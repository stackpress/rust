//! Source-client planning preserves compiled binding/order and portable projection boundaries.
use stackpress_framework::commands::{
    schema::rust::{self, transformer::*},
    source_client::{self, Binding, Options},
};
use stackpress_framework_schema::{Schema, codegen::SourceGraph};
use std::error::Error;

/// Compose exactly the application's maintained four-model extension fixture.
fn config() -> rust::parser::SchemaConfig {
    let mut loader = MemoryLoader::new("/source-client").unwrap();
    loader.insert(
        "schema.idea",
        include_str!("../../consumers/build-host/source.idea"),
    );
    Composer::new(rust::package_loader(loader).unwrap(), Default::default())
        .compose("schema.idea")
        .unwrap()
        .config
}
fn graph(context: &TransformContext<'_>) -> SourceGraph {
    Schema::normalize_source(context.schema(), Default::default(), &[]).unwrap()
}
/// All callbacks are actual compiled function paths, not executable config text.
fn emit(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let mut options = Options::new("source-app", "crate::framework");
    options.session = Some(Binding {
        path: "stackpress_framework_session::composite::register_with".into(),
        feature: None,
    });
    options.api = Some(Binding {
        path: "stackpress_framework_api::register_with".into(),
        feature: Some("api".into()),
    });
    options.admin = Some(Binding {
        path: "super::super::admin".into(),
        feature: Some("ssr".into()),
    });
    source_client::emit(
        &graph(context),
        &stackpress_framework_admin::codegen::source_definitions().unwrap(),
        &options,
        output,
    )
}

#[test]
fn generated_source_client_catalog_and_portable_ui_come_from_one_graph() {
    let mut registry = Registry::new();
    registry
        .add(
            TransformSpec::new("source-client").owns(&["source", "stores", "ui", "client"]),
            emit,
        )
        .unwrap();
    let plan = registry.plan(&config(), Vec::new()).unwrap();
    let client = plan
        .files()
        .find(|(path, _, _)| *path == "client/framework.rs")
        .unwrap()
        .2;
    assert!(client.contains("FrameworkBindings"));
    assert!(client.contains("composite::register_with::<super::super::stores::family::Models>"));
    assert!(client.contains("#[cfg(feature=\"api\")]"));
    assert!(client.contains("#[cfg(feature=\"ssr\")]"));
    // G19 composition retains the app's local Profile first, then imported names in
    // first-encounter order (idea-transformer/tests/compose_rules.rs:620–651).
    let names = [
        "name:\"Profile\"",
        "name:\"Application\"",
        "name:\"Session\"",
        "name:\"Auth\"",
    ];
    let positions = names.map(|name| client.find(name).unwrap());
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    let public = plan
        .files()
        .filter(|(path, _, _)| path.starts_with("ui/"))
        .map(|(_, _, text)| text)
        .collect::<String>();
    assert!(public.contains("Nickname"));
    for private in [
        "@hashed",
        "@encrypted",
        "SourceRecordSchema",
        "SourceEnvironment",
        "stackpress_server",
        "stackpress_sql::",
    ] {
        assert!(!public.contains(private), "{private}");
    }
    assert!(
        plan.files()
            .any(|(path, _, text)| path == "source/profile.rs" && text.contains("nickname"))
    );
}

/// Malformed code paths fail at planning, before any output workspace or Engine exists.
#[test]
fn source_client_rejects_expression_bindings_before_contributing_files() {
    fn invalid(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
        let options = Options::new("app", "crate::framework; panic!()");
        source_client::emit(
            &graph(context),
            &stackpress_framework_admin::codegen::source_definitions().unwrap(),
            &options,
            output,
        )
    }
    let mut registry = Registry::new();
    registry
        .add(
            TransformSpec::new("source-client").owns(&["source", "stores", "ui", "client"]),
            invalid,
        )
        .unwrap();
    assert!(
        registry
            .plan(&config(), Vec::new())
            .unwrap_err()
            .source()
            .unwrap()
            .to_string()
            .contains("Rust path")
    );
}
