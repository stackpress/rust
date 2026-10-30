//! Real existing Idea/schema/SQL registrars and file output, behind an explicit build-host binding.

use super::*;
use stackpress_framework::commands::schema::rust::{self, transformer::*};
use stackpress_framework_schema::{Schema, codegen::*};

/// Normalize one app-owned shape with explicitly approved public field policies.
fn normalized(context: &TransformContext<'_>) -> stackpress_idea::transformer::Result<Schema> {
    let mut definitions = DefinitionRegistry::standard();
    for field in ["id", "title"] {
        definitions = definitions
            .with_field_policy(
                "Note",
                field,
                stackpress_framework_schema::FieldAccess {
                    readable: true,
                    writable: true,
                    sensitive: false,
                    publish_default: false,
                },
            )
            .map_err(transform_error)?;
    }
    Schema::normalize(context.schema(), &definitions, NormalizeOptions::default())
        .map_err(transform_error)
}
/// Invoke the real schema emitter, not a stand-in output string.
fn schema(
    context: &TransformContext<'_>,
    output: &mut OutputPlan,
) -> stackpress_idea::transformer::Result<()> {
    emit_contracts(&normalized(context)?, output)
}
/// Invoke the real generated SQL store emitter over the identical normalized shape.
fn sql(
    context: &TransformContext<'_>,
    output: &mut OutputPlan,
) -> stackpress_idea::transformer::Result<()> {
    stackpress_framework_sql::codegen::emit_stores(&normalized(context)?, output)
}
/// Compiled Leptos view metadata replaces the source Node import transform.
fn view(
    _: &TransformContext<'_>,
    output: &mut OutputPlan,
) -> stackpress_idea::transformer::Result<()> {
    output.write("views/catalog.rs", "//! Compiled finite view catalog.\n")
}
/// Select ordinary real registrars in source schema/SQL/view order.
fn registry() -> stackpress_idea::transformer::Result<Registry> {
    let mut registry = Registry::new();
    rust::register(&mut registry, schema, Some(sql), Some(view))?;
    Ok(registry)
}
/// App owns the root; transform partitions remain independent.
fn aggregate(
    _: &stackpress_idea::parser::SchemaConfig,
) -> stackpress_idea::transformer::Result<Vec<GeneratedFile>> {
    Ok(vec![GeneratedFile::new(
        "application.rs",
        "//! Finite generated application.\n",
    )])
}

/// Full source command inputs reach real generation, repeat install and explicit freshness checking.
#[tokio::test]
async fn real_generation_and_provider_errors() {
    let root = super::sql::Fixture::new();
    std::fs::write(root.0.join("schema.idea"),"plugin \"framework-schema\" {}\nplugin \"framework-sql\" {}\nplugin \"framework-view\" {}\nmodel Note {\n id String @id\n title String\n}\n").unwrap();
    let build = root.0.join("generated");
    let mut app = application(
        json!({"server":{"cwd":root.0},"client":{"tsconfig":"source.json","build":build},"cli":{"idea":root.0.join("schema.idea")}}),
    );
    app.on_config(task(|start: &mut StartupContext| {
        Box::pin(async move {
            let mut bindings = Bindings::default();
            bindings.bind("generate", rust::provider);
            start.register(Arc::new(bindings))?;
            start.register(Arc::new(rust::Host {
                registry,
                aggregates: aggregate,
                mode: rust::GenerationMode::Install,
            }))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    let host = app.build().await.unwrap();
    assert!(!build.exists());
    for _ in 0..2 {
        assert_eq!(
            host.resolve("generate", json!({}))
                .await
                .unwrap()
                .response
                .code,
            Some(200)
        );
    }
    assert!(build.join("contracts/note.rs").exists());
    assert!(build.join("stores/note.rs").exists());
    assert!(build.join("views/catalog.rs").exists());
    let entry = root.0.join("schema.idea");
    let request = rust::GenerateRequest::new(
        entry.to_str().unwrap(),
        aggregate,
        rust::GenerationMode::Check,
    );
    assert!(
        rust::generate(
            request,
            ProjectLoader::new(&root.0).unwrap(),
            &registry().unwrap(),
            &OutputWorkspace::new(&build).unwrap()
        )
        .unwrap()
        .is_current()
    );
    assert!(
        host.resolve("generate", json!({"i":"missing.idea"}))
            .await
            .is_err()
    );
    let mut duplicate = registry().unwrap();
    assert!(rust::register(&mut duplicate, schema, None, None).is_err());
}
