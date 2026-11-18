//! Application-facing queues and snapshots; the protocol driver alone owns native IO.
use super::WebSocketOptions;
use crate::report::lock;
use stackpress_lib::{Error, Request, Result};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore, mpsc, watch};

/// Complete application data message; control frames remain driver-owned.
#[derive(Debug, PartialEq, Eq)]
pub enum WsMessage {
    /// UTF-8 text, bounded before queueing.
    Text(String),
    /// Owned binary data, bounded before queueing.
    Binary(Vec<u8>),
}
impl WsMessage {
    /// Inspect byte cost without cloning the payload.
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Text(s) => s.len(),
            Self::Binary(b) => b.len(),
        }
    }
}
/// Validated wire close code and UTF-8 reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WsClose {
    pub(crate) code: u16,
    pub(crate) reason: String,
}
impl WsClose {
    /// Validate wire-transmittable codes and <=123 UTF-8 bytes before copying.
    pub fn new(code: u16, reason: impl AsRef<str>) -> Result<Self> {
        let reason = reason.as_ref();
        if !(matches!(code,1000..=1003|1007..=1014|3000..=4999)) || reason.len() > 123 {
            return Err(Error::invalid("invalid WebSocket close code/reason"));
        }
        Ok(Self {
            code,
            reason: reason.into(),
        })
    }
    /// Read the validated code.
    pub fn code(&self) -> u16 {
        self.code
    }
    /// Read the bounded reason.
    pub fn reason(&self) -> &str {
        &self.reason
    }
    /// Create a library-owned constant close, without parsing or a fallible invariant.
    pub(crate) fn library(code: u16, reason: &str) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }
}
/// One terminal outcome, separate from the possibly full inbound data queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WsCloseInfo {
    /// Valid peer-supplied close metadata, if any.
    pub peer: Option<WsClose>,
    /// Whether the peer completed the close handshake.
    pub clean: bool,
}
/// Explicit checked handshake snapshot, never the body or a borrowed Request/Invocation.
pub struct WsRequestSnapshot {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    sessions: Vec<(String, String)>,
}
impl std::fmt::Debug for WsRequestSnapshot {
    /// Debug exposes only field counts, never credentials, URL parameters or selected values.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsRequestSnapshot")
            .field("headers", &self.headers.len())
            .field("sessions", &self.sessions.len())
            .finish_non_exhaustive()
    }
}
impl WsRequestSnapshot {
    /// Charge selected values before cloning; headers/session contents are never logged by default.
    pub(crate) fn capture(request: &Request, options: &WebSocketOptions) -> Result<Self> {
        let mut bytes = request
            .url
            .as_str()
            .len()
            .saturating_add(request.method.as_str().len());
        if bytes > 16384 {
            return Err(Error::limit("WebSocket request snapshot limit exceeded"));
        }
        let mut headers = Vec::new();
        let mut sessions = Vec::new();
        for name in &options.headers {
            for value in request.headers.get_all(name).unwrap_or_default() {
                charge(&mut bytes, name, value)?;
                headers.push((name.clone(), value.clone()));
            }
        }
        for name in &options.sessions {
            if let Some(value) = request.session.get(name).and_then(|v| v.as_str()) {
                charge(&mut bytes, name, value)?;
                sessions.push((name.clone(), value.into()));
            }
        }
        Ok(Self {
            url: request.url.as_str().into(),
            method: request.method.as_str().into(),
            headers,
            sessions,
        })
    }
    /// Normalized informational URL captured at intent selection.
    pub fn url(&self) -> &str {
        &self.url
    }
    /// Original selected method; successful native handshakes require GET.
    pub fn method(&self) -> &str {
        &self.method
    }
    /// Explicitly selected repeated headers, preserving occurrences.
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }
    /// Explicitly selected textual cookie/session entries.
    pub fn sessions(&self) -> &[(String, String)] {
        &self.sessions
    }
}
/// Check both name/value storage before copying one snapshot entry.
fn charge(bytes: &mut usize, name: &str, value: &str) -> Result<()> {
    *bytes = bytes.saturating_add(name.len()).saturating_add(value.len());
    if *bytes > 16384 {
        return Err(Error::limit("WebSocket snapshot byte limit exceeded"));
    }
    Ok(())
}
/// A queued message owns its byte reservation until dequeue/drop; mpsc charges count separately.
pub(super) struct Queued {
    pub(super) message: WsMessage,
    pub(super) _bytes: OwnedSemaphorePermit,
}
/// Terminal state remains deliverable even when the data queue is full.
pub(super) struct Terminal {
    pub(super) result: Mutex<Option<(WsCloseInfo, Option<Arc<Error>>)>>,
    pub(super) changed: Notify,
    pub(super) closing: AtomicBool,
}
impl Terminal {
    /// Publish exactly once outside queue capacity; callers retain the first cause.
    pub(super) fn finish(&self, info: WsCloseInfo, error: Option<Arc<Error>>) {
        let mut result = lock(&self.result);
        if result.is_none() {
            *result = Some((info, error));
        }
        drop(result);
        self.closing.store(true, Ordering::Release);
        self.changed.notify_waiters();
    }
}
/// Owned nonclonable application connection; sends never require an application recv call.
pub struct WsConnection {
    pub(super) outbound: mpsc::Sender<Queued>,
    pub(super) inbound: mpsc::Receiver<Queued>,
    pub(super) outbound_bytes: Arc<Semaphore>,
    pub(super) close: watch::Sender<Option<WsClose>>,
    pub(super) terminal: Arc<Terminal>,
    pub(super) snapshot: Arc<WsRequestSnapshot>,
    pub(super) max_message: usize,
    pub(super) max_queue_bytes: usize,
    pub(super) send_timeout: Duration,
    pub(super) ended: bool,
}
impl WsConnection {
    /// Await bounded count/byte capacity and reject oversized/closing messages before enqueue.
    pub async fn send(&mut self, message: WsMessage) -> Result<()> {
        if message.len() > self.max_message || message.len() > self.max_queue_bytes {
            return Err(Error::limit("WebSocket send byte limit exceeded"));
        }
        if self.terminal.closing.load(Ordering::Acquire) {
            return Err(Error::new("WebSocket connection closing"));
        }
        let size = u32::try_from(message.len())
            .map_err(|_| Error::limit("WebSocket queue byte cost overflow"))?;
        let reserve = async {
            let bytes = Arc::clone(&self.outbound_bytes)
                .acquire_many_owned(size)
                .await
                .map_err(|_| Error::new("WebSocket byte queue closed"))?;
            let slot = self
                .outbound
                .reserve()
                .await
                .map_err(|_| Error::new("WebSocket outbound queue closed"))?;
            if self.terminal.closing.load(Ordering::Acquire) {
                return Err(Error::new("WebSocket connection closing"));
            }
            slot.send(Queued {
                message,
                _bytes: bytes,
            });
            Ok(())
        };
        tokio::time::timeout(self.send_timeout, reserve)
            .await
            .map_err(|e| crate::external("WebSocket send capacity deadline exceeded", e))?
    }
    /// Receive complete Text/Binary in order, then one error and fused EOF on terminal failure.
    pub async fn recv(&mut self) -> Result<Option<WsMessage>> {
        if self.ended {
            return Ok(None);
        }
        loop {
            let notified = self.terminal.changed.notified();
            match self.inbound.try_recv() {
                Ok(queued) => return Ok(Some(queued.message)),
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => {}
            }
            let terminal = lock(&self.terminal.result).clone();
            if let Some((_, error)) = terminal {
                self.ended = true;
                return match error {
                    Some(error) => Err(Error::new("WebSocket connection failed").with_cause(error)),
                    None => Ok(None),
                };
            }
            tokio::select! {biased;()=notified=>{},item=self.inbound.recv()=>{if let Some(item)=item {return Ok(Some(item.message));}}}
        }
    }
    /// Prioritize one validated close independently of outbound queue capacity.
    pub async fn close(&mut self, close: WsClose) -> Result<()> {
        if self.terminal.closing.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        self.close
            .send(Some(close))
            .map_err(|_| Error::new("WebSocket close driver unavailable"))
    }
    /// Read the explicitly captured request metadata; it grants no fresh authorization.
    pub fn request(&self) -> &WsRequestSnapshot {
        &self.snapshot
    }
    /// Read peer metadata and cleanliness once the driver reaches a terminal state.
    pub fn close_info(&self) -> Option<WsCloseInfo> {
        lock(&self.terminal.result)
            .as_ref()
            .map(|(info, _)| info.clone())
    }
}
impl Drop for WsConnection {
    /// Sender drop wakes the driver; an explicit close retains its separate code and priority.
    fn drop(&mut self) {
        // Do not synthesize an explicit close: normal callback completion drains accepted data.
        self.terminal.closing.store(true, Ordering::Release);
    }
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod tests;
