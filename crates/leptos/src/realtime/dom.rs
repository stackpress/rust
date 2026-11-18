//! Thin browser leases; portable bounds live in the feature modules and run in native tests.
use super::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Event, EventTarget};
#[cfg(feature = "client-sse")]
#[path = "dom_sse.rs"]
mod sse;
#[cfg(feature = "client-sse")]
pub(super) use sse::{EventSourceLease, event_source};
#[cfg(feature = "client-websocket")]
#[path = "dom_ws.rs"]
mod ws;
#[cfg(feature = "client-websocket")]
pub(super) use ws::{Timer, WebSocketLease, close, send, websocket};

/// Registration and retained callback share one lifetime; removal precedes closure destruction.
struct Listener {
    target: EventTarget,
    name: String,
    callback: Closure<dyn FnMut(Event)>,
}
impl Listener {
    /// Return a removable lease; partial connection setup drops already installed listeners.
    fn new(
        target: EventTarget,
        name: &str,
        callback: impl FnMut(Event) + 'static,
    ) -> RealtimeResult<Self> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>);
        target
            .add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())
            .map_err(|_| RealtimeError::Browser)?;
        Ok(Self {
            target,
            name: name.into(),
            callback,
        })
    }
}
impl Drop for Listener {
    /// Browser removal is idempotent; no callback is intentionally leaked with Closure::forget.
    fn drop(&mut self) {
        let _ = self.target.remove_event_listener_with_callback(
            &self.name,
            self.callback.as_ref().unchecked_ref(),
        );
    }
}
/// Bound UTF-16 before copying out of JS, then enforce the public UTF-8 retention ceiling.
fn text(value: JsValue, max: usize) -> RealtimeResult<String> {
    if !value.is_string() {
        return Err(RealtimeError::Invalid);
    }
    let value = js_sys::JsString::from(value);
    if value.length() as usize > max {
        return Err(RealtimeError::Invalid);
    }
    let text = value.as_string().ok_or(RealtimeError::Invalid)?;
    if text.len() > max {
        return Err(RealtimeError::Invalid);
    }
    Ok(text)
}

/// Inspect native string properties without an unbounded Rust String conversion first.
#[cfg(feature = "client-sse")]
fn property_text(value: &JsValue, name: &str, max: usize) -> RealtimeResult<String> {
    text(
        js_sys::Reflect::get(value, &JsValue::from_str(name))
            .map_err(|_| RealtimeError::Browser)?,
        max,
    )
}
