//! Generated typed page adapters and explicit host route/operation selection.
use super::emit::{control_source, public_type};
use super::templates::{Asset, Renderer, literal, source_error};
use serde_json::{Value, json};
use stackpress_framework_schema::{
    Role, Schema, SchemaResult, UiModel, codegen::DefinitionRegistry,
};
use stackpress_idea_transformer::{OutputPlan, Result};

/// Generate typed adapters for the application's explicit, currently present model selection.
///
/// Call `emit_roles` first for its complete role/source-coordinate preflight. Pure adapters
/// compile in the frontend; the separate host file is explicitly included only by the native
/// application. Both files belong to the existing UI partition and the same output writer.
///
/// # Errors
/// Refuses duplicate selections/private identities and preserves schema/generator causes.
/// Template failures identify their static asset; source limits preserve the direct Limit
/// category. The registry continues to add its ordinary outer Callback context.
pub fn emit_pages(
    schema: &Schema,
    definitions: &DefinitionRegistry,
    selected: &[&str],
    output: &mut OutputPlan,
) -> Result<()> {
    let mut unique = std::collections::BTreeSet::new();
    if selected.len() > 128 || selected.iter().any(|name| !unique.insert(*name)) {
        return Err(stackpress_idea_transformer::TransformError::new(
            "invalid or duplicate admin model selection",
        ));
    }
    let ui = schema.project_ui().map_err(source_error)?;
    // Import selection depends only on admitted fields, never on embedded label text.
    let has_controls = ui
        .models
        .iter()
        .filter(|model| selected.contains(&model.id.as_str()))
        .any(|model| has_role(model, Role::Field));
    let mut renderer = Renderer::new().map_err(source_error)?;
    let mut models = Vec::new();
    let mut pages = Vec::new();
    let mut hosts = Vec::new();
    let mut route_models = Vec::new();
    for name in selected {
        // A reduced schema removes the old model's adapters and registrations in one install.
        let Some(model) = schema.models().iter().find(|model| model.name == *name) else {
            continue;
        };
        let projected = ui
            .models
            .iter()
            .find(|item| item.id == *name)
            .ok_or_else(|| {
                stackpress_idea_transformer::TransformError::new(
                    "admin model has no public projection",
                )
            })?;
        if projected.key.is_empty() {
            return Err(stackpress_idea_transformer::TransformError::new(
                "admin requires an approved complete identity",
            ));
        }
        let module = &model.module;
        let soft = model.fields.iter().any(|field| field.active);
        models.push((module.clone(), model.name.clone()));
        pages.push(
            model_data(&mut renderer, module, projected, soft, definitions)
                .map_err(source_error)?,
        );
        hosts.push(json!({
            "module_literal": literal(module), "native_module": native_module(module),
            "store_type": model.rust_name.clone() + "s", "soft_remove": soft,
        }));
        // These finite action inventories retain source ordering and soft-delete selection.
        let page_actions: Vec<_> = [
            "search", "detail", "create", "update", "copy", "remove", "restore", "import",
        ]
        .into_iter()
        .filter(|action| soft || !matches!(*action, "remove" | "restore"))
        .collect();
        let api_actions: Vec<_> = [
            "search", "get", "detail", "create", "update", "remove", "restore",
        ]
        .into_iter()
        .filter(|action| soft || !matches!(*action, "remove" | "restore"))
        .collect();
        route_models.push(json!({
            "module": module, "page_actions": page_actions,
            "import_actions": ["preview", "confirm"], "api_actions": api_actions,
        }));
    }
    // Every selected model and control passed before either final file reaches the writer.
    let dispatch = dispatch(&mut renderer, &models).map_err(source_error)?;
    let pure = renderer
        .render(
            Asset::Pages,
            &json!({
                "has_controls": has_controls, "models": pages, "dispatch": dispatch,
            }),
        )
        .map_err(source_error)?;
    let present = !models.is_empty();
    let direct_routes = routes(&mut renderer, &route_models, false).map_err(source_error)?;
    let (registration, mounted_routes) = if present {
        // A removed selection has no registrar, service claim, or private Route helper.
        (
            renderer
                .render(Asset::Registration, &json!({}))
                .map_err(source_error)?,
            routes(&mut renderer, &route_models, true).map_err(source_error)?,
        )
    } else {
        (String::new(), String::new())
    };
    let host = renderer
        .render(
            Asset::Host,
            &json!({
                "has_models": present, "models": hosts, "direct_routes": direct_routes,
                "registration": registration, "mounted_routes": mounted_routes,
            }),
        )
        .map_err(source_error)?;
    output.write("ui/pages.rs", pure)?;
    output.contribute_module("ui", "pages")?;
    output.write("ui/pages_host.rs", host)?;
    Ok(())
}

/// Keep SQL's reserved aggregate-module escape separate from source-template rendering.
fn native_module(module: &str) -> String {
    if matches!(module, "schema" | "operations") {
        format!("_{module}")
    } else {
        module.into()
    }
}

/// Check one explicit public capability without substituting a different role.
fn has_role(model: &UiModel, role: Role) -> bool {
    model.fields.iter().any(|field| {
        field
            .roles
            .iter()
            .any(|binding| binding.role == role && crate::draft::admitted(field, role))
    })
}

/// Prepare one minimal model context; literals remain data and controls share one template.
fn model_data(
    renderer: &mut Renderer,
    module: &str,
    model: &UiModel,
    soft: bool,
    definitions: &DefinitionRegistry,
) -> SchemaResult<Value> {
    let public = public_type(&model.id)?;
    let name = public.strip_prefix("Public").expect("public type prefix");
    let spec = json!({"module":module,"model":model,"soft_remove":soft}).to_string();
    let mut controls = Vec::new();
    for field in &model.fields {
        let Some(binding) = field
            .roles
            .iter()
            .find(|binding| binding.role == Role::Field)
        else {
            continue;
        };
        if !crate::draft::admitted(field, Role::Field) {
            continue;
        }
        controls.push(json!({
            "key_guard": model.key.contains(&field.wire_name),
            "body": control_source(renderer, field, binding, Role::Field, definitions, false)?,
        }));
    }
    Ok(json!({
        "module": module, "type_name": name, "public_type": public,
        "spec_literal": literal(&spec), "has_controls": has_role(model, Role::Field),
        "has_filter": has_role(model, Role::Filter), "has_list": has_role(model, Role::List),
        "has_view": has_role(model, Role::View), "controls": controls,
    }))
}

/// Render identical direct/private route bodies without rewriting already emitted Rust.
fn routes(renderer: &mut Renderer, models: &[Value], private: bool) -> SchemaResult<String> {
    renderer.render(
        Asset::Routes,
        &json!({
            "private": private, "has_models": !models.is_empty(), "models": models,
        }),
    )
}

/// Turn finite authored signature atoms into records, never formatted declarations or calls.
fn adapter(name: &str, parameters: &[(&str, &str)], result: &str, arguments: &[&str]) -> Value {
    let parameters: Vec<_> = parameters
        .iter()
        .map(|(name, ty)| json!({"name": name, "ty": ty}))
        .collect();
    json!({"name": name, "parameters": parameters, "result": result, "arguments": arguments})
}

/// Generate the same closed dispatch table for shared UI and native host callers.
fn dispatch(renderer: &mut Renderer, models: &[(String, String)]) -> SchemaResult<String> {
    let identities: Vec<_> = models
        .iter()
        .map(|(module, name)| {
            json!({
                "module": module, "module_literal": literal(module), "name_literal": literal(name),
            })
        })
        .collect();
    // Signatures remain compiler-authored; templates own separators, unused names and calls.
    let adapters = [
        adapter("spec", &[("module", "&str")], "PageSpec", &[]),
        adapter(
            "input",
            &[
                ("module", "&str"),
                ("action", "Action"),
                ("value", "serde_json::Value"),
            ],
            "serde_json::Value",
            &["action", "value"],
        ),
        adapter(
            "query",
            &[("module", "&str"), ("value", "serde_json::Value")],
            "serde_json::Value",
            &["value"],
        ),
        adapter(
            "validate_import",
            &[
                ("module", "&str"),
                ("value", "serde_json::Value"),
                ("context", "&stackpress_framework_schema::ValueContext<'_>"),
            ],
            "Vec<stackpress_framework_schema::FieldIssue>",
            &["value", "context"],
        ),
        adapter(
            "form",
            &[("module", "&str"), ("form", "Form")],
            "AnyView",
            &["form"],
        ),
        adapter(
            "filters",
            &[
                ("module", "&str"),
                ("draft", "&Draft"),
                ("instance", "&str"),
            ],
            "AnyView",
            &["draft", "instance"],
        ),
        adapter(
            "table",
            &[
                ("module", "&str"),
                ("value", "&serde_json::Value"),
                ("instance", "&str"),
            ],
            "AnyView",
            &["value", "instance"],
        ),
        adapter(
            "detail",
            &[
                ("module", "&str"),
                ("value", "&serde_json::Value"),
                ("instance", "&str"),
            ],
            "AnyView",
            &["value", "instance"],
        ),
    ];
    renderer.render(
        Asset::Dispatch,
        &json!({
            "has_models": !models.is_empty(), "models": identities, "adapters": adapters,
        }),
    )
}
