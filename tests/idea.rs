//! Opt-in facade paths compile independently without registering generators implicitly.

/// The parser-only feature exposes the standalone pure source compiler.
#[cfg(feature = "idea-parser")]
#[test]
fn parser_feature_exposes_standalone_parser() {
    // Access through the facade, rather than importing the optional dependency directly.
    let schema = stackpress::idea_parser::parse(
        "facade.idea",
        "model Profile { id String name String }",
        &Default::default(),
    )
    .unwrap();
    assert_eq!(schema.model["Profile"].columns.len(), 2);
}

/// The transformer feature exposes ordinary registry construction without the wrapper feature.
#[cfg(feature = "idea-transformer")]
#[test]
fn transformer_feature_exposes_registry() {
    // Construction has no callbacks, process runtime or filesystem effects.
    let _registry = stackpress::idea_transformer::Registry::new();
}

/// The convenience feature includes both component paths through the wrapper namespace.
#[cfg(feature = "idea")]
#[test]
fn idea_feature_exposes_both_components() {
    // Nested component exports are usable even if the separate facade flags are disabled.
    let schema = stackpress::idea::parser::parse("s", "", &Default::default()).unwrap();
    let _registry = stackpress::idea::transformer::Registry::new();
    assert!(schema.model.is_empty());
}
