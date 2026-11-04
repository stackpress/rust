//! Derive finite relation/query metadata without changing exact winning source declarations.
use super::{SourceGraph, invalid};
use crate::{Field, Model, SchemaResult, SourceCatalog, SourceQuery, SourceRelation, ValueType};

impl SourceGraph {
    /// Build the complete private catalog, including inferred inverse and wildcard metadata.
    ///
    /// This never resolves a virtual external target into a hidden model or table. Source
    /// attributes retain their exact original values; derivations occupy separate fields.
    /// # Errors
    /// Rejects a query path that cannot resolve to a declared scalar member.
    pub fn catalog(&self) -> SchemaResult<SourceCatalog> {
        let mut relations = Vec::new();
        let mut queries = Vec::new();
        // Catalog order is composed model order, then each declaration's own field order.
        for model in self.models() {
            for field in &model.fields {
                if let ValueType::Model(target) = &field.value_type {
                    relations.push(relation(self, model, field, target));
                }
            }
            queries.push(query(self, model)?);
        }
        Ok(SourceCatalog {
            tables: self.tables.clone(),
            types: self.types.clone(),
            fieldsets: self.fieldsets().to_vec(),
            enums: self.enums().to_vec(),
            models: self.models().to_vec(),
            external_models: self.external_models().to_vec(),
            relations,
            queries,
        })
    }
}

/// Derive one edge from the validated graph, retaining unresolved external mappings.
fn relation(
    graph: &SourceGraph,
    model: &Model,
    field: &Field,
    target_name: &str,
) -> SourceRelation {
    let target = graph
        .models()
        .iter()
        .find(|target| target.name == target_name);
    let inverse = target.and_then(|target| {
        target.fields.iter().find(|other| {
            // The validated inverse has the opposite ownership direction, not just its type.
            other.value_type == ValueType::Model(model.name.clone())
                && field.relation.is_some() != other.relation.is_some()
        })
    });
    // Owning mappings stay exact. Reverse mappings swap the sole owning edge; declared
    // externals keep their members unresolved instead of inventing keys or hidden tables.
    let mapping = field
        .relation
        .as_ref()
        .map(|mapping| (mapping.local.clone(), mapping.foreign.clone()))
        .or_else(|| {
            inverse?
                .relation
                .as_ref()
                .map(|mapping| (mapping.foreign.clone(), mapping.local.clone()))
        });
    SourceRelation {
        model: model.name.clone(),
        field: field.name.clone(),
        target: target_name.into(),
        multiple: field.multiple,
        local: mapping.as_ref().map(|mapping| mapping.0.clone()),
        foreign: mapping.map(|mapping| mapping.1),
        inverse: inverse.map(|field| field.name.clone()),
        external: target.is_none(),
    }
}

/// Preserve selectors independently of expansion so wildcard source spelling remains visible.
fn query(graph: &SourceGraph, model: &Model) -> SchemaResult<SourceQuery> {
    let selectors = if model.query.is_empty() {
        vec!["*".into()]
    } else {
        model.query.clone()
    };
    let mut paths = Vec::new();
    for selector in &selectors {
        expand(graph, model, selector, &mut paths)?;
    }
    Ok(SourceQuery {
        model: model.name.clone(),
        selectors,
        paths,
    })
}

/// Expand only the selected path; cycles never trigger recursive eager graph traversal.
fn expand<'a>(
    graph: &'a SourceGraph,
    mut model: &'a Model,
    selector: &str,
    paths: &mut Vec<String>,
) -> SchemaResult<()> {
    let mut prefix = String::new();
    let segments: Vec<_> = selector
        .split(':')
        .next()
        .unwrap_or(selector)
        .split('.')
        .collect();
    for (index, segment) in segments.iter().enumerate() {
        // Wildcards select stored members in declaration order, excluding virtual edges.
        if *segment == "*" {
            paths.extend(
                model
                    .fields
                    .iter()
                    .filter(|field| !matches!(field.value_type, ValueType::Model(_)))
                    .map(|field| format!("{prefix}{}", field.name)),
            );
            break;
        }
        let field = model
            .fields
            .iter()
            .find(|field| field.name == *segment)
            .ok_or_else(|| invalid("unknown source query member"))?;
        if index + 1 == segments.len() {
            // A terminal relation is retained as the original selected expression. The SQL
            // store owns its source path/column behavior, including any driver failure.
            paths.push(if selector.contains(':') {
                selector.into()
            } else {
                format!("{prefix}{}", field.name)
            });
        } else {
            // A relation lookup consumes one bounded selector segment, not another query.
            let (ValueType::Model(name) | ValueType::Fieldset(name)) = &field.value_type else {
                return Err(invalid("source query cannot traverse scalar"));
            };
            model = graph
                .models()
                .iter()
                .chain(graph.fieldsets())
                .find(|model| model.name == *name)
                .ok_or_else(|| invalid("source query target is external"))?;
            prefix.push_str(segment);
            prefix.push('.');
        }
    }
    Ok(())
}
