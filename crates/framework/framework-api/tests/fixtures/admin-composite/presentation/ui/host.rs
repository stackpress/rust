//! Generated source host catalog; the application selects concrete route/event registrars.
/// Native callbacks traverse this catalog once in source order, with admin registered last.
pub const MODELS: &[(&str, &str)] = &[("catalog", "Catalog"), ("slugged", "Slugged")];

/// Only source-authored labels/icons are published; no native model or codec metadata.
pub fn presentation() -> stackpress_framework_schema::__private::serde_json::Value {
    stackpress_framework_schema::__private::serde_json::json!({
        "catalog":{"label":"Catalog","plural":"Catalog","icon":null},
        "slugged":{"label":"Slugged","plural":"Slugged","icon":null},
    })
}
