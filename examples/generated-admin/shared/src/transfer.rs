//! Root-owned import preview/confirmation UI; file IO and network transport stay in thin adapters.
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use stackpress_framework_admin::{
    pages::{Command, Reply, Snapshot},
    transfer::{Phase, Preview},
};

/// Browser adapter result, never a server file name/path capability.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileDraft {
    /// Display-only bounded base name.
    pub name: String,
    /// Whole bounded UTF-8 upload retained only by this mounted browser root.
    pub csv: String,
    /// Safe adapter error; native/browser IO diagnostics are never copied here.
    pub error: String,
    /// A bounded file read is pending; no import request has been sent.
    pub reading: bool,
    /// Monotonic browser selection/cancel generation discards late file reads.
    pub generation: u64,
}
/// Root context installed by the browser adapter; SSR starts with the same empty draft.
#[derive(Clone, Copy, Default)]
pub struct UploadState(pub RwSignal<FileDraft>);

/// Render a preview-first import without sending any request during SSR/mount.
pub fn form(snapshot: Snapshot, transport: Callback<Command>) -> AnyView {
    let upload = use_context::<UploadState>().unwrap_or_default();
    let phase = RwSignal::new(Phase::Idle);
    let preview = RwSignal::new(None::<Preview>);
    let decode = RwSignal::new(false);
    let confirmed = RwSignal::new(false);
    // File/encoding changes invalidate every prior preview and its explicit confirmation.
    Effect::watch(
        move || (upload.0.get(), decode.get()),
        move |_, _, _| {
            if !phase.get_untracked().locked() {
                preview.set(None);
                confirmed.set(false);
                phase.set(Phase::Idle);
            }
        },
        false,
    );
    let base = crate::crud::endpoint(&snapshot, stackpress_framework_admin::pages::Action::Import);
    let base = base.replacen("/import", "/preview", 1);
    let csrf = snapshot.csrf;
    let limits = snapshot.transfer_limits;
    let submit = Callback::new(move |write: bool| {
        if phase.get_untracked().locked() {
            return;
        }
        let file = upload.0.get_untracked();
        if file.csv.is_empty() || file.reading || !file.error.is_empty() {
            return;
        }
        if write
            && (!confirmed.get_untracked()
                || !preview.get_untracked().is_some_and(|p| p.valid())
                || phase.get_untracked() != Phase::Previewed)
        {
            return;
        }
        let selected_decode = decode.get_untracked();
        let generation = file.generation;
        phase.set(if write {
            Phase::Applying
        } else {
            Phase::Previewing
        });
        let complete = Callback::new(move |reply: Reply| {
            if upload.0.get_untracked().generation != generation
                || decode.get_untracked() != selected_decode
            {
                phase.set(if write { Phase::Unknown } else { Phase::Idle });
                return;
            }
            let next = Phase::reply(write, reply.status, &reply.envelope);
            if !write || !matches!(next, Phase::Imported(_)) {
                preview.set(
                    reply
                        .envelope
                        .get("results")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()),
                );
            }
            confirmed.set(false);
            phase.set(next);
        });
        transport.run(Command {
            endpoint: if write {
                base.replacen("/preview", "/confirm", 1)
            } else {
                base.clone()
            },
            input: json!({"csv":file.csv,"decode_export_guards":selected_decode,"confirm":write}),
            csrf: csrf.clone(),
            write,
            complete,
        });
    });
    view!{<section id="admin-import">
        <p>"Create-only import. Every row is checked before confirmation. Existing or repeated identities prevent the whole batch. No server file storage."</p>
        <p>{format!("Maximum {} bytes, {} rows, {} columns, {} bytes per cell.",limits.bytes,limits.rows,limits.columns,limits.cell)}</p>
        <label for="admin-import-file">"CSV file"</label><input id="admin-import-file" type="file" accept=".csv,text/csv" disabled=move ||phase.get().locked()/>
        <p id="import-file-name">{move ||upload.0.get().name}</p>
        <p role="alert" id="import-file-error">{move ||upload.0.get().error}</p>
        <label><input id="import-decode" type="checkbox" checked=move ||decode.get() prop:checked=move ||decode.get() disabled=move ||phase.get().locked() on:change=move |event|decode.set(event_target_checked(&event))/>"Decode Stackpress export guards"</label>
        <p>"Off by default: a leading apostrophe stays literal. Enable only for Stackpress exports; preview shows raw and decoded values. Use \\N for null, \\O to leave unset/use default, and double a literal leading backslash."</p>
        <p id="import-status" role="status">{move ||if upload.0.get().reading{"Reading selected file…".into()}else{phase.get().message()}}</p>
        <div class="admin-actions"><button id="import-preview" type="button" disabled=move ||phase.get().locked()||upload.0.get().csv.is_empty()||upload.0.get().reading||!upload.0.get().error.is_empty() on:click=move |_|submit.run(false)>"Preview import"</button>
        <button id="admin-import-cancel" type="button" disabled=move ||phase.get().locked() on:click=move |_|{upload.0.update(|draft|{draft.csv.clear();draft.name.clear();draft.error.clear();draft.reading=false;draft.generation+=1;});preview.set(None);phase.set(Phase::Idle);confirmed.set(false);}>"Cancel / clear file"</button></div>
        {move ||preview.get().map(preview_table)}
        <label><input id="import-confirm-check" type="checkbox" checked=move ||confirmed.get() prop:checked=move ||confirmed.get() disabled=move ||phase.get().locked()||!preview.get().is_some_and(|p|p.valid())||phase.get()!=Phase::Previewed on:change=move |event|confirmed.set(event_target_checked(&event))/>"I confirm applying every previewed row in one transaction"</label>
        <button id="import-confirm" type="button" disabled=move ||phase.get().locked()||!confirmed.get()||!preview.get().is_some_and(|p|p.valid())||phase.get()!=Phase::Previewed on:click=move |_|submit.run(true)>"Confirm import"</button>
    </section>}.into_any()
}

/// Render every bounded row as inert text inside a labelled scroll region.
fn preview_table(preview: Preview) -> AnyView {
    let valid = preview.valid();
    view!{<p id="import-row-count">{format!("{} rows; {}",preview.rows.len(),if valid{"ready for explicit confirmation"}else{"correct the errors and preview again"})}</p>
            <div class="import-preview-scroll" role="region" aria-label="Import preview" tabindex="0"><table id="import-preview-table"><caption>"Raw CSV and decoded public values"</caption><thead><tr><th scope="col">"Row"</th><th scope="col">"Raw cells"</th><th scope="col">"Decoded values"</th><th scope="col">"Errors"</th></tr></thead><tbody>
            {preview.rows.into_iter().map(|row|view!{<tr><th scope="row">{row.number}</th><td><pre>{serde_json::to_string(&row.raw).unwrap_or_default()}</pre></td><td><pre>{row.values.to_string()}</pre></td><td><ul>{row.errors.into_iter().map(|issue|view!{<li>{issue.message}</li>}).collect_view()}</ul></td></tr>}).collect_view()}
            </tbody></table></div>}.into_any()
}
