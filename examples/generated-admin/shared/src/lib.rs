//! Actual generated role consumers over the generated-workspace three-model fixture.
//!
//! Only public projections enter role wrappers. Stores and host tool dependencies are not
//! included in this crate's module graph. This example performs no database writes.

#[cfg_attr(not(feature = "format"), path = "generated/contracts/mod.rs")]
#[cfg_attr(feature = "format", path = "generated-format/contracts/mod.rs")]
#[allow(
    missing_docs,
    reason = "the writer owns aggregate declarations; generated leaves carry Rustdoc"
)]
/// Exact generated value contracts, independent of stores.
pub mod contracts;
pub mod crud;
pub mod plugin;
pub mod transfer;
#[cfg_attr(not(feature = "format"), path = "generated/ui/mod.rs")]
#[cfg_attr(feature = "format", path = "generated-format/ui/mod.rs")]
#[allow(
    missing_docs,
    reason = "the writer owns aggregate declarations; generated leaves carry Rustdoc"
)]
/// Generated model role functions over public contracts.
pub mod ui;

use leptos::prelude::*;
use serde_json::json;
use stackpress_framework_admin::Draft;
use stackpress_framework_schema::{Role, SchemaResult};

/// Render fixture-backed list, view, field, filter and span roles for all three real models.
#[component]
pub fn Admin() -> impl IntoView {
    match contents() {
        Ok(contents) => contents,
        Err(_) => view! { <p role="alert">"Unable to render public fixture"</p> }.into_any(),
    }
}

/// Construct only generated Public DTOs; stored rows/host fields never seed the root.
fn contents() -> SchemaResult<AnyView> {
    let profile: contracts::public::PublicProfile =
        serde_json::from_value(json!({"id":"p1", "name":"Ada", "active":false, "address":null}))
            .map_err(codec)?;
    let project: contracts::public::PublicProject = serde_json::from_value(
        json!({"id":"j1", "ownerId":"p1", "title":"Compile roles", "state":"open", "tags":[]}),
    )
    .map_err(codec)?;
    let task: contracts::public::PublicTask = serde_json::from_value(
        json!({"id":"t1", "projectId":"j1", "title":"Review generated UI", "done":false,"priority":0}),
    )
    .map_err(codec)?;
    let proof: contracts::public::PublicRoleProof =
        serde_json::from_value(json!({"id":"proof", "text":null, "flag":false, "zero":0}))
            .map_err(codec)?;
    let proof_draft = Draft::new(
        &ui::role_proof::descriptors()?,
        &json!({"rows":["", "0"],"metadata":{"initial":"0"},"score":0,"slug":"A New Slug"}),
        Role::Field,
    )?;
    let proof_filter = Draft::new(&ui::role_proof::descriptors()?, &json!({}), Role::Filter)?;
    let proof_span = Draft::new(&ui::role_proof::descriptors()?, &json!({}), Role::Span)?;
    let proof_roles = (
        ui::role_proof::field(&proof_draft, "proof")?,
        ui::role_proof::filter(&proof_filter, "proof")?,
        ui::role_proof::span(&proof_span, "proof")?,
        ui::role_proof::list(&proof, "proof-list")?,
        ui::role_proof::view(&proof, "proof-view")?,
    );
    let profile_draft = Draft::new(
        &ui::profile::descriptors()?,
        &json!({"name":"Ada"}),
        Role::Field,
    )?;
    let project_draft = Draft::new(
        &ui::project::descriptors()?,
        &json!({"title":"Compile roles", "state":"open", "tags":["rust","0"]}),
        Role::Field,
    )?;
    let task_draft = Draft::new(
        &ui::task::descriptors()?,
        &json!({"title":"Review generated UI"}),
        Role::Field,
    )?;
    let profile_filter = Draft::new(&ui::profile::descriptors()?, &json!({}), Role::Filter)?;
    let project_filter = Draft::new(&ui::project::descriptors()?, &json!({}), Role::Filter)?;
    let task_filter = Draft::new(&ui::task::descriptors()?, &json!({}), Role::Filter)?;
    let profile_span = Draft::new(&ui::profile::descriptors()?, &json!({}), Role::Span)?;
    let project_span = Draft::new(&ui::project::descriptors()?, &json!({}), Role::Span)?;
    let task_span = Draft::new(&ui::task::descriptors()?, &json!({}), Role::Span)?;
    let profile_fields = ui::profile::field(&profile_draft, "profile")?;
    let project_fields = ui::project::field(&project_draft, "project")?;
    let task_fields = ui::task::field(&task_draft, "task")?;
    let profiles = ui::profile::list(&profile, "profile-list")?;
    let projects = ui::project::list(&project, "project-list")?;
    let tasks = ui::task::list(&task, "task-list")?;
    let table = ui::task::table(&[("task-one".into(), task.clone())], "task-table")?;
    let profile_view = ui::profile::view(&profile, "profile-view")?;
    let project_view = ui::project::view(&project, "project-view")?;
    let task_view = ui::task::view(&task, "task-view")?;
    let filters = (
        ui::profile::filter(&profile_filter, "profile")?,
        ui::project::filter(&project_filter, "project")?,
        ui::task::filter(&task_filter, "task")?,
    );
    let spans = (
        ui::profile::span(&profile_span, "profile")?,
        ui::project::span(&project_span, "project")?,
        ui::task::span(&task_span, "task")?,
    );
    // Snapshot reacts to edited controls; this is a local draft proof, not a mutation endpoint.
    let draft = task_draft.clone();
    Ok(view! { <main><h1>"Generated workspace admin"</h1>
        <p>"Fixture data · generated role wrappers · no database writes"</p>
        <section><h2>"Profile"</h2><h3>"List"</h3><ul aria-label="Profile list"><li>{profiles}</li></ul><h3>"View"</h3>{profile_view}<h3>"Fields"</h3><form aria-label="Profile field form" on:submit=move |event| event.prevent_default()>{profile_fields}</form></section>
        <section><h2>"Project"</h2><h3>"List"</h3><ul aria-label="Project list"><li>{projects}</li></ul><h3>"View"</h3>{project_view}<h3>"Fields"</h3><form aria-label="Project field form" on:submit=move |event| event.prevent_default()>{project_fields}</form></section>
        <section><h2>"Task"</h2><h3>"List"</h3><ul aria-label="Task list"><li>{tasks}</li></ul><h3>"Table"</h3>{table}<h3>"View"</h3>{task_view}<h3>"Fields"</h3><form aria-label="Task field form" on:submit=move |event| event.prevent_default()>{task_fields}</form></section>
        <section><h2>"Filters"</h2>{filters}</section><section><h2>"Spans"</h2>{spans}</section>
        <section id="role-proof"><h2>"Nullable and defaults proof"</h2>{proof_roles}
            <output id="proof-draft">{move || { for field in ["text","zero","rows","metadata","score","slug"] {let _ = proof_draft.get(field).map(|value| value.get());} proof_draft.snapshot().map(|value| value.to_string()).unwrap_or_default() }}</output>
        </section>
        <output id="project-draft">{move || {for field in ["state","tags"] { let _ = project_draft.get(field).map(|value|value.get()); } project_draft.snapshot().map(|value|value.to_string()).unwrap_or_default() }}</output>
        <output id="task-draft">{move || { let _ = draft.get("title").map(|value| value.get()); let _ = draft.get("done").map(|value| value.get()); draft.snapshot().map(|value| value.to_string()).unwrap_or_default() }}</output>
    </main> }.into_any())
}

/// Preserve codec cause beneath one safe public-fixture context.
fn codec(cause: serde_json::Error) -> stackpress_framework_schema::SchemaError {
    stackpress_framework_schema::SchemaError::new(
        stackpress_framework_schema::SchemaErrorKind::Codec,
        "public fixture decoding failed",
    )
    .with_cause(cause)
}
