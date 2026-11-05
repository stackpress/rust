//! Generated ordinary source page dispatch; no model-name special cases.
use leptos::prelude::*;
use stackpress_framework_admin::{
    Draft,
    pages::{Form, PageSpec},
};
use stackpress_framework_schema::__private::serde_json::Value;
use stackpress_framework_schema::{SchemaError, SchemaErrorKind, SchemaResult};
/// Exact source model order, selected before compilation.
pub const MODELS: &[&str] = &["profile", "application", "session", "auth"];
/// Resolve a source module without a runtime model registry.
pub fn spec(module: &str) -> SchemaResult<PageSpec> {
    match module {
        "profile" => Ok(PageSpec {
            module: module.into(),
            model: super::profile::model(),
            soft_remove: true,
        }),
        "application" => Ok(PageSpec {
            module: module.into(),
            model: super::application::model(),
            soft_remove: true,
        }),
        "session" => Ok(PageSpec {
            module: module.into(),
            model: super::session::model(),
            soft_remove: true,
        }),
        "auth" => Ok(PageSpec {
            module: module.into(),
            model: super::auth::model(),
            soft_remove: true,
        }),
        _ => Err(unavailable()),
    }
}
/// Source create/update controls use the application's full generated draft.
pub fn form(module: &str, form: Form) -> SchemaResult<AnyView> {
    match module {
        "profile" => super::profile::field(&form.draft, &form.instance),
        "application" => super::application::field(&form.draft, &form.instance),
        "session" => super::session::field(&form.draft, &form.instance),
        "auth" => super::auth::field(&form.draft, &form.instance),
        _ => Err(unavailable()),
    }
}
/// Source filters remain presentation; handlers decide how to execute them.
pub fn filters(module: &str, draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "profile" => super::profile::filter(draft, instance),
        "application" => super::application::filter(draft, instance),
        "session" => super::session::filter(draft, instance),
        "auth" => super::auth::filter(draft, instance),
        _ => Err(unavailable()),
    }
}
/// Render exact source rows through their selected list adapters.
pub fn table(module: &str, value: &Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "profile" => stackpress_framework_admin::source::pages::table(
            &spec(module)?,
            value,
            instance,
            super::profile::list_cells,
        ),
        "application" => stackpress_framework_admin::source::pages::table(
            &spec(module)?,
            value,
            instance,
            super::application::list_cells,
        ),
        "session" => stackpress_framework_admin::source::pages::table(
            &spec(module)?,
            value,
            instance,
            super::session::list_cells,
        ),
        "auth" => stackpress_framework_admin::source::pages::table(
            &spec(module)?,
            value,
            instance,
            super::auth::list_cells,
        ),
        _ => Err(unavailable()),
    }
}
/// Render exact source detail data, including application-added columns.
pub fn detail(module: &str, value: &Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "profile" => super::profile::view(value, instance),
        "application" => super::application::view(value, instance),
        "session" => super::session::view(value, instance),
        "auth" => super::auth::view(value, instance),
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
