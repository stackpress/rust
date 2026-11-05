//! Generated portable source roles; regenerate from the application's composed Idea input.
use leptos::prelude::*;
use stackpress_framework_schema::{SchemaResult,UiField,UiModel};
[%#?:rendering%]use stackpress_framework_schema::Role;
use stackpress_framework_admin::RoleContext;
[%/?:rendering%]
use stackpress_framework_schema::__private::serde_json;
use stackpress_framework_admin::Draft;
/// Only portable presentation metadata; no native descriptors or providers.
pub fn model()->UiModel { serde_json::from_str([%model%]).expect("generated source UI descriptor") }
/// Source field order and props, detached from private metadata.
pub fn descriptors()->SchemaResult<Vec<UiField>> {Ok(model().fields)}
[%#@:roles role%]
/// Render the source-selected [%role.name%] roles for this instance.
[%#?:role.display%]pub fn [%role.function%](value:&serde_json::Value,instance:&str)->SchemaResult<[%#?:role.list%]Vec<AnyView>[%|%]AnyView[%/?:role.list%]> {
[%#?:role.has_controls%]
[%#?:role.uses_row%]    let value_row=value;
[%/?:role.uses_row%]
    let fields=descriptors()?;
    let draft=Draft::new(&fields,value,Role::[%role.variant%])?;
    let draft=&draft;
[%|%]    let _=(value,instance);
[%/?:role.has_controls%]
[%|%]pub fn [%role.name%](draft:&Draft,instance:&str)->SchemaResult<AnyView> {
[%#?:role.has_controls%]    let fields=descriptors()?;
[%|%]    let _=(draft,instance);
[%/?:role.has_controls%]
[%/?:role.display%]
    let [%#?:role.has_controls%]mut [%/?:role.has_controls%]controls=Vec::<AnyView>::new();
[%#@:role.controls control%]    {
        let field=fields.iter().find(|field|field.wire_name==[%control.wire%]).expect("generated role field").clone();
        let value=draft.get([%control.wire%]).expect("source role admitted draft");
        let context=RoleContext {id:format!("{instance}-[%role.name%]-{}",[%control.wire%]),field,role:Role::[%role.variant%],value};
[%#?:control.row%]        let row=value_row.clone();
[%/?:control.row%]        controls.push(view!{<[%control.module%]::[%control.symbol%] context [%#?:control.row%]row[%/?:control.row%]/>}.into_any());
    }
[%/@:role.controls%]    [%#?:role.list%]Ok(controls)[%|%]Ok(controls.collect_view().into_any())[%/?:role.list%]
}
[%#?:role.list%]/// Render source list controls outside a table.
pub fn list(value:&serde_json::Value,instance:&str)->SchemaResult<AnyView> {Ok(list_cells(value,instance)?.collect_view().into_any())}
[%/?:role.list%]
[%/@:roles%]
