//! Source SSR boundary cases for appearance, native names and nested fieldsets.
#![cfg(all(feature = "codegen", feature = "source-ssr"))]
use leptos::prelude::*;
use serde_json::{Value, json};
use stackpress_framework_admin::source::roles::{
    FieldsetField, FieldsetView, InputField, fieldset_context,
};
use stackpress_framework_admin::{Draft, RoleContext, codegen::source_definitions};
use stackpress_framework_schema::{Input, Role, Schema, UiField};

/// Normalize real authored declarations before rendering their public field descriptors.
fn fields(source: &str) -> (Vec<UiField>, Vec<UiField>) {
    let parsed =
        stackpress_idea_parser::parse("publication.idea", source, &Default::default()).unwrap();
    let graph = Schema::normalize_source(&parsed, Default::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&source_definitions().unwrap())
        .unwrap();
    (
        ui.models[0].fields.clone(),
        ui.fieldsets
            .first()
            .map(|model| model.fields.clone())
            .unwrap_or_default(),
    )
}

/// Numeric CSS respects unitless properties, zero and custom variables in escaped native output.
#[test]
fn authored_appearance_and_filter_names_survive_ssr() {
    let (mut fields, _) = fields("model Proof { values String[] @field.input @filter.input }");
    let style = json!({"marginTop":4,"opacity":0.5,"zIndex":2,"width":0,"--custom":3,"backgroundColor":"red"});
    for role in [Role::Field, Role::Filter] {
        let field = &mut fields[0];
        let binding = field
            .roles
            .iter_mut()
            .find(|binding| binding.role == role)
            .unwrap();
        binding.props.insert("style".into(), style.clone());
        binding.props.insert("className".into(), json!("caller"));
        let owner = Owner::new();
        let html = owner.with(|| {
            let context = RoleContext {
                id: "appearance".into(),
                field: field.clone(),
                role,
                value: RwSignal::new(Input::Missing),
            };
            view! {<InputField context/>}.to_html()
        });
        assert!(html.contains("caller"), "{html}");
        for declaration in [
            "margin-top:4px",
            "opacity:0.5",
            "z-index:2",
            "width:0",
            "--custom:3",
            "background-color:red",
        ] {
            assert!(html.contains(declaration), "missing {declaration}: {html}");
        }
        let name = if role == Role::Filter {
            "eq[values][]"
        } else {
            "values[]"
        };
        assert!(html.contains(&format!("name=\"{name}\"")), "{html}");
        owner.cleanup();
    }
}

/// Populated source fieldsets invoke real child controls with row-specific successful names.
#[test]
fn populated_fieldsets_render_child_drafts_and_numbered_names() {
    let (fields, children) = fields(
        "type Detail { label String @default(\"fallback\") @field.input } model Proof { details Detail[] @field.fieldset }",
    );
    let owner = Owner::new();
    let html = owner.with(|| {
        let context = RoleContext {
            id: "nested".into(),
            field: fields[0].clone(),
            role: Role::Field,
            value: RwSignal::new(Input::Value(
                json!([{"label":"first"},null,{"label":"third"}]),
            )),
        };
        let child_fields = children.clone();
        let render = Callback::new(move |(draft, id): (Draft, String)| {
            let field = child_fields[0].clone();
            let value = draft.get("label").unwrap();
            let context = fieldset_context(RoleContext {
                id,
                field,
                role: Role::Field,
                value,
            });
            view! {<InputField context/>}.into_any()
        });
        view! {<FieldsetField context fields=children.clone() render/>}.to_html()
    });
    assert!(html.contains("name=\"details[0][label]\""), "{html}");
    assert!(html.contains("name=\"details[2][label]\""), "{html}");
    assert!(html.contains("value=\"first\""));
    assert!(html.contains("value=\"third\""));
    assert!(!html.contains("details[1][label]"));
    assert!(html.contains("Remove row 3"));
    owner.cleanup();
}

/// Required singular fieldsets initialize declared public defaults while optional ones stay empty.
#[test]
fn required_singular_fieldset_uses_child_defaults() {
    let (fields, children) = fields(
        "type Detail { label String @default(\"fallback\") @field.input } model Proof { detail Detail @field.fieldset }",
    );
    let owner = Owner::new();
    let html = owner.with(|| {
        let context = RoleContext {
            id: "singular".into(),
            field: fields[0].clone(),
            role: Role::Field,
            value: RwSignal::new(Input::Missing),
        };
        let child_fields = children.clone();
        let render = Callback::new(move |(draft, id): (Draft, String)| {
            let field = child_fields[0].clone();
            let context = fieldset_context(RoleContext {
                id,
                value: draft.get("label").unwrap(),
                field,
                role: Role::Field,
            });
            view! {<InputField context/>}.into_any()
        });
        view! {<FieldsetField context fields=children.clone() render/>}.to_html()
    });
    assert!(html.contains("name=\"detail[label]\""), "{html}");
    assert!(html.contains("value=\"fallback\""));
    assert!(!html.contains("Remove row"));
    owner.cleanup();
}

/// Fieldset display retains row cardinality and applies source truthiness only to optional cells.
#[test]
fn fieldset_display_preserves_required_and_optional_cells() {
    let (fields, children) = fields(
        "type Detail { required String @view.text optional String? @view.text ignored String? } model Proof { details Detail[] @view.fieldset }",
    );
    let owner = Owner::new();
    let html = owner.with(|| {
        let context = RoleContext {
            id: "display".into(),
            field: fields[0].clone(),
            role: Role::View,
            value: RwSignal::new(Input::Value(
                json!([{"required":"","optional":"visible"},{"required":"second","optional":""}]),
            )),
        };
        let render = Callback::new(|(row, id): (Value, String)| {
            vec![
                view! {<b data-row=id>{row["required"].as_str().unwrap_or("").to_owned()}</b>}
                    .into_any(),
                view! {<i>{row["optional"].as_str().unwrap_or("").to_owned()}</i>}.into_any(),
            ]
        });
        view! {<FieldsetView context fields=children.clone() render/>}.to_html()
    });
    assert!(html.contains("data-row=\"display-0\""), "{html}");
    assert!(html.contains("<i>visible</i>"));
    assert!(!html.contains("<i></i>"));
    assert!(html.contains("second"));
    assert!(!html.contains("ignored"));
    owner.cleanup();
}

/// Nonempty media/link values retain authored dimensions and destinations; tags retain source text.
#[test]
fn media_links_and_tags_preserve_source_rendering() {
    use stackpress_framework_admin::source::roles::{ImageView, LinkView, TagsView};
    let (mut fields, _) =
        fields("model Proof { value String @view.image({ width 24 height 32 }) }");
    let owner = Owner::new();
    let html = owner.with(|| {
        let field = &mut fields[0];
        let props = &mut field.roles[0].props;
        props.insert("target".into(), json!("_blank"));
        props.insert("warning".into(), json!(true));
        props.insert("pill".into(), json!(true));
        let context = RoleContext {
            id: "media".into(),
            field: field.clone(),
            role: Role::View,
            value: RwSignal::new(Input::Value(json!("/image?a=1&b=2"))),
        };
        let tags = RoleContext {
            value: RwSignal::new(Input::Value(json!(["label", 17, false]))),
            ..context.clone()
        };
        view! {<ImageView context=context.clone()/><LinkView context/><TagsView context=tags/>}
            .to_html()
    });
    for expected in [
        "width=\"24\"",
        "height=\"32\"",
        "target=\"_blank\"",
        "api-tag-warning",
        "api-tag-pill",
        ">label</span>",
        ">17</span>",
        ">false</span>",
    ] {
        assert!(html.contains(expected), "missing {expected}: {html}");
    }
    owner.cleanup();
}

/// Joined templates substitute nested data as escaped text and keep relation fallback on missing paths.
#[test]
fn joined_templates_use_data_paths_and_fallback_without_evaluation() {
    use stackpress_framework_admin::source::roles::TemplateView;
    let (mut fields, _) = fields("model Proof { value String @view.template }");
    for (template, expected) in [
        ("Hello {{ nested.name }} {{count}}!", "Hello Ada 2!"),
        ("{{missing.path}}", "fallback"),
        ("{{unclosed", "fallback"),
    ] {
        fields[0].roles[0]
            .props
            .insert("template".into(), json!(template));
        let owner = Owner::new();
        let html = owner.with(|| {
            let context = RoleContext {
                id: "template".into(),
                field: fields[0].clone(),
                role: Role::View,
                value: RwSignal::new(Input::Value(json!("fallback"))),
            };
            view! {<TemplateView context row=json!({"nested":{"name":"Ada"},"count":2})/>}.to_html()
        });
        assert!(html.contains(expected), "{html}");
        owner.cleanup();
    }
}

/// Array metadata uses source Object.entries indexes and String values without changing the draft.
#[test]
fn metadata_arrays_and_relation_selection_keep_original_drafts() {
    use stackpress_framework_admin::source::roles::{MetadataField, RelationField};
    let (relation_fields, _) = fields("model Proof { value String @field.relation }");
    let (fields, _) = fields("model Proof { value Hash @field.metadata }");
    let owner = Owner::new();
    owner.with(|| {
        let original = Input::Value(json!([null,true,2,["a",null,"b"],{"x":1}]));
        let draft = RwSignal::new(original.clone());
        let context = RoleContext {
            id: "array-metadata".into(),
            field: fields[0].clone(),
            role: Role::Field,
            value: draft,
        };
        let html = view! {<MetadataField context/>}.to_html();
        for text in ["null", "true", "2", "a,,b", "[object Object]"] {
            assert!(html.contains(&format!("value=\"{text}\"")), "{html}");
        }
        assert_eq!(draft.get_untracked(), original);
        // A selected relation omitted by the available choices remains a real selected choice.
        let selected = RwSignal::new(Input::Value(json!("omitted-choice")));
        let context = RoleContext {
            id: "relation".into(),
            field: relation_fields[0].clone(),
            role: Role::Field,
            value: selected,
        };
        let html = view! {<RelationField context/>}.to_html();
        assert!(html.contains("omitted-choice"), "{html}");
        assert_eq!(
            selected.get_untracked(),
            Input::Value(json!("omitted-choice"))
        );
    });
    owner.cleanup();
}

/// Source switches retain native field/filter names and duplicate relation choices report an alert.
#[test]
fn switch_names_and_relation_choice_refusal_do_not_mutate_drafts() {
    use stackpress_framework_admin::source::roles::{
        Choice, RelationField, Relations, SwitchField,
    };
    let (switch_fields, _) = fields("model Proof { enabled Boolean @field.switch @filter.switch }");
    let (relation_fields, _) = fields("model Proof { value String @field.relation }");
    for role in [Role::Field, Role::Filter] {
        let owner = Owner::new();
        owner.with(|| {
            let original = Input::Value(json!(true));
            let draft = RwSignal::new(original.clone());
            let context = RoleContext {
                id: "source-switch".into(),
                field: switch_fields[0].clone(),
                role,
                value: draft,
            };
            let html = view! {<SwitchField context/>}.to_html();
            let name = if role == Role::Filter {
                "eq[enabled]"
            } else {
                "enabled"
            };
            assert!(html.contains(&format!("name=\"{name}\"")), "{html}");
            assert_eq!(draft.get_untracked(), original);
        });
        owner.cleanup();
    }
    let owner = Owner::new();
    owner.with(|| {
        let choice = Choice {
            id: "duplicate".into(),
            label: "Choice".into(),
            row: json!({"id":"duplicate"}),
        };
        provide_context(Relations(std::collections::BTreeMap::from([(
            "value".into(),
            vec![choice.clone(), choice],
        )])));
        let draft = RwSignal::new(Input::Missing);
        let context = RoleContext {
            id: "duplicate-relation".into(),
            field: relation_fields[0].clone(),
            role: Role::Field,
            value: draft,
        };
        let html = view! {<RelationField context/>}.to_html();
        assert!(
            html.contains("Relation choices exceed the UI representation budget"),
            "{html}"
        );
        assert_eq!(draft.get_untracked(), Input::Missing);
    });
    owner.cleanup();
}

/// Empty optional tag input stays empty, and disabled adornments do not add warning/pill classes.
#[test]
fn optional_and_plain_tags_preserve_unadorned_source_output() {
    use stackpress_framework_admin::source::roles::TagsView;
    let (mut fields, _) = fields("model Proof { values String[] @view.tags }");
    fields[0].roles[0]
        .props
        .insert("warning".into(), json!(false));
    fields[0].roles[0].props.insert("pill".into(), json!(false));
    for value in [Input::Missing, Input::Value(json!(["first"]))] {
        let has_tag = matches!(&value, Input::Value(_));
        let owner = Owner::new();
        let html = owner.with(|| {
            let draft = RwSignal::new(value.clone());
            let context = RoleContext {
                id: "plain-tags".into(),
                field: fields[0].clone(),
                role: Role::View,
                value: draft,
            };
            let html = view! {<TagsView context/>}.to_html();
            assert_eq!(draft.get_untracked(), value);
            html
        });
        assert_eq!(html.contains(">first</span>"), has_tag, "{html}");
        assert!(!html.contains("api-tag-warning"), "{html}");
        assert!(!html.contains("api-tag-pill"), "{html}");
        owner.cleanup();
    }
}
