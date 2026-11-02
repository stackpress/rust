/// Render at most 100 public rows through selected list adapters.
/// Caller-supplied unique nonempty keys (at most 128 bytes) retain row identity.
/// Invalid row identities fail before constructing any controls.
pub fn table(rows: &[(String, super::super::contracts::public::[%public_type%])], instance: &str) -> SchemaResult<AnyView> {
    use stackpress_leptos::components::{Table, TableHead, TableGroup, TableRow, TableHeader, TableCell};
    let mut keys = std::collections::BTreeSet::new();
    if rows.len() > 100 || rows.iter().any(|(key,_)| key.is_empty() || key.len() > 128 || !keys.insert(key)) {
        return Err(SchemaError::new(SchemaErrorKind::Invalid, "invalid public table row identities"));
    }
    let fields = descriptors()?;
    let mut rendered = Vec::new();
    for (key,value) in rows {
        let seed = stackpress_framework_schema::to_bounded_value(value)?;
        let draft = Draft::new(&fields,&seed,Role::List)?;
        // Hex encoding is injective and cannot collide with role/field delimiters.
        let identity: String = key.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect();
        let instance = format!("{instance}-row-{identity}");
        let mut controls = Vec::new();
[%#@:controls control%][%control%][%/@:controls%]        let cells = controls.into_iter().map(|control| view! { <TableCell>{control}</TableCell> }).collect_view();
        rendered.push(view! { <TableRow row_key=key.clone()>{cells}</TableRow> });
    }
    let headers = vec![
[%#@:headers header%]view! { <TableHeader>{[%header.literal%]}</TableHeader> },
[%/@:headers%]    ].collect_view();
    let rows = rendered.collect_view();
    Ok(view! { <Table caption=[%caption_literal%]><TableHead><TableRow row_key="header">{headers}</TableRow></TableHead><TableGroup>{rows}</TableGroup></Table> }.into_any())
}
