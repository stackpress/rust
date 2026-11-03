//! Preflight public descriptors, then render structural role templates into the UI-owned plan.

use super::bindings::{import, locate, validate_binding};
use super::templates::{Asset, Renderer, source_error};
use std::collections::BTreeSet;

use stackpress_framework_schema::{
    Role, RoleBinding, SchemaError, SchemaErrorKind, SchemaResult, UiField, UiModel, UiSchema,
    codegen::{DefinitionRegistry, DiagnosticMap, portable_name},
};
use stackpress_idea_transformer::{OutputPlan, Result};

/// G38 selects revision 3 explicitly; older descriptors cannot mix with new contracts.
const UI_CONTRACT_REVISION: u32 = 3;

/// Emit deterministic model role functions from revision-3 public descriptors only.
///
/// The ordinary function transform must own `ui`. Its sibling schema transform emits the
/// `contracts::public::Public<Model>` types. Missing roles contribute no function. The import
/// registry remains build-host data and is resolved before emitting any model file.
/// # Errors
/// Refuses revision/name collisions, unavailable/missing/invalid imports, incompatible native
/// controls, oversized descriptors, and output ownership failures. Semantic errors retain
/// Idea byte positions or normalized-JSON structural paths through `diagnostics`.
/// Template failures retain an authored asset identity and concrete engine cause. A template
/// output limit keeps the direct emitter's Limit category beneath registry callback context.
pub fn emit_roles(
    ui: &UiSchema,
    definitions: &DefinitionRegistry,
    diagnostics: &DiagnosticMap,
    output: &mut OutputPlan,
) -> Result<()> {
    let files = prepare(ui, definitions, diagnostics).map_err(source_error)?;
    // Publish contributions only after every model and import passed preflight.
    for (module, text) in files {
        output.write(format!("ui/{module}.rs"), text)?;
        output.contribute_module("ui", &module)?;
    }
    Ok(())
}

/// Render a bounded complete set, detecting collisions before mutating the output plan.
fn prepare(
    ui: &UiSchema,
    definitions: &DefinitionRegistry,
    diagnostics: &DiagnosticMap,
) -> SchemaResult<Vec<(String, String)>> {
    if ui.contract_revision != UI_CONTRACT_REVISION
        || stackpress_framework_schema::CONTRACT_REVISION != UI_CONTRACT_REVISION
    {
        return Err(invalid("unsupported UI contract revision"));
    }
    let value = stackpress_framework_schema::to_bounded_value(ui)?;
    stackpress_framework_schema::check_browser_json(&value)?;
    if ui.models.len() + ui.fieldsets.len() > 128 {
        return Err(invalid("UI model budget exceeded"));
    }
    // Compilation is invocation-local; each used asset is reused across every selected model.
    let mut renderer = Renderer::new()?;
    let mut modules = BTreeSet::new();
    let mut files = Vec::new();
    for (kind, model) in ui
        .models
        .iter()
        .map(|model| ("model", model))
        .chain(ui.fieldsets.iter().map(|model| ("type", model)))
    {
        let module = portable_name(&model.id)?;
        if !modules.insert(module.clone()) {
            return Err(invalid("UI module name collision"));
        }
        if model.fields.len() > 128 {
            return Err(invalid("UI field budget exceeded"));
        }
        let fields: Vec<_> = model
            .fields
            .iter()
            .filter(|field| !field.sensitive && (field.readable || field.writable))
            .cloned()
            .collect();
        let mut names = BTreeSet::new();
        for field in &fields {
            if !names.insert(&field.wire_name) {
                return Err(invalid("duplicate public UI field"));
            }
            let mut roles = BTreeSet::new();
            for binding in &field.roles {
                if !roles.insert(binding.role) {
                    return Err(locate(
                        invalid("duplicate public UI role"),
                        kind,
                        model,
                        field,
                        binding.role,
                        diagnostics,
                    ));
                }
                // An explicit filter on an unsupported type must diagnose, not disappear.
                if binding.role == Role::Filter && field.readable && !field.filterable {
                    return Err(locate(
                        invalid("selected UI filter has no public query capability"),
                        kind,
                        model,
                        field,
                        binding.role,
                        diagnostics,
                    ));
                }
                if crate::draft::admitted(field, binding.role) {
                    validate_binding(field, binding, definitions).map_err(|error| {
                        locate(error, kind, model, field, binding.role, diagnostics)
                    })?;
                }
            }
        }
        files.push((
            module,
            model_source(&mut renderer, model, &fields, definitions)?,
        ));
    }
    Ok(files)
}

/// Emit literals only through Rust's escaped Debug-string spelling; source names are checked.
fn model_source(
    renderer: &mut Renderer,
    model: &UiModel,
    fields: &[UiField],
    definitions: &DefinitionRegistry,
) -> SchemaResult<String> {
    let json = serde_json::to_string(fields)
        .map_err(|cause| invalid("public UI descriptor encoding failed").with_cause(cause))?;
    // Choose compiler-owned imports before embedding labels; never rewrite literal data.
    let rendering = fields.iter().any(|field| {
        field
            .roles
            .iter()
            .any(|binding| crate::draft::admitted(field, binding.role))
    });
    let mut roles = Vec::new();
    for role in [
        Role::Field,
        Role::Filter,
        Role::List,
        Role::Span,
        Role::View,
    ] {
        // Preserve admission and role order before rendering any source structure.
        let selected: Vec<_> = fields
            .iter()
            .filter(|field| crate::draft::admitted(field, role))
            .filter_map(|field| {
                field
                    .roles
                    .iter()
                    .find(|binding| binding.role == role)
                    .map(|binding| (field, binding))
            })
            .collect();
        if selected.is_empty() {
            continue;
        }
        let typed_input = matches!(role, Role::List | Role::View);
        let public = if typed_input {
            public_type(&model.id)?
        } else {
            String::new()
        };
        // Child fragments come from the actual shared control template, never a formatter.
        let controls = selected
            .into_iter()
            .map(|(field, binding)| {
                control_source(renderer, field, binding, role, definitions, true)
            })
            .collect::<SchemaResult<Vec<_>>>()?;
        roles.push(serde_json::json!({
            "name": role_name(role), "variant": format!("{role:?}"),
            "typed_input": typed_input, "public_type": public, "controls": controls,
        }));
    }
    let table = super::table::source(renderer, model, fields, definitions)?;
    renderer.render(
        Asset::Roles,
        &serde_json::json!({
            "rendering": rendering, "fields_literal": super::templates::literal(&json),
            "roles": roles, "has_table": !table.is_empty(), "table": table,
        }),
    )
}

/// Build one literal field context and a statically resolved component invocation.
pub(super) fn control_source(
    renderer: &mut Renderer,
    field: &UiField,
    binding: &RoleBinding,
    role: Role,
    definitions: &DefinitionRegistry,
    labeled: bool,
) -> SchemaResult<String> {
    let import = import(binding, definitions)?;
    // Labels and props are inert descriptor data; only checked imports enter source positions.
    renderer.render(
        Asset::Control,
        &serde_json::json!({
            "wire_literal": super::templates::literal(&field.wire_name),
            "role_name": role_name(role), "role_variant": format!("{role:?}"),
            "component_module": import.module, "component_symbol": import.symbol,
            "labeled": labeled && matches!(role, Role::List | Role::View),
        }),
    )
}

/// Match schema's conventional public type spelling without accepting emitted type fragments.
pub(super) fn public_type(source: &str) -> SchemaResult<String> {
    let module = portable_name(source)?;
    let mut name = String::new();
    for part in module.split('_').filter(|part| !part.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            name.push(first.to_ascii_uppercase());
            name.extend(chars);
        }
    }
    if name == "Self" {
        name.push_str("Value");
    }
    Ok(format!("Public{name}"))
}

/// Finite function names are authored here, never supplied by metadata.
pub(super) fn role_name(role: Role) -> &'static str {
    match role {
        Role::Field => "field",
        Role::Filter => "filter",
        Role::List => "list",
        Role::Span => "span",
        Role::View => "view",
    }
}

/// Add one safe context without disclosing rejected values through Display.
fn invalid(context: &'static str) -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, context)
}
