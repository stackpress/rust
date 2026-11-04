//! Minimal M0 contract probes plus independent production source emission checks.

use super::schema;
use stackpress_framework_schema::{Schema, SchemaError, SchemaErrorKind, codegen::*};
use stackpress_idea_transformer::{OutputPlan, Registry, TransformContext, TransformSpec};
use stackpress_lib::{TemplateEngine, TemplateOptions};
use std::{error::Error, path::Path};

/// Put primitive type names, encoded metadata and role choices into actual Rust structure.
pub(super) fn emit_probe(schema: &SourceGraph, semantics: EmissionSemantics) -> String {
    assert_eq!(semantics, EmissionSemantics::Source);
    let models: Vec<_> = schema
        .models()
        .iter()
        .map(|model| {
            // The probe supports these roles only; an unknown role must not become a default.
            let role = match model.name.as_str() {
                "Profile" => "ProfileRole",
                "Auth" => "AuthRole",
                "Application" => "ApplicationRole",
                "Session" => "ApiSessionRole",
                _ => panic!("unsupported probe role"),
            };
            // Encode metadata as one Rust string literal; templates never evaluate its contents.
            let metadata = format!("{:?}", serde_json::to_string(model).unwrap());
            serde_json::json!({
                "name": model.rust_name,
                "json": metadata,
                "role": role,
                "witnesses":emit_source_witnesses(model).unwrap(),
            })
        })
        .collect();
    TemplateEngine::new(TemplateOptions {
        delimiters: ["[%".into(), "%]".into()],
        ..Default::default()
    })
    .unwrap()
    .render(
        include_str!("model.rs.tpl"),
        &serde_json::json!({"models":models,"api":schema.models().iter().any(|m|m.name=="Application"),"family":schema.models()[0].fields.len()>5}),
    )
    .unwrap()
}

/// Publish disposable source only when the outer recorded proof runner requests it.
#[test]
fn structural_source_emitter_default_and_extended_family() {
    for (extended, name) in [(false, "default"), (true, "app")] {
        let schema = super::source_schema(extended);
        let source = emit_probe(&schema, EmissionSemantics::Source);
        assert!(source.contains("pub type Profile = support::Model"));
        assert!(source.contains("pub type Session = support::Model"));
        assert!(source.contains("impl SessionModels for Family"));
        assert!(source.contains("type FieldValue = String"));
        assert_eq!(source.contains("nickname"), extended);
        assert_eq!(source, emit_probe(&schema, EmissionSemantics::Source));
        if let Ok(root) = std::env::var("G50_PROBE_OUT") {
            let out = Path::new(&root).join(name);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(out.join("generated.rs"), source).unwrap();
        }
    }
}

/// Raw winning source composition retains external names; no stub/API model is synthesized.
#[test]
fn session_only_external_reverse_links_remain_declarations() {
    use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader, PackageLoader};
    let mut app = MemoryLoader::new("/app").unwrap();
    app.insert(
        "schema.idea",
        "use \"stackpress-session/schema.idea\"\nmodel Profile { nickname String? }",
    );
    let loader = PackageLoader::new(app)
        .with_source(
            "stackpress-session/schema.idea",
            include_str!("../../../../framework-session/schema.idea"),
        )
        .unwrap();
    let graph = Composer::new(loader, CompositionLimits::default())
        .compose("schema.idea")
        .unwrap();
    assert_eq!(
        graph
            .config
            .model
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["Profile", "Auth"]
    );
    // Serialize only for inspection: original external type declarations remain in the AST.
    let profile = serde_json::to_value(&graph.config.model["Profile"]).unwrap();
    let text = profile.to_string();
    assert!(text.contains("Application"));
    assert!(text.contains("Session"));
    assert!(!graph.config.model.contains_key("Application"));
    assert!(!graph.config.model.contains_key("Session"));
    let source = Schema::normalize_source(
        &graph.config,
        NormalizeOptions::default(),
        &["Application", "Session"],
    )
    .unwrap();
    assert_eq!(source.models().len(), 2);
    let emitted = emit_probe(&source, EmissionSemantics::Source);
    assert!(emitted.contains("impl SessionModels for Family"));
    assert!(!emitted.contains("impl ApiModels for Family"));
    // Production schema/store emission uses the same external graph, not the M0 probe template.
    let mut registry = Registry::new();
    registry
        .add(
            TransformSpec::new("native").owns(&["source", "stores"]),
            |context: &TransformContext<'_>, plan: &mut OutputPlan| {
                let source = Schema::normalize_source(
                    context.schema(),
                    NormalizeOptions::default(),
                    &["Application", "Session"],
                )
                .unwrap();
                emit_source_contracts(&source, plan)?;
                stackpress_framework_sql::codegen::emit_source_stores(&source, plan)
            },
        )
        .unwrap();
    let plan = registry.plan(&graph.config, vec![]).unwrap();
    let paths = plan
        .files()
        .map(|(path, _, _)| path.to_owned())
        .collect::<Vec<_>>();
    assert!(paths.iter().any(|path| path == "stores/profile.rs"));
    assert!(
        !paths
            .iter()
            .any(|path| path.contains("application.rs") || path.contains("session.rs"))
    );
    let temporary = tempfile::tempdir().unwrap();
    // macOS aliases /var through /private/var; the output writer requires real ancestors.
    let temporary_root = temporary.path().canonicalize().unwrap();
    let destination = std::env::var_os("G50_NATIVE_PROBE_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| temporary_root.join("generated"));
    stackpress_idea_transformer::OutputWorkspace::new(destination)
        .unwrap()
        .install(plan)
        .unwrap();

    if let Ok(root) = std::env::var("G50_PROBE_OUT") {
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(Path::new(&root).join("session-only.rs"), emitted).unwrap();
    }
}

/// Legacy and explicit Typed emission must be byte-identical for all owned partitions.
fn typed(
    context: &TransformContext<'_>,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let _ = context;
    emit_contracts_with_semantics(&schema(false), output, EmissionSemantics::Typed)?;
    stackpress_framework_sql::codegen::emit_stores_with_semantics(
        &schema(false),
        output,
        EmissionSemantics::Typed,
    )
}
/// Existing public entry points remain the unchanged Typed default.
fn legacy(
    context: &TransformContext<'_>,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let _ = context;
    emit_contracts(&schema(false), output)?;
    stackpress_framework_sql::codegen::emit_stores(&schema(false), output)
}
/// Source refuses before mutating the plan and retains Unsupported beneath the typed cause.
fn source(
    context: &TransformContext<'_>,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let _ = context;
    let before = output.files().count();
    let a = emit_contracts_with_semantics(&schema(false), output, EmissionSemantics::Source)
        .unwrap_err();
    let b = stackpress_framework_sql::codegen::emit_stores_with_semantics(
        &schema(false),
        output,
        EmissionSemantics::Source,
    )
    .unwrap_err();
    for error in [a, b] {
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<SchemaError>()
                .unwrap()
                .kind(),
            SchemaErrorKind::Unsupported
        );
    }
    assert_eq!(output.files().count(), before);
    Ok(())
}

#[test]
fn compiled_emission_selection_preserves_typed_and_refuses_partial_source() {
    let snapshot = |callback| {
        let mut registry = Registry::new();
        registry
            .add(
                TransformSpec::new("probe").owns(&["contracts", "metadata", "stores"]),
                callback,
            )
            .unwrap();
        let plan = registry.plan(&Default::default(), vec![]).unwrap();
        plan.files()
            .map(|(path, _, source)| (path.to_owned(), source.to_owned()))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    assert_eq!(EmissionSemantics::default(), EmissionSemantics::Typed);
    assert_eq!(snapshot(legacy), snapshot(typed));
    assert!(snapshot(source).is_empty());
}
