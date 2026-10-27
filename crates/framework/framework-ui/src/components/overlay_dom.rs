//! Thin top-layer measurement, listener and timer leases. Dropping an Owner releases all.
use super::DelayMs;
use leptos::prelude::*;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{Event, EventTarget, HtmlElement};

/// A registered callback owns both the matching removal identity and its JS closure.
struct Listener {
    /// The exact target/capture registration is retained for removal.
    target: EventTarget,
    /// Fixed event name, paired with the retained callback identity.
    name: &'static str,
    /// Owned until native removal completes.
    callback: Closure<dyn FnMut(Event)>,
}
impl Listener {
    /// Register a capturing listener, including scroll from nested overflow containers.
    fn new(
        target: EventTarget,
        name: &'static str,
        callback: impl FnMut(Event) + 'static,
    ) -> Option<Self> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>);
        target
            .add_event_listener_with_callback_and_bool(
                name,
                callback.as_ref().unchecked_ref(),
                true,
            )
            .ok()?;
        Some(Self {
            target,
            name,
            callback,
        })
    }
}
impl Drop for Listener {
    /// Remove before freeing the callback, including unmount while open.
    fn drop(&mut self) {
        let _ = self.target.remove_event_listener_with_callback_and_bool(
            self.name,
            self.callback.as_ref().unchecked_ref(),
            true,
        );
    }
}
#[derive(Clone)]
struct Handles {
    /// Native surface; clones do not own a second cleanup registration.
    element: HtmlElement,
    /// App-supplied connected trigger identity used for positioning.
    anchor: String,
    /// Close/focus restoration happens once across native and controlled dismissal.
    was_open: Rc<Cell<bool>>,
    /// The pre-open control, cloned out before any reentrant focus call.
    opener: Rc<std::cell::RefCell<Option<HtmlElement>>>,
}
impl Handles {
    /// Use bounded fixed coordinates; document/scroll changes never alter application data.
    fn position(&self) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(anchor) = self
            .element
            .owner_document()
            .and_then(|d| d.get_element_by_id(&self.anchor))
        else {
            return;
        };
        let a = anchor.get_bounding_client_rect();
        // Read the non-inherited computed zoom on each ancestor. Rounded offsetWidth
        // ratios oscillate at 200% and would retrigger ResizeObserver indefinitely.
        let mut ancestor = Some(self.element.clone().unchecked_into::<web_sys::Element>());
        let mut zooms = Vec::new();
        for _ in 0..64 {
            let Some(node) = ancestor.take() else {
                break;
            };
            let zoom = window
                .get_computed_style(&node)
                .ok()
                .flatten()
                .and_then(|style| style.get_property_value("zoom").ok())
                .unwrap_or_else(|| "1".into());
            zooms.push(zoom);
            ancestor = node.parent_element();
        }
        if ancestor.is_some() {
            return;
        }
        let Ok(scale) = super::effective_zoom(&zooms) else {
            return;
        };
        let size = [
            window
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or_default(),
            window
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or_default(),
        ];
        let _ = self.element.style().set_property(
            "max-width",
            &format!("{}px", ((size[0] - 16.0) / scale).max(0.0)),
        );
        let _ = self.element.style().set_property(
            "max-height",
            &format!("{}px", ((size[1] - 16.0) / scale).max(0.0)),
        );
        let p = self.element.get_bounding_client_rect();
        if let Ok([left, top]) = super::place_popup(
            [a.x(), a.y(), a.width(), a.height()],
            [p.width(), p.height()],
            size,
        ) {
            let _ = self
                .element
                .style()
                .set_property("left", &format!("{}px", left / scale));
            let _ = self
                .element
                .style()
                .set_property("top", &format!("{}px", top / scale));
        }
    }
    /// Close exactly once, restoring a still-connected trigger without retaining a root.
    fn close(&self, restore: bool) {
        if !self.was_open.replace(false) {
            return;
        }
        // Preserve a newly focused outside control on pointer/Tab dismissal.
        let return_focus = self
            .element
            .owner_document()
            .and_then(|d| d.active_element())
            .is_some_and(|active| {
                self.element.contains(Some(&active))
                    || (!self.element.is_connected() && active.tag_name() == "BODY")
            });
        if self.element.matches(":popover-open").unwrap_or(false) {
            let _ = self.element.hide_popover();
        }
        // Clone before focus: focus events can synchronously reenter application code.
        let opener = self.opener.borrow().clone();
        if restore
            && return_focus
            && let Some(opener) = opener.filter(|node| node.is_connected())
        {
            let _ = opener.focus();
        }
    }
}
/// ResizeObserver covers asynchronous option/content changes without a polling loop.
struct ResizeLease {
    /// Disconnect before freeing the corresponding callback.
    observer: web_sys::ResizeObserver,
    /// Keep JS callable while observation is active.
    _callback: Closure<dyn FnMut()>,
}
impl ResizeLease {
    /// Own the observer and its callback together; observation starts only while open.
    fn new(handles: Handles) -> Option<Self> {
        let element = handles.element.clone();
        let callback = Closure::wrap(Box::new(move || handles.position()) as Box<dyn FnMut()>);
        let observer = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()).ok()?;
        observer.observe(&element);
        Some(Self {
            observer,
            _callback: callback,
        })
    }
}
impl Drop for ResizeLease {
    /// Disconnect before freeing its closure on close and unmount.
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}
/// One stored lease per component; no registry or global listeners survive its owner.
struct PopupLease {
    handles: Handles,
    _listeners: Vec<Listener>,
    _resize: Option<ResizeLease>,
    restore: bool,
}
impl Drop for PopupLease {
    /// Close native top-layer state before its detached node becomes unreachable.
    fn drop(&mut self) {
        self.handles.close(self.restore);
    }
}
/// Reconcile a native auto/manual popover with root-owned visibility after hydration.
pub(super) fn install(
    element: NodeRef<leptos::html::Div>,
    anchor: String,
    open: RwSignal<bool>,
    initial: String,
    manual: bool,
) {
    let lease = StoredValue::new_local(None::<PopupLease>);
    Effect::new(move |_| {
        let requested = open.get();
        let Some(node) = element.get() else {
            return;
        };
        let existing = lease.with_value(|lease| lease.as_ref().map(|lease| lease.handles.clone()));
        if requested {
            let handles = existing.unwrap_or_else(|| Handles {
                element: node.clone().into(),
                anchor: anchor.clone(),
                was_open: Rc::new(Cell::new(false)),
                opener: Rc::new(std::cell::RefCell::new(None)),
            });
            if !handles.was_open.get() {
                *handles.opener.borrow_mut() = node
                    .owner_document()
                    .and_then(|d| d.active_element())
                    .and_then(|e| e.dyn_into().ok());
                if node.show_popover().is_err() {
                    open.try_set(false);
                    return;
                }
                handles.was_open.set(true);
                handles.position();
                if !initial.is_empty()
                    && let Some(target) = node
                        .owner_document()
                        .and_then(|d| d.get_element_by_id(&initial))
                    && node.contains(Some(&target))
                {
                    super::dialog_dom::focus_id(&initial);
                }
                let mut listeners = Vec::new();
                if let Some(window) = web_sys::window() {
                    for name in ["resize", "scroll"] {
                        let copy = handles.clone();
                        if let Some(listener) =
                            Listener::new(window.clone().unchecked_into(), name, move |_| {
                                copy.position()
                            })
                        {
                            listeners.push(listener);
                        }
                    }
                }
                lease.set_value(Some(PopupLease {
                    _resize: ResizeLease::new(handles.clone()),
                    handles,
                    _listeners: listeners,
                    restore: !manual,
                }));
            }
        } else {
            // Take out first: native close/focus can dispatch events synchronously.
            let mut previous = None;
            lease.update_value(|slot| previous = slot.take());
            drop(previous);
        }
    });
}
/// Native outside-click/Escape closure updates the same controller as explicit actions.
pub(super) fn sync_toggle(event: &Event, open: RwSignal<bool>) {
    if let Some(node) = event
        .current_target()
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
    {
        open.try_set(node.matches(":popover-open").unwrap_or(false));
    }
}
/// A cancellable callback is owned as long as its native timer may fire.
struct Timer {
    id: i32,
    _callback: Closure<dyn FnMut()>,
}
impl Timer {
    /// Schedule without leaking a once_into_js callback on cancellation.
    fn new(delay: DelayMs, callback: impl FnMut() + 'static) -> Option<Self> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
        let id = web_sys::window()?
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                delay.get() as i32,
            )
            .ok()?;
        Some(Self {
            id,
            _callback: callback,
        })
    }
}
impl Drop for Timer {
    /// Clear before freeing the closure on state changes and unmount.
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(self.id);
        }
    }
}
/// Delay visibility; leaving/dismissing cancels pending work before it can publish.
pub(super) fn delayed(open: RwSignal<bool>, requested: Signal<bool>, delay: DelayMs) {
    let timer = StoredValue::new_local(None::<Timer>);
    Effect::new(move |_| {
        let mut previous = None;
        timer.update_value(|slot| previous = slot.take());
        drop(previous);
        if requested.get() {
            timer.set_value(Timer::new(delay, move || {
                open.try_set(true);
            }));
        } else if open.get_untracked() {
            // A short exit grace makes the hover description itself reachable.
            timer.set_value(Timer::new(DelayMs::default(), move || {
                open.try_set(false);
            }));
        }
    });
}
/// Expiration pauses for interaction; leaving restarts one full bounded duration.
pub(super) fn expire(visible: RwSignal<bool>, paused: Signal<bool>, duration: DelayMs) {
    let timer = StoredValue::new_local(None::<Timer>);
    Effect::new(move |_| {
        let mut previous = None;
        timer.update_value(|slot| previous = slot.take());
        drop(previous);
        if visible.get() && !paused.get() {
            timer.set_value(Timer::new(duration, move || {
                visible.try_set(false);
            }));
        }
    });
}
