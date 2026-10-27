//! Validate complete public input before contributing escaped Rust source to the writer.

use super::bindings::{import, locate, validate_binding};
use std::collections::BTreeSet;

use stackpress_framework_schema::{
    Role, RoleBinding, SchemaError, SchemaErrorKind, SchemaResult, UiField, UiModel, UiSchema,
    codegen::{DefinitionRegistry, DiagnosticMap, portable_name, transform_error},
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
pub fn emit_roles(
    ui: &UiSchema,
    definitions: &DefinitionRegistry,
    diagnostics: &DiagnosticMap,
    output: &mut OutputPlan,
) -> Result<()> {
    let files = prepare(ui, definitions, diagnostics).map_err(transform_error)?;
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
        files.push((module, model_source(model, &fields, definitions)?));
    }
    Ok(files)
}

/// Emit literals only through Rust's escaped Debug-string spelling; source names are checked.
fn model_source(
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
    let imports = if rendering {
        "use leptos::prelude::*;\nuse stackpress_framework_admin::{Draft, RoleContext};\nuse stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult, UiField};\n"
    } else {
        "use stackpress_framework_schema::{SchemaError, SchemaErrorKind, SchemaResult, UiField};\n"
    };
    let mut text = format!(
        "//! Generated public role wrappers; handwritten adapters remain outside this partition.\n{imports}/// Decode only approved field descriptors; imports and source positions stay tool-only.\npub fn descriptors() -> SchemaResult<Vec<UiField>> {{ serde_json::from_str({json:?}).map_err(|cause| SchemaError::new(SchemaErrorKind::Codec, \"public role descriptor decoding failed\").with_cause(cause)) }}\n"
    );
    for role in [
        Role::Field,
        Role::Filter,
        Role::List,
        Role::Span,
        Role::View,
    ] {
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
        let name = role_name(role);
        text.push_str(&format!(
            "/// Render the explicitly selected {name} controls for one caller-owned instance.\n"
        ));
        if matches!(role, Role::List | Role::View) {
            let public = public_type(&model.id)?;
            text.push_str(&format!("pub fn {name}(value: &super::super::contracts::public::{public}, instance: &str) -> SchemaResult<AnyView> {{\nlet seed = stackpress_framework_schema::to_bounded_value(value)?;\nlet fields = descriptors()?;\nlet draft = Draft::new(&fields, &seed, Role::{role:?})?;\n"));
        } else {
            text.push_str(&format!("pub fn {name}(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {{\nlet fields = descriptors()?;\n"));
        }
        text.push_str("let mut controls = Vec::new();\n");
        for (field, binding) in selected {
            text.push_str(&control_source(field, binding, role, definitions, true)?);
        }
        text.push_str("Ok(controls.collect_view().into_any())\n}\n");
    }
    text.push_str(&super::table::source(model, fields, definitions)?);
    Ok(text)
}

/// Build one literal field context and a statically resolved component invocation.
pub(super) fn control_source(
    field: &UiField,
    binding: &RoleBinding,
    role: Role,
    definitions: &DefinitionRegistry,
    labeled: bool,
) -> SchemaResult<String> {
    let import = import(binding, definitions)?;
    let member = &field.wire_name;
    let name = role_name(role);
    let mut text = format!(
        "{{\n// This exact field was approved by the app before normalization.\nlet field = fields.iter().find(|field| field.wire_name == {member:?}).cloned().ok_or_else(|| SchemaError::new(SchemaErrorKind::Invalid, \"missing generated field descriptor\"))?;\nlet value = draft.get({member:?}).ok_or_else(|| SchemaError::new(SchemaErrorKind::Invalid, \"missing generated draft field\"))?;\nlet context = RoleContext {{ id: format!(\"{{}}-{name}-{{}}\", instance, {member:?}), field, role: Role::{role:?}, value }};\n"
    );
    // Model display roles label each value so multiple selected fields cannot run together.
    // Both the label and the component props remain escaped data in the generated view.
    if labeled && matches!(role, Role::List | Role::View) {
        text.push_str(&format!("let label = context.text_prop(\"label\", &context.field.label);\ncontrols.push(view! {{ <div class=\"sp-role-value\"><span class=\"sp-role-label\">{{label}} \": \"</span><{}::{} context/></div> }}.into_any());\n", import.module, import.symbol));
    } else {
        text.push_str(&format!(
            "controls.push(view! {{ <{}::{} context/> }}.into_any());\n",
            import.module, import.symbol
        ));
    }
    text.push_str("}\n");
    Ok(text)
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
