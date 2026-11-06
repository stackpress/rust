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
[%#?:role.display%]pub fn [%role.function%](value:&serde_json::Value,instance:&str)->SchemaResult<[%#?:role.cells%]Vec<AnyView>[%|%]AnyView[%/?:role.cells%]> {
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
[%#?:role.prefix%]        let context=stackpress_framework_admin::source::roles::fieldset_context(context);
[%/?:role.prefix%][%#?:control.row%]        let row=value_row.clone();
[%/?:control.row%][%#?:control.nested%]        let fields=super::[%control.nested_module%]::descriptors()?;
[%#?:control.nested_display%]        let render=Callback::new(|(value,instance):(serde_json::Value,String)|super::[%control.nested_module%]::[%control.nested_function%](&value,&instance).expect("generated fieldset display"));
[%|%]        let fieldset_label=super::[%control.nested_module%]::model().label;
        let render=Callback::new(|(draft,instance):(Draft,String)|super::[%control.nested_module%]::[%control.nested_function%](&draft,&instance).expect("generated fieldset inputs"));
[%/?:control.nested_display%][%/?:control.nested%]        controls.push(view!{<[%control.module%]::[%control.symbol%] context [%#?:control.row%]row[%/?:control.row%][%#?:control.nested%] fields render [%#?:control.nested_display%][%|%]fieldset_label[%/?:control.nested_display%][%/?:control.nested%]/>}.into_any());
    }
[%/@:role.controls%]    [%#?:role.cells%]Ok(controls)[%|%]Ok(controls.collect_view().into_any())[%/?:role.cells%]
}
[%#?:role.cells%]/// Render source [%role.name%] controls outside a table.
pub fn [%role.name%](value:&serde_json::Value,instance:&str)->SchemaResult<AnyView> {Ok([%role.function%](value,instance)?.collect_view().into_any())}
[%/?:role.cells%]
[%/@:roles%]
