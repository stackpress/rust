//! Source-template boundaries: encoded contexts, unchanged output caps and inert branches.

use super::{Asset, OUTPUT_BYTES, Renderer, literal};
use serde_json::json;
use stackpress_framework_schema::SchemaErrorKind;
use stackpress_lib::template::{TemplateError, TemplateErrorKind};
use std::error::Error;

/// Normalize real composite keys, optionally leaving them outside the public projection.
fn table_source(expose_keys: bool) -> String {
    use stackpress_framework_schema::{FieldAccess, Schema, codegen::DefinitionRegistry};
    let config = stackpress_idea_parser::parse(
        "table-actions.idea",
        "model Example { tenant String @id id String @id name String @list.string }",
        &Default::default(),
    )
    .unwrap();
    let mut definitions = DefinitionRegistry::standard();
    for name in ["tenant", "id", "name"] {
        definitions = definitions
            .with_field_policy(
                "Example",
                name,
                FieldAccess {
                    readable: name == "name" || expose_keys,
                    ..Default::default()
                },
            )
            .unwrap();
    }
    let definitions = crate::codegen::register_components(definitions, vec![]).unwrap();
    let model = Schema::normalize(&config, &definitions, Default::default())
        .unwrap()
        .project_ui()
        .unwrap()
        .models
        .remove(0);
    assert_eq!(model.key.is_empty(), !expose_keys);
    super::super::table::source(
        &mut Renderer::new().unwrap(),
        &model,
        &model.fields,
        &definitions,
    )
    .unwrap()
}

/// Table actions retain schema key order and the source's empty sticky heading.
#[test]
fn table_actions_preserve_ordered_keys_and_empty_heading() {
    let source = table_source(true);
    assert!(source.contains("detail_action(\"example\", &[\"tenant\", \"id\"], &seed)"));
    assert!(source.contains("<TableCell sticky_right=true>{action}</TableCell>"));
    assert!(
        source.contains("<TableHeader sticky=true sticky_right=true>{().into_any()}</TableHeader>")
    );
    // The generated DOM key remains independent of the DTO keys used by detail_action.
    assert!(source.contains("format!(\"{byte:02x}\")"));
}

/// Keyless tables omit both action slots without looking up selected-only context.
#[test]
fn keyless_tables_do_not_read_action_context() {
    let actual = table_source(false);
    assert!(!actual.contains("detail_action"));
    assert!(!actual.contains("sticky_right"));
    // The template also remains lazy when selected-only keys are entirely absent.
    let source = Renderer::new()
        .unwrap()
        .render(
            Asset::Table,
            &json!({
                "public_type": "PublicExample", "controls": [], "headers": [],
                "caption_literal": literal("Examples"), "has_identity": false,
            }),
        )
        .unwrap();
    assert!(!source.contains("detail_action"));
    assert!(!source.contains("sticky_right"));
    assert!(source.contains("<TableRow row_key=key.clone()>{cells}</TableRow>"));
}

/// Exercise the production engine options with arbitrary small boundary-test source.
fn render(
    asset: &'static str,
    source: &str,
    data: &serde_json::Value,
) -> stackpress_framework_schema::SchemaResult<String> {
    Renderer::new()?
        .engine
        .render(source, data)
        .map_err(|cause| super::template_error(asset, cause))
}

/// JSON encoding overhead must not reduce the existing one-MiB emitted-source allowance.
#[test]
fn encoded_context_headroom_preserves_output_boundary() {
    // Quotes, backslashes and controls cost two JSON bytes each but one output byte.
    let payload = "\"\\\n".repeat(200_000);
    let data = json!({"payload": payload});
    assert!(serde_json::to_vec(&data).unwrap().len() > OUTPUT_BYTES);
    assert_eq!(
        render("admin/boundary.rs.tpl", "[%payload%]", &data).unwrap(),
        payload
    );
    // The local data allowance does not expand the generated source/artifact ceiling.
    let exact = "x".repeat(OUTPUT_BYTES);
    assert_eq!(
        render(
            "admin/boundary.rs.tpl",
            "[%payload%]",
            &json!({"payload": exact})
        )
        .unwrap()
        .len(),
        OUTPUT_BYTES
    );
    let cause = render(
        "admin/boundary.rs.tpl",
        "x[%payload%]",
        &json!({"payload": exact}),
    )
    .unwrap_err();
    assert_eq!(cause.kind(), SchemaErrorKind::Limit);
    assert!(cause.to_string().contains("admin/boundary.rs.tpl"));
    assert_eq!(
        cause
            .source()
            .unwrap()
            .downcast_ref::<TemplateError>()
            .unwrap()
            .kind(),
        TemplateErrorKind::Limit
    );
}

/// Empty host/role branches must not read unavailable selected-only fields.
#[test]
fn empty_assets_need_no_selected_branch_data() {
    let routes = render(
        "admin/routes.rs.tpl",
        include_str!("routes.rs.tpl"),
        &json!({
            "private": false, "has_models": false, "models": [],
        }),
    )
    .unwrap();
    let host = render(
        "admin/pages_host.rs.tpl",
        include_str!("pages_host.rs.tpl"),
        &json!({
            "has_models": false, "models": [], "direct_routes": routes,
        }),
    )
    .unwrap();
    assert!(!host.contains("pub fn register("));
    assert!(!host.contains("mount_routes"));
    assert!(!host.contains("builder.get("));
    assert!(host.contains("_start"));
    assert!(host.contains("No import model is selected."));
    let role = render(
        "admin/roles.rs.tpl",
        include_str!("roles.rs.tpl"),
        &json!({
            "rendering": false, "fields_literal": "\"[]\"", "roles": [], "has_table": false,
        }),
    )
    .unwrap();
    assert!(role.contains("pub fn descriptors()"));
    assert!(!role.contains("use leptos"));
    assert!(!role.contains("pub fn field("));
}

/// Source-looking data is escaped by the emitter and inserted once, without template evaluation.
#[test]
fn control_literals_and_runtime_formatting_stay_inert() {
    let wire = "[%missing%] \" \\ \n </script> 🦀";
    let control = render("admin/control.rs.tpl", include_str!("control.rs.tpl"), &json!({
        "wire_literal": literal(wire), "role_name": "field", "role_variant": "Field",
        "component_module": "crate::controls", "component_symbol": "TextField", "labeled": false,
    })).unwrap();
    assert!(control.contains(&literal(wire)));
    assert!(control.contains("format!(\"{}-field-{}\", instance,"));
    assert!(control.contains("<crate::controls::TextField context/>"));
    assert!(!control.contains("let label ="));
}

/// The new adapter changes only concrete template failures, never ordinary schema error policy.
#[test]
fn unrelated_schema_limits_keep_the_existing_callback_mapping() {
    let cause = stackpress_framework_schema::SchemaError::new(
        SchemaErrorKind::Limit,
        "existing schema budget",
    );
    assert_eq!(
        super::source_error(cause).kind(),
        stackpress_idea_transformer::TransformErrorKind::Callback
    );
}

/// Reusing an asset must never retain earlier data, failed lookups, or spent render budgets.
#[test]
fn reused_controls_start_with_fresh_data_after_a_failed_render() {
    let mut renderer = Renderer::new().unwrap();
    let context = |wire: &str| {
        json!({
            "wire_literal": literal(wire), "role_name": "field", "role_variant": "Field",
            "component_module": "crate::controls", "component_symbol": "TextField", "labeled": false,
        })
    };
    let first = renderer.render(Asset::Control, &context("first")).unwrap();
    assert!(first.contains("draft.get(\"first\")"));
    let error = renderer.render(Asset::Control, &json!({})).unwrap_err();
    assert!(error.to_string().contains("admin/control.rs.tpl"));
    let second = renderer
        .render(Asset::Control, &context("second [%inert%]"))
        .unwrap();
    assert!(second.contains("draft.get(\"second [%inert%]\")"));
    assert!(!second.contains("draft.get(\"first\")"));
}
