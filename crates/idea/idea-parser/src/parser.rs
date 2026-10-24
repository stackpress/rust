//! Extracted recursive-descent syntax; all cursor offsets remain UTF-8 boundaries.

use serde_json::Value;

use crate::ast::{
    ArrayToken, DataToken, DeclarationToken, DeclaratorToken, IdentifierToken, ImportToken,
    LiteralToken, ObjectToken, PropertyToken, SchemaToken, Statement,
};
use crate::error::{ErrorCode, IdeaError};
use crate::{CompileInputs, ParseLimits};

/// Parse a complete source and reject trailing nondeclaration input.
pub(crate) fn parse(source: &str, inputs: &CompileInputs) -> Result<SchemaToken, IdeaError> {
    let mut parser = Parser::new(source, inputs);
    let start = parser.index;
    let mut body = Vec::new();

    // Skip leading trivia once; each statement then owns its complete grammar extent.
    parser.skip_noncode()?;
    while !parser.is_eof() {
        parser.charge_node()?;
        if body.len() >= inputs.limits.max_collection() {
            return Err(parser.limit("Declaration collection limit exceeded"));
        }
        body.push(parser.parse_statement()?);
        // Top-level statements need trivia between them, but never after the final statement.
        let end = parser.index;
        parser.skip_noncode()?;
        if !parser.is_eof() && parser.index == end {
            return Err(IdeaError::new(
                ErrorCode::UnexpectedToken,
                "Expected declaration separator",
            )
            .with_span(end, end + 1));
        }
    }

    // Cross-kind uniqueness is a syntax invariant as well as a compilation invariant.
    let mut names = std::collections::HashMap::new();
    for item in &body {
        if let Statement::Declaration(declaration) = item {
            let name = &declaration.declarations[0].id;
            if let Some((start, end)) = names.insert(&name.name, (name.start, name.end)) {
                return Err(
                    IdeaError::new(ErrorCode::DuplicateName, "Duplicate declaration")
                        .with_span(name.start, name.end)
                        .with_related(start, end),
                );
            }
        }
    }
    Ok(SchemaToken {
        node_type: "Program",
        kind: "schema",
        start,
        end: parser.index,
        body,
    })
}

pub(crate) struct Parser<'a> {
    source: &'a str,
    index: usize,
    /// Active recursive data depth, checked before descent.
    depth: usize,
    /// Explicit environment snapshot and caller-lowered limits.
    inputs: &'a CompileInputs,
    /// Whether nested data is currently inside an attribute argument list.
    attribute_data: bool,
    /// Aggregate syntax-node count, charged before growing a collection.
    nodes: usize,
}

impl<'a> Parser<'a> {
    /// Initialize an isolated cursor at the beginning of this source.
    fn new(source: &'a str, inputs: &'a CompileInputs) -> Self {
        Self {
            source,
            index: 0,
            depth: 0,
            inputs,
            attribute_data: false,
            nodes: 0,
        }
    }

    /// Produce a positioned limit error at the cursor before allocating another node.
    fn limit(&self, message: &str) -> IdeaError {
        IdeaError::new(ErrorCode::Limit, message).with_span(self.index, self.index)
    }

    /// Charge one syntax node before allocating it.
    fn charge_node(&mut self) -> Result<(), IdeaError> {
        if self.nodes >= self.inputs.limits.max_nodes() {
            return Err(self.limit("Syntax node budget exceeded"));
        }
        self.nodes += 1;
        Ok(())
    }

    /// Guard a collection before parsing and allocating its next member.
    fn check_collection(&mut self, length: usize) -> Result<(), IdeaError> {
        if length >= self.inputs.limits.max_collection() {
            return Err(self.limit("Syntax collection budget exceeded"));
        }
        self.charge_node()
    }

    /// Require trivia between values while allowing an adjacent closing delimiter.
    fn separate_or_close(&mut self, closer: char) -> Result<(), IdeaError> {
        if self.check_char(closer) {
            return Ok(());
        }
        self.expect_whitespace()
    }

    /// Reject recursive data before constructing another stack frame's children.
    fn enter_data(&mut self) -> Result<(), IdeaError> {
        // The hard ceiling cannot be raised by caller configuration.
        if self.depth >= self.inputs.limits.max_depth.min(ParseLimits::HARD_DEPTH) {
            return Err(
                IdeaError::new(ErrorCode::Limit, "Idea nesting limit exceeded")
                    .with_span(self.index, self.index),
            );
        }
        self.depth += 1;
        Ok(())
    }

    /// Check whether the cursor reached the end of the UTF-8 source.
    fn is_eof(&self) -> bool {
        self.index >= self.source.len()
    }

    /// Read the next character at the maintained character-boundary cursor.
    fn current(&self) -> Option<char> {
        self.source[self.index..].chars().next()
    }

    /// Advance by the full UTF-8 width of the current character.
    fn advance_char(&mut self) -> Option<char> {
        let ch = self.current()?;
        // Byte offsets advance by the entire scalar, never into a UTF-8 continuation byte.
        self.index += ch.len_utf8();
        Some(ch)
    }

    /// Test a grammar prefix without changing cursor state.
    fn starts_with(&self, value: &str) -> bool {
        self.source[self.index..].starts_with(value)
    }

    /// Consume whitespace and comments, rejecting unterminated block comments.
    fn skip_noncode(&mut self) -> Result<(), IdeaError> {
        loop {
            let start = self.index;
            while matches!(self.current(), Some(ch) if ch.is_whitespace()) {
                self.advance_char();
            }

            // Line and block comments must be recognized before a slash can be a token.
            if self.starts_with("//") {
                while let Some(ch) = self.advance_char() {
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                }
            } else if self.starts_with("/*") {
                self.index += 2;
                while !self.is_eof() && !self.starts_with("*/") {
                    self.advance_char();
                }
                if self.is_eof() {
                    return Err(IdeaError::new(
                        ErrorCode::UnexpectedEof,
                        "Unterminated block comment",
                    )
                    .with_span(self.source.len(), self.source.len()));
                }
                self.index += 2;
            }

            // Stop only after a full trivia pass makes no progress, preventing empty loops.
            if self.index == start {
                break;
            }
        }

        Ok(())
    }

    /// Dispatch exactly one of the six supported declaration forms.
    fn parse_statement(&mut self) -> Result<Statement, IdeaError> {
        let word = self.peek_word().ok_or_else(|| {
            IdeaError::new(ErrorCode::UnexpectedToken, "Expected declaration")
                .with_span(self.index, self.index + 1)
        })?;

        // Declaration keywords are closed; unknown metadata stays open inside attributes.
        match word.as_str() {
            "plugin" => Ok(Statement::Declaration(self.parse_plugin()?)),
            "use" => Ok(Statement::Import(self.parse_use()?)),
            "prop" => Ok(Statement::Declaration(self.parse_prop()?)),
            "enum" => Ok(Statement::Declaration(self.parse_enum()?)),
            "type" => Ok(Statement::Declaration(self.parse_type_like("type")?)),
            "model" => Ok(Statement::Declaration(self.parse_type_like("model")?)),
            _ => Err(IdeaError::new(
                ErrorCode::UnexpectedToken,
                format!("Unexpected token {}", word),
            )
            .with_span(self.index, self.index + word.len())),
        }
    }

    /// Record an import specifier without resolving or loading a file.
    fn parse_use(&mut self) -> Result<ImportToken, IdeaError> {
        let start = self.index;
        self.expect_keyword("use")?;
        self.expect_whitespace()?;
        let source = self.parse_string_literal()?;
        Ok(ImportToken {
            node_type: "ImportDeclaration",
            start,
            end: self.index,
            specifiers: vec![],
            source,
        })
    }

    /// Read one named plugin configuration without executing any code.
    fn parse_plugin(&mut self) -> Result<DeclarationToken, IdeaError> {
        let start = self.index;
        self.expect_keyword("plugin")?;
        self.expect_whitespace()?;
        let id_literal = self.parse_string_literal()?;
        self.expect_whitespace()?;
        let init = self.parse_object_literal()?;
        let id = IdentifierToken::new(
            id_literal.start,
            id_literal.end,
            id_literal.value.as_str().unwrap_or_default(),
        );

        Ok(self.declaration("plugin", None, start, self.index, id, init))
    }

    /// Read an ordered reusable property declaration.
    fn parse_prop(&mut self) -> Result<DeclarationToken, IdeaError> {
        let start = self.index;
        self.expect_keyword("prop")?;
        self.expect_whitespace()?;
        let id = self.parse_capital_identifier()?;
        self.expect_whitespace()?;
        let init = self.parse_object_literal()?;
        Ok(self.declaration("prop", None, start, init.end, id, init))
    }

    /// Read an ordered scalar option declaration.
    fn parse_enum(&mut self) -> Result<DeclarationToken, IdeaError> {
        let start = self.index;
        self.expect_keyword("enum")?;
        self.expect_whitespace()?;
        let id = self.parse_capital_identifier()?;
        self.expect_whitespace()?;
        self.expect_char('{')?;
        self.skip_noncode()?;

        let mut properties = Vec::new();
        // Each successful member parser advances; EOF and invalid members return an error.
        while !self.check_char('}') {
            self.check_collection(properties.len())?;
            let key = self.parse_upper_identifier()?;
            self.expect_whitespace()?;
            let value = if self.starts_with("env(\"") {
                DataToken::Environment(self.parse_env_literal()?)
            } else {
                DataToken::Literal(self.parse_scalar_literal()?)
            };
            let end = value_end(&value);
            properties.push(PropertyToken::new(key.start, end, key, value));
            self.separate_or_close('}')?;
        }
        self.expect_char('}')?;
        unique_properties(&properties)?;

        Ok(self.declaration(
            "enum",
            None,
            start,
            self.index,
            id,
            ObjectToken::new(start, self.index, properties),
        ))
    }

    /// Read mutable/immutable model or type structure and ordered columns.
    fn parse_type_like(&mut self, kind: &'static str) -> Result<DeclarationToken, IdeaError> {
        let start = self.index;
        self.expect_keyword(kind)?;
        self.expect_whitespace()?;
        let id = self.parse_capital_identifier()?;
        // Preserve the explicit marker; composition, not parsing, gives G19 its meaning.
        let mutable = !self.consume_char('!');
        self.expect_whitespace()?;

        // Attributes stay uninterpreted so framework metadata does not leak into grammar.
        let attributes = self.parse_attribute_properties()?;
        self.skip_noncode()?;
        self.expect_char('{')?;
        self.skip_noncode()?;

        let mut columns = Vec::new();
        // Each successful member parser advances; EOF and invalid members return an error.
        while !self.check_char('}') {
            self.check_collection(columns.len())?;
            columns.push(self.parse_column_property()?);
        }
        self.expect_char('}')?;
        unique_properties(&columns)?;

        let init = ObjectToken::new(
            start,
            self.index,
            vec![
                PropertyToken::new(
                    start,
                    self.index,
                    IdentifierToken::new(start, start, "attributes"),
                    DataToken::Object(ObjectToken::new(start, self.index, attributes)),
                ),
                PropertyToken::new(
                    start,
                    self.index,
                    IdentifierToken::new(start, start, "columns"),
                    DataToken::Object(ObjectToken::new(start, self.index, columns)),
                ),
            ],
        );

        Ok(self.declaration(kind, Some(mutable), start, self.index, id, init))
    }

    /// Read one field type and its open attribute metadata.
    fn parse_column_property(&mut self) -> Result<PropertyToken, IdeaError> {
        let key = self.parse_camel_identifier()?;
        self.expect_whitespace()?;
        let value = self.parse_type_literal()?;
        self.separate_or_close('}')?;
        // Attributes stay uninterpreted so framework metadata does not leak into grammar.
        let attributes = self.parse_attribute_properties()?;
        self.skip_noncode()?;

        let value_start = value.start;
        let end = self.index;
        Ok(PropertyToken::new(
            key.start,
            end,
            key,
            DataToken::Object(ObjectToken::new(
                value_start,
                end,
                vec![
                    PropertyToken::new(
                        value_start,
                        value.end,
                        IdentifierToken::new(value_start, value.end, "type"),
                        DataToken::Literal(value.clone()),
                    ),
                    PropertyToken::new(
                        value_start,
                        end,
                        IdentifierToken::new(value_start, value.end, "attributes"),
                        DataToken::Object(ObjectToken::new(value_start, end, attributes)),
                    ),
                ],
            )),
        ))
    }

    /// Read attributes in source order and reject duplicate names.
    fn parse_attribute_properties(&mut self) -> Result<Vec<PropertyToken>, IdeaError> {
        let mut properties = Vec::new();
        while self.check_char('@') {
            self.check_collection(properties.len())?;
            properties.push(self.parse_attribute_property()?);
            // Metadata methods/flags cannot run directly into another attribute or field.
            self.separate_or_close('}')?;
        }
        unique_properties(&properties)?;
        Ok(properties)
    }

    /// Attributes are the trickiest part of the grammar because they mix two forms:
    /// `@flag` and `@method(arg1 arg2)`.
    ///
    /// The parser keeps them normalized as object properties so finalization can
    /// resolve prop references later without caring whether the source came from a
    /// top-level declaration attribute or a column attribute.
    fn parse_attribute_property(&mut self) -> Result<PropertyToken, IdeaError> {
        let start = self.index;
        self.expect_char('@')?;
        let key = self.parse_attribute_identifier(start)?;
        // Parentheses choose method arguments; no parentheses means a true flag.
        let value = if self.consume_char('(') {
            self.skip_noncode()?;
            self.attribute_data = true;
            let mut elements = Vec::new();
            // Whitespace/comments separate complete arguments without comma syntax.
            while !self.check_char(')') {
                self.check_collection(elements.len())?;
                elements.push(self.parse_attribute_data()?);
                self.separate_or_close(')')?;
            }
            self.expect_char(')')?;
            self.attribute_data = false;
            DataToken::Array(ArrayToken::new(start, self.index, elements))
        } else {
            DataToken::Literal(LiteralToken::new(
                start,
                self.index,
                Value::Bool(true),
                "true",
            ))
        };

        Ok(PropertyToken::new(start, self.index, key, value))
    }

    /// Allow prior prop identifiers only in attribute argument positions.
    fn parse_attribute_data(&mut self) -> Result<DataToken, IdeaError> {
        // Composite delimiters take precedence over scalar and identifier interpretation.
        if self.current() == Some('{') {
            return Ok(DataToken::Object(self.parse_object_literal()?));
        }
        if self.current() == Some('[') {
            return Ok(DataToken::Array(self.parse_array_literal()?));
        }
        if self.current() == Some('"')
            || self.starts_with("env(\"")
            || matches!(self.current(), Some(ch) if ch.is_ascii_digit() || ch == '-')
        {
            return self.parse_data();
        }

        if let Some(word) = self.peek_word() {
            if matches!(word.as_str(), "true" | "false" | "null") {
                return self.parse_data();
            }
            if is_capital_identifier(&word) {
                let start = self.index;
                let identifier = self.parse_capital_identifier()?;
                return Ok(DataToken::Identifier(IdentifierToken::new(
                    start,
                    self.index,
                    identifier.name,
                )));
            }
        }

        self.parse_data()
    }

    /// Read literal scalar, object or array data without executing expressions.
    fn parse_data(&mut self) -> Result<DataToken, IdeaError> {
        self.charge_node()?;
        // Composite delimiters take precedence over scalar and identifier interpretation.
        if self.current() == Some('{') {
            return Ok(DataToken::Object(self.parse_object_literal()?));
        }
        if self.current() == Some('[') {
            return Ok(DataToken::Array(self.parse_array_literal()?));
        }
        // Environment syntax uses only this invocation's explicit map.
        if self.starts_with("env(\"") {
            return Ok(DataToken::Environment(self.parse_env_literal()?));
        }
        if self.current() == Some('"') {
            return Ok(DataToken::Literal(self.parse_string_literal()?));
        }
        if matches!(self.current(), Some(ch) if ch.is_ascii_digit() || ch == '-') {
            return Ok(DataToken::Literal(self.parse_number_literal()?));
        }

        let word = self.peek_word().ok_or_else(|| {
            IdeaError::new(
                if self.is_eof() {
                    ErrorCode::UnexpectedEof
                } else {
                    ErrorCode::UnexpectedToken
                },
                "Expected data",
            )
            .with_span(self.index, self.index + 1)
        })?;
        if matches!(word.as_str(), "true" | "false" | "null") {
            return Ok(DataToken::Literal(self.parse_scalar_literal()?));
        }

        // Attribute argument composites retain typed references just like direct arguments.
        if self.attribute_data && is_capital_identifier(&word) {
            return Ok(DataToken::Identifier(self.parse_capital_identifier()?));
        }

        Err(
            IdeaError::new(ErrorCode::UnexpectedToken, "Expected literal data")
                .with_span(self.index, self.index + word.len()),
        )
    }

    /// Read bounded nested object data with unique member names.
    fn parse_object_literal(&mut self) -> Result<ObjectToken, IdeaError> {
        let start = self.index;
        self.enter_data()?;
        self.expect_char('{')?;
        self.skip_noncode()?;
        let mut properties = Vec::new();
        // Each successful member parser advances; EOF and invalid members return an error.
        while !self.check_char('}') {
            self.check_collection(properties.len())?;
            let key = self.parse_any_identifier()?;
            self.expect_whitespace()?;
            let value = self.parse_data()?;
            let end = value_end(&value);
            properties.push(PropertyToken::new(key.start, end, key, value));
            self.separate_or_close('}')?;
        }
        self.expect_char('}')?;
        self.depth -= 1;
        unique_properties(&properties)?;
        Ok(ObjectToken::new(start, self.index, properties))
    }

    /// Read bounded whitespace-separated array elements.
    fn parse_array_literal(&mut self) -> Result<ArrayToken, IdeaError> {
        let start = self.index;
        self.enter_data()?;
        self.expect_char('[')?;
        self.skip_noncode()?;
        let mut elements = Vec::new();
        // Nested elements spend depth before descending and release it only on success.
        while !self.check_char(']') {
            self.check_collection(elements.len())?;
            elements.push(self.parse_data()?);
            self.separate_or_close(']')?;
        }
        self.expect_char(']')?;
        self.depth -= 1;
        Ok(ArrayToken::new(start, self.index, elements))
    }

    /// Retain one environment name without reading or copying any environment value.
    fn parse_env_literal(&mut self) -> Result<LiteralToken, IdeaError> {
        let start = self.index;
        self.expect_keyword("env")?;
        self.expect_char('(')?;
        let key = self.parse_string_literal()?;
        self.expect_char(')')?;
        // Retain the name and exact syntax; compilation alone receives environment values.
        let mut node = LiteralToken::new(
            start,
            self.index,
            key.value,
            &self.source[start..self.index],
        );
        node.node_type = "Environment";
        Ok(node)
    }

    /// Read the existing raw double-quoted grammar without inventing escapes.
    fn parse_string_literal(&mut self) -> Result<LiteralToken, IdeaError> {
        let start = self.index;
        self.expect_char('"')?;
        let mut value = String::new();
        while let Some(ch) = self.current() {
            if ch == '"' {
                self.advance_char();
                return Ok(LiteralToken::new(
                    start,
                    self.index,
                    Value::String(value.clone()),
                    format!("'{}'", value),
                ));
            }
            value.push(ch);
            self.advance_char();
        }
        Err(
            IdeaError::new(ErrorCode::UnexpectedEof, "Unterminated string literal")
                .with_span(self.source.len(), self.source.len()),
        )
    }

    /// Read an exact 64-bit integer or finite decimal float.
    fn parse_number_literal(&mut self) -> Result<LiteralToken, IdeaError> {
        let start = self.index;
        if self.current() == Some('-') {
            self.advance_char();
        }
        let digits = self.index;
        while matches!(self.current(), Some(ch) if ch.is_ascii_digit()) {
            self.advance_char();
        }
        if digits == self.index {
            return Err(
                IdeaError::new(ErrorCode::InvalidLiteral, "Expected numeric digits")
                    .with_span(start, self.index),
            );
        }
        // A decimal point selects finite-float parsing; integer precision is never coerced.
        let mut is_float = false;
        if self.current() == Some('.') {
            is_float = true;
            self.advance_char();
            let fractional_start = self.index;
            while matches!(self.current(), Some(ch) if ch.is_ascii_digit()) {
                self.advance_char();
            }
            if fractional_start == self.index {
                return Err(IdeaError::new(
                    ErrorCode::InvalidLiteral,
                    "Expected fractional digits",
                )
                .with_span(start, self.index));
            }
        }
        let raw = &self.source[start..self.index];
        if is_float {
            let value = raw.parse::<f64>().map_err(|_| {
                IdeaError::new(ErrorCode::InvalidLiteral, "Invalid float literal")
                    .with_span(start, self.index)
            })?;
            if !value.is_finite() {
                return Err(
                    IdeaError::new(ErrorCode::InvalidLiteral, "Nonfinite number")
                        .with_span(start, self.index),
                );
            }
            Ok(LiteralToken::new(
                start,
                self.index,
                Value::from(value),
                raw,
            ))
        } else {
            let value = raw
                .parse::<i64>()
                .map(serde_json::Number::from)
                .or_else(|_| raw.parse::<u64>().map(serde_json::Number::from))
                .map_err(|_| {
                    IdeaError::new(ErrorCode::InvalidLiteral, "Integer outside 64-bit range")
                        .with_span(start, self.index)
                })?;
            Ok(LiteralToken::new(
                start,
                self.index,
                Value::Number(value),
                raw,
            ))
        }
    }

    /// Read literal scalar data, including explicit environment values.
    fn parse_scalar_literal(&mut self) -> Result<LiteralToken, IdeaError> {
        // Enums use the same explicit environment grammar as other scalar positions.
        // Environment syntax uses only this invocation's explicit map.
        if self.starts_with("env(\"") {
            return self.parse_env_literal();
        }
        if self.current() == Some('"') {
            return self.parse_string_literal();
        }
        if matches!(self.current(), Some(ch) if ch.is_ascii_digit() || ch == '-') {
            return self.parse_number_literal();
        }

        let start = self.index;
        let word = self.parse_word()?;
        // Declaration keywords are closed; unknown metadata stays open inside attributes.
        match word.as_str() {
            "true" => Ok(LiteralToken::new(
                start,
                self.index,
                Value::Bool(true),
                "true",
            )),
            "false" => Ok(LiteralToken::new(
                start,
                self.index,
                Value::Bool(false),
                "false",
            )),
            "null" => Ok(LiteralToken::new(start, self.index, Value::Null, "null")),
            _ => Err(IdeaError::new(
                ErrorCode::InvalidLiteral,
                format!("Invalid literal {}", word),
            )
            .with_span(start, self.index)),
        }
    }

    /// Validate exactly the approved scalar/list optionality modifiers.
    fn parse_type_literal(&mut self) -> Result<LiteralToken, IdeaError> {
        let start = self.index;
        let mut word = self.parse_word()?;
        // Brackets belong to type modifiers only here, never to a reference identifier.
        while matches!(self.current(), Some('[' | ']' | '?')) {
            word.push(self.advance_char().unwrap_or_default());
        }
        if !is_type_identifier(&word) {
            return Err(IdeaError::new(
                ErrorCode::InvalidLiteral,
                format!("Invalid type {}", word),
            )
            .with_span(start, self.index));
        }
        Ok(LiteralToken::new(
            start,
            self.index,
            Value::String(word.clone()),
            format!("\"{}\"", word),
        ))
    }

    /// Read a lowercase dotted metadata namespace.
    fn parse_attribute_identifier(&mut self, start: usize) -> Result<IdentifierToken, IdeaError> {
        let mut value = String::new();
        while self
            .current()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.'))
        {
            value.push(self.advance_char().unwrap_or_default());
        }

        if !valid_attribute(&value) {
            return Err(IdeaError::new(
                ErrorCode::UnexpectedToken,
                "Expected lowercase attribute namespace",
            )
            .with_span(start, self.index));
        }

        Ok(IdentifierToken::new(start, self.index, value))
    }

    /// Read an object key with ordinary ASCII identifier syntax.
    fn parse_any_identifier(&mut self) -> Result<IdentifierToken, IdeaError> {
        let start = self.index;
        let word = self.parse_word()?;
        if !is_any_identifier(&word) {
            return Err(IdeaError::new(
                ErrorCode::UnexpectedToken,
                format!("Invalid identifier {}", word),
            )
            .with_span(start, self.index));
        }
        Ok(IdentifierToken::new(start, self.index, word))
    }

    /// Require the capital declaration-name convention.
    fn parse_capital_identifier(&mut self) -> Result<IdentifierToken, IdeaError> {
        let start = self.index;
        let word = self.parse_word()?;
        if !is_capital_identifier(&word) {
            return Err(IdeaError::new(
                ErrorCode::UnexpectedToken,
                format!("Invalid identifier {}", word),
            )
            .with_span(start, self.index));
        }
        Ok(IdentifierToken::new(start, self.index, word))
    }

    /// Require uppercase enum option keys.
    fn parse_upper_identifier(&mut self) -> Result<IdentifierToken, IdeaError> {
        let start = self.index;
        let word = self.parse_word()?;
        if !is_upper_identifier(&word) {
            return Err(IdeaError::new(
                ErrorCode::UnexpectedToken,
                format!("Invalid identifier {}", word),
            )
            .with_span(start, self.index));
        }
        Ok(IdentifierToken::new(start, self.index, word))
    }

    /// Require the lowercase field-name convention.
    fn parse_camel_identifier(&mut self) -> Result<IdentifierToken, IdeaError> {
        let start = self.index;
        let word = self.parse_word()?;
        if !is_camel_identifier(&word) {
            return Err(IdeaError::new(
                ErrorCode::UnexpectedToken,
                format!("Invalid identifier {}", word),
            )
            .with_span(start, self.index));
        }
        Ok(IdentifierToken::new(start, self.index, word))
    }

    /// Consume the lexical word characters while preserving UTF-8 cursor validity.
    fn parse_word(&mut self) -> Result<String, IdeaError> {
        let start = self.index;
        let mut word = String::new();
        while !self.starts_with("//")
            && !self.starts_with("/*")
            && matches!(self.current(), Some(ch)
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '?' | '/' | '-'))
        {
            word.push(self.advance_char().unwrap_or_default());
        }
        if word.is_empty() {
            return Err(IdeaError::new(
                if self.is_eof() {
                    ErrorCode::UnexpectedEof
                } else {
                    ErrorCode::UnexpectedToken
                },
                "Expected word",
            )
            .with_span(start, start + 1));
        }
        Ok(word)
    }

    /// Inspect the next lexical word without committing cursor advancement.
    fn peek_word(&self) -> Option<String> {
        let mut index = self.index;
        let mut word = String::new();
        while index < self.source.len() {
            if self.source[index..].starts_with("//") || self.source[index..].starts_with("/*") {
                break;
            }
            let ch = self.source[index..].chars().next()?;
            if !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '?' | '/' | '-')) {
                break;
            }
            word.push(ch);
            index += ch.len_utf8();
        }
        if word.is_empty() { None } else { Some(word) }
    }

    /// Consume one required declaration keyword.
    fn expect_keyword(&mut self, value: &str) -> Result<(), IdeaError> {
        let start = self.index;
        let word = self.parse_word()?;
        if word != value {
            return Err(
                IdeaError::new(ErrorCode::UnexpectedToken, format!("Expected {}", value))
                    .with_span(start, self.index),
            );
        }
        Ok(())
    }

    /// Require a separator so adjacent values cannot merge ambiguously.
    fn expect_whitespace(&mut self) -> Result<(), IdeaError> {
        let start = self.index;
        self.skip_noncode()?;
        if self.index == start {
            return Err(IdeaError::new(
                if self.is_eof() {
                    ErrorCode::UnexpectedEof
                } else {
                    ErrorCode::UnexpectedToken
                },
                "Expected whitespace or comment",
            )
            .with_span(start, start + usize::from(!self.is_eof())));
        }
        Ok(())
    }

    /// Consume one required delimiter or return its source location.
    fn expect_char(&mut self, expected: char) -> Result<(), IdeaError> {
        match self.current() {
            Some(ch) if ch == expected => {
                self.advance_char();
                Ok(())
            }
            _ => Err(IdeaError::new(
                if self.is_eof() {
                    ErrorCode::UnexpectedEof
                } else {
                    ErrorCode::UnexpectedToken
                },
                format!("Expected {}", expected),
            )
            .with_span(self.index, self.index + 1)),
        }
    }

    /// Consume an optional delimiter while leaving other input untouched.
    fn consume_char(&mut self, expected: char) -> bool {
        if self.current() == Some(expected) {
            self.advance_char();
            true
        } else {
            false
        }
    }

    /// Inspect an optional delimiter without changing parser state.
    fn check_char(&mut self, expected: char) -> bool {
        self.current() == Some(expected)
    }

    /// Assemble the extracted private declaration shape from validated components.
    fn declaration(
        &self,
        kind: &'static str,
        mutable: Option<bool>,
        start: usize,
        end: usize,
        id: IdentifierToken,
        init: ObjectToken,
    ) -> DeclarationToken {
        DeclarationToken {
            node_type: "VariableDeclaration",
            kind,
            mutable,
            start,
            end,
            declarations: vec![DeclaratorToken {
                node_type: "VariableDeclarator",
                start: id.start,
                end,
                id,
                init,
            }],
        }
    }
}

/// Return the source endpoint of a typed data node.
fn value_end(value: &DataToken) -> usize {
    match value {
        DataToken::Identifier(value) => value.end,
        DataToken::Literal(value) | DataToken::Environment(value) => value.end,
        DataToken::Object(value) => value.end,
        DataToken::Array(value) => value.end,
    }
}

/// Validate ordinary ASCII identifier characters.
pub(crate) fn is_any_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Validate a capitalized declaration/type name.
pub(crate) fn is_capital_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_uppercase() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Validate an uppercase enum member name.
pub(crate) fn is_upper_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_uppercase() || ch == '_')
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

/// Validate a lowercase field name.
pub(crate) fn is_camel_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Accept exactly required/optional scalar and required/optional single list.
pub(crate) fn is_type_identifier(value: &str) -> bool {
    // Removing at most one suffix keeps repeated modifiers invalid.
    let base = value.strip_suffix('?').unwrap_or(value);
    let base = base.strip_suffix("[]").unwrap_or(base);
    is_capital_identifier(base)
}

/// Require lowercase namespace segments rather than accepting arbitrary dots/digits.
pub(crate) fn valid_attribute(value: &str) -> bool {
    value.split('.').all(|part| {
        let mut chars = part.chars();
        matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
            && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    })
}

/// Reject ambiguous member replacement while retaining both declaration locations.
fn unique_properties(properties: &[PropertyToken]) -> Result<(), IdeaError> {
    let mut names = std::collections::HashMap::new();
    // Inspect source order so the later member is the primary diagnostic.
    for property in properties {
        if let Some((first, first_end)) =
            names.insert(&property.key.name, (property.key.start, property.key.end))
        {
            return Err(IdeaError::new(
                ErrorCode::DuplicateName,
                format!(
                    "Duplicate member {} (first at byte {first})",
                    property.key.name
                ),
            )
            .with_span(property.key.start, property.key.end)
            .with_related(first, first_end));
        }
    }
    Ok(())
}

/// Tokenize with the same literal, identifier, attribute and trivia scanner as the parser.
pub(crate) fn tokenize_source(
    source: &crate::SourceFile,
    limits: ParseLimits,
) -> Result<Vec<crate::Token>, IdeaError> {
    let inputs = CompileInputs {
        limits,
        ..Default::default()
    };
    crate::check_inputs(source.name(), source.text(), &inputs)?;
    let mut scanner = Parser::new(source.text(), &inputs);
    let mut tokens = Vec::new();
    let mut delimiter_depth = 0usize;
    let result = (|| {
        scanner.skip_noncode()?;
        while !scanner.is_eof() {
            scanner.charge_node()?;
            let start = scanner.index;
            // Composite punctuation stays lexical; parsing alone checks balanced grammar.
            let (kind, value) = match scanner.current() {
                Some('@') => {
                    scanner.advance_char();
                    let identifier = scanner.parse_attribute_identifier(start)?;
                    (
                        crate::TokenKind::Attribute,
                        crate::TokenValue::Text(identifier.name),
                    )
                }
                Some('"') => {
                    let literal = scanner.parse_string_literal()?;
                    (
                        crate::TokenKind::Literal,
                        crate::TokenValue::Literal(literal.value),
                    )
                }
                Some(ch) if matches!(ch, '{' | '}' | '[' | ']' | '(' | ')' | '!') => {
                    if matches!(ch, '{' | '[' | '(') {
                        if delimiter_depth >= limits.max_depth.min(ParseLimits::HARD_DEPTH) {
                            return Err(scanner.limit("Token nesting limit exceeded"));
                        }
                        delimiter_depth += 1;
                    } else if matches!(ch, '}' | ']' | ')') {
                        delimiter_depth = delimiter_depth.saturating_sub(1);
                    }
                    scanner.advance_char();
                    (crate::TokenKind::Delimiter, crate::TokenValue::Symbol(ch))
                }
                Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                    let literal = scanner.parse_number_literal()?;
                    (
                        crate::TokenKind::Literal,
                        crate::TokenValue::Literal(literal.value),
                    )
                }
                _ if scanner.starts_with("env(\"") => {
                    let environment = scanner.parse_env_literal()?;
                    (
                        crate::TokenKind::Environment,
                        crate::TokenValue::Literal(environment.value),
                    )
                }
                _ => {
                    let mut word = scanner.parse_word()?;
                    if is_capital_identifier(word.trim_end_matches('?')) {
                        while scanner.starts_with("[]") || scanner.current() == Some('?') {
                            if scanner.starts_with("[]") {
                                word.push('[');
                                word.push(']');
                                scanner.index += 2;
                            } else {
                                word.push(scanner.advance_char().unwrap_or_default());
                            }
                        }
                    }
                    match word.as_str() {
                        "true" => (
                            crate::TokenKind::Literal,
                            crate::TokenValue::Literal(true.into()),
                        ),
                        "false" => (
                            crate::TokenKind::Literal,
                            crate::TokenValue::Literal(false.into()),
                        ),
                        "null" => (
                            crate::TokenKind::Literal,
                            crate::TokenValue::Literal(Value::Null),
                        ),
                        _ if is_any_identifier(&word) || is_type_identifier(&word) => {
                            (crate::TokenKind::Word, crate::TokenValue::Text(word))
                        }
                        _ => {
                            return Err(IdeaError::new(
                                ErrorCode::UnexpectedToken,
                                "Invalid lexical word",
                            )
                            .with_span(start, scanner.index));
                        }
                    }
                }
            };
            tokens.push(crate::Token {
                kind,
                value,
                span: crate::SourceSpan::new(source, start, scanner.index),
                raw: source.text()[start..scanner.index].into(),
            });
            scanner.skip_noncode()?;
        }
        Ok(tokens)
    })();
    result.map_err(|error| error.locate(source.name(), source.text()))
}
