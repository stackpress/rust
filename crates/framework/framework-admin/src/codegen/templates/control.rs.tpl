{
// This exact field was approved by the app before normalization.
let field = fields.iter().find(|field| field.wire_name == [%wire_literal%]).cloned().ok_or_else(|| SchemaError::new(SchemaErrorKind::Invalid, "missing generated field descriptor"))?;
let value = draft.get([%wire_literal%]).ok_or_else(|| SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field"))?;
let context = RoleContext {
    id: format!("{}-[%role_name%]-{}", instance, [%wire_literal%]),
    field,
    role: Role::[%role_variant%],
    value,
};
[%#?:labeled%]let label = context.text_prop("label", &context.field.label);
controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><[%component_module%]::[%component_symbol%] context/></div> }.into_any());
[%|%]controls.push(view! { <[%component_module%]::[%component_symbol%] context/> }.into_any());
[%/?:labeled%]}
