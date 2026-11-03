//! Shipped package sources compose without a source checkout or runtime plugin selection.
#![cfg(all(feature = "codegen", not(target_arch = "wasm32")))]

use stackpress_framework::commands::schema::rust::{
    self,
    transformer::{Composer, CompositionLimits, MemoryLoader, SourceLoader},
};

#[test]
fn shipped_catalog_composes_all_models_and_mutable_app_extensions() {
    assert_eq!(
        stackpress_framework::SCHEMA_SOURCE,
        "use \"stackpress-api/schema.idea\""
    );
    let mut app = MemoryLoader::new("/application").unwrap();
    app.insert(
        "nested/schema.idea",
        "use \"stackpress/stackpress.idea\"\nmodel Profile { nickname String? }",
    );
    let loader = rust::package_loader(app).unwrap();
    let source = Composer::new(&loader, CompositionLimits::default())
        .compose("nested/schema.idea")
        .unwrap();
    assert_eq!(source.sources.len(), 4);
    assert_eq!(
        source
            .imports
            .iter()
            .map(|edge| edge.specifier.as_str())
            .collect::<Vec<_>>(),
        [
            "stackpress/stackpress.idea",
            "stackpress-api/schema.idea",
            "stackpress-session/schema.idea"
        ]
    );
    assert_eq!(
        source
            .config
            .model
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["Profile", "Application", "Session", "Auth"],
        "local declarations retain their position under G19"
    );
    let profile = &source.config.model["Profile"];
    assert_eq!(profile.columns.len(), 14);
    assert_eq!(profile.columns.last().unwrap().name, "nickname");
    assert_eq!(profile.columns.first().unwrap().name, "id");
    assert!(profile.attributes.contains_key("labels"));
    assert_eq!(
        loader
            .load(None, "stackpress-session/schema.idea", 10000)
            .unwrap()
            .text,
        stackpress_framework_session::SCHEMA_SOURCE
    );
    assert_eq!(
        loader
            .load(None, "stackpress-api/schema.idea", 10000)
            .unwrap()
            .text,
        stackpress_framework_api::SCHEMA_SOURCE
    );
    assert!(
        loader.load(None, "stackpress/schema.idea", 10000).is_err(),
        "no unapproved aggregate alias"
    );
}

#[test]
fn direct_session_import_and_immutable_redeclaration_follow_g19() {
    for (suffix, has_extra) in [("", true), ("!", false)] {
        let mut app = MemoryLoader::new("/application").unwrap();
        app.insert("schema.idea", format!("use \"stackpress-session/schema.idea\"\nmodel Profile{suffix} {{ nickname String? }}"));
        let source = Composer::new(
            rust::package_loader(app).unwrap(),
            CompositionLimits::default(),
        )
        .compose("schema.idea")
        .unwrap();
        assert_eq!(source.config.model.len(), 2);
        assert_eq!(
            source.config.model["Profile"]
                .columns
                .iter()
                .any(|field| field.name == "nickname"),
            has_extra
        );
        assert!(source.config.model.contains_key("Auth"));
    }
}
