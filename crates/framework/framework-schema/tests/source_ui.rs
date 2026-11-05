//! Source props are original presentation data, not Typed role arguments or executable imports.
#![cfg(feature = "codegen")]
use stackpress_framework_schema::{
    Role, Schema,
    codegen::{NormalizeOptions, source_definitions},
};

#[test]
fn source_original_blog_editor_and_select_props_normalize_exactly() {
    // templates/blog/schema.idea:184-200,233. style=true is also the original Frui
    // toolbar flag, despite the attribute dictionary's CSS-object description.
    let source = r#"
enum PublishStatus { DRAFT "DRAFT" REVIEW "REVIEW" PUBLISHED "PUBLISHED" ARCHIVED "ARCHIVED" }
model Article {
 contents Text? @field.editor({ history true font true size true format true paragraph true blockquote true style true color true highlight true text true textStyle true align true list true code true })
 status PublishStatus @field.select({ options [ "DRAFT" "REVIEW" "PUBLISHED" "ARCHIVED" ] })
}
"#;
    let config =
        stackpress_idea_parser::parse("blog-ui.idea", source, &Default::default()).unwrap();
    let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&source_definitions().unwrap())
        .unwrap();
    let editor = &ui.models[0].fields[0].roles[0];
    assert_eq!(editor.role, Role::Field);
    assert_eq!(editor.props.len(), 14);
    assert!(
        editor
            .props
            .values()
            .all(|value| value.as_bool() == Some(true))
    );
    assert!(
        !editor.available,
        "normalization does not invent a rich editor"
    );
    let select = &ui.models[0].fields[1].roles[0];
    assert_eq!(
        select.props["options"],
        serde_json::json!(["DRAFT", "REVIEW", "PUBLISHED", "ARCHIVED"])
    );
}
