//! Compare the API's optional URL fields before replacing their source-specific adapter.
use leptos::prelude::*;
use serde_json::json;
use stackpress_framework_admin::{
    RoleContext, codegen::register_components, components::TextField,
};
use stackpress_framework_api::{codegen, views::roles::InputField};
use stackpress_framework_schema::{
    FieldAccess, Input, Role, RoleBinding, Schema, codegen::DefinitionRegistry,
};

/// Obtain the shared URL binding through its actual registry and normalizer.
fn shared_url_binding() -> RoleBinding {
    let config = stackpress_idea_parser::parse(
        "url-comparison.idea",
        "model Example { id String @id site String? @field.url }",
        &Default::default(),
    )
    .unwrap();
    let registry = DefinitionRegistry::standard()
        .with_field_policy(
            "Example",
            "site",
            FieldAccess {
                readable: true,
                writable: true,
                ..Default::default()
            },
        )
        .unwrap();
    let registry = register_components(registry, vec![]).unwrap();
    Schema::normalize(&config, &registry, Default::default())
        .unwrap()
        .project_ui()
        .unwrap()
        .models
        .remove(0)
        .fields
        .remove(0)
        .roles
        .remove(0)
}

/// Read an escaped native attribute without depending on unrelated wrapper markup.
fn attribute<'a>(html: &'a str, name: &str) -> &'a str {
    let input = html
        .split("<input")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    input
        .split_once(&format!(" {name}=\""))
        .unwrap_or_else(|| panic!("missing {name} in {input}"))
        .1
        .split('"')
        .next()
        .unwrap()
}

/// Shared native URL semantics match, but nullable presence and associations do not.
#[test]
fn api_url_fields_keep_their_adapter_until_shared_markup_is_equivalent() {
    // This failed at registry composition on 443dfc0, before either control could render.
    let registry = codegen::definitions().unwrap();
    let ui = codegen::normalize(&registry).unwrap().project_ui().unwrap();
    let application = ui
        .models
        .iter()
        .find(|model| model.id == "Application")
        .unwrap();
    let shared = shared_url_binding();
    assert_eq!(shared.component, "stackpress.ui.field.url");
    assert_eq!(shared.props["input_type"], "url");
    for name in ["logo", "website"] {
        let field = application
            .fields
            .iter()
            .find(|field| field.wire_name == name)
            .unwrap();
        let binding = field
            .roles
            .iter()
            .find(|binding| binding.role == Role::Field)
            .unwrap();
        assert_eq!(binding.component, "api.source.field.url");
        assert_eq!(binding.props["type"], "url");
        assert!(field.nullable);
        for seed in [
            Input::Missing,
            Input::Null,
            Input::Value(json!("")),
            Input::Value(json!("https://example.invalid/a?x=1&y=\"🦀\"")),
        ] {
            let owner = Owner::new();
            owner.with(|| {
                let value = RwSignal::new(seed.clone());
                let context = RoleContext {
                    id: name.into(),
                    field: field.clone(),
                    role: Role::Field,
                    value,
                };
                let api = view! { <InputField context/> }.to_html();
                assert_eq!(value.get_untracked(), seed);
                // Change only the selected binding; retain the real API field and draft.
                let mut field = field.clone();
                *field
                    .roles
                    .iter_mut()
                    .find(|binding| binding.role == Role::Field)
                    .unwrap() = shared.clone();
                let context = RoleContext {
                    id: name.into(),
                    field,
                    role: Role::Field,
                    value,
                };
                let generic = view! { <TextField context/> }.to_html();
                assert_eq!(value.get_untracked(), seed);
                for name in ["type", "id", "name", "value"] {
                    assert_eq!(attribute(&api, name), attribute(&generic, name));
                }
                assert_eq!(attribute(&api, "type"), "url");
                assert_eq!(attribute(&api, "name"), name);
                // Both native values match without promoting the optional draft to empty.
                assert!(!api.contains("<select"));
                assert!(generic.contains(&format!("id=\"{name}-presence\"")));
                assert!(generic.contains("Clear (empty)"));
                assert!(!api.contains("class=\"sp-role\""));
                assert!(generic.contains("class=\"sp-role\""));
                assert!(!api.contains("aria-describedby="));
                assert!(generic.contains("aria-describedby="));
                assert_ne!(api, generic);
            });
            owner.cleanup();
        }
    }
}
