//! Generated ordinary source page dispatch; no model-name special cases.
use leptos::prelude::*;
use stackpress_framework_admin::{Draft,pages::{Form,PageSpec}};
use stackpress_framework_schema::{SchemaResult,SchemaError,SchemaErrorKind};
use stackpress_framework_schema::__private::serde_json::Value;
/// Exact source model order, selected before compilation.
pub const MODELS:&[&str]=&[[%#@:models model%][%model.module_literal%],[%/@:models%]];
/// Resolve a source module without a runtime model registry.
pub fn spec(module:&str)->SchemaResult<PageSpec> {
    match module {
[%#@:models model%]        [%model.module_literal%]=>Ok(PageSpec {module:module.into(),model:super::[%model.module%]::model(),soft_remove:[%model.soft_remove%]}),
[%/@:models%]        _=>Err(unavailable()),
    }
}
/// Source create/update controls use the application's full generated draft.
pub fn form(module:&str,form:Form)->SchemaResult<AnyView> {
    match module {
[%#@:models model%]        [%model.module_literal%]=>super::[%model.module%]::field(&form.draft,&form.instance),
[%/@:models%]        _=>Err(unavailable()),
    }
}
/// Source filters remain presentation; handlers decide how to execute them.
pub fn filters(module:&str,draft:&Draft,instance:&str)->SchemaResult<AnyView> {
    match module {
[%#@:models model%]        [%model.module_literal%]=>super::[%model.module%]::filter(draft,instance),
[%/@:models%]        _=>Err(unavailable()),
    }
}
/// Render exact source rows through their selected list adapters.
pub fn table(module:&str,value:&Value,instance:&str)->SchemaResult<AnyView> {
    match module {
[%#@:models model%]        [%model.module_literal%]=>stackpress_framework_admin::source::pages::table(&spec(module)?,value,instance,super::[%model.module%]::list_cells),
[%/@:models%]        _=>Err(unavailable()),
    }
}
/// Render exact source detail data, including application-added columns.
pub fn detail(module:&str,value:&Value,instance:&str)->SchemaResult<AnyView> {
    match module {
[%#@:models model%]        [%model.module_literal%]=>super::[%model.module%]::view(value,instance),
[%/@:models%]        _=>Err(unavailable()),
    }
}
/// App-owned IO and routing receive only these compiled source adapters.
pub fn ui()->stackpress_framework_admin::source::pages::Ui {stackpress_framework_admin::source::pages::Ui {spec,form,filters,table,detail}}
/// Unknown models never fall back to another model's process.
fn unavailable()->SchemaError {SchemaError::new(SchemaErrorKind::Invalid,"source model unavailable")}
