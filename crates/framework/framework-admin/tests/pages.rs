//! Page generation preserves public identities and native/pure separation.
#![cfg(feature = "codegen")]
use stackpress_framework_admin::codegen::{emit_pages, emit_roles, register_components};
use stackpress_framework_schema::{
    FieldAccess, Schema,
    codegen::{DefinitionRegistry, DiagnosticMap, NormalizeOptions},
};
use stackpress_idea_transformer::{
    Composer, MemoryLoader, OutputPlan, Registry, TransformContext, TransformInputs,
    TransformSelection, TransformSpec,
};

/// Use actual Idea source, policies and normalization; test output remains unpublished.
fn fixture(source: &str, public: bool) -> (Schema, DefinitionRegistry, DiagnosticMap) {
    let mut loader = MemoryLoader::new("/pages").unwrap();
    loader.insert("fixture.idea", source);
    let composed = Composer::new(loader, Default::default())
        .compose("fixture.idea")
        .unwrap();
    let diagnostics = DiagnosticMap::from_composed(&composed).unwrap();
    let mut definitions = DefinitionRegistry::standard();
    for model in ["Profile", "PublicPublic", "Schema", "Operations"] {
        for field in ["id", "name"] {
            definitions = definitions
                .with_field_policy(
                    model,
                    field,
                    FieldAccess {
                        readable: public,
                        writable: public,
                        sensitive: false,
                        publish_default: true,
                    },
                )
                .unwrap();
        }
    }
    let definitions = register_components(definitions, Vec::new()).unwrap();
    let schema = Schema::normalize_with_diagnostics(
        &composed.config,
        &definitions,
        NormalizeOptions::default(),
        &diagnostics,
    )
    .unwrap();
    (schema, definitions, diagnostics)
}

/// A complete public key is required; repeated selection cannot duplicate dispatch or registration.
#[test]
fn invalid_page_selection_cannot_publish() {
    for public in [true, false] {
        let (schema, definitions, _) =
            fixture("model Profile { id String @id name String }", public);
        let mut output = OutputPlan::default();
        let selected = if public {
            vec!["Profile", "Profile"]
        } else {
            vec!["Profile"]
        };
        assert!(emit_pages(&schema, &definitions, &selected, &mut output).is_err());
        assert_eq!(output.files().count(), 0);
    }
}

/// Reserved native aggregate modules and repeated Public prefixes retain concrete model identities.
#[test]
fn deterministic_pages_keep_native_registration_separate() {
    let source = "model PublicPublic { id String @id name String @field.input } model Schema { id String @id name String } model Operations { id String @id name String }";
    fn transform(
        context: &TransformContext<'_>,
        output: &mut OutputPlan,
    ) -> stackpress_idea_transformer::Result<()> {
        let (schema, definitions, diagnostics) = fixture(context.input("source")?, true);
        emit_roles(
            &schema.project_ui().unwrap(),
            &definitions,
            &diagnostics,
            output,
        )?;
        emit_pages(
            &schema,
            &definitions,
            &["PublicPublic", "Schema", "Operations", "Profile"],
            output,
        )
    }
    let generate = || {
        let mut inputs = TransformInputs::new();
        inputs.insert("source", source).unwrap();
        let mut registry = Registry::new();
        registry
            .add(TransformSpec::new("ui").owns(&["ui"]), transform)
            .unwrap();
        registry
            .plan_with(
                &Default::default(),
                &TransformSelection::new().with_inputs(inputs),
                vec![],
            )
            .unwrap()
    };
    let first = generate();
    let second = generate();
    assert_eq!(
        first.files().collect::<Vec<_>>(),
        second.files().collect::<Vec<_>>()
    );
    let pure = first
        .files()
        .find(|(path, _, _)| *path == "ui/pages.rs")
        .unwrap()
        .2;
    let host = first
        .files()
        .find(|(path, _, _)| *path == "ui/pages_host.rs")
        .unwrap()
        .2;
    assert!(pure.contains("PublicCreatePublicPublic"));
    assert!(!pure.contains("stackpress_server"));
    assert!(!pure.contains("stackpress_framework_sql"));
    assert!(host.contains("operations::_schema::register"));
    assert!(host.contains("operations::_operations::register"));
    assert!(!host.contains("profile/search"));
    assert!(!host.contains("Operation::Restore"));
}
