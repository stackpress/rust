//! Native head/body/foot/group/cell slots plus a controlled local data-table adapter.
use super::icons::{IconKind, icon};
use super::{Pagination, SortDirection, Table, TableState};
use leptos::prelude::*;
/// Native table header slot; no source child inspection.
#[component]
pub fn TableHead(
    /// Explicit rows and column header cells.
    children: Children,
) -> impl IntoView {
    view! {<thead>{children()}</thead>}
}
/// Native table body/group slot; multiple groups preserve semantic document order.
#[component]
pub fn TableGroup(
    /// Explicit body rows.
    children: Children,
) -> impl IntoView {
    view! {<tbody>{children()}</tbody>}
}
/// Native table footer slot.
#[component]
pub fn TableFoot(
    /// Explicit summary rows.
    children: Children,
) -> impl IntoView {
    view! {<tfoot>{children()}</tfoot>}
}
/// Native row with a host-owned stable identity and optional selected state.
#[component]
pub fn TableRow(
    /// Stable application row identity, not an array index.
    #[prop(into)]
    row_key: String,
    /// Whether the application has selected this row.
    #[prop(optional, into)]
    selected: Signal<bool>,
    /// Explicit native cells.
    children: Children,
) -> impl IntoView {
    view! {<tr data-row-id=row_key data-selected=move ||selected.get().to_string()>{children()}</tr>}
}
/// Header cell scope is explicit and cannot become an arbitrary ARIA grid role.
#[derive(Clone, Copy, Debug, Default)]
pub enum HeaderScope {
    /// Ordinary column header.
    #[default]
    Column,
    /// Row header.
    Row,
    /// Column group header.
    ColumnGroup,
    /// Row group header.
    RowGroup,
}
impl HeaderScope {
    /// Native scope token.
    fn token(self) -> &'static str {
        match self {
            Self::Column => "col",
            Self::Row => "row",
            Self::ColumnGroup => "colgroup",
            Self::RowGroup => "rowgroup",
        }
    }
}
/// Explicit head cell with bounded spans and opt-in sticky positioning.
#[component]
pub fn TableHeader(
    /// Native scope.
    #[prop(optional)]
    scope: HeaderScope,
    /// Bounded native column span.
    #[prop(default = 1)]
    colspan: u16,
    /// Native row span: zero reaches the end of the row group; positive spans cap at 100.
    #[prop(default = 1)]
    rowspan: u16,
    /// Optional sticky header within its own scrolling region.
    #[prop(optional)]
    sticky: bool,
    /// Header content.
    children: Children,
) -> impl IntoView {
    view! {<th scope=scope.token() colspan=colspan.clamp(1,100) rowspan=rowspan.min(100) class:sp-sticky=sticky>{children()}</th>}
}
/// Explicit data cell with bounded spans.
#[component]
pub fn TableCell(
    /// Native column span.
    #[prop(default = 1)]
    colspan: u16,
    /// Native row span: zero reaches the end of the row group; positive spans cap at 100.
    #[prop(default = 1)]
    rowspan: u16,
    /// Escaped/semantic cell content.
    children: Children,
) -> impl IntoView {
    view! {<td colspan=colspan.clamp(1,100) rowspan=rowspan.min(100)>{children()}</td>}
}
/// Invisible width ruler inside a cell, preserving the source TableRule contract.
/// This decorative element contributes a bounded minimum width without adding a row.
#[component]
pub fn TableRule(
    /// Requested pixel width, clamped to 1..=4096; no arbitrary style string.
    width: u16,
) -> impl IntoView {
    view! {<hr aria-hidden="true" style=format!("border:0;margin:0;width:{}px",width.clamp(1,4096))/>}
}
/// Presentation for a data column, independent of its value and controller.
/// Missing entries use wrapping with no ruler; extra entries are ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableColumnLayout {
    /// Optional minimum pixel width, clamped to 1..=4096 by the cell ruler.
    pub min_width: Option<u16>,
    /// Permit prose wrapping. False keeps compact values on one line.
    pub wrap: bool,
}
impl Default for TableColumnLayout {
    /// Preserve the existing wrapping presentation when no layout is supplied.
    fn default() -> Self {
        Self {
            min_width: None,
            wrap: true,
        }
    }
}

/// Full controlled local table: literal filter, lexical sort, paging and keyed selection.
/// The host owns datasets and authoritative operations; this component never fetches.
#[component]
pub fn DataTable(
    /// Unique prefix for filter/selection control relationships.
    #[prop(into)]
    id: String,
    /// Caption and labelled overflow region.
    #[prop(into)]
    caption: String,
    /// Root-owned controller over checked public rows.
    state: TableState,
    /// Opt-in neutral stripes.
    #[prop(optional)]
    striped: bool,
    /// Sticky column heads inside the labelled overflow.
    #[prop(optional)]
    sticky: bool,
    /// Optional per-column presentation; never changes values, sorting or selection.
    #[prop(optional)]
    column_layout: Vec<TableColumnLayout>,
) -> impl IntoView {
    // Store immutable presentation once; keyed header/cell closures only borrow it.
    let layout = StoredValue::new(column_layout);
    let column_layout =
        move |index| layout.with_value(|items| items.get(index).copied().unwrap_or_default());
    let filter = format!("{id}-filter");
    let select_all = format!("{id}-select-page");
    // Native checkboxes carry selection; CSS follows :checked even before JavaScript.
    // Duplicating this as a row attribute cannot track an early native checkbox edit.
    let rows_id = id.clone();
    let page_label = format!("{caption} pages");
    // Stable column positions retain focused sort controls while labels/data change.
    let columns = move || (0..state.column_count()).collect::<Vec<_>>();
    let header = move |index| {
        let presentation = column_layout(index);
        view! {<th scope="col" class:sp-sticky=sticky class:sp-table-nowrap=!presentation.wrap
        aria-sort=move ||match state.sort(){Some((column,SortDirection::Ascending)) if column==index=>"ascending",Some((column,SortDirection::Descending)) if column==index=>"descending",_=>"none"}>
        <button class="sp-table-sort" type="button" on:click=move |_|{state.toggle_sort(index);}>{move ||state.column(index)}
            {move ||icon(match state.sort() {
                Some((column,SortDirection::Ascending)) if column==index => IconKind::ArrowUp,
                Some((column,SortDirection::Descending)) if column==index => IconKind::ArrowDown,
                _ => IconKind::ArrowUpDown,
            })}</button>
            {presentation.min_width.map(|width|view!{<TableRule width/>})}</th>}
    };
    view! {<section class="sp-data-table" class:sp-striped=striped>
        <label for=filter.clone()>"Filter rows"</label><input class="sp-input" id=filter type="search" maxlength="1024" value=move ||state.filter() prop:value=move ||state.filter() on:input=move |event|{let _=state.set_filter(event_target_value(&event));}/>
        <Table caption=caption><thead><tr><th scope="col" class:sp-sticky=sticky><label class="sp-sr-only" for=select_all.clone()>"Select page"</label><input id=select_all type="checkbox"
            checked=move ||{let rows=state.rows();!rows.is_empty() && rows.iter().all(|row|state.selected().contains(&row.key))}
            prop:checked=move ||{let rows=state.rows();!rows.is_empty() && rows.iter().all(|row|state.selected().contains(&row.key))}
            prop:indeterminate=move ||{let rows=state.rows();let selected=state.selected();rows.iter().any(|row|selected.contains(&row.key)) && !rows.iter().all(|row|selected.contains(&row.key))}
            on:change=move |event|state.select_page(event_target_checked(&event))/></th>
            <For each=columns key=|index|*index children=header/>
        </tr></thead><tbody><For each=move ||state.rows() key=|row|row.key.clone() children=move |row|{
            let key=row.key.clone();let checked=key.clone();let cells_key=key.clone();
            let cells=move ||(0..state.column_count()).collect::<Vec<_>>();
            let cell_view=move |index:usize|{
                let key=cells_key.clone(); let presentation=column_layout(index);
                // Preserve old whitespace defaults for omitted layouts. Explicit wrapping
                // entries opt prose into hard line breaks as well as soft wrapping.
                let prose=presentation.wrap && layout.with_value(|items|index<items.len());
                view!{<td class:sp-table-nowrap=!presentation.wrap class:sp-table-prose=prose>{move ||state.cell(&key,index)}</td>}
            };let label=format!("Select row {}",row.key);let checkbox=format!("{rows_id}-row-{}",row.key);
            view!{<tr data-row-id=row.key.clone()><td><input type="checkbox" id=checkbox aria-label=label
                checked={let key=checked.clone();move ||state.selected().contains(&key)} prop:checked=move ||state.selected().contains(&checked) on:change=move |event|{state.select(&key,event_target_checked(&event));}/></td>
                <For each=cells key=|index|*index children=cell_view/></tr>}
        }/></tbody><tfoot><tr><td colspan=move ||state.column_count()+1><span role="status">{move ||format!("{} matching · {} selected",state.matching().len(),state.selected().len())}</span></td></tr></tfoot></Table>
        <p class="sp-empty" hidden=move ||!state.rows().is_empty()>"No matching rows"</p><Pagination label=page_label page=state.page()/>
    </section>}
}
