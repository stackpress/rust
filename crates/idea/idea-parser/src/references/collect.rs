//! Structural reference discovery over checked syntax, before composition moves declarations.
//!
//! This visitor only records sites and prior-declaration provenance. Resolution belongs to
//! the parent module, so literal strings cannot accidentally become reference instructions.

use crate::ast::{DataToken, Statement};
use crate::{
    Ast, DeclarationKind, ErrorCode, IdeaError, ParseLimits, Reference, SourceSpan, ValuePath,
};
use indexmap::IndexMap;

/// Collect source-ordered reference sites under one bounded metadata-copy budget.
pub(crate) fn collect(ast: &Ast, limits: ParseLimits) -> Result<Vec<Reference>, IdeaError> {
    let mut collector = ReferenceCollector::new(ast, limits);
    // Original statement positions establish forward visibility across declaration kinds.
    for (index, item) in ast.syntax.body.iter().enumerate() {
        if let Statement::Declaration(declaration) = item {
            collector.collect_declaration(index, declaration)?;
        }
    }
    Ok(collector.references)
}

/// Per-source reference collection state; every owned result spends the same byte budget.
struct ReferenceCollector<'a> {
    /// Original source and checked syntax, borrowed throughout collection.
    ast: &'a Ast,
    /// Local declaration positions used to mark forward/self references.
    declarations: IndexMap<&'a str, usize>,
    /// Source-ordered structural sites ready for composition.
    references: Vec<Reference>,
    /// Bytes still available for repeated owner, source and path metadata.
    remaining: usize,
}

/// Borrowed structural owner shared by every nested argument within one metadata object.
#[derive(Clone, Copy)]
struct ReferenceOwner<'a> {
    /// Type or model namespace, unchanged when the declaration moves between files.
    kind: DeclarationKind,
    /// Globally unique name of the declaration containing the argument.
    declaration: &'a str,
    /// Named column, or None for declaration metadata.
    column: Option<&'a str>,
    /// Original order used to enforce prior-declaration visibility.
    index: usize,
}

impl<'a> ReferenceCollector<'a> {
    /// Index original declarations without treating import directives as reference targets.
    fn new(ast: &'a Ast, limits: ParseLimits) -> Self {
        let declarations = ast
            .syntax
            .body
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                if let Statement::Declaration(declaration) = item {
                    Some((declaration.declarations[0].id.name.as_str(), index))
                } else {
                    None
                }
            })
            .collect();
        Self {
            ast,
            declarations,
            references: Vec::new(),
            remaining: limits.max_bytes,
        }
    }

    /// Select only fieldsets, then visit metadata before columns in original normalized order.
    fn collect_declaration(
        &mut self,
        index: usize,
        declaration: &crate::DeclarationToken,
    ) -> Result<(), IdeaError> {
        let kind = match declaration.kind {
            "type" => DeclarationKind::Type,
            "model" => DeclarationKind::Model,
            _ => return Ok(()),
        };
        let value = &declaration.declarations[0];
        let owner = ReferenceOwner {
            kind,
            declaration: &value.id.name,
            column: None,
            index,
        };
        for property in &value.init.properties {
            let DataToken::Object(object) = &property.value else {
                continue;
            };
            match property.key.name.as_str() {
                "attributes" => self.collect_attributes(object, owner)?,
                "columns" => self.collect_columns(object, owner)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// Attach the stable column name before inspecting its normalized metadata member.
    fn collect_columns(
        &mut self,
        columns: &crate::ObjectToken,
        owner: ReferenceOwner<'_>,
    ) -> Result<(), IdeaError> {
        for column in &columns.properties {
            let DataToken::Object(fields) = &column.value else {
                continue;
            };
            let owner = ReferenceOwner {
                column: Some(&column.key.name),
                ..owner
            };
            for field in &fields.properties {
                // Ignore normalized type data; only the metadata object can contain sites.
                if field.key.name == "attributes"
                    && let DataToken::Object(attributes) = &field.value
                {
                    self.collect_attributes(attributes, owner)?;
                }
            }
        }
        Ok(())
    }

    /// Walk each argument iteratively while preserving source order and literal distinction.
    fn collect_attributes(
        &mut self,
        attributes: &crate::ObjectToken,
        owner: ReferenceOwner<'_>,
    ) -> Result<(), IdeaError> {
        for attribute in &attributes.properties {
            let mut pending = vec![(&attribute.value, Vec::new())];
            while let Some((data, path)) = pending.pop() {
                match data {
                    DataToken::Identifier(target) => self.record(owner, attribute, target, path)?,
                    DataToken::Array(array) => {
                        // Reverse stack insertion restores source order on removal.
                        for (index, data) in array.elements.iter().enumerate().rev() {
                            let mut child = path.clone();
                            child.push(ValuePath::Index(index));
                            pending.push((data, child));
                        }
                    }
                    DataToken::Object(object) => {
                        for property in object.properties.iter().rev() {
                            let mut child = path.clone();
                            child.push(ValuePath::Key(property.key.name.clone()));
                            pending.push((&property.value, child));
                        }
                    }
                    DataToken::Literal(_) | DataToken::Environment(_) => {}
                }
            }
        }
        Ok(())
    }

    /// Charge repeated metadata before creating an owned structural reference record.
    fn record(
        &mut self,
        owner: ReferenceOwner<'_>,
        attribute: &crate::PropertyToken,
        target: &crate::IdentifierToken,
        path: Vec<ValuePath>,
    ) -> Result<(), IdeaError> {
        let path_bytes: usize = path
            .iter()
            .map(|step| match step {
                ValuePath::Key(key) => key.len() + 16,
                ValuePath::Index(_) => 24,
            })
            .sum();
        // Preserve the established accounting constants and subtraction/error precedence.
        let bytes = owner
            .declaration
            .len()
            .saturating_add(owner.column.map_or(0, str::len))
            .saturating_add(attribute.key.name.len())
            .saturating_add(target.name.len())
            .saturating_add(self.ast.source.name().len())
            .saturating_add(path_bytes)
            .saturating_add(256);
        self.remaining = self.remaining.checked_sub(bytes).ok_or_else(|| {
            IdeaError::new(ErrorCode::Limit, "Reference metadata budget exceeded")
                .with_span(target.start, target.end)
                .locate(self.ast.source.name(), self.ast.source.text())
        })?;
        self.references.push(Reference {
            kind: owner.kind,
            declaration: owner.declaration.into(),
            column: owner.column.map(Into::into),
            attribute: attribute.key.name.clone(),
            path,
            target: target.name.clone(),
            span: SourceSpan::new(&self.ast.source, target.start, target.end),
            forward: self
                .declarations
                .get(target.name.as_str())
                .is_some_and(|order| *order >= owner.index),
        });
        Ok(())
    }
}
