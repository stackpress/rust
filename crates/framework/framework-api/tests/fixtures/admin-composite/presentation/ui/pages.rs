//! Generated ordinary source page dispatch; no model-name special cases.
use leptos::prelude::*;
use stackpress_framework_admin::{
    Draft,
    pages::{Form, PageSpec},
};
use stackpress_framework_schema::__private::serde_json::Value;
use stackpress_framework_schema::{SchemaError, SchemaErrorKind, SchemaResult};
/// Exact source model order, selected before compilation.
pub const MODELS: &[&str] = &["catalog", "slugged"];
/// Resolve a source module without a runtime model registry.
pub fn spec(module: &str) -> SchemaResult<PageSpec> {
    match module {
        "catalog" => Ok(PageSpec {
            module: module.into(),
            model: super::catalog::model(),
            soft_remove: true,
        }),
        "slugged" => Ok(PageSpec {
            module: module.into(),
            model: super::slugged::model(),
            soft_remove: false,
        }),
        _ => Err(unavailable()),
    }
}
/// Source create/update controls use the application's full generated draft.
pub fn form(module: &str, form: Form) -> SchemaResult<AnyView> {
    match module {
        "catalog" => super::catalog::field(&form.draft, &form.instance),
        "slugged" => super::slugged::field(&form.draft, &form.instance),
        _ => Err(unavailable()),
    }
}
/// Source filters remain presentation; handlers decide how to execute them.
pub fn filters(module: &str, draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "catalog" => super::catalog::filter(draft, instance),
        "slugged" => super::slugged::filter(draft, instance),
        _ => Err(unavailable()),
    }
}
/// Render exact source rows through their selected list adapters.
pub fn table(module: &str, value: &Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "catalog" => stackpress_framework_admin::source::pages::table(
            &spec(module)?,
            value,
            instance,
            super::catalog::list_cells,
        ),
        "slugged" => stackpress_framework_admin::source::pages::table(
            &spec(module)?,
            value,
            instance,
            super::slugged::list_cells,
        ),
        _ => Err(unavailable()),
    }
}
/// Render exact source detail data, including application-added columns.
pub fn detail(module: &str, value: &Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "catalog" => super::catalog::view(value, instance),
        "slugged" => super::slugged::view(value, instance),
        _ => Err(unavailable()),
    }
}
/// App-owned IO and routing receive only these compiled source adapters.
pub fn ui() -> stackpress_framework_admin::source::pages::Ui {
    stackpress_framework_admin::source::pages::Ui {
        spec,
        form,
        filters,
        table,
        detail,
    }
}
/// Unknown models never fall back to another model's process.
fn unavailable() -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, "source model unavailable")
}
