//! Actual generated CRUD page consumers. Transport and authority remain outside this module.
use crate::ui;
use leptos::prelude::*;
use serde_json::{Value, json};
use stackpress_framework_admin::{
    Draft,
    pages::{Action, Command, Form, ParentScope, Reply, Snapshot, WriteState},
    search::{SearchQuery, encode},
};
use stackpress_framework_schema::{Role, SchemaResult};

/// One selected generated page; loaders supplied its actual public operation result.
#[component]
pub fn Admin(
    /// Host-selected public state, identical in SSR and initial hydrate/CSR.
    snapshot: Snapshot,
    /// Platform-owned finite HTTP adapter; no transport choice is read from schema props.
    transport: Callback<Command>,
) -> impl IntoView {
    match contents(snapshot, transport) {
        Ok(view) => view,
        Err(_) => view! {<p role="alert">"Unable to render this public page"</p>}.into_any(),
    }
}

/// Fixed same-mount action link. Keys and parent scope remain escaped URL data.
pub fn link(snapshot: &Snapshot, action: Action, key: &Value) -> String {
    let mut query = Vec::new();
    if matches!(
        action,
        Action::Detail | Action::Update | Action::Copy | Action::Remove | Action::Restore
    ) {
        query.push(format!("key={}", encode(&key.to_string())));
    }
    if action == Action::Export {
        query.push(snapshot.query.query().unwrap_or_default());
    }
    if let Some(parent) = &snapshot.parent {
        query.push(format!(
            "parent={}",
            encode(&serde_json::to_string(parent).expect("public scope serialization"))
        ));
    }
    format!(
        "{}{}/{}{}{}",
        snapshot.mount,
        snapshot.spec.module,
        action.name(),
        if query.is_empty() { "" } else { "?" },
        query.join("&")
    )
}

/// One root owns drafts, acknowledgement state and returned public data independently.
fn contents(snapshot: Snapshot, transport: Callback<Command>) -> SchemaResult<AnyView> {
    let config = stackpress_framework_admin::pages::AdminConfig::new(
        &snapshot.mount,
        ui::pages::MODELS
            .iter()
            .map(|module| module.to_string())
            .collect(),
    )?;
    let data = RwSignal::new(snapshot.data.clone());
    let status = RwSignal::new(WriteState::Idle);
    let fields = RwSignal::new(Vec::<stackpress_framework_schema::FieldIssue>::new());
    provide_context(stackpress_framework_admin::pages::FormIssues(fields));
    let snapshot = StoredValue::new(snapshot);
    let current = snapshot.get_value();
    provide_context(stackpress_framework_admin::pages::LockedFields(
        current
            .parent
            .as_ref()
            .map(|parent| vec![parent.field.clone()])
            .unwrap_or_default(),
    ));
    let heading = format!("{} · {}", current.spec.model.label, current.action.name());
    let actions = current
        .allowed
        .iter()
        .filter(|action| {
            **action != current.action
                && (**action == Action::Create
                    || **action == Action::Search
                    || matches!(**action, Action::Import | Action::Export)
                    || !current.key.as_object().is_none_or(|key| key.is_empty()))
        })
        .map(|action| (*action, link(&current, *action, &current.key)))
        .collect();
    let body = if current.action == Action::Import {
        crate::transfer::form(current.clone(), transport)
    } else if current.action == Action::Search {
        search(snapshot, data, status, transport)?
    } else if current.action == Action::Detail {
        detail(snapshot)?
    } else {
        editor(snapshot, data, status, fields, transport)?
    };
    Ok(view!{<stackpress_framework_admin::pages::LayoutAdmin title=heading config actions>
        {current.parent.as_ref().map(|parent|view!{<p id="parent-scope">{format!("Related to {} {}",parent.module,parent.value)}</p>})}
        <p id="operation-status" role="status">{move ||status.get().message()}</p>
        <ul id="operation-errors" role="alert">{move ||stackpress_framework_admin::presentation::issues(&snapshot.get_value().spec.model,fields.get()).into_iter().map(|issue|view!{<li>{issue.message}</li>}).collect_view()}</ul>
        {body}</stackpress_framework_admin::pages::LayoutAdmin>}.into_any())
}

/// Complete one request in its existing root; errors never replace draft signals.
fn complete(
    data: RwSignal<Value>,
    status: RwSignal<WriteState>,
    fields: RwSignal<Vec<stackpress_framework_schema::FieldIssue>>,
    write: bool,
    validation: Option<(stackpress_framework_schema::UiModel, Value)>,
) -> Callback<Reply> {
    Callback::new(move |reply: Reply| {
        let state = match (write, WriteState::reply(&reply)) {
            (false, WriteState::Unknown) => {
                WriteState::Failed("Unable to load records; no mutation was submitted".into())
            }
            (_, state) => state,
        };
        let issues = reply
            .envelope
            .pointer("/errors/fields")
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        fields.set(if let Some((model, values)) = &validation {
            stackpress_framework_admin::presentation::submitted_issues(model, issues, values)
        } else {
            issues
        });
        if state == WriteState::Saved {
            if let Some(result) = reply.envelope.get("results") {
                data.set(result.clone());
            }
            status.set(if write {
                WriteState::Saved
            } else {
                WriteState::Idle
            });
        } else {
            status.set(state);
        }
    })
}

/// Creation/copy/update and explicit remove/restore share one exact draft and mutation gate.
fn editor(
    snapshot: StoredValue<Snapshot>,
    data: RwSignal<Value>,
    status: RwSignal<WriteState>,
    fields: RwSignal<Vec<stackpress_framework_schema::FieldIssue>>,
    transport: Callback<Command>,
) -> SchemaResult<AnyView> {
    let seed = snapshot.get_value();
    provide_context(stackpress_framework_admin::pages::EditorAction(seed.action));
    let pending = stackpress_framework_admin::pages::PendingEdits::default();
    provide_context(pending.clone());
    let submission_pending = pending.clone();
    let mut value =
        stackpress_framework_admin::pages::input(&seed.spec, seed.action, seed.data.clone());
    if let Some(parent) = &seed.parent
        && let Some(object) = value.as_object_mut()
    {
        object.insert(parent.field.clone(), parent.value.clone());
    }
    let draft = Draft::new(&seed.spec.model.fields, &value, Role::Field)?;
    let observed = draft.clone();
    let names = seed
        .spec
        .model
        .fields
        .iter()
        .map(|field| field.wire_name.clone())
        .collect::<Vec<_>>();
    let edits = pending.clone();
    // A previous acknowledgement describes only the saved draft, never subsequent edits.
    Effect::watch(
        move || {
            (
                names
                    .iter()
                    .filter_map(|name| observed.get(name))
                    .map(|value| value.get())
                    .collect::<Vec<_>>(),
                edits.blocked(),
            )
        },
        move |_, _, _| {
            if status.get_untracked() == WriteState::Saved {
                status.set(WriteState::Idle);
            }
        },
        false,
    );
    let content = if matches!(seed.action, Action::Remove | Action::Restore) {
        view! {<p>"This action requires explicit confirmation."</p>}.into_any()
    } else {
        ui::pages::form(
            &seed.spec.module,
            Form {
                action: seed.action,
                draft: draft.clone(),
                instance: "admin".into(),
                locked: seed.parent.as_ref().map(|parent| parent.field.clone()),
            },
        )?
    };
    let confirmation = RwSignal::new(false);
    let confirm = matches!(seed.action, Action::Remove | Action::Restore);
    let submitted = draft.clone();
    let button = if confirm {
        format!("Confirm {}", seed.action.name())
    } else {
        "Save".into()
    };
    let initial = snapshot.get_value();
    let url = endpoint(
        &initial,
        if initial.action == Action::Copy {
            Action::Create
        } else {
            initial.action
        },
    );
    Ok(view!{<section><p>{if seed.action==Action::Copy {"Copy begins without identity. Supply a new identity before saving."}else{"The draft stays here if validation, authorization or acknowledgement fails."}}</p>
        <form id="admin-form" aria-label="Record form" method="post" action=url novalidate on:submit=move |event| {
            event.prevent_default();
            if submission_pending.blocked() {status.set(WriteState::Failed("Finish or cancel the edited field before saving".into()));return;}
            if matches!(status.get_untracked(),WriteState::Pending|WriteState::Unknown) || confirm && !confirmation.get_untracked() {return;}
            let seed=snapshot.get_value();
            let result=if confirm {Ok(json!({"key":seed.key}))} else {submitted.snapshot().and_then(|value|ui::pages::input(&seed.spec.module,seed.action,stackpress_framework_admin::pages::input(&seed.spec,if seed.action==Action::Update {Action::Update} else {Action::Create},value))).map(|value|if seed.action==Action::Update {json!({"key":seed.key,"patch":value})}else{json!({"input":value})})};
            match result {Ok(input)=>{status.set(WriteState::Pending);fields.set(Vec::new());let validation=Some((seed.spec.model.clone(),input.get(if seed.action==Action::Update {"patch"}else{"input"}).cloned().unwrap_or(Value::Null)));transport.run(Command {endpoint:endpoint(&seed,if seed.action==Action::Copy {Action::Create}else{seed.action}),input,csrf:seed.csrf,write:true,complete:complete(data,status,fields,true,validation)});},Err(_)=>status.set(WriteState::Failed("Invalid form values; draft preserved".into()))}
        }>{content}
        {confirm.then(||view!{<label><input id="confirm-action" type="checkbox" checked=move ||confirmation.get() on:change=move |event|confirmation.set(event_target_checked(&event))/>"I confirm this action"</label>})}
        <button type="submit" disabled=move ||matches!(status.get(),WriteState::Pending|WriteState::Unknown) || confirm && !confirmation.get() || pending.blocked()>{button}</button>
        </form><output id="admin-draft" hidden>{move ||{for field in &draft_fields(&snapshot.get_value()) {let _=draft.get(field).map(|value|value.get());}draft.snapshot().map(|value|value.to_string()).unwrap_or_default()}}</output>
        <output id="operation-result" hidden>{move ||data.get().to_string()}</output>
        {move ||{let seed=snapshot.get_value();(status.get()==WriteState::Saved).then(||stackpress_framework_admin::pages::key(&seed.spec,&data.get()).ok()).flatten().map(|key|view!{<a id="saved-record" href=link(&seed,Action::Detail,&key)>"View record"</a>})}}
        {(seed.spec.soft_remove && seed.action==Action::Remove).then(||view!{<a id="restore-link" href=link(&seed,Action::Restore,&seed.key)>"Restore record"</a>})}
        </section>}.into_any())
}

/// Exact fields observed before the untracked snapshot; this never submits hidden DOM controls.
fn draft_fields(snapshot: &Snapshot) -> Vec<String> {
    snapshot
        .spec
        .model
        .fields
        .iter()
        .map(|field| field.wire_name.clone())
        .collect()
}

/// Build only one compiled same-model API endpoint, preserving authorized related scope.
pub(crate) fn endpoint(snapshot: &Snapshot, action: Action) -> String {
    let mut endpoint = format!(
        "{}api/{}/{}",
        snapshot.mount,
        snapshot.spec.module,
        action.name()
    );
    if let Some(parent) = &snapshot.parent {
        endpoint.push_str(&format!(
            "?parent={}",
            encode(&serde_json::to_string(parent).expect("public scope serialization"))
        ));
    }
    endpoint
}

/// Bounded list/search uses host totals, actual query contracts and generated filter roles.
fn search(
    snapshot: StoredValue<Snapshot>,
    data: RwSignal<Value>,
    status: RwSignal<WriteState>,
    transport: Callback<Command>,
) -> SchemaResult<AnyView> {
    let seed = snapshot.get_value();
    let query = RwSignal::new(seed.query.clone());
    let text = RwSignal::new(seed.query.q.clone().unwrap_or_default());
    // Offer text search only when the generated public query admits searchable fields.
    let searchable = seed.spec.model.fields.iter().any(|field| field.searchable);
    let filter_seed = Value::Object(
        seed.query
            .filters
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    );
    let draft = Draft::new(&seed.spec.model.fields, &filter_seed, Role::Filter)?;
    let filters = ui::pages::filters(&seed.spec.module, &draft, "admin-search")?;
    let fields = use_context::<stackpress_framework_admin::pages::FormIssues>()
        .expect("root issues context")
        .0;
    let request = Callback::new(move |next: SearchQuery| {
        let seed = snapshot.get_value();
        let checked = next
            .operation(&seed.spec.model)
            .and_then(|value| ui::pages::query(&seed.spec.module, value));
        match checked {
            Ok(input) => {
                query.set(next);
                status.set(WriteState::Pending);
                transport.run(Command {
                    endpoint: endpoint(&seed, Action::Search),
                    input,
                    csrf: seed.csrf,
                    write: false,
                    complete: complete(data, status, fields, false, None),
                });
            }
            Err(_) => status.set(WriteState::Failed("Invalid search state".into())),
        }
    });
    let filtered = draft.clone();
    let orders=seed.spec.model.fields.iter().filter(|field|field.sortable).map(|field|{
        let name=field.wire_name.clone();let label=format!("Order {}",field.label);
        view!{<button type="button" data-order=name.clone() on:click=move |_|{let mut next=query.get();next.cycle_order(&name);request.run(next);}>{label}</button>}
    }).collect_view();
    Ok(view!{<section><form aria-label="Search form" novalidate on:submit=move |event|{
        event.prevent_default();if status.get_untracked()==WriteState::Pending {return;}
        let mut next=query.get();next.q=(!text.get().is_empty()).then(||text.get());next.skip=0;
        if let Ok(values)=filtered.snapshot() && let Some(object)=values.as_object() {next.filters=object.iter().filter(|(_,value)|!value.is_null()).map(|(key,value)|(key.clone(),value.clone())).collect();}
        if let Some(parent)=snapshot.get_value().parent {next.filter(&parent.field,parent.value);}
        request.run(next);
    } >{searchable.then(||view!{<label for="admin-q">"Search text"</label><input id="admin-q" value=move ||text.get() prop:value=move ||text.get() on:input=move |event|text.set(event_target_value(&event))/>})}{filters}
        <label for="admin-limit">"Page size"</label><select id="admin-limit" on:change=move |event|{if let Ok(limit)=event_target_value(&event).parse(){query.update(|query|query.limit=limit);}}>
        {[1u16,5,20,100].into_iter().map(|limit|view!{<option value=limit selected=move ||query.get().limit==limit prop:selected=move ||query.get().limit==limit>{limit}</option>}).collect_view()}</select>
        <button type="submit" disabled=move ||status.get()==WriteState::Pending>"Search"</button></form>{orders}
        <p id="search-total">{move ||data.get().get("total").map(Value::to_string).unwrap_or_else(||"Total unavailable".into())}</p>
        <div class="admin-actions" role="group" aria-label="Pagination"><a id="search-deep-link" href=move ||{let seed=snapshot.get_value();format!("{}?{}{}",link(&Snapshot {parent:None,..seed.clone()},Action::Search,&json!({})),query.get().query().unwrap_or_default(),seed.parent.as_ref().map(|parent|format!("&parent={}",encode(&serde_json::to_string(parent).expect("public scope")))).unwrap_or_default())}>"Link to this search"</a>
        {snapshot.get_value().allowed.contains(&Action::Export).then(||view!{<a id="export-csv" href=move ||{let seed=snapshot.get_value();link(&Snapshot{query:query.get(),..seed},Action::Export,&json!({}))}>"Export all filtered rows (CSV)"</a>})}
        <button id="previous-page" disabled=move ||query.get().skip==0 || status.get()==WriteState::Pending on:click=move |_|{let mut next=query.get();next.skip=next.skip.saturating_sub(u64::from(next.limit));request.run(next);}>"Previous"</button>
        <button id="next-page" disabled=move ||{let query=query.get();status.get()==WriteState::Pending || data.get().get("total").and_then(Value::as_u64).is_none_or(|total|query.skip+u64::from(query.limit)>=total)} on:click=move |_|{let mut next=query.get();next.skip=next.skip.saturating_add(u64::from(next.limit));request.run(next);}>"Next"</button></div>
        <div id="public-table">{move ||ui::pages::table(&snapshot.get_value().spec.module,&data.get(),"admin-list").unwrap_or_else(|_|view!{<p role="alert">"Invalid public rows"</p>}.into_any())}</div>
        <ul aria-label="Records">{move ||data.get().get("rows").and_then(Value::as_array).cloned().unwrap_or_default().into_iter().filter_map(|value|{
            let row=value.get("row").unwrap_or(&value);let seed=snapshot.get_value();let key=stackpress_framework_admin::pages::key(&seed.spec,row).ok()?;
            Some(view!{<li><a href=link(&seed,Action::Detail,&key)>{stackpress_framework_admin::presentation::record_label(&seed.spec.model,row)}</a></li>})
        }).collect_view()}</ul></section>}.into_any())
}

/// Detail keeps actual selected operation data and finite related navigation in the same scope.
fn detail(snapshot: StoredValue<Snapshot>) -> SchemaResult<AnyView> {
    let seed = snapshot.get_value();
    let key = stackpress_framework_admin::pages::key(&seed.spec, &seed.data)?;
    let related=ui::pages::MODELS.iter().filter_map(|module|{
        let child=ui::pages::spec(module).ok()?;
        let mapping=child.model.fields.iter().filter_map(|field|field.relation.as_ref()).find(|relation|relation.target==seed.spec.model.id)?.mapping.as_ref()?;
        let value=key.get(&mapping.foreign)?.clone();
        let parent=ParentScope {module:seed.spec.module.clone(),key:key.clone(),field:mapping.local.clone(),value};
        let related=Snapshot {spec:child,parent:Some(parent),..seed.clone()};
        Some(view!{<li><a data-related=module.to_string() href=link(&related,Action::Search,&json!({}))>{format!("Related {module}")}</a></li>})
    }).collect_view();
    let detail = ui::pages::detail(&seed.spec.module, &seed.data, "admin-detail")?;
    let identity=key.as_object().expect("public key object").iter().map(|(name,value)|view!{<div><dt>{name.clone()}</dt><dd>{value.as_str().map(str::to_owned).unwrap_or_else(||value.to_string())}</dd></div>}).collect_view();
    Ok(view!{<section><dl aria-label="Identity">{identity}</dl><div id="record-detail">{detail}</div><ul aria-label="Related records">{related}</ul></section>}.into_any())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Acknowledgements update status/issues without replacing a draft or inventing read writes.
    #[test]
    fn completion_keeps_public_errors_and_honest_states() {
        let owner = Owner::new();
        owner.with(|| {
            let data=RwSignal::new(json!({"original":true}));let status=RwSignal::new(WriteState::Idle);
            let issues=RwSignal::new(Vec::new());let write=complete(data,status,issues,true,None);
            write.run(Reply{status:400,envelope:json!({"code":400,"error":"input validation failed","errors":{"fields":[{"path":["name"],"code":"invalid","message":"field validation failed"}]}})});
            assert!(matches!(status.get_untracked(),WriteState::Failed(_)));assert_eq!(issues.get_untracked()[0].path,["name"]);assert_eq!(data.get_untracked(),json!({"original":true}));
            write.run(Reply{status:500,envelope:json!({"code":500,"errors":{"outcome":"unknown"}})});assert_eq!(status.get_untracked(),WriteState::Unknown);
            let read=complete(data,status,issues,false,None);read.run(Reply{status:500,envelope:json!({"code":500,"errors":{"outcome":"unknown"}})});assert!(matches!(status.get_untracked(),WriteState::Failed(_)));
            read.run(Reply{status:200,envelope:json!({"code":200,"results":{"rows":[],"total":0}})});assert_eq!(status.get_untracked(),WriteState::Idle);assert_eq!(data.get_untracked()["total"],0);
            write.run(Reply{status:200,envelope:json!({"code":200,"results":{"id":"created"}})});assert_eq!(status.get_untracked(),WriteState::Saved);assert!(issues.get_untracked().is_empty());
        });
        owner.cleanup();
    }
}
