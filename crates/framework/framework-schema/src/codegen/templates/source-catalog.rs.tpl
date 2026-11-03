//! Generated complete private catalog; no renderer, store, actions or plugin registration.
use stackpress_framework_schema::Model;

/// Complete derived relation/query catalog; raw attributes remain in each model.
pub fn describe() -> stackpress_framework_schema::SourceCatalog {
    stackpress_framework_schema::source::__private::serde_json::from_str([%catalog%]).expect("generator emitted valid source catalog")
}

/// Complete source model inventory in composed declaration order.
pub fn models() -> Vec<Model> {
    vec![
[%#@:models model%]
        super::[%model.module%]::describe(),
[%/@:models%]
    ]
}

/// Unresolved external targets, retained without creating hidden models or tables.
pub const EXTERNAL_MODELS: &[&str] = &[
[%#@:externals name%]
    [%name%],
[%/@:externals%]
];
