//! Real revision-2 catalog normalization, source diagnostics and SSR adapter admission.
#![cfg(all(feature = "codegen", feature = "ssr"))]
use leptos::prelude::*;
use serde_json::json;
use stackpress_framework_admin::{
    Draft, RoleContext,
    codegen::{emit_roles, register_components},
    components::*,
};
use stackpress_framework_schema::{
    FieldAccess, Role, Schema, UiSchema,
    codegen::{DefinitionRegistry, DiagnosticMap},
};
use stackpress_idea_transformer::{
    Composer, MemoryLoader, OutputPlan, Registry, TransformContext, TransformInputs,
    TransformSelection, TransformSpec,
};

/// Exercise the real Idea composition and app-owned public policy before UI projection.
fn fixture(source: &str) -> (UiSchema, DiagnosticMap) {
    let mut loader = MemoryLoader::new("/catalog").unwrap();
    loader.insert("fixture.idea", source);
    let composed = Composer::new(loader, Default::default())
        .compose("fixture.idea")
        .unwrap();
    let diagnostics = DiagnosticMap::from_composed(&composed).unwrap();
    let mut registry = DefinitionRegistry::standard();
    for field in [
        "id", "rows", "tags", "metadata", "range", "rating", "slug", "choice",
    ] {
        registry = registry
            .with_field_policy(
                "Example",
                field,
                FieldAccess {
                    readable: true,
                    writable: true,
                    publish_default: true,
                    ..Default::default()
                },
            )
            .unwrap();
    }
    let registry = register_components(registry, vec![]).unwrap();
    (
        Schema::normalize_with_diagnostics(
            &composed.config,
            &registry,
            Default::default(),
            &diagnostics,
        )
        .unwrap()
        .project_ui()
        .unwrap(),
        diagnostics,
    )
}

/// A regular function transform owns UI output; no alternate output writer is used.
fn transform(
    context: &TransformContext<'_>,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let ui: UiSchema = serde_json::from_str(context.input("ui")?).unwrap();
    let diagnostics: DiagnosticMap = serde_json::from_str(context.input("diagnostics")?).unwrap();
    emit_roles(
        &ui,
        &register_components(DefinitionRegistry::standard(), vec![]).unwrap(),
        &diagnostics,
        output,
    )
}

/// The catalog is deterministic and new scalar/collection props are checked at source sites.
#[test]
fn new_catalog_roles_and_invalid_configuration_diagnostics() {
    let valid = "enum Choice { A \"a\" B \"b\" } model Example { id String @id rows String[] @field.textlist @span.strings tags String[] @field.taglist metadata Object @field.metadata range Number @field.slider({ min -1 max 1 step 0.5 }) rating Integer @field.rating({ max 10 }) slug String @field.slug choice Choice @field.select({ searchable true }) @list.text }";
    let generate = |source: &str| {
        let (ui, diagnostics) = fixture(source);
        let mut inputs = TransformInputs::new();
        inputs
            .insert("ui", serde_json::to_string(&ui).unwrap())
            .unwrap();
        inputs
            .insert("diagnostics", serde_json::to_string(&diagnostics).unwrap())
            .unwrap();
        let mut registry = Registry::new();
        registry
            .add(TransformSpec::new("ui").owns(&["ui"]), transform)
            .unwrap();
        registry.plan_with(
            &Default::default(),
            &TransformSelection::new().with_inputs(inputs),
            vec![],
        )
    };
    let first = generate(valid).unwrap();
    let second = generate(valid).unwrap();
    assert_eq!(
        first.files().collect::<Vec<_>>(),
        second.files().collect::<Vec<_>>()
    );
    let source = first
        .files()
        .find(|(path, _, _)| *path == "ui/example.rs")
        .unwrap()
        .2;
    for symbol in [
        "TextListField",
        "MetadataField",
        "SliderField",
        "RatingField",
        "SlugField",
        "SelectField",
        "pub fn table(",
    ] {
        assert!(source.contains(symbol));
    }
    for declaration in [
        "rows String @field.tags",
        "metadata String @field.metadata",
        "range Integer @field.slider({ step 0.5 })",
        "range Number @field.slider({ min 2 max 1 })",
        "range Number @field.slider({ step 0 })",
        "rating Number @field.rating",
        "rating Integer @field.rating({ max 101 })",
        "slug Boolean @field.slug",
        "choice Choice @field.select({ searchable true placeholder \"ignored\" })",
    ] {
        let original = format!(
            "// 🦀\nenum Choice {{ A \"a\" B \"b\" }}\nmodel Example {{ id String @id {declaration} }}"
        );
        let error = generate(&original).unwrap_err();
        let mut cause: &dyn std::error::Error = &error;
        let schema = loop {
            if let Some(error) = cause.downcast_ref::<stackpress_framework_schema::SchemaError>() {
                break error;
            }
            cause = cause.source().unwrap();
        };
        let (start, end) = schema.location().unwrap().bytes.unwrap();
        assert!(original[start..end].starts_with("@field."));
    }
    #[cfg(feature = "format")]
    for declaration in [
        "range Number @field.number({ scale -1 })",
        "range Integer @field.integer({ scale 2 })",
        "range Number @field.integer",
        "range Number @field.number({ digits 0 })",
        "range Number @field.number({ locale \"ambient\" })",
        "slug String @field.date",
        "slug Date @field.time",
        "slug Date @field.date({ precision 1 })",
        "slug Datetime @field.datetime({ precision 10 })",
        "slug String @field.phone({ region \"invalid\" })",
    ] {
        let original = format!("// 🦀\nmodel Example {{ id String @id {declaration} }}");
        let error = generate(&original).unwrap_err();
        let mut cause: &dyn std::error::Error = &error;
        let schema = loop {
            if let Some(error) = cause.downcast_ref::<stackpress_framework_schema::SchemaError>() {
                break error;
            }
            cause = cause.source().unwrap();
        };
        let (start, end) = schema.location().unwrap().bytes.unwrap();
        assert!(original[start..end].starts_with("@field."));
    }
    for role in [
        "field.number",
        "field.date",
        "field.phone",
        "field.editor",
        "field.file",
        "field.relation",
        "field.suggest",
    ] {
        if cfg!(feature = "format") && matches!(role, "field.number" | "field.date" | "field.phone")
        {
            continue;
        }
        let source = format!("model Example {{ id String @id slug String @{role} }}");
        // Unsupported vocabulary remains represented but cannot emit a compiled wrapper.
        let (ui, _) = fixture(&source);
        assert!(
            !ui.models[0]
                .fields
                .iter()
                .find(|field| field.wire_name == "slug")
                .unwrap()
                .roles[0]
                .available
        );
        assert!(generate(&source).is_err());
    }
}

/// SSR admits exact values, escapes labels and refuses incompatible collection seeds.
#[test]
fn new_native_controls_have_escaped_labels_and_exact_seeds() {
    let (ui, _) = fixture(
        "enum Choice { A \"a\" B \"b\" } model Example { id String @id rows String[]? @label(\"</script> 🦀\") @field.strings tags String[] @field.tags({ tags false }) metadata Object @field.metadata range Integer @field.slider rating Integer? @field.rating slug String @field.slug choice Choice @field.select({ searchable true }) }",
    );
    let owner = Owner::new();
    owner.with(||{
        let fields=&ui.models[0].fields;let draft=Draft::new(fields,&json!({"rows":["", "0", "</script>"],"tags":[],"metadata":{"":"","zero":"0"},"range":0,"rating":0,"slug":"A New Slug","choice":"a"}),Role::Field).unwrap();
        let context=|name:&str|RoleContext{id:format!("catalog-{name}"),field:fields.iter().find(|field|field.wire_name==name).unwrap().clone(),role:Role::Field,value:draft.get(name).unwrap()};
        let rows=context("rows");let tags=context("tags");let metadata=context("metadata");let range=context("range");let rating=context("rating");let slug=context("slug");let choice=context("choice");
        let html=view!{<TextListField context=rows/><TextListField context=tags/><MetadataField context=metadata/><SliderField context=range/><RatingField context=rating/><SlugField context=slug/><SelectField context=choice/>}.to_html();
        assert!(html.contains("sp-tag-list"),"fixed tag mode wins over a supplied false prop");
        assert!(!html.contains("<script>"));assert!(!html.contains("</script>"));assert!(html.contains("&lt;/script&gt;"));
        for marker in ["<legend>","type=\"range\"","type=\"radio\"","Apply slug","role=\"combobox\"","catalog-rows-1-value"] {assert!(html.contains(marker),"{marker}");}
        assert_eq!(draft.snapshot().unwrap()["range"],0);assert_eq!(draft.snapshot().unwrap()["rating"],0);
        for (name,value) in [("rows",json!([false])),("metadata",json!({"x":0})),("range",json!(200)),("rating",json!(6)),("choice",json!("unknown"))] {
            draft.get(name).unwrap().set(stackpress_framework_schema::Input::Value(value));
            let context=context(name);let html=match name {"rows"=>view!{<TextListField context/>}.to_html(),"metadata"=>view!{<MetadataField context/>}.to_html(),"range"=>view!{<SliderField context/>}.to_html(),"rating"=>view!{<RatingField context/>}.to_html(),_=>view!{<ComboboxField context/>}.to_html()};
            assert!(html.contains("incompatible"));
        }
    });
    owner.cleanup();
}
