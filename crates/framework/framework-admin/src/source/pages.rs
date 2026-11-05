//! Portable source page callbacks. No server or model source runtime enters the browser graph.
use crate::{
    Draft,
    pages::{Form, PageSpec},
};
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::{SchemaError, SchemaErrorKind, SchemaResult};

/// Compiled source model adapters. Registration and IO remain application-owned.
#[derive(Clone, Copy)]
pub struct Ui {
    /// Source model descriptor.
    pub spec: fn(&str) -> SchemaResult<PageSpec>,
    /// Source create/update/copy fields.
    pub form: fn(&str, Form) -> SchemaResult<AnyView>,
    /// Source query controls.
    pub filters: fn(&str, &Draft, &str) -> SchemaResult<AnyView>,
    /// Source search presentation.
    pub table: fn(&str, &Value, &str) -> SchemaResult<AnyView>,
    /// Source detail presentation.
    pub detail: fn(&str, &Value, &str) -> SchemaResult<AnyView>,
}

/// Render source search rows without Typed page-size/identity admission policy.
/// # Errors
/// Requires the source search result array (or its rows envelope), never a substitute row.
pub fn table(
    spec: &PageSpec,
    value: &Value,
    instance: &str,
    render: fn(&Value, &str) -> SchemaResult<Vec<AnyView>>,
) -> SchemaResult<AnyView> {
    use stackpress_leptos::components::{
        Table, TableCell, TableGroup, TableHead, TableHeader, TableRow,
    };
    let rows = value
        .get("rows")
        .unwrap_or(value)
        .as_array()
        .ok_or_else(|| SchemaError::new(SchemaErrorKind::Invalid, "source search rows missing"))?;
    let mut rendered = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let row = row.get("row").unwrap_or(row);
        let identity = format!("{instance}-{index}");
        let cells = render(row, &identity)?
            .into_iter()
            .map(|cell| view! {<TableCell>{cell}</TableCell>})
            .collect_view();
        let action = crate::row_actions::detail_action(
            &spec.module,
            &spec
                .model
                .key
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            row,
        );
        rendered.push(view!{<TableRow row_key=identity>{cells}<TableCell sticky_right=true>{action}</TableCell></TableRow>});
    }
    let headers = spec
        .model
        .fields
        .iter()
        .filter(|field| {
            field
                .roles
                .iter()
                .any(|role| role.role == stackpress_framework_schema::Role::List)
        })
        .map(|field| {
            let label = field.label.clone();
            view! {<TableHeader>{label}</TableHeader>}
        })
        .collect_view();
    let caption = spec.model.label.clone();
    Ok(view!{<Table caption><TableHead><TableRow row_key="header">{headers}<TableHeader sticky=true sticky_right=true>{().into_any()}</TableHeader></TableRow></TableHead><TableGroup>{rendered.collect_view()}</TableGroup></Table>}.into_any())
}
