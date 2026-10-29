//! Native event/selection/timer leases. No policy or application callback spans a guard.
use super::EditError;
use super::{EditPolicy, EditSelection, EditState, SelectionDirection};
use js_sys::{Function, JsString, Reflect};
use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{HtmlDocument, HtmlInputElement};
type InputRef = NodeRef<leptos::html::Input>;

/// Local event suppression covers only synchronous native normalization events.
#[derive(Clone, Copy)]
pub(super) struct Adapter {
    /// One bounded next-task composition settlement, cancelled by Owner cleanup.
    pub settle: Callback<()>,
    /// Prevent a native edit command from reentering pure input/selection transitions.
    pub syncing: StoredValue<bool>,
}
/// Copy UTF-16 offsets/direction before borrowing reactive state.
fn selection(node: &HtmlInputElement) -> EditSelection {
    EditSelection {
        start: node.selection_start().ok().flatten().unwrap_or(0),
        end: node.selection_end().ok().flatten().unwrap_or(0),
        direction: match node.selection_direction().ok().flatten().as_deref() {
            Some("forward") => SelectionDirection::Forward,
            Some("backward") => SelectionDirection::Backward,
            _ => SelectionDirection::None,
        },
    }
}
/// Inspect JS-owned text before converting it into an owned UTF-8 Rust string.
fn bounded_text(value: wasm_bindgen::JsValue, budget: (usize, usize)) -> Result<String, EditError> {
    let text = value
        .dyn_into::<JsString>()
        .map_err(|_| EditError::Invalid)?;
    // UTF-16 length is a cheap lower bound on UTF-8 bytes, avoiding long JS scans.
    if text.length() as usize > budget.0 {
        return Err(EditError::Limit);
    }
    super::transport::admit_utf16(
        (0..text.length()).map(|i| text.char_code_at(i) as u16),
        budget.0,
        budget.1,
    )?;
    Ok(text.into())
}
/// Read a native property without an unbounded String-returning web-sys getter.
fn bounded_value(node: &HtmlInputElement, budget: (usize, usize)) -> Result<String, EditError> {
    let value = Reflect::get(node.as_ref(), &"value".into()).map_err(|_| EditError::Invalid)?;
    bounded_text(value, budget)
}
/// Equality stays in JavaScript; an oversized rejected value is never copied for comparison.
fn value_matches(node: &HtmlInputElement, text: &str) -> bool {
    Reflect::get(node.as_ref(), &"value".into())
        .ok()
        .is_some_and(|value| value == wasm_bindgen::JsValue::from_str(text))
}
/// Adopt an existing native field before hydrate, without allocating beyond its limits.
/// Missing fields are expected during CSR. No commit or application callback is emitted.
/// # Errors
/// Rejects excessive/malformed text or invalid selections while retaining the typed value.
pub fn adopt_native<P: EditPolicy>(
    id: &str,
    state: RwSignal<EditState<P>>,
) -> Result<(), EditError> {
    let Some(node) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    else {
        return Ok(());
    };
    let node = node
        .dyn_into::<HtmlInputElement>()
        .map_err(|_| EditError::Invalid)?;
    let budget = state.with_untracked(EditState::transport_budget);
    let selected = selection(&node);
    let text = match bounded_value(&node, budget) {
        Ok(text) => text,
        Err(error) => {
            state.update(|s| s.reject_native(error, selected));
            return Err(error);
        }
    };
    let mut result = Ok(());
    state.update(|s| result = s.adopt(&text, selected));
    result
}
/// Copy a property snapshot; normalization events never become another user edit.
pub(super) fn read<P: EditPolicy>(
    node: InputRef,
    state: RwSignal<EditState<P>>,
    syncing: StoredValue<bool>,
) {
    if syncing.try_get_value() != Some(false) {
        return;
    }
    if let Some(node) = node.get() {
        let selected = selection(&node);
        let budget = state.with_untracked(EditState::transport_budget);
        // The JS getter and UTF-16 scan run after releasing the reactive read guard.
        match bounded_value(&node, budget) {
            Ok(value) => state.update(|s| {
                let _ = s.input(&value, selected);
            }),
            Err(error) => state.update(|s| s.reject_native(error, selected)),
        }
    }
}
/// Track native selection without parsing or rewriting text.
pub(super) fn select<P: EditPolicy>(
    node: InputRef,
    state: RwSignal<EditState<P>>,
    syncing: StoredValue<bool>,
) {
    if syncing.try_get_value() != Some(false) {
        return;
    }
    if let Some(node) = node.get() {
        let selected = selection(&node);
        state.update(|s| {
            let _ = s.select(selected);
        });
    }
}
/// Reject clipboard proposals before default paste; accepted paste remains native/trusted.
pub(super) fn paste<P: EditPolicy>(
    event: &web_sys::ClipboardEvent,
    node: InputRef,
    state: RwSignal<EditState<P>>,
) {
    let Some(node) = node.get() else {
        return;
    };
    let Some(data) = event.clipboard_data() else {
        return;
    };
    let budget = state.with_untracked(EditState::transport_budget);
    let selected = selection(&node);
    // getData returns a JS string handle here; UTF-8 copying happens only after admission.
    let text = (|| {
        let getter = Reflect::get(data.as_ref(), &"getData".into())
            .map_err(|_| EditError::Invalid)?
            .dyn_into::<Function>()
            .map_err(|_| EditError::Invalid)?;
        let value = getter
            .call1(data.as_ref(), &"text/plain".into())
            .map_err(|_| EditError::Invalid)?;
        bounded_text(value, budget)
    })();
    let text = match text {
        Ok(text) => text,
        Err(error) => {
            state.update(|s| s.reject_native(error, selected));
            event.prevent_default();
            return;
        }
    };
    let mut rejected = false;
    state.update(|s| {
        rejected = s.paste(&text, selected).is_err();
    });
    if rejected {
        event.prevent_default();
    }
}
/// Blur preserves raw edit history; explicit Enter requests native normalization.
/// Application callbacks execute only after releasing the reactive write guard.
pub(super) fn commit<P: EditPolicy>(
    state: RwSignal<EditState<P>>,
    nullable: bool,
    callback: Option<Callback<Option<P::Value>>>,
    normalize: bool,
) {
    let mut changed = false;
    state.try_update(|s| {
        changed = if normalize {
            s.commit(nullable)
        } else {
            s.commit_preserving_draft(nullable)
        }
        .unwrap_or(false)
    });
    if changed
        && let Some(callback) = callback
        && let Some(value) = state.try_with(|s| s.value().cloned())
    {
        callback.run(value);
    }
}
/// Own the native callback until it fires, is superseded, or the root is disposed.
struct Timer {
    id: i32,
    _callback: Closure<dyn FnMut()>,
}
impl Drop for Timer {
    /// Cancel before releasing the closure; no callback reaches disposed state.
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(self.id);
        }
    }
}
/// Apply mapped selection without moving focus, including backward anchor/focus order.
fn set_selection(node: &HtmlInputElement, selected: EditSelection) {
    if selection(node) != selected {
        let direction = match selected.direction {
            SelectionDirection::Forward => "forward",
            SelectionDirection::Backward => "backward",
            SelectionDirection::None => "none",
        };
        let _ = node.set_selection_range_with_direction(selected.start, selected.end, direction);
    }
}
/// Preserve native undo on focused normalization; external clean resets may replace text.
pub(super) fn install<P: EditPolicy>(
    node: InputRef,
    state: RwSignal<EditState<P>>,
    nullable: bool,
    callback: Option<Callback<Option<P::Value>>>,
) -> Adapter {
    let syncing = StoredValue::new(false);
    let native_locale = StoredValue::new(state.with_untracked(EditState::draft_locale));
    Effect::new(move |_| {
        let snapshot = state.get();
        let Some(node) = node.get() else {
            return;
        };
        if snapshot.composing() {
            return;
        }
        let before = bounded_value(&node, snapshot.transport_budget());
        let before_selection = selection(&node);
        syncing.set_value(true);
        let mut applied = true;
        if !value_matches(&node, snapshot.draft()) {
            let document = node.owner_document();
            let focused = document
                .as_ref()
                .and_then(|d| d.active_element())
                .is_some_and(|active| active.is_same_node(Some(&node)));
            if focused && let Ok(before) = before {
                // Native insertText keeps Chromium's undo transaction. Never synthesize paste.
                node.select();
                applied = document
                    .and_then(|d| d.dyn_into::<HtmlDocument>().ok())
                    .is_some_and(|d| {
                        d.exec_command_with_show_ui_and_value("insertText", false, snapshot.draft())
                            .unwrap_or(false)
                    })
                    && value_matches(&node, snapshot.draft());
                if !applied {
                    // Unsupported hosts keep their native draft rather than lose edit history.
                    let locale = native_locale.get_value();
                    state.try_update(|s| s.retain_native(&before, before_selection, locale));
                }
            } else {
                // Blur itself never requests a rewrite. A clean model/locale reset or
                // rejected native text uses a bounded setter without stealing focus.
                node.set_value(snapshot.draft());
            }
        }
        set_selection(
            &node,
            if applied {
                snapshot.selection()
            } else {
                before_selection
            },
        );
        if applied {
            native_locale.try_set_value(snapshot.draft_locale());
        }
        syncing.try_set_value(false);
    });
    let timer = StoredValue::new_local(None::<Timer>);
    let settle = Callback::new(move |_| {
        // End may precede/follow final input; next-task settlement sees both orders.
        timer.set_value(None);
        let closure = Closure::wrap(Box::new(move || {
            let mut deferred = false;
            if state
                .try_update(|s| deferred = s.settle_composition())
                .is_some()
                && deferred
            {
                commit(state, nullable, callback, false);
            }
        }) as Box<dyn FnMut()>);
        if let Some(window) = web_sys::window()
            && let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                0,
            )
        {
            timer.set_value(Some(Timer {
                id,
                _callback: closure,
            }));
        }
    });
    Adapter { settle, syncing }
}
