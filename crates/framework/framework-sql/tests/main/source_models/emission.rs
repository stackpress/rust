//! A minimal structural emitter proves trait implementations; production source emission stays gated.

use super::schema;
use stackpress_framework_schema::{Schema, SchemaError, SchemaErrorKind, codegen::*};
use stackpress_idea_transformer::{OutputPlan, Registry, TransformContext, TransformSpec};
use stackpress_lib::{TemplateEngine, TemplateOptions};
use std::{error::Error, path::Path};

/// Put primitive type names, encoded metadata and role choices into actual Rust structure.
fn emit_probe(schema: &Schema, semantics: EmissionSemantics) -> String {
    assert_eq!(semantics, EmissionSemantics::Source);
    let models: Vec<_> = schema
        .models()
        .iter()
        .map(|model| {
            let id = model
                .fields
                .iter()
                .find(|field| field.name == "id")
                .unwrap();
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
                "id_type": rust_type(schema, id).unwrap(),
                "json": metadata,
                "role": role,
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
        &serde_json::json!({"models":models,"api":true}),
    )
    .unwrap()
}

/// Publish disposable source only when the outer recorded proof runner requests it.
#[test]
fn structural_source_emitter_default_and_extended_family() {
    for (extended, name) in [(false, "default"), (true, "app")] {
        let schema = schema(extended);
        let source = emit_probe(&schema, EmissionSemantics::Source);
        assert!(source.contains("impl SessionModels for Family"));
        assert!(source.contains("impl ApiModels for Family"));
        assert!(source.contains("type Id = String"));
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
    // Semantic normalization of external relations is explicitly M1 work, not hidden tables.
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
