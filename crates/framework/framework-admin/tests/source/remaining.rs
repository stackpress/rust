//! Every formerly unavailable canonical placement normalizes, emits and renders its real adapter.
use super::*;
use leptos::prelude::*;
use serde_json::{Value, json};
use stackpress_framework_admin::{RoleContext, source::roles::*};
use stackpress_framework_schema::Input;

/// One realistic source value and type for each capability family.
fn carrier(name: &str) -> (&'static str, Value) {
    match name {
        "field.small" => ("Integer", json!(3)),
        "field.datelist" | "field.datetimelist" | "field.numberlist" | "field.timelist"
        | "field.filelist" | "field.imagelist" => ("String[]", json!(["a", "b"])),
        "view.comma" | "view.line" | "view.spread" | "view.list" | "view.ol" | "view.ul"
        | "view.carousel" | "view.film" => ("String[]", json!(["a", "b"])),
        "view.tabular" => ("Object[]", json!([{"first":"one"},{"second":"two"}])),
        "field.fieldset" | "view.fieldset" => ("Detail", json!({"text":"nested"})),
        "view.rating" | "view.formula" => ("Number", json!(2.5)),
        "field.color" | "view.color" => ("String", json!("rebeccapurple")),
        "field.country" => ("String", json!("PH")),
        "view.country" => ("String", json!("PHL")),
        "field.currency" | "view.currency" => ("String", json!("PHP")),
        "view.rel" | "view.relative" => ("String", json!("2026-11-01T00:00:00Z")),
        _ => ("String", json!("source <b>text</b>")),
    }
}

/// Compile actual component APIs rather than treating import strings as rendering evidence.
fn render_component(symbol: &str, context: RoleContext) -> AnyView {
    match symbol{
    "InputField"=>view!{<InputField context/>}.into_any(),
    "ColorField"=>view!{<ColorField context/>}.into_any(),
    "CountryField"=>view!{<CountryField context/>}.into_any(),
    "FileField"=>view!{<FileField context/>}.into_any(),
    "RadioField"=>view!{<RadioField context/>}.into_any(),
    "SmallField"=>view!{<SmallField context/>}.into_any(),
    "SuggestField"=>view!{<SuggestField context/>}.into_any(),
    "SourceTextListField"=>view!{<SourceTextListField context/>}.into_any(),
    "RichField"=>view!{<RichField context/>}.into_any(),
    "OverflowView"=>view!{<OverflowView context/>}.into_any(),
    "SpreadView"=>view!{<SpreadView context/>}.into_any(),
    "CountryView"=>view!{<CountryView context/>}.into_any(),
    "EmailView"=>view!{<EmailView context/>}.into_any(),
    "FormulaView"=>view!{<FormulaView context row=json!({"price":3})/>}.into_any(),
    "RelativeView"=>view!{<RelativeView context/>}.into_any(),
    "TransformView"=>view!{<TransformView context/>}.into_any(),
    "MediaView"=>view!{<MediaView context/>}.into_any(),
    "RichView"=>view!{<RichView context/>}.into_any(),
    "ColorView"=>view!{<ColorView context/>}.into_any(),
    "RatingView"=>view!{<RatingView context/>}.into_any(),
    "ListView"=>view!{<ListView context/>}.into_any(),
    "TabularView"=>view!{<TabularView context/>}.into_any(),
    "FieldsetField"=>view!{<FieldsetField context fields=vec![] render=Callback::new(|_: (stackpress_framework_admin::Draft,String)|().into_any())/>}.into_any(),
    "FieldsetView"=>view!{<FieldsetView context fields=vec![] render=Callback::new(|_: (Value,String)|vec![])/>}.into_any(),
    _=>panic!("unproved source import {symbol}"),
}
}

/// All 108 former placement/name gaps have available compiled imports and real SSR paths.
#[test]
fn source_all_44_remaining_roles_emit_and_render_in_every_placement() {
    let definitions = source_definitions().unwrap();
    let mut count = 0;
    for (family, names, placements) in [
        (
            "field",
            "code color country currency email file image json markdown mask radio small suggest datelist datetimelist filelist imagelist numberlist timelist fieldset",
            vec!["field", "filter", "span"],
        ),
        (
            "view",
            "chars comma country currency email formula line relative transform words carousel code color film json markdown rating list ol spread tabular ul fieldset rel",
            vec!["view", "list"],
        ),
    ] {
        for name in names.split_whitespace() {
            let canonical = format!("{family}.{name}");
            let (kind, seed) = carrier(&canonical);
            for placement in &placements {
                let args = if family == "view" && name == "code" {
                    "({language \"rust\"})"
                } else if name == "mask" {
                    "({mask \"99-99\"})"
                } else if name == "formula" {
                    "({formula \"{this}+{price}\"})"
                } else {
                    ""
                };
                let source = format!(
                    "type Detail {{ text String? @field.input @span.input @filter.input @view.text @list.text }}\nmodel Proof {{ id String @id value {kind}? @{placement}.{name}{args} }}"
                );
                let config =
                    stackpress_idea_parser::parse("remaining.idea", &source, &Default::default())
                        .unwrap();
                let graph = Schema::normalize_source(&config, Default::default(), &[])
                    .unwrap_or_else(|error| {
                        panic!("normalize {placement}.{name}: {error:?}; config={config:?}")
                    });
                let ui = graph
                    .project_source_ui(&definitions)
                    .unwrap_or_else(|error| panic!("project {placement}.{name}: {error:?}"));
                let field = ui.models[0]
                    .fields
                    .iter()
                    .find(|field| field.wire_name == "value")
                    .unwrap()
                    .clone();
                let binding = field.roles.first().unwrap();
                assert!(binding.available, "{placement}.{name}");
                let import = definitions
                    .imports()
                    .iter()
                    .find(|import| import.component == binding.component)
                    .unwrap();
                assert_eq!(import.module, "stackpress_framework_admin::source::roles");
                let mut registry = Registry::new();
                registry
                    .add(TransformSpec::new("remaining").owns(&["ui"]), emit)
                    .unwrap();
                let plan = registry.plan(&config, vec![]).unwrap();
                let generated = plan
                    .files()
                    .find(|(path, _, _)| *path == "ui/proof.rs")
                    .unwrap()
                    .2;
                assert!(
                    generated.contains(&format!("::{}", import.symbol)),
                    "{placement}.{name}"
                );
                let role = binding.role;
                let owner = Owner::new();
                let html = owner.with(|| {
                    let context = RoleContext {
                        id: "remaining".into(),
                        field,
                        role,
                        value: RwSignal::new(Input::Value(seed.clone())),
                    };
                    render_component(&import.symbol, context).to_html()
                });
                owner.cleanup();
                assert!(!html.contains("unavailable"), "{placement}.{name}: {html}");
                count += 1;
            }
        }
    }
    assert_eq!(count, 108);
}

/// Build a binding through the public source projection, keeping registry internals private.
fn selected(
    name: &str,
) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::RoleBinding> {
    let config = stackpress_idea_parser::parse(
        "fixed.idea",
        &format!("model Proof {{ value String? @{name} }}"),
        &Default::default(),
    )
    .unwrap();
    let graph = Schema::normalize_source(&config, Default::default(), &[])?;
    let ui = graph.project_source_ui(&source_definitions()?)?;
    Ok(ui.models[0].fields[0].roles[0].clone())
}

/// Required authored mask is diagnosed, and fixed values remain source-authoritative.
#[test]
fn source_role_fixed_attributes_and_required_mask_keep_their_contract() {
    assert!(selected("field.mask").is_err());
    for (name, key, expected) in [
        ("field.email", "type", json!("email")),
        ("field.json", "language", json!("json")),
        ("field.json", "setup", json!("basic")),
        ("field.small", "max", json!(9)),
        ("field.datelist", "type", json!("date")),
        ("view.words", "words", json!(true)),
        ("view.ol", "ordered", json!(true)),
        ("view.comma", "separator", json!(", ")),
    ] {
        let binding = selected(name).unwrap();
        assert_eq!(binding.props.get(key), Some(&expected), "{name}");
    }
}

/// G55 uses the established Rust text conversion for structured JSON, with no pretty-print pass.
#[test]
fn source_json_structured_values_use_existing_compact_text_conversion() {
    let config = stackpress_idea_parser::parse(
        "json.idea",
        "model Proof { value Object? @field.json @view.json }",
        &Default::default(),
    )
    .unwrap();
    let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&source_definitions().unwrap())
        .unwrap();
    let field = ui.models[0].fields[0].clone();
    for role in [Role::Field, Role::View] {
        let owner = Owner::new();
        let html = owner.with(|| {
            let context = RoleContext {
                id: "json".into(),
                field: field.clone(),
                role,
                value: RwSignal::new(Input::Value(json!({"key":"value"}))),
            };
            if role == Role::Field {
                view! {<RichField context/>}.to_html()
            } else {
                view! {<RichView context/>}.to_html()
            }
        });
        assert!(html.contains("key"));
        assert!(!html.contains("[object Object]"));
        assert!(!html.contains("\n  "));
        owner.cleanup();
    }
}

/// Frui renders IEEE arithmetic results as text; parse errors remain the original numeric zero.
#[test]
fn source_formula_display_keeps_javascript_infinity_nan_and_error_spelling() {
    for (formula, expected) in [
        ("1/0", "Infinity"),
        ("-1/0", "-Infinity"),
        ("0/0", "NaN"),
        ("(1+)", "0"),
    ] {
        let config = stackpress_idea_parser::parse(
            "formula.idea",
            &format!("model Proof {{ value Number? @view.formula({{formula \"{formula}\"}}) }}"),
            &Default::default(),
        )
        .unwrap();
        let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
        let field = graph
            .project_source_ui(&source_definitions().unwrap())
            .unwrap()
            .models[0]
            .fields[0]
            .clone();
        let owner = Owner::new();
        let html = owner.with(|| {
            let context = RoleContext {
                id: "formula".into(),
                field,
                role: Role::View,
                value: RwSignal::new(Input::Value(json!(0))),
            };
            view! {<FormulaView context/>}.to_html()
        });
        owner.cleanup();
        assert_eq!(html, expected, "{formula}");
    }
}
