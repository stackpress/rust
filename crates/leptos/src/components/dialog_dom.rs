//! Thin native-dialog adapter; no policy, network or domain state lives here.

use leptos::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::JsCast;
use web_sys::{HtmlDialogElement, HtmlElement};

/// Focus a known application control without holding a reactive borrow guard.
pub(super) fn focus_id(id: &str) {
    if let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
        .and_then(|element| element.dyn_into::<HtmlElement>().ok())
    {
        let _ = element.focus();
    }
}

/// Nested dialogs/popovers own their Escape; a parent modeless fallback must wait.
pub(super) fn from_current_dialog(event: &web_sys::KeyboardEvent) -> bool {
    let current = event
        .current_target()
        .and_then(|target| target.dyn_into::<web_sys::Node>().ok());
    let nearest = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|target| {
            target
                .closest("dialog, [popover]:popover-open")
                .ok()
                .flatten()
        });
    current
        .zip(nearest)
        .is_some_and(|(current, nearest)| nearest.is_same_node(Some(&current)))
}

/// Cloneable DOM handles, distinct from the one cleanup lease that owns them.
#[derive(Clone)]
struct Handles {
    /// The actual native modal; showModal enforces focus containment/inertness.
    dialog: HtmlDialogElement,
    /// Updated before opening; no borrow is held during focus/event callbacks.
    opener: Rc<RefCell<Option<HtmlElement>>>,
    /// Tracks native Escape/close as well as controlled closure.
    was_open: Rc<Cell<bool>>,
    /// An app-chosen deterministic fallback when the opener is disconnected.
    fallback: String,
    /// Modal return-focus rules differ from modeless background interaction.
    mode: super::DialogMode,
}

impl Handles {
    /// Return focus to a connected opener or the explicit fallback.
    fn restore_focus(&self) {
        let opener = self.opener.borrow().clone();
        if let Some(opener) = opener.filter(|element| element.is_connected()) {
            let _ = opener.focus();
            // A connected opener can still be hidden/disabled/inert. Verify actual
            // focus rather than treating an Ok DOM call as successful restoration.
            let restored = opener
                .owner_document()
                .and_then(|document| document.active_element())
                .is_some_and(|active| active.is_same_node(Some(&opener)));
            if !restored {
                focus_id(&self.fallback);
            }
        } else {
            focus_id(&self.fallback);
        }
    }

    /// Inspect before closing, since native close can already restore focus itself.
    fn wants_return_focus(&self) -> bool {
        let active = self
            .dialog
            .owner_document()
            .and_then(|document| document.active_element());
        let inside = active
            .as_ref()
            .is_some_and(|node| self.dialog.contains(Some(node)));
        let at_opener = active.as_ref().is_some_and(|node| {
            self.opener
                .borrow()
                .as_ref()
                .is_some_and(|opener| node.is_same_node(Some(opener)))
        });
        let detached_body = !self.dialog.is_connected()
            && active
                .as_ref()
                .is_some_and(|node| node.tag_name() == "BODY");
        self.mode.returns_focus(inside, at_opener, detached_body)
    }
    /// Reconcile controlled state after hydration without touching initial focus.
    fn synchronize(&self, open: bool, initial: &str) {
        if open && !self.dialog.open() {
            // Capture first; showModal synchronously moves focus and can dispatch events.
            let opener = self
                .dialog
                .owner_document()
                .and_then(|document| document.active_element())
                .and_then(|element| element.dyn_into::<HtmlElement>().ok());
            *self.opener.borrow_mut() = opener;
            if (if self.mode != super::DialogMode::Nonmodal {
                self.dialog.show_modal()
            } else {
                self.dialog.show();
                Ok(())
            })
            .is_ok()
            {
                self.was_open.set(true);
                // Only an actual descendant may override the native initial target.
                if let Some(target) = self
                    .dialog
                    .owner_document()
                    .and_then(|document| document.get_element_by_id(initial))
                    && self.dialog.contains(Some(&target))
                {
                    focus_id(initial);
                }
            }
        } else if !open && self.was_open.replace(false) {
            let restore = self.wants_return_focus();
            if self.dialog.open() {
                self.dialog.close();
            }
            if restore {
                self.restore_focus();
            }
        }
    }
}

/// Exactly one root-local lease closes an unmounted modal and returns focus.
struct Lease(Handles);

impl Drop for Lease {
    /// Native handles are local storage, never Send-wrapped or retained globally.
    fn drop(&mut self) {
        if self.0.was_open.replace(false) {
            let restore = self.0.wants_return_focus();
            if self.0.dialog.open() {
                self.0.dialog.close();
            }
            if restore {
                self.0.restore_focus();
            }
        }
    }
}

/// Install effects only in the real browser, after the SSR nodes are hydrated.
pub(super) fn install(
    element: NodeRef<leptos::html::Dialog>,
    open: RwSignal<bool>,
    initial: String,
    fallback: String,
    mode: super::DialogMode,
) {
    let lease = StoredValue::new_local(None::<Lease>);
    Effect::new(move |_| {
        let requested = open.get();
        let Some(element) = element.get() else {
            return;
        };
        // Copy handles out before invoking any DOM method that can reenter the app.
        let handles = lease.with_value(|lease| lease.as_ref().map(|lease| lease.0.clone()));
        let handles = handles.unwrap_or_else(|| {
            let handles = Handles {
                dialog: element.clone(),
                opener: Rc::new(RefCell::new(None)),
                was_open: Rc::new(Cell::new(false)),
                fallback: fallback.clone(),
                mode,
            };
            lease.set_value(Some(Lease(handles.clone())));
            handles
        });
        handles.synchronize(requested, &initial);
    });
}

/// Keep Tab/Shift-Tab inside the innermost active modal, including browser-chrome boundaries.
pub(super) fn trap_tab(event: &web_sys::KeyboardEvent) {
    if event.key() != "Tab" {
        return;
    }
    let Some(dialog) = event
        .current_target()
        .and_then(|target| target.dyn_into::<HtmlDialogElement>().ok())
    else {
        return;
    };
    let target_dialog = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|element| element.closest("dialog").ok().flatten());
    if !dialog.open()
        || target_dialog
            .as_ref()
            .is_none_or(|target| !target.is_same_node(Some(&dialog)))
    {
        return;
    }
    let Ok(nodes) = dialog.query_selector_all("a[href],button:not([disabled]),input:not([disabled]):not([type=hidden]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex='-1'])") else { return; };
    let mut focusable = Vec::new();
    for index in 0..nodes.length() {
        if let Some(element) = nodes
            .item(index)
            .and_then(|node| node.dyn_into::<HtmlElement>().ok())
        {
            let rect = element.get_bounding_client_rect();
            // Explicit negative tabindex opts a control out of keyboard order.
            if element.tab_index() >= 0 && rect.width() > 0.0 && rect.height() > 0.0 {
                focusable.push(element);
            }
        }
    }
    // Native positive tabindex precedes ordinary DOM-order controls; stable sort
    // preserves document order within each tabindex group.
    focusable.sort_by_key(|element| {
        let index = element.tab_index();
        if index > 0 { (0, index) } else { (1, 0) }
    });
    let active = dialog
        .owner_document()
        .and_then(|document| document.active_element());
    let first = focusable.first();
    let last = focusable.last();
    let at_boundary = if event.shift_key() { first } else { last };
    let inside = active
        .as_ref()
        .is_some_and(|active| dialog.contains(Some(active)));
    if !inside
        || at_boundary.is_some_and(|boundary| {
            active
                .as_ref()
                .is_some_and(|active| active.is_same_node(Some(boundary)))
        })
    {
        event.prevent_default();
        if let Some(target) = if event.shift_key() { last } else { first } {
            let _ = target.focus();
        }
    } else if focusable.is_empty() {
        event.prevent_default();
        let _ = dialog.focus();
    }
}
