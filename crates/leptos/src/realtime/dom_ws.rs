//! Native WebSocket lease, bounded immediate sends and one cancellable retry timer.
use super::*;
use std::{cell::Cell, rc::Rc};
use web_sys::{BinaryType, MessageEvent, WebSocket};

/// One native socket and its exact listener registrations.
pub(in crate::realtime) struct WebSocketLease {
    socket: WebSocket,
    listeners: Vec<Listener>,
}
impl Drop for WebSocketLease {
    /// Remove callbacks before closing a connecting/open socket; closed sockets need no second close.
    fn drop(&mut self) {
        self.listeners.clear();
        if self.socket.ready_state() < WebSocket::CLOSING {
            let _ = self.socket.close();
        }
    }
}
/// One browser timeout and its retained closure; dropping cancels it before freeing the callback.
pub(in crate::realtime) struct Timer {
    id: i32,
    _callback: Closure<dyn FnMut()>,
}
impl Drop for Timer {
    /// A missing Window already makes the timer irrelevant; normal disposal cancels it explicitly.
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(self.id);
        }
    }
}
/// Attach mount and terminal-cleanup effects, without opening during render or native SSR.
pub(in crate::realtime) fn websocket(url: String, handle: WebSocketHandle) {
    let attempts = Rc::new(Cell::new(0u8));
    Effect::new(move |_| {
        let running = handle.running.get();
        let _ = handle.generation.get();
        let _ = handle.timer.try_update_value(Option::take);
        let _ = handle.lease.try_update_value(Option::take);
        if !running {
            return;
        }
        match connect(&url, handle, Rc::clone(&attempts)) {
            Ok(lease) => {
                let _ = handle.lease.try_set_value(Some(lease));
            }
            Err(error) => {
                let _ = handle.error.try_set(Some(error));
                let _ = handle.state.try_set(ConnectionState::Disconnected);
            }
        }
    });
    Effect::new(move |_| {
        if handle.state.get() == ConnectionState::Disconnected {
            // Free a closed native object's listeners after its event callback has returned.
            let _ = handle.lease.try_update_value(Option::take);
        }
    });
}
/// Configure binary delivery and register every event under one fallible lease.
fn connect(
    url: &str,
    handle: WebSocketHandle,
    attempts: Rc<Cell<u8>>,
) -> RealtimeResult<WebSocketLease> {
    let socket = WebSocket::new(url).map_err(|_| RealtimeError::Browser)?;
    socket.set_binary_type(BinaryType::Arraybuffer);
    let mut lease = WebSocketLease {
        socket,
        listeners: Vec::new(),
    };
    let _ = handle.state.try_set(ConnectionState::Connecting);
    let opened = Rc::clone(&attempts);
    lease.listeners.push(Listener::new(
        lease.socket.clone().into(),
        "open",
        move |_| {
            opened.set(0);
            let _ = handle.error.try_set(None);
            let _ = handle.state.try_set(ConnectionState::Open);
        },
    )?);
    lease.listeners.push(Listener::new(
        lease.socket.clone().into(),
        "error",
        move |_| {
            let _ = handle.error.try_set(Some(RealtimeError::Browser));
        },
    )?);
    lease.listeners.push(Listener::new(
        lease.socket.clone().into(),
        "message",
        move |event| {
            let message = event
                .dyn_into::<MessageEvent>()
                .map_err(|_| RealtimeError::Invalid)
                .and_then(|event| message(event.data(), handle.options.message));
            match message {
                Ok(message) => {
                    let _ = handle.latest.try_set(Some(message));
                }
                Err(error) => {
                    let _ = handle.error.try_set(Some(error));
                    let _ = handle.running.try_set(false);
                    let _ = handle.state.try_set(ConnectionState::Disconnected);
                }
            }
        },
    )?);
    lease.listeners.push(Listener::new(
        lease.socket.clone().into(),
        "close",
        move |_| {
            let _ = handle.state.try_set(ConnectionState::Disconnected);
            if handle.running.try_get_untracked() != Some(true) {
                return;
            }
            let Some(delay) = handle.options.retry_delay(attempts.get()) else {
                return;
            };
            let Some(next) = attempts.get().checked_add(1) else {
                return;
            };
            attempts.set(next);
            match retry(handle, delay) {
                Ok(timer) => {
                    let _ = handle.timer.try_set_value(Some(timer));
                }
                Err(error) => {
                    let _ = handle.error.try_set(Some(error));
                }
            }
        },
    )?);
    Ok(lease)
}
/// Retain only a bounded complete String/ArrayBuffer; Blob delivery is not enabled.
fn message(data: JsValue, max: usize) -> RealtimeResult<BrowserWsMessage> {
    if data.is_string() {
        return text(data, max).map(BrowserWsMessage::Text);
    }
    let buffer = data
        .dyn_into::<js_sys::ArrayBuffer>()
        .map_err(|_| RealtimeError::Invalid)?;
    if buffer.byte_length() as usize > max {
        return Err(RealtimeError::Invalid);
    }
    Ok(BrowserWsMessage::Binary(
        js_sys::Uint8Array::new(&buffer).to_vec(),
    ))
}
/// Set one bounded retry; generation changes only schedule a fresh mount effect, never resend.
fn retry(handle: WebSocketHandle, delay: i32) -> RealtimeResult<Timer> {
    let window = web_sys::window().ok_or(RealtimeError::Browser)?;
    let callback = Closure::wrap(Box::new(move || {
        if handle.running.try_get_untracked() == Some(true) {
            let _ = handle.reconnect();
        }
    }) as Box<dyn FnMut()>);
    let id = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            delay,
        )
        .map_err(|_| RealtimeError::Browser)?;
    Ok(Timer {
        id,
        _callback: callback,
    })
}
/// Clone only the JS handle out of storage before a potentially reentrant browser operation.
pub(in crate::realtime) fn send(
    handle: WebSocketHandle,
    message: BrowserWsMessage,
) -> RealtimeResult<()> {
    let socket = handle
        .lease
        .try_with_value(|lease| lease.as_ref().map(|lease| lease.socket.clone()))
        .flatten()
        .ok_or(RealtimeError::Disconnected)?;
    if socket.ready_state() != WebSocket::OPEN {
        return Err(RealtimeError::Disconnected);
    }
    handle
        .options
        .check_send(message.len(), socket.buffered_amount() as usize)?;
    match message {
        BrowserWsMessage::Text(text) => socket.send_with_str(&text),
        BrowserWsMessage::Binary(bytes) => socket.send_with_u8_array(&bytes),
    }
    .map_err(|_| RealtimeError::Browser)
}
/// Stop retries, detach listeners and issue one checked browser-supported close.
pub(in crate::realtime) fn close(
    handle: WebSocketHandle,
    code: u16,
    reason: &str,
) -> RealtimeResult<()> {
    let _ = handle.timer.try_update_value(Option::take);
    if let Some(Some(mut lease)) = handle.lease.try_update_value(Option::take) {
        lease.listeners.clear();
        lease
            .socket
            .close_with_code_and_reason(code, reason)
            .map_err(|_| RealtimeError::Browser)?;
    }
    Ok(())
}
