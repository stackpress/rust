//! API-local build-input adapters for two shared framework-schema codegen gaps.
//!
//! `codegen/metadata.rs::normalize_model` validates literal query path segments rather
//! than expanding wildcards. `codegen/mod.rs::Schema::check_relations` requires an explicit
//! inverse name instead of deriving the sole matching reverse array. Remove each adapter
//! when that shared behavior supports the original API input; never patch the source Idea,
//! runtime query semantics, generated output, or shared checker here.
use stackpress_idea_parser::{AttributeValue, SchemaConfig};

/// Adapt only the composed API build input; neither adapter acquires runtime resources.
pub(super) fn apply(config: &mut SchemaConfig) {
    expand_query_wildcards(config);
    derive_session_inverse(config);
}

/// Expand stored columns in declaration order, retaining explicit paths and duplicates.
fn expand_query_wildcards(config: &mut SchemaConfig) {
    // The existing typed normalizer requires finite paths. Expand source wildcards
    // against the same composed stored columns; runtime StoreQuery stays unchanged.
    let snapshot = config.clone();
    for model in config.model.values_mut() {
        let Some(AttributeValue::Array(paths)) = model.attributes.get_mut("query") else {
            continue;
        };
        let mut expanded = Vec::new();
        for path in paths.iter().filter_map(serde_json::Value::as_str) {
            let (prefix, model_name) = if let Some((prefix, _)) = path.rsplit_once(".*") {
                let target = snapshot.model[&model.name]
                    .columns
                    .iter()
                    .find(|column| column.name == prefix)
                    .map(|column| column.column_type.as_str());
                (format!("{prefix}."), target.unwrap_or(&model.name))
            } else {
                (String::new(), model.name.as_str())
            };
            if path == "*" || path.ends_with(".*") {
                expanded.extend(
                    snapshot.model[model_name]
                        .columns
                        .iter()
                        .filter(|column| !snapshot.model.contains_key(&column.column_type))
                        .map(|column| {
                            serde_json::Value::String(format!("{prefix}{}", column.name))
                        }),
                );
            } else {
                expanded.push(path.into());
            }
        }
        *paths = expanded;
    }
}

/// Supply the source's singular Application.session inverse to the shared relation checker.
fn derive_session_inverse(config: &mut SchemaConfig) {
    // The native relation checker requires the inverse name that upstream derives
    // from its sole matching reverse array. This is metadata, not a new store join.
    if let Some(column) = config.model.get_mut("Session").and_then(|model| {
        model
            .columns
            .iter_mut()
            .find(|column| column.name == "application")
    }) && let Some(AttributeValue::Array(args)) = column.attributes.get_mut("relation")
        && let Some(serde_json::Value::Object(mapping)) = args.first_mut()
    {
        mapping.insert("name".into(), "session".into());
    }
}
