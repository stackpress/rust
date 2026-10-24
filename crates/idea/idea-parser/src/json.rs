//! Bounded, duplicate-aware normalized JSON input and checked export.

use std::collections::HashSet;
use std::io::{self, Write};

use indexmap::IndexMap;
use serde::Serialize;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

use crate::parser::{
    is_any_identifier, is_camel_identifier, is_capital_identifier, is_upper_identifier,
    valid_attribute,
};
use crate::{
    AttributeValue, ColumnConfig, CompiledSchema, ErrorCode, IdeaError, ParseLimits, Reference,
    SchemaConfig, TypeConfig,
};

impl SchemaConfig {
    /// Import ordered normalized JSON, rejecting duplicates and malformed configuration.
    ///
    /// JSON strings are literal data. Use [`CompiledSchema::from_json`] for typed references.
    /// Limits bound input bytes, JSON container depth and materialized export size.
    ///
    /// # Errors
    /// Returns named `<json>` syntax, validation, duplicate or resource diagnostics.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::{SchemaConfig, ParseLimits};
    /// let schema = SchemaConfig::from_json(r#"{"prop":{"P":{"x":1}}}"#, ParseLimits::default())?;
    /// assert_eq!(schema.prop["P"]["x"], 1);
    /// # Ok::<(), stackpress_idea_parser::IdeaError>(())
    /// ```
    pub fn from_json(source: &str, limits: ParseLimits) -> Result<Self, IdeaError> {
        Self::from_json_named("<json>", source, limits)
    }

    /// Import normalized JSON with an explicit logical diagnostic name.
    ///
    /// # Errors
    /// Returns the same checked errors as [`Self::from_json`].
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::{SchemaConfig, ParseLimits};
    /// assert!(SchemaConfig::from_json_named("empty.json", "{}", ParseLimits::default()).is_ok());
    /// ```
    pub fn from_json_named(
        file: &str,
        source: &str,
        limits: ParseLimits,
    ) -> Result<Self, IdeaError> {
        let result = read_json(file, source, limits).and_then(|value| schema_from_value(&value));
        result.map_err(|error| locate_json(error, file, source))
    }

    /// Validate public, manually assembled config before clone, export or finalization.
    ///
    /// Existing public config fields remain compatible. Direct `Clone`/Serde on arbitrary
    /// caller-built recursive `Value`s does not provide this checked boundary.
    ///
    /// # Errors
    /// Returns invalid names/structure, duplicate declarations/columns or resource limits.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::{SchemaConfig, ParseLimits};
    /// SchemaConfig::default().validate(ParseLimits::default())?;
    /// # Ok::<(), stackpress_idea_parser::IdeaError>(())
    /// ```
    pub fn validate(&self, limits: ParseLimits) -> Result<(), IdeaError> {
        // Reject caller-built wide sections before visiting their values or allocating JSON.
        let maximum = limits.max_collection();
        if self.r#use.len() > maximum
            || self.plugin.len() > maximum
            || self.prop.len() > maximum
            || self.r#enum.len() > maximum
            || self.r#type.len() > maximum
            || self.model.len() > maximum
        {
            return Err(limit("Schema collection budget exceeded"));
        }
        // Inspect borrowed recursive values before invoking recursive Serde or Clone.
        for values in self
            .plugin
            .values()
            .chain(self.prop.values())
            .chain(self.r#enum.values())
        {
            if values.len() > maximum {
                return Err(limit("Member collection budget exceeded"));
            }
            for value in values.values() {
                validate_value(value, limits)?;
            }
        }
        for value in self.r#type.values().chain(self.model.values()) {
            if value.columns.len() > maximum {
                return Err(limit("Column collection budget exceeded"));
            }
            validate_attributes(&value.attributes, limits)?;
            for column in &value.columns {
                validate_attributes(&column.attributes, limits)?;
            }
        }
        // A bounded serializer prevents a wide caller-built config from allocating its JSON.
        let encoded = encode(self, limits)?;
        let value = read_json("<config>", &encoded, limits)?;
        schema_from_value(&value)?;
        Ok(())
    }

    /// Export checked ordered config as compact JSON with no ambient inputs or IO.
    ///
    /// # Errors
    /// Rejects invalid manually constructed config or output exceeding the supplied limits.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::{SchemaConfig, ParseLimits};
    /// assert_eq!(SchemaConfig::default().to_json(ParseLimits::default())?, "{}");
    /// # Ok::<(), stackpress_idea_parser::IdeaError>(())
    /// ```
    pub fn to_json(&self, limits: ParseLimits) -> Result<String, IdeaError> {
        self.validate(limits)?;
        encode(self, limits)
    }
}

impl CompiledSchema {
    /// Export config together with actual reference sites; literal lookalikes remain data.
    ///
    /// # Errors
    /// Rejects malformed config/reference sites, invalid coordinates or exceeded budgets.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::*;
    /// let compiled = CompiledSchema { config: SchemaConfig::default(), references: vec![] };
    /// assert!(compiled.to_json(ParseLimits::default())?.contains("references"));
    /// # Ok::<(), IdeaError>(())
    /// ```
    pub fn to_json(&self, limits: ParseLimits) -> Result<String, IdeaError> {
        self.config.validate(limits)?;
        crate::references::validate_references(&self.config, &self.references, limits)?;
        encode(self, limits)
    }

    /// Import the typed config/reference envelope using duplicate-aware bounded decoding.
    ///
    /// # Errors
    /// Rejects unknown fields, invalid config/sites/coordinates and exceeded budgets.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::*;
    /// let compiled = CompiledSchema::from_json(r#"{"config":{},"references":[]}"#,
    ///     ParseLimits::default())?;
    /// assert!(compiled.references.is_empty());
    /// # Ok::<(), IdeaError>(())
    /// ```
    pub fn from_json(source: &str, limits: ParseLimits) -> Result<Self, IdeaError> {
        let result = (|| {
            // Typed envelopes have a closed shape; plain-schema imports never infer references.
            let value = read_json("<json>", source, limits)?;
            let object = object(&value)?;
            fields(object, &["config", "references"])?;
            let config = schema_from_value(required(object, "config")?)?;
            let references: Vec<Reference> =
                serde_json::from_value(required(object, "references")?.clone())
                    .map_err(|_| validation("Invalid typed reference records"))?;
            crate::references::validate_references(&config, &references, limits)?;
            Ok(Self { config, references })
        })();
        result.map_err(|error| locate_json(error, "<json>", source))
    }
}

/// Check metadata values without recursively serializing untrusted caller-built data.
fn validate_attributes(
    attributes: &IndexMap<String, AttributeValue>,
    limits: ParseLimits,
) -> Result<(), IdeaError> {
    if attributes.len() > limits.max_collection() {
        return Err(limit("Attribute collection budget exceeded"));
    }
    for value in attributes.values() {
        if let AttributeValue::Array(values) = value {
            if values.len() > limits.max_collection() {
                return Err(limit("Argument collection budget exceeded"));
            }
            for value in values {
                validate_value(value, limits)?;
            }
        }
    }
    Ok(())
}

/// Bound borrowed value depth, nodes, key/string bytes before clone or serialization.
pub(crate) fn validate_value(root: &Value, limits: ParseLimits) -> Result<(), IdeaError> {
    // Work holds borrowed nodes, so rejecting a hostile tree does not recursively drop it.
    let mut pending = vec![(root, 1usize)];
    let mut bytes = 0usize;
    let mut nodes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        // Check aggregate traversal work before queuing another node's children.
        nodes += 1;
        if nodes > limits.max_nodes() {
            return Err(limit("JSON node budget exceeded"));
        }
        if matches!(value, Value::Array(_) | Value::Object(_))
            && depth > limits.max_depth.min(ParseLimits::HARD_DEPTH)
        {
            return Err(limit("JSON nesting limit exceeded"));
        }
        match value {
            Value::Array(values) => {
                // Width is checked before extending the traversal stack.
                if values.len() > limits.max_collection() {
                    return Err(limit("JSON collection budget exceeded"));
                }
                pending.extend(values.iter().map(|value| (value, depth + 1)));
            }
            Value::Object(values) => {
                // Key bytes contribute to allocation pressure just like string values.
                if values.len() > limits.max_collection() {
                    return Err(limit("JSON collection budget exceeded"));
                }
                for (key, value) in values {
                    bytes = bytes.saturating_add(key.len());
                    pending.push((value, depth + 1));
                }
            }
            Value::String(value) => bytes = bytes.saturating_add(value.len()),
            _ => {}
        }
        if bytes > limits.max_bytes {
            return Err(limit("JSON byte budget exceeded"));
        }
    }
    Ok(())
}

/// Deserialize structure only after a nonrecursive text pass has bounded depth and bytes.
pub(crate) fn read_json(file: &str, source: &str, limits: ParseLimits) -> Result<Value, IdeaError> {
    preflight(source, limits).map_err(|error| error.locate(file, source))?;
    let mut decoder = serde_json::Deserializer::from_str(source);
    let value = UniqueValue::deserialize(&mut decoder)
        .and_then(|value| {
            decoder.end()?;
            Ok(value.0)
        })
        .map_err(|error| {
            // Serde reports byte columns. Convert them to a UTF-8 source boundary once.
            let start = source
                .split_inclusive('\n')
                .take(error.line().saturating_sub(1))
                .map(str::len)
                .sum::<usize>()
                + error.column().saturating_sub(1);
            let code = if error.to_string().contains("duplicate JSON key") {
                ErrorCode::DuplicateName
            } else {
                ErrorCode::ValidationError
            };
            IdeaError::new(code, "Invalid normalized JSON")
                .with_span(start, start)
                .locate(file, source)
        })?;
    validate_value(&value, limits).map_err(|error| error.locate(file, source))?;
    Ok(value)
}

/// Bound nesting before recursive decoding, respecting JSON quoting and escaped quotes.
fn preflight(source: &str, limits: ParseLimits) -> Result<(), IdeaError> {
    if source.len() > limits.max_bytes {
        return Err(limit("JSON byte budget exceeded").with_span(0, 0));
    }
    // This iterative pass runs before Serde allocates any recursive container.
    let mut depth = 0usize;
    let mut index = 0usize;
    let mut nodes = 0usize;
    let mut collections: Vec<usize> = Vec::new();
    let bytes = source.as_bytes();
    while index < bytes.len() {
        if matches!(
            bytes[index],
            b'"' | b'[' | b'{' | b'-' | b'0'..=b'9' | b't' | b'f' | b'n'
        ) {
            nodes += 1;
            if nodes > limits.max_nodes() {
                return Err(limit("JSON node budget exceeded").with_span(index, index));
            }
        }
        match bytes[index] {
            b'"' => {
                index += 1;
                // Escapes are JSON syntax, separate from Idea's intentionally raw strings.
                while index < bytes.len() && bytes[index] != b'"' {
                    if bytes[index] == b'\\' {
                        index += 1;
                    }
                    index += 1;
                }
            }
            b'[' | b'{' => {
                // The hard ceiling cannot be raised by caller-supplied JSON limits.
                depth += 1;
                collections.push(1);
                if depth > limits.max_depth.min(ParseLimits::HARD_DEPTH) {
                    return Err(limit("JSON nesting limit exceeded").with_span(index, index + 1));
                }
            }
            b']' | b'}' => {
                depth = depth.saturating_sub(1);
                collections.pop();
            }
            b',' => {
                // Commas inside strings were skipped; each remaining comma starts an entry.
                if let Some(entries) = collections.last_mut() {
                    *entries += 1;
                    if *entries > limits.max_collection() {
                        return Err(
                            limit("JSON collection budget exceeded").with_span(index, index)
                        );
                    }
                }
            }
            b'-' | b'0'..=b'9' => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && matches!(bytes[index], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
                {
                    index += 1;
                }
                let raw = &source[start..index];
                // JSON's default number visitor otherwise rounds overflowing integers to f64.
                if !raw.contains(['.', 'e', 'E'])
                    && raw.parse::<i64>().is_err()
                    && raw.parse::<u64>().is_err()
                {
                    return Err(IdeaError::new(
                        ErrorCode::InvalidLiteral,
                        "Integer outside 64-bit range",
                    )
                    .with_span(start, index));
                }
                // Decimal/exponent overflow is a literal-range failure, not JSON null data.
                if raw.contains(['.', 'e', 'E'])
                    && raw.parse::<f64>().is_ok_and(|value| !value.is_finite())
                {
                    return Err(
                        IdeaError::new(ErrorCode::InvalidLiteral, "Nonfinite JSON number")
                            .with_span(start, index),
                    );
                }
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    Ok(())
}

/// Ordered value visitor that refuses duplicate keys instead of silently keeping the last.
struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    /// Decode through the duplicate-aware visitor for every nested map and sequence.
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        decoder.deserialize_any(UniqueVisitor)
    }
}

/// Recursively used only after the text-level hard depth check.
struct UniqueVisitor;

impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = UniqueValue;
    /// Describe normalized input without echoing confidential data into errors.
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bounded JSON data")
    }
    /// Preserve JSON null.
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }
    /// Preserve boolean data.
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }
    /// Preserve signed integer precision.
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }
    /// Preserve unsigned integer precision.
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }
    /// Accept only finite JSON numbers.
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(value)
            .map(|value| UniqueValue(Value::Number(value)))
            .ok_or_else(|| E::custom("nonfinite number"))
    }
    /// Own decoded strings only within the already bounded source.
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }
    /// Retain sequence order.
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(UniqueValue(Value::Array(values)))
    }
    /// Retain map order while detecting duplicates at every nesting level.
    fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = entries.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON key"));
            }
            let value = entries.next_value::<UniqueValue>()?;
            values.insert(key, value.0);
        }
        Ok(UniqueValue(Value::Object(values)))
    }
}

/// Validate known sections and convert normalized values without interpreting metadata.
pub(crate) fn schema_from_value(value: &Value) -> Result<SchemaConfig, IdeaError> {
    // Structural sections are closed; plugin/prop/attribute payloads remain open data.
    let root = object(value)?;
    fields(root, &["use", "plugin", "prop", "enum", "type", "model"])?;
    let mut schema = SchemaConfig::default();
    let mut names = HashSet::new();
    for (section, value) in root {
        if section == "use" {
            // Import order is preserved and no source is loaded by this data-only boundary.
            schema.r#use = value
                .as_array()
                .ok_or_else(|| validation("Expected import array"))?
                .iter()
                .map(|value| string(value).map(Into::into))
                .collect::<Result<_, _>>()?;
            continue;
        }
        for (name, value) in object(value)? {
            // Cross-kind name collisions are errors, even if both payloads are valid.
            if !names.insert(name) {
                return Err(IdeaError::new(
                    ErrorCode::DuplicateName,
                    "Duplicate declaration",
                ));
            }
            if section != "plugin" && !is_capital_identifier(name) {
                return Err(validation("Invalid declaration name"));
            }
            match section.as_str() {
                "type" => {
                    // Reuse the same fieldset validation for models and reusable types.
                    schema.r#type.insert(name.clone(), fieldset(name, value)?);
                }
                "model" => {
                    schema.model.insert(name.clone(), fieldset(name, value)?);
                }
                "plugin" | "prop" | "enum" => {
                    // Preserve ordinary data verbatim after validating the member-name grammar.
                    let mut data = IndexMap::new();
                    for (key, value) in object(value)? {
                        if !is_any_identifier(key)
                            || (section == "enum" && !is_upper_identifier(key))
                        {
                            return Err(validation("Invalid member name"));
                        }
                        if section == "enum" && matches!(value, Value::Array(_) | Value::Object(_))
                        {
                            return Err(validation("Enum values must be scalar"));
                        }
                        data.insert(key.clone(), value.clone());
                    }
                    match section.as_str() {
                        "plugin" => {
                            schema.plugin.insert(name.clone(), data);
                        }
                        "prop" => {
                            schema.prop.insert(name.clone(), data);
                        }
                        _ => {
                            schema.r#enum.insert(name.clone(), data);
                        }
                    }
                }
                _ => return Err(validation("Unknown schema section")),
            }
        }
    }
    Ok(schema)
}

/// Validate a model/type's exact normalized fields and ordered unique columns.
fn fieldset(name: &str, value: &Value) -> Result<TypeConfig, IdeaError> {
    // Required normalized fields cannot be defaulted without hiding malformed imports.
    let value = object(value)?;
    fields(value, &["name", "mutable", "attributes", "columns"])?;
    if string(required(value, "name")?)? != name {
        return Err(validation("Declaration name mismatch"));
    }
    let mutable = required(value, "mutable")?
        .as_bool()
        .ok_or_else(|| validation("Expected mutable boolean"))?;
    let attributes = attributes(required(value, "attributes")?)?;
    let mut columns = Vec::new();
    let mut names = HashSet::new();
    for column in required(value, "columns")?
        .as_array()
        .ok_or_else(|| validation("Expected columns array"))?
    {
        // Columns are ordered arrays with unique names; their types are already normalized.
        let column = object(column)?;
        fields(
            column,
            &["name", "type", "required", "multiple", "attributes"],
        )?;
        let name = string(required(column, "name")?)?;
        if !is_camel_identifier(name) {
            return Err(validation("Invalid column name"));
        }
        if !names.insert(name) {
            return Err(IdeaError::new(ErrorCode::DuplicateName, "Duplicate column"));
        }
        let column_type = string(required(column, "type")?)?;
        if !is_capital_identifier(column_type) {
            return Err(validation("Expected unmodified base type"));
        }
        // Preserve caller-supplied flags only after verifying each is a real JSON boolean.
        columns.push(ColumnConfig {
            name: name.into(),
            column_type: column_type.into(),
            attributes: attributes_from_column(column)?,
            required: required(column, "required")?
                .as_bool()
                .ok_or_else(|| validation("Expected required boolean"))?,
            multiple: required(column, "multiple")?
                .as_bool()
                .ok_or_else(|| validation("Expected multiple boolean"))?,
        });
    }
    Ok(TypeConfig {
        name: name.into(),
        mutable,
        attributes,
        columns,
    })
}

/// Extract column metadata without shadowing the parent fieldset's metadata binding.
fn attributes_from_column(
    column: &Map<String, Value>,
) -> Result<IndexMap<String, AttributeValue>, IdeaError> {
    attributes(required(column, "attributes")?)
}

/// Require lowercase metadata namespaces and boolean/argument-array values.
fn attributes(value: &Value) -> Result<IndexMap<String, AttributeValue>, IdeaError> {
    let mut result = IndexMap::new();
    for (name, value) in object(value)? {
        if !valid_attribute(name) {
            return Err(validation("Invalid attribute namespace"));
        }
        // Flags and method arguments are the only normalized metadata shapes.
        let attribute = match value {
            Value::Bool(value) => AttributeValue::Boolean(*value),
            Value::Array(values) => AttributeValue::Array(values.clone()),
            _ => return Err(validation("Expected attribute boolean or argument array")),
        };
        result.insert(name.clone(), attribute);
    }
    Ok(result)
}

/// Require an object at a normalized structural boundary.
fn object(value: &Value) -> Result<&Map<String, Value>, IdeaError> {
    value
        .as_object()
        .ok_or_else(|| validation("Expected normalized object"))
}

/// Require a string without coercing other scalar types.
fn string(value: &Value) -> Result<&str, IdeaError> {
    value.as_str().ok_or_else(|| validation("Expected string"))
}

/// Require a known structural field; metadata objects themselves stay open.
fn required<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a Value, IdeaError> {
    object
        .get(key)
        .ok_or_else(|| IdeaError::new(ErrorCode::MissingProperty, "Missing normalized property"))
}

/// Reject unknown normalized fields rather than silently dropping incorrect input.
fn fields(object: &Map<String, Value>, allowed: &[&str]) -> Result<(), IdeaError> {
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(validation("Unknown normalized property"));
    }
    Ok(())
}

/// Bounded serializer sink that can count bytes without allocating an intermediate string.
struct Sink {
    bytes: Vec<u8>,
    remaining: usize,
    collect: bool,
}
impl Write for Sink {
    /// Charge serialized bytes before extending the output allocation.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.remaining = self
            .remaining
            .checked_sub(bytes.len())
            .ok_or_else(|| io::Error::other("JSON byte budget exceeded"))?;
        if self.collect {
            self.bytes.extend_from_slice(bytes);
        }
        Ok(bytes.len())
    }
    /// There is no external IO to flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Measure already validated, depth-bounded data without allocating serialized output.
pub(crate) fn measure<T: Serialize>(value: &T, maximum: usize) -> Result<usize, IdeaError> {
    let mut sink = Sink {
        bytes: Vec::new(),
        remaining: maximum,
        collect: false,
    };
    serde_json::to_writer(&mut sink, value)
        .map_err(|_| limit("Serialized byte budget exceeded"))?;
    Ok(maximum - sink.remaining)
}

/// Serialize bounded data with an allocation limit enforced by the writer.
pub(crate) fn encode<T: Serialize>(value: &T, limits: ParseLimits) -> Result<String, IdeaError> {
    let mut sink = Sink {
        bytes: Vec::new(),
        remaining: limits.max_bytes,
        collect: true,
    };
    serde_json::to_writer(&mut sink, value)
        .map_err(|_| limit("Serialized byte budget exceeded"))?;
    String::from_utf8(sink.bytes).map_err(|_| validation("JSON serialization failed"))
}

/// Assign source coordinates to structural errors lacking a lexical location.
fn locate_json(error: IdeaError, file: &str, source: &str) -> IdeaError {
    if error.file.is_empty() {
        error.with_span(0, 0).locate(file, source)
    } else {
        error
    }
}

/// Produce a sanitized structural failure without including config or environment data.
pub(crate) fn validation(message: &str) -> IdeaError {
    IdeaError::new(ErrorCode::ValidationError, message)
}

/// Produce a sanitized budget failure shared by checked JSON operations.
fn limit(message: &str) -> IdeaError {
    IdeaError::new(ErrorCode::Limit, message)
}
