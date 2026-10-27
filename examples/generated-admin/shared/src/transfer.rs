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

/// Exact preview identity: replacing a file requires fresh preview even if its bytes happen to match.
#[derive(Clone, Debug, PartialEq)]
struct Selection {
    generation: u64,
    csv: String,
    decode: bool,
}
impl Selection {
    /// Retain only this bounded root-owned selection; no host token or file storage is needed.
    fn new(file: &FileDraft, decode: bool) -> Self {
        Self {
            generation: file.generation,
            csv: file.csv.clone(),
            decode,
        }
    }
}
/// A successful preview grants confirmation only for its exact selection and explicit decode mode.
#[derive(Clone, Default)]
struct PreviewGuard(Option<Selection>);
impl PreviewGuard {
    /// Invalidate synchronously on a decode change, including toggling away and back again.
    fn invalidate(&mut self) {
        self.0 = None;
    }
    /// Bind a complete valid preview to the selection that actually produced it.
    fn approve(&mut self, selection: Selection) {
        self.0 = Some(selection);
    }
    /// Compare again at submit; a deferred reactive invalidation cannot permit a stale write.
    fn permits(&self, selection: &Selection) -> bool {
        self.0.as_ref() == Some(selection)
    }
}

/// Render a preview-first import without sending any request during SSR/mount.
pub fn form(snapshot: Snapshot, transport: Callback<Command>) -> AnyView {
    let upload = use_context::<UploadState>().unwrap_or_default();
    let phase = RwSignal::new(Phase::Idle);
    let preview = RwSignal::new(None::<Preview>);
    let decode = RwSignal::new(false);
    let confirmed = RwSignal::new(false);
    let guard = RwSignal::new(PreviewGuard::default());
    let submitted = RwSignal::new(None::<Selection>);
    // Editing remains allowed during IO. Invalidate permission, without releasing its request lock.
    Effect::watch(
        move || (upload.0.get(), decode.get()),
        move |_, _, _| {
            guard.update(PreviewGuard::invalidate);
            preview.set(None);
            confirmed.set(false);
            if matches!(
                phase.get_untracked(),
                Phase::Idle | Phase::Previewed | Phase::Failed(_)
            ) {
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
        let selection = Selection::new(&file, decode.get_untracked());
        if write
            && (!guard.get_untracked().permits(&selection)
                || !confirmed.get_untracked()
                || !preview.get_untracked().is_some_and(|p| p.valid())
                || phase.get_untracked() != Phase::Previewed)
        {
            return;
        }
        let selected_decode = decode.get_untracked();
        if write {
            submitted.set(Some(selection.clone()));
        } else {
            submitted.set(None);
        }
        phase.set(if write {
            Phase::Applying
        } else {
            Phase::Previewing
        });
        let complete = Callback::new(move |reply: Reply| {
            let current = Selection::new(&upload.0.get_untracked(), decode.get_untracked());
            let newer = selection != current;
            // A stale read cannot approve the current selection. A write's known acknowledgement
            // remains known even if newer edits exist; never invent uncertainty from that difference.
            if !write && newer {
                phase.set(Phase::Idle);
                return;
            }
            let next = Phase::reply(write, reply.status, &reply.envelope);
            let result = (!newer)
                .then(|| {
                    reply
                        .envelope
                        .get("results")
                        .and_then(|value| serde_json::from_value::<Preview>(value.clone()).ok())
                })
                .flatten();
            guard.update(PreviewGuard::invalidate);
            if !write && result.as_ref().is_some_and(Preview::valid) {
                guard.update(|guard| guard.approve(selection.clone()));
            }
            preview.set(result);
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
        <p>{format!("Maximum {} per file, {}, {} columns, {} per cell.",human_bytes(limits.bytes),row_count(limits.rows),limits.columns,human_bytes(limits.cell))}</p>
        <label for="admin-import-file">"CSV file"</label><input id="admin-import-file" type="file" accept=".csv,text/csv" disabled=move ||phase.get()==Phase::Unknown/>
        <p id="import-file-name">{move ||upload.0.get().name}</p>
        <p role="alert" id="import-file-error">{move ||upload.0.get().error}</p>
        <label><input id="import-decode" type="checkbox" checked=move ||decode.get() prop:checked=move ||decode.get() disabled=move ||phase.get()==Phase::Unknown on:change=move |event|{guard.update(PreviewGuard::invalidate);preview.set(None);confirmed.set(false);decode.set(event_target_checked(&event));}/>"Decode Stackpress export guards"</label>
        <p>"Off by default: a leading apostrophe stays literal. Enable only for Stackpress exports; preview shows raw and decoded values. Use \\N for null, \\O to leave unset/use default, and double a literal leading backslash."</p>
        <p id="import-status" role="status">{move ||if upload.0.get().reading{"Reading selected file…".into()}else{import_message(&phase.get(), submitted.get().is_some_and(|sent|sent != Selection::new(&upload.0.get(),decode.get())))}}</p>
        <div class="admin-actions"><button id="import-preview" type="button" disabled=move ||phase.get().locked()||upload.0.get().csv.is_empty()||upload.0.get().reading||!upload.0.get().error.is_empty() on:click=move |_|submit.run(false)>"Preview import"</button>
        <button id="admin-import-cancel" type="button" disabled=move ||phase.get()==Phase::Unknown on:click=move |_|{upload.0.update(|draft|{draft.csv.clear();draft.name.clear();draft.error.clear();draft.reading=false;draft.generation+=1;});guard.update(PreviewGuard::invalidate);preview.set(None);if !phase.get_untracked().locked(){phase.set(Phase::Idle);}confirmed.set(false);}>"Cancel / clear file"</button></div>
        {move ||preview.get().map(preview_table)}
        <label><input id="import-confirm-check" type="checkbox" checked=move ||confirmed.get() prop:checked=move ||confirmed.get() disabled=move ||phase.get().locked()||!preview.get().is_some_and(|p|p.valid())||phase.get()!=Phase::Previewed||!guard.get().permits(&Selection::new(&upload.0.get(),decode.get())) on:change=move |event|confirmed.set(event_target_checked(&event))/>"I confirm applying every previewed row in one transaction"</label>
        <button id="import-confirm" type="button" disabled=move ||phase.get().locked()||!confirmed.get()||!preview.get().is_some_and(|p|p.valid())||phase.get()!=Phase::Previewed||!guard.get().permits(&Selection::new(&upload.0.get(),decode.get())) on:click=move |_|submit.run(true)>"Confirm import"</button>
    </section>}.into_any()
}

/// Render every bounded row as inert text inside a labelled scroll region.
fn preview_table(preview: Preview) -> AnyView {
    let valid = preview.valid();
    view!{<p id="import-row-count">{format!("{}; {}",row_count(preview.rows.len()),if valid{"ready for explicit confirmation"}else{"correct the errors and preview again"})}</p>
            <div class="import-preview-scroll" role="region" aria-label="Import preview" tabindex="0"><table id="import-preview-table"><caption>"Raw CSV and decoded public values"</caption><thead><tr><th scope="col">"Row"</th><th scope="col">"Raw cells"</th><th scope="col">"Decoded values"</th><th scope="col">"Errors"</th></tr></thead><tbody>
            {preview.rows.into_iter().map(|row|view!{<tr><th scope="row">{row.number}</th><td data-label="Raw cells"><pre>{serde_json::to_string(&row.raw).unwrap_or_default()}</pre></td><td data-label="Decoded values"><pre>{row.values.to_string()}</pre></td><td data-label="Errors"><ul>{row.errors.into_iter().map(|issue|view!{<li>{issue.message}</li>}).collect_view()}</ul></td></tr>}).collect_view()}
            </tbody></table></div>}.into_any()
}

/// Feedback always names the submitted file; new selections never change an acknowledged outcome.
fn import_message(phase: &Phase, newer: bool) -> String {
    match phase {
        Phase::Imported(count) if newer => format!(
            "Imported {} from the submitted file. You have newer unsaved changes; preview the current file again.",
            row_count(*count)
        ),
        Phase::Unknown => format!(
            "Write outcome is unknown for the submitted file. Rows may have been imported. {}Verify records before another import; no automatic retry.",
            if newer {
                "You have newer unsaved changes. "
            } else {
                ""
            }
        ),
        Phase::Failed(error) if newer => {
            format!("Submitted file: {error}. You have newer unsaved changes.")
        }
        _ => phase.message(),
    }
}
/// English quantity shared by preview and acknowledgement feedback.
fn row_count(count: usize) -> String {
    format!("{count} {}", if count == 1 { "row" } else { "rows" })
}
/// Familiar visible units use 1024-byte increments; server admission always uses exact byte counts.
fn human_bytes(bytes: usize) -> String {
    let (size, unit) = if bytes >= 1_048_576 {
        (bytes as f64 / 1_048_576.0, "MB")
    } else if bytes >= 1024 {
        (bytes as f64 / 1024.0, "KB")
    } else {
        return format!("{bytes} bytes");
    };
    format!(
        "{} {unit}",
        format!("{size:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
    )
}

#[cfg(test)]
#[path = "transfer_tests.rs"]
mod tests;
