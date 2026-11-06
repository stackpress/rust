//! Editor/HTML source metadata reaches compiled role imports and shared SSR components.
use super::*;
use leptos::prelude::*;
use stackpress_framework_admin::{
    RoleContext,
    source::roles::{EditorField, HtmlView},
};
use stackpress_framework_schema::Input;

/// Source HTML has no Typed null-label presentation policy.
#[test]
fn source_editor_and_html_null_and_missing_content_stay_empty() {
    let config = stackpress_idea_parser::parse(
        "optional-html.idea",
        "model Article { contents Text? @field.editor @view.html }",
        &Default::default(),
    )
    .unwrap();
    let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&source_definitions().unwrap())
        .unwrap();
    let field = ui.models[0].fields[0].clone();
    for value in [
        Input::Missing,
        Input::Null,
        Input::Value(serde_json::Value::Null),
        Input::Value(serde_json::json!("")),
    ] {
        for role in [Role::Field, Role::View] {
            let owner = Owner::new();
            let html = owner.with(|| {
                let context = RoleContext {
                    id: "empty-html".into(),
                    field: field.clone(),
                    role,
                    value: RwSignal::new(value.clone()),
                };
                if role == Role::Field {
                    view! {<EditorField context/>}.to_html()
                } else {
                    view! {<HtmlView context/>}.to_html()
                }
            });
            if role == Role::Field {
                assert!(html.contains("name=\"contents\""));
                assert!(html.contains("value=\"\""));
            } else {
                assert!(html.contains("<div></div>"));
            }
            assert!(!html.contains('—'));
            owner.cleanup();
        }
    }
}

/// Use the blog's actual role shape without consulting the other example branch.
fn editor_config() -> stackpress_idea_parser::SchemaConfig {
    stackpress_idea_parser::parse("editor.idea", r#"model Article {
        id String @id
        contents Text @field.editor({
            history true font true size true format true paragraph true blockquote true
            style true color true highlight true text true textStyle true align true list true code true
        }) @view.html @list.html
    }"#, &Default::default()).unwrap()
}

/// Formerly unavailable selections now emit checked source-role imports and fresh files.
#[test]
fn editor_and_html_source_roles_emit_with_the_blog_props() {
    let config = editor_config();
    let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
    let definitions = source_definitions().unwrap();
    let ui = graph.project_source_ui(&definitions).unwrap();
    let field = ui.models[0]
        .fields
        .iter()
        .find(|field| field.wire_name == "contents")
        .unwrap();
    assert_eq!(field.roles.len(), 3);
    assert!(field.roles.iter().all(|role| role.available));
    let editor = field
        .roles
        .iter()
        .find(|role| role.role == Role::Field)
        .unwrap();
    assert_eq!(editor.props.len(), 14);
    assert!(
        editor
            .props
            .values()
            .all(|value| value.as_bool() == Some(true))
    );
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("source-ui").owns(&["ui"]), emit)
        .unwrap();
    let plan = registry.plan(&config, Vec::new()).unwrap();
    let source = plan
        .files()
        .find(|(path, _, _)| *path == "ui/article.rs")
        .unwrap()
        .2;
    assert!(source.contains("stackpress_framework_admin::source::roles::EditorField"));
    assert!(source.contains("stackpress_framework_admin::source::roles::HtmlView"));
    let temporary = tempfile::tempdir_in("/private/tmp").unwrap();
    let workspace =
        stackpress_idea_transformer::OutputWorkspace::new(temporary.path().join("generated"))
            .unwrap();
    workspace.install(plan).unwrap();
    assert!(
        workspace
            .check(&registry.plan(&config, Vec::new()).unwrap())
            .unwrap()
            .is_current()
    );
}

/// No editor option disappears at the schema-to-shared-component boundary.
#[test]
fn complete_editor_named_props_and_source_truthiness_render() {
    let config = stackpress_idea_parser::parse(
        "all-editor.idea",
        r#"model Article { contents Text @field.editor({
        history true
        font true
        size true
        format true
        paragraph true
        blockquote true
        style true
        color true
        highlight true
        text true
        textStyle true
        remove true
        indent true
        align true
        rule true
        list true
        lineheight true
        table true
        link true
        image true
        imageGallery true
        video true
        audio true
        math true
        fullscreen true
        showblocks true
        code true
        preview true
        print true
        save true
        template true
        dir true className "hidden-source-class" name "authored-name" id "hidden-id"
    }) @view.html }"#,
        &Default::default(),
    )
    .unwrap();
    let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&source_definitions().unwrap())
        .unwrap();
    let field = ui.models[0].fields[0].clone();
    let owner = Owner::new();
    owner.with(|| {
        let draft = RwSignal::new(Input::Value(serde_json::json!("<p><b>stored</b></p>")));
        let context = RoleContext {
            id: "visible-id".into(),
            field: field.clone(),
            role: Role::Field,
            value: draft,
        };
        let html = view! {<EditorField context/>}.to_html();
        for label in [
            "Undo",
            "Bold",
            "Insert Template",
            "Insert from Gallery",
            "Insert Audio",
            "Toggle Text Direction",
        ] {
            assert!(html.contains(&format!("aria-label=\"{label}\"")), "{label}");
        }
        assert!(html.contains("name=\"authored-name\""));
        assert!(html.contains("id=\"hidden-id\""));
        assert!(html.contains("id=\"visible-id\""));
        assert!(html.contains("class=\"hidden-source-class\""));
        assert!(!html.contains("dir=\"true\""));
        assert!(html.contains("<p><b>stored</b></p>"));
        let context = RoleContext {
            id: "html".into(),
            field,
            role: Role::View,
            value: draft,
        };
        let html = view! {<HtmlView context/>}.to_html();
        assert!(html.contains("<div><p><b>stored</b></p></div>"));
        assert_eq!(
            draft.get_untracked(),
            Input::Value(serde_json::json!("<p><b>stored</b></p>"))
        );
    });
    owner.cleanup();
}

/// Dictionary CSS-object style is truthy to Frui; false preserves an empty default toolbar.
#[test]
fn editor_source_object_style_and_false_flags_keep_frui_semantics() {
    for (style, bold) in [
        ("false", false),
        ("{}", true),
        ("{ marginTop \"10px\" }", true),
    ] {
        let input = format!(
            "model Article {{ contents Text @field.editor({{style {style} history false dir false}}) }}"
        );
        let config =
            stackpress_idea_parser::parse("style.idea", &input, &Default::default()).unwrap();
        let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
        let ui = graph
            .project_source_ui(&source_definitions().unwrap())
            .unwrap();
        let field = ui.models[0].fields[0].clone();
        let owner = Owner::new();
        let html = owner.with(|| {
            let context = RoleContext {
                id: "editor".into(),
                field,
                role: Role::Field,
                value: RwSignal::new(Input::Missing),
            };
            view! {<EditorField context/>}.to_html()
        });
        assert_eq!(html.contains("aria-label=\"Bold\""), bold);
        assert!(!html.contains("aria-label=\"Undo\""));
        assert!(!html.contains("aria-label=\"Toggle Text Direction\""));
        owner.cleanup();
    }
}
