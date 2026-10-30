//! Application/Session page composition over generated roles and existing shared controls.
use super::roles::Relations;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stackpress_framework_admin::{
    Draft,
    pages::{Action, Form, PageSpec, PendingEdits},
};
use stackpress_framework_schema::{Role, SchemaResult};
use stackpress_leptos::components::{
    Button, ButtonType, FieldControl, Input, LayoutHead, LayoutLeft, LayoutMain, LayoutMenu,
    LayoutPanel, Textarea,
};

/// Compiled application's generated page functions; no dynamic model or role loading.
#[derive(Clone, Copy)]
pub struct Ui {
    /// Generated selected model descriptor.
    pub spec: fn(&str) -> SchemaResult<PageSpec>,
    /// Generated create/update/copy controls.
    pub form: fn(&str, Form) -> SchemaResult<AnyView>,
    /// Generated query controls.
    pub filters: fn(&str, &Draft, &str) -> SchemaResult<AnyView>,
    /// Generated list roles.
    pub table: fn(&str, &Value, &str) -> SchemaResult<AnyView>,
    /// Generated read roles.
    pub detail: fn(&str, &Value, &str) -> SchemaResult<AnyView>,
}
/// Source-selected public page state; stored secrets deliberately remain visible.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Management {
    /// Selected generated module, application or session.
    pub module: String,
    /// Original selected admin action.
    pub action: Action,
    /// Source row or search result array, including joined data.
    pub data: Value,
    /// Original aggregate request fields, retaining relation filter/default inputs.
    pub query: Value,
    /// Actual source relation rows and labels.
    pub relations: Relations,
    /// Source CSRF field name and generated outgoing token, where that page uses it.
    pub csrf: Option<(String, String)>,
    /// Selected admin config: name/base/menu, plus original brand/view values.
    pub config: Value,
    /// Explicit saved theme; unset consumes the shared initial theme bootstrap.
    pub theme: Option<stackpress_framework_view::preferences::ThemePreference>,
    /// Public model error, retaining the same submitted draft.
    pub error: Option<String>,
}
impl Management {
    /// Original admin.base fallback, retaining an explicitly empty string.
    pub fn base(&self) -> &str {
        self.config
            .pointer("/admin/base")
            .and_then(Value::as_str)
            .unwrap_or("/admin")
    }
    /// Current source route, including its original path identity spelling.
    pub fn url(&self, action: Action, id: Option<&str>) -> String {
        format!(
            "{}/{}/{}{}",
            self.base(),
            self.module,
            action.name(),
            id.map(|id| format!("/{id}")).unwrap_or_default()
        )
    }
}
/// Render generated controls with source action paths, compact links and native form fallback.
#[component]
pub fn ManagementPage(
    /// Same source DTO for SSR/hydration/CSR.
    page: Management,
    /// Compile-selected generated functions.
    ui: Ui,
    /// Browser transport submits the exact current draft; absent uses native forms.
    #[prop(optional)]
    submit: Option<Callback<Value>>,
    /// Browser query transport, independent of mutation submission.
    #[prop(optional)]
    search: Option<Callback<Value>>,
) -> impl IntoView {
    provide_context(page.relations.clone());
    let spec = (ui.spec)(&page.module);
    let Ok(spec) = spec else {
        return view! {<p role="alert">"Unknown management module"</p>}.into_any();
    };
    let id = page
        .query
        .get("id")
        .and_then(Value::as_str)
        .or_else(|| page.data.get("id").and_then(Value::as_str));
    let title = format!("{} {}", spec.model.label, action_label(page.action));
    // Source detail actions distinguish active rows from removed rows. Other pages retain
    // collection actions; the breadcrumb provides the way back to search.
    let active = page
        .data
        .get("active")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let actions: &[Action] = match page.action {
        Action::Search => &[Action::Import, Action::Export, Action::Create],
        Action::Detail | Action::Remove | Action::Restore => {
            if active {
                &[Action::Copy, Action::Update, Action::Remove]
            } else {
                &[Action::Copy, Action::Update, Action::Restore]
            }
        }
        _ => &[],
    };
    let links = actions.iter().copied().map(|action| {
        let collection = matches!(action, Action::Search | Action::Create | Action::Import | Action::Export);
        view! {<a class=format!("api-action api-action-{}",action.name()) href=page.url(action,if collection {None} else {id})>
            {super::icons::named(action_icon(action))}<span>{action_label(action)}</span>
        </a>}
    }).collect_view();
    let content = content(&page, ui, &spec, submit, search);
    let base = page.base().to_owned();
    let model_label = if page.module == "application" {
        "Apps"
    } else {
        "Sessions"
    };
    let menu = page
        .config
        .pointer("/admin/menu")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            let label = super::string(item.get("name").or_else(|| item.get("label")), "");
            let href = super::string(
                item.get("path")
                    .or_else(|| item.get("href"))
                    .or_else(|| item.get("url")),
                "",
            );
            view! {<a href=href>{label}</a>}
        })
        .collect_view();
    let theme = page.theme.map(|theme| theme.as_str()).unwrap_or("light");
    let brand = super::string(page.config.pointer("/brand/name"), "Stackpress");
    let admin_name = super::string(page.config.pointer("/admin/name"), "Admin");
    let is_detail = matches!(
        page.action,
        Action::Detail | Action::Remove | Action::Restore
    );
    let sidebar_name = admin_name.clone();
    let applications_url = format!("{base}/application/search");
    let brand_url = applications_url.clone();
    let sessions_url = format!("{base}/session/search");
    let search_url = page.url(Action::Search, None);
    let is_application = page.module == "application";
    let pending_theme = page.theme.is_none();
    let action = page.action;
    let sessions_tab = (is_application && is_detail)
        .then(|| {
            id.map(|id| {
        view! {<a href=format!("{base}/session/search?eq[applicationId]={id}")>"Sessions"</a>}
    })
        })
        .flatten();
    let error = page.error.map(|error| view! {<p role="alert">{error}</p>});
    // Existing semantic layout slots carry the original 220px sidebar/60px header shell.
    // This application-owned skin does not replace generated controls or shared layouts.
    view! {<div class="sp-root api-admin-shell" data-theme=theme data-initial-theme-pending=pending_theme.then_some("")>
        <LayoutPanel><LayoutLeft label="Administration">
            <a class="api-brand" href=brand_url>{super::icons::named("blocks")}<span>{brand}</span></a>
            <p class="api-menu-label">{sidebar_name}</p>
            <LayoutMenu label="Models">
                <a aria-current=is_application.then_some("page") href=applications_url>{super::icons::named("blocks")}<span>"Apps"</span></a>
                <a aria-current=(!is_application).then_some("page") href=sessions_url>{super::icons::named("key")}<span>"Sessions"</span></a>{menu}
            </LayoutMenu>
        </LayoutLeft><div class="api-workspace">
            <LayoutHead><span>{admin_name}</span><span class="api-section-label">{model_label}</span></LayoutHead>
            <LayoutMain><div class="api-admin">
                <nav class="api-crumbs" aria-label="Breadcrumb"><a href=search_url>{super::icons::named(if is_application {"blocks"} else {"key"})}{model_label}</a>{super::icons::named("chevron-right")}<span aria-current="page">{action_label(action)}</span></nav>
                <div class="api-page-heading"><h1>{title}</h1><nav class="api-actions" aria-label="Actions">{links}</nav></div>
                {is_detail.then(||view!{<nav class="api-tabs" aria-label="Record"><span aria-current="page">"Info"</span>{sessions_tab}</nav>})}
                <div class="api-content">{error}{content}</div>
            </div></LayoutMain>
        </div></LayoutPanel>
    </div>}.into_any()
}
/// Visible source action labels are title case; route names remain lowercase.
fn action_label(action: Action) -> &'static str {
    match action {
        Action::Search => "Search",
        Action::Create => "Create",
        Action::Detail => "Detail",
        Action::Update => "Update",
        Action::Copy => "Copy",
        Action::Remove => "Remove",
        Action::Restore => "Restore",
        Action::Import => "Import",
        Action::Export => "Export",
    }
}
/// Source action meanings expressed with the selected Lucide family.
fn action_icon(action: Action) -> &'static str {
    match action {
        Action::Search => "search",
        Action::Create => "plus",
        Action::Detail => "file-text",
        Action::Update => "square-pen",
        Action::Copy => "copy",
        Action::Remove => "trash",
        Action::Restore => "circle-check",
        Action::Import => "upload",
        Action::Export => "download",
    }
}
/// Project joined rows to exactly the generated scalar public fields; joined labels stay context-local.
fn scalar(spec: &PageSpec, row: &Value) -> Value {
    let mut output = serde_json::Map::new();
    for field in spec
        .model
        .fields
        .iter()
        .filter(|field| field.relation.is_none())
    {
        if let Some(value) = row.get(&field.wire_name) {
            output.insert(field.wire_name.clone(), value.clone());
        }
    }
    Value::Object(output)
}
/// Keep native source routes while browser submission reads rich shared controls' draft state.
fn content(
    page: &Management,
    ui: Ui,
    spec: &PageSpec,
    submit: Option<Callback<Value>>,
    search: Option<Callback<Value>>,
) -> AnyView {
    let model = page.module.clone();
    if page.action == Action::Search {
        let draft = Draft::new(
            &spec.model.fields,
            page.query.get("eq").unwrap_or(&json!({})),
            Role::Filter,
        );
        let filters = draft
            .as_ref()
            .map_err(|_| {
                stackpress_framework_schema::SchemaError::new(
                    stackpress_framework_schema::SchemaErrorKind::Invalid,
                    "query draft unavailable",
                )
            })
            .and_then(|draft| (ui.filters)(&model, draft, "api-filter"));
        let q = RwSignal::new(super::string(page.query.get("q"), ""));
        let rows = page.data.as_array().cloned().unwrap_or_default();
        let table = (ui.table)(
            &model,
            &json!({"rows":rows.iter().map(|row|scalar(spec,row)).collect::<Vec<_>>()}),
            "api-table",
        );
        // Shared tables have no row-action slot yet; retain the approved fallback until
        // framework-admin supplies a trailing action column, without forking its emitter.
        let links = rows.iter().filter_map(|row|row.get("id").and_then(Value::as_str)).map(|id| {
            view! {<a class="api-row-link" aria-label=format!("Detail {id}") href=page.url(Action::Detail,Some(id))>
                {super::icons::named("file-text")}<span>"Detail"</span>
            </a>}
        }).collect_view();
        // Upstream keeps collection text search separate from the filter drawer. A native
        // disclosure retains the same panel distinction without requiring JavaScript.
        let filter_search = search;
        let filter_draft = draft.as_ref().ok().cloned();
        return view! {
            <div class="api-search-toolbar">
                <form class="api-search-form" method="get" on:submit=move |event| {
                    if let Some(search)=&search {event.prevent_default();if let Ok(draft)=&draft && let Ok(eq)=draft.snapshot(){search.run(json!({"q":q.get_untracked(),"eq":eq}));}}
                }><FieldControl for_id="api-search" label="Search"><Input id="api-search" name="q" value=q/></FieldControl>
                <Button button_type=ButtonType::Submit>{super::icons::named("search")}"Search"</Button></form>
                <details class="api-filter-panel"><summary>{super::icons::named("menu")}"Filters"</summary>
                    <form method="get" on:submit=move |event| {
                        if let Some(search)=&filter_search {event.prevent_default();if let Some(draft)=&filter_draft && let Ok(eq)=draft.snapshot(){search.run(json!({"q":q.get_untracked(),"eq":eq}));}}
                    }><h2>"Filters"</h2><input type="hidden" name="q" value=move ||q.get()/>{role(filters)}
                    <Button button_type=ButtonType::Submit>"Filter"</Button></form>
                </details>
            </div>
            <div class="api-table" role="region" aria-label="Search results" tabindex="0">{role(table)}</div><nav aria-label="Rows">{links}</nav>
        }.into_any();
    }
    if matches!(page.action, Action::Create | Action::Update | Action::Copy) {
        let mut initial = if page.action == Action::Create {
            page.query.clone()
        } else {
            page.data.clone()
        };
        if let (Some(initial), Some(query)) = (initial.as_object_mut(), page.query.as_object()) {
            initial.extend(query.clone());
        }
        let draft = Draft::new(&spec.model.fields, &initial, Role::Field);
        let Ok(draft) = draft else {
            return view! {<p role="alert">"Cannot represent this draft"</p>}.into_any();
        };
        let pending = PendingEdits::default();
        provide_context(pending.clone());
        let fields = (ui.form)(
            &model,
            Form {
                action: page.action,
                draft: draft.clone(),
                instance: "api-form".into(),
                locked: None,
            },
        );
        let csrf = page.csrf.clone();
        let query = page.query.clone();
        return view!{<form method="post" on:submit=move |event| {
            if let Some(submit)=&submit {event.prevent_default();if pending.blocked(){return;}
                if let Ok(mut data)=draft.snapshot() {if let Some(object)=data.as_object_mut() {
                    if let Some(id)=query.get("id") {object.insert("id".into(),id.clone());}
                    if let Some((name,token))=&csrf {object.insert(name.clone(),Value::String(token.clone()));}
                }submit.run(data);}
            }
        }>{csrf_input(&page.csrf)}{role(fields)}<Button button_type=ButtonType::Submit>"Save"</Button></form>}.into_any();
    }
    if page.action == Action::Import {
        return view! {<CsvImport submit/>}.into_any();
    }
    let detail = role((ui.detail)(&model, &scalar(spec, &page.data), "api-detail"));
    let confirm=matches!(page.action,Action::Remove|Action::Restore).then(||view!{<form method="post">{csrf_input(&page.csrf)}<input type="hidden" name="confirmed" value="1"/><Button button_type=ButtonType::Submit>"Confirm"</Button></form>});
    let related = if page.module == "application" {
        page.data.get("id").and_then(Value::as_str).map(|id|view!{<nav aria-label="Related sessions"><a class="api-action api-action-create" href=format!("{}/session/create?applicationId={id}",page.base())>{super::icons::named("plus")}"Create session"</a></nav>})
    } else {
        None
    };
    view! {<div class="api-read">{detail}</div>{confirm}{related}}.into_any()
}
/// Source token generation is page-owned and its hidden field remains plain form data.
fn csrf_input(csrf: &Option<(String, String)>) -> Option<AnyView> {
    csrf.as_ref().map(|(name, token)| {
        view! {<input type="hidden" name=name.clone() value=token.clone()/>}.into_any()
    })
}
/// Preserve representational failures as visible errors instead of silently empty generated UI.
fn role(value: SchemaResult<AnyView>) -> AnyView {
    value.unwrap_or_else(|_| {
        view! {<p role="alert">"Cannot represent this generated role"</p>}.into_any()
    })
}

/// Existing textarea and native file selection feed the original rows-array batch input.
#[component]
fn CsvImport(
    /// Optional native-form transport selected by the browser host.
    #[prop(optional_no_strip)]
    submit: Option<Callback<Value>>,
) -> impl IntoView {
    let csv = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    view! {<form method="post" on:submit=move |event| {
        if let Some(submit)=&submit {event.prevent_default();match import_csv(&csv.get_untracked()) {Ok(rows)=>submit.run(json!({"rows":rows})),Err(_)=>error.set("CSV parsing error".into())}}
    }><FieldControl for_id="api-csv" label="CSV file"><input id="api-csv" type="file" accept=".csv,text/csv" data-api-csv/></FieldControl>
    <FieldControl for_id="api-csv-text" label="CSV rows"><Textarea id="api-csv-text" name="csv" value=csv required=true/></FieldControl>
    <p role="alert">{move ||error.get()}</p><Button button_type=ButtonType::Submit>"Import"</Button></form>}
}
/// Parse source header/value strings with the existing shared CSV parser, without write policy.
/// # Errors
/// Existing parser representation ceilings/quote errors fail before constructing a batch.
pub fn import_csv(csv: &str) -> SchemaResult<Vec<Value>> {
    let records = stackpress_framework_admin::transfer::parse(csv.as_bytes(), Default::default())?;
    let Some(head) = records.first() else {
        return Ok(Vec::new());
    };
    Ok(records
        .iter()
        .skip(1)
        .filter(|row| !row.iter().all(String::is_empty))
        .map(|row| {
            Value::Object(
                head.iter()
                    .cloned()
                    .zip(row.iter().cloned().map(Value::String))
                    .collect(),
            )
        })
        .collect())
}
