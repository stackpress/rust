//! Bounded server-sent-event encoding and owned response selection.
//!
//! Applications own authentication, replay, cursor meaning and bounded subscriptions. Native
//! hosts enforce clocks; ordinary in-process body polling yields application events only.
//! Buffered adapters reject the native descriptor without polling it. EventSource cannot attach
//! arbitrary Authorization headers; cross-origin credentials and proxy streaming need app policy.
//!
//! ```
//! use stackpress_server::http::sse::{channel, SseEvent, SseOptions, SseResponseExt};
//! # fn example() -> stackpress_lib::Result<()> {
//! let (sender, events) = channel(8, 65_536)?;
//! sender.try_send(SseEvent::data("ready")?.id("cursor-1")?)?;
//! let mut response = stackpress_lib::Response::new();
//! response.sse(events, SseOptions::default())?;
//! drop(sender); // Accepted events drain before EOF.
//! # Ok(())
//! # }
//! ```

use super::stream::{HttpBodySlot, OwnedSource};
use futures_core::Stream;
use stackpress_lib::{Body, Error, Request, Response, Result, SharedNative, Value};
use std::{
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll},
    time::Duration,
};

/// Checked per-event wire ceilings, measured in UTF-8 bytes.
#[derive(Clone, Copy, Debug)]
pub struct SseEncodeLimits {
    event: usize,
    id: usize,
    name: usize,
}
impl Default for SseEncodeLimits {
    /// Choose the approved 64 KiB event, 1 KiB cursor and 256-byte name ceilings.
    fn default() -> Self {
        Self {
            event: 65536,
            id: 1024,
            name: 256,
        }
    }
}
impl SseEncodeLimits {
    /// Select smaller positive ceilings; construction cannot increase the hard wire bounds.
    pub fn new(event: usize, id: usize, name: usize) -> Result<Self> {
        if event == 0 || event > 65536 || id == 0 || id > 1024 || name == 0 || name > 256 {
            return Err(Error::invalid("invalid SSE encoding limits"));
        }
        Ok(Self { event, id, name })
    }
    /// Select the stricter bound in each field.
    pub fn intersection(self, other: Self) -> Self {
        Self {
            event: self.event.min(other.event),
            id: self.id.min(other.id),
            name: self.name.min(other.name),
        }
    }
    /// Largest complete encoded event, including its blank-line terminator.
    pub fn max_event_bytes(&self) -> usize {
        self.event
    }
}
/// One validated owned event or a comment that cannot carry message metadata.
#[derive(Clone, Debug)]
pub struct SseEvent {
    text: String,
    comment: bool,
    id: Option<String>,
    name: Option<String>,
    retry: Option<u32>,
}
impl SseEvent {
    /// Validate the complete normalized wire cost before allocating owned data.
    pub fn data(value: impl AsRef<str>) -> Result<Self> {
        Self::text(value.as_ref(), false)
    }
    /// Build a comment-only event; comments do not create messages or update the cursor.
    pub fn comment(value: impl AsRef<str>) -> Result<Self> {
        Self::text(value.as_ref(), true)
    }
    /// Normalize CRLF and bare CR once, after rejecting an excessive encoded cost.
    fn text(value: &str, comment: bool) -> Result<Self> {
        // Encoded lines always cost at least their input, even after CRLF normalization.
        // Reject before scanning so arbitrary caller strings cannot create unbounded work.
        if value.len() > 65536 {
            return Err(Error::limit("SSE event byte limit exceeded"));
        }
        let prefix = if comment { 2 } else { 6 };
        let cost = text_cost(value, prefix)?;
        if cost > 65536 {
            return Err(Error::limit("SSE event byte limit exceeded"));
        }
        Ok(Self {
            text: value.replace("\r\n", "\n").replace('\r', "\n"),
            comment,
            id: None,
            name: None,
            retry: None,
        })
    }
    /// Attach an opaque cursor; empty resets it. CR, LF and NUL are rejected.
    pub fn id(mut self, value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        field(value, 1024, true)?;
        self.metadata()?;
        self.check_extra(
            value.len().saturating_add(5),
            self.id.as_ref().map_or(0, |s| s.len() + 5),
        )?;
        self.id = Some(value.into());
        Ok(self)
    }
    /// Attach a nonempty named event; omit this field for the default message type.
    pub fn event(mut self, value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        field(value, 256, false)?;
        self.metadata()?;
        if value.is_empty() {
            return Err(Error::invalid("empty SSE event name"));
        }
        self.check_extra(
            value.len().saturating_add(8),
            self.name.as_ref().map_or(0, |s| s.len() + 8),
        )?;
        self.name = Some(value.into());
        Ok(self)
    }
    /// Set decimal browser retry milliseconds, including zero.
    pub fn retry(mut self, milliseconds: u32) -> Result<Self> {
        self.metadata()?;
        self.check_extra(
            digits(milliseconds) + 8,
            self.retry.map_or(0, |n| digits(n) + 8),
        )?;
        self.retry = Some(milliseconds);
        Ok(self)
    }
    /// Reject metadata on comments before any mutation or allocation.
    fn metadata(&self) -> Result<()> {
        if self.comment {
            return Err(Error::invalid("SSE comments cannot carry metadata"));
        }
        Ok(())
    }
    /// Charge replacement metadata against the already checked event cost.
    fn check_extra(&self, added: usize, replaced: usize) -> Result<()> {
        if self
            .cost()?
            .checked_sub(replaced)
            .and_then(|n| n.checked_add(added))
            .is_none_or(|n| n > 65536)
        {
            return Err(Error::limit("SSE aggregate byte limit exceeded"));
        }
        Ok(())
    }
    /// Compute every prefix, decimal digit and delimiter before allocation.
    fn cost(&self) -> Result<usize> {
        let mut cost = text_cost(&self.text, if self.comment { 2 } else { 6 })?;
        for extra in [
            self.id.as_ref().map_or(0, |s| s.len() + 5),
            self.name.as_ref().map_or(0, |s| s.len() + 8),
            self.retry.map_or(0, |n| digits(n) + 8),
        ] {
            cost = cost
                .checked_add(extra)
                .ok_or_else(|| Error::limit("SSE byte cost overflow"))?;
        }
        Ok(cost)
    }
    /// Encode a complete event atomically after all limits pass; no raw lines are accepted.
    pub fn encode(&self, limits: &SseEncodeLimits) -> Result<Vec<u8>> {
        let cost = self.cost()?;
        if cost > limits.event
            || self.id.as_ref().is_some_and(|s| s.len() > limits.id)
            || self.name.as_ref().is_some_and(|s| s.len() > limits.name)
        {
            return Err(Error::limit("SSE encoding limit exceeded"));
        }
        let mut bytes = Vec::with_capacity(cost);
        // Metadata precedes data; every normalized line receives its own safe field prefix.
        for (prefix, value) in [
            ("id: ", self.id.as_deref()),
            ("event: ", self.name.as_deref()),
        ] {
            if let Some(value) = value {
                bytes.extend_from_slice(prefix.as_bytes());
                bytes.extend_from_slice(value.as_bytes());
                bytes.push(b'\n');
            }
        }
        if let Some(retry) = self.retry {
            bytes.extend_from_slice(format!("retry: {retry}\n").as_bytes());
        }
        let prefix = if self.comment {
            b": ".as_slice()
        } else {
            b"data: ".as_slice()
        };
        for line in self.text.split('\n') {
            bytes.extend_from_slice(prefix);
            bytes.extend_from_slice(line.as_bytes());
            bytes.push(b'\n');
        }
        bytes.push(b'\n');
        Ok(bytes)
    }
}
/// Count decimal digits without a temporary allocation.
fn digits(mut n: u32) -> usize {
    let mut count = 1;
    while n >= 10 {
        n /= 10;
        count += 1;
    }
    count
}
/// Charge normalized UTF-8 plus a prefix/newline for every line, including empty trailing ones.
fn text_cost(value: &str, prefix: usize) -> Result<usize> {
    let mut lines = 1usize;
    let mut bytes = 0usize;
    let mut cr = false;
    for b in value.bytes() {
        if b == b'\n' && cr {
            cr = false;
            continue;
        }
        cr = b == b'\r';
        if b == b'\n' || cr {
            lines = lines
                .checked_add(1)
                .ok_or_else(|| Error::limit("SSE line overflow"))?;
        } else {
            bytes = bytes
                .checked_add(1)
                .ok_or_else(|| Error::limit("SSE byte overflow"))?;
        }
    }
    lines
        .checked_mul(prefix + 1)
        .and_then(|n| n.checked_add(bytes))
        .and_then(|n| n.checked_add(1))
        .ok_or_else(|| Error::limit("SSE wire cost overflow"))
}
/// Reject single-line field injection before copying untrusted data.
fn field(value: &str, max: usize, nul: bool) -> Result<()> {
    if value.len() > max || value.contains(['\r', '\n']) || (nul && value.contains('\0')) {
        return Err(Error::invalid("invalid SSE field"));
    }
    Ok(())
}
/// Read one bounded untrusted Last-Event-ID, distinguishing missing from an empty reset.
pub fn last_event_id(request: &Request, max_bytes: usize) -> Result<Option<&str>> {
    if max_bytes == 0 || max_bytes > 1024 {
        return Err(Error::invalid("invalid SSE cursor ceiling"));
    }
    let values = request.headers.get_all("last-event-id").unwrap_or_default();
    if values.len() > 1 {
        return Err(Error::invalid("duplicate Last-Event-ID"));
    }
    let value = values.first().map(String::as_str);
    if let Some(value) = value {
        field(value, max_bytes, true)?;
    }
    Ok(value)
}
/// Finite requested session controls; a native host may only reduce these limits.
#[derive(Clone, Copy, Debug)]
pub struct SseOptions {
    pub(crate) encode: SseEncodeLimits,
    age: Duration,
    idle: Duration,
    heartbeat: Duration,
    bytes: usize,
}
impl Default for SseOptions {
    /// Choose the G63 defaults, including a separate one-hour session clock.
    fn default() -> Self {
        Self {
            encode: Default::default(),
            age: Duration::from_secs(3600),
            idle: Duration::from_secs(30),
            heartbeat: Duration::from_secs(15),
            bytes: 64 * 1024 * 1024,
        }
    }
}
impl SseOptions {
    /// Select positive bounded age, heartbeat, write idle and aggregate wire bytes.
    pub fn new(age: Duration, heartbeat: Duration, idle: Duration, bytes: usize) -> Result<Self> {
        if age.is_zero()
            || age > Duration::from_secs(3600)
            || heartbeat.is_zero()
            || heartbeat >= idle
            || idle > Duration::from_secs(3600)
            || !(13..=64 * 1024 * 1024).contains(&bytes)
        {
            return Err(Error::invalid("invalid SSE session controls"));
        }
        Ok(Self {
            age,
            heartbeat,
            idle,
            bytes,
            ..Default::default()
        })
    }
    /// Maximum session age after publication, independent of ordinary HTTP clocks.
    pub fn max_age(&self) -> Duration {
        self.age
    }
    /// Maximum interval between native writes, never reset by producer polling.
    pub fn write_idle(&self) -> Duration {
        self.idle
    }
    /// Quiet interval before a heartbeat; no catch-up bursts are emitted.
    pub fn heartbeat_interval(&self) -> Duration {
        self.heartbeat
    }
    /// Aggregate wire ceiling, including comments.
    pub fn max_bytes(&self) -> usize {
        self.bytes
    }
    /// Read the validated encoding ceilings.
    pub fn encode_limits(&self) -> SseEncodeLimits {
        self.encode
    }
    /// Choose stricter event encoding ceilings.
    pub fn with_encode_limits(mut self, limits: SseEncodeLimits) -> Self {
        self.encode = limits;
        self
    }
}
/// Attach an owned subscription without polling or starting tasks.
pub trait SseResponseExt {
    /// Set event-stream headers/status and a private one-shot bounded descriptor.
    fn sse<S>(&mut self, events: S, options: SseOptions) -> Result<&mut Self>
    where
        S: Stream<Item = Result<SseEvent>> + Send + 'static;
}
impl SseResponseExt for Response {
    /// Retain the producer only after the foundation accepts the response representation.
    fn sse<S>(&mut self, events: S, options: SseOptions) -> Result<&mut Self>
    where
        S: Stream<Item = Result<SseEvent>> + Send + 'static,
    {
        self.status_code(200, None)?;
        self.headers_mut()?.set("cache-control", "no-cache")?;
        self.headers_mut()?.set("x-accel-buffering", "no")?;
        self.set(
            "text/event-stream; charset=utf-8",
            Body::Value(Value::Shared(SharedNative::new(HttpBodySlot {
                stream: Mutex::new(Some(OwnedSource::Stream(Box::pin(Encoder {
                    source: Box::pin(events),
                    limits: options.encode,
                })))),
                length: None,
                limits: super::stream::StreamLimits {
                    max_chunk_bytes: options.encode.event,
                    max_bytes: options.bytes,
                    total_timeout: options.age,
                    idle_timeout: options.idle,
                },
                sse: Some(options),
            }))),
        )
    }
}
/// Translate one event per body demand; outer HttpBody fuses errors and owns aggregate accounting.
struct Encoder<S> {
    source: Pin<Box<S>>,
    limits: SseEncodeLimits,
}
impl<S: Stream<Item = Result<SseEvent>>> Stream for Encoder<S> {
    type Item = Result<Vec<u8>>;
    /// Encode only after the consumer requests its next complete event.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.source
            .as_mut()
            .poll_next(cx)
            .map(|item| item.map(|event| event.and_then(|e| e.encode(&this.limits))))
    }
}
/// Runtime-neutral finite heartbeat driver with an injected elapsed-time stream.
///
/// Ticks must be monotonic elapsed durations since publication. The caller owns tick scheduling;
/// EOF drops the source. Native hosts use demand-driven Tokio sleeps and actual-write monitoring.
pub struct SseDriver<S, T> {
    source: Option<Pin<Box<S>>>,
    ticks: Pin<Box<T>>,
    options: SseOptions,
    elapsed: Duration,
    last: Duration,
    bytes: usize,
    initial: bool,
}
impl<S, T> SseDriver<S, T> {
    /// Create an unpolled driver; the initial comment commits headers on first demand.
    pub fn new(source: S, ticks: T, options: SseOptions) -> Self {
        Self {
            source: Some(Box::pin(source)),
            ticks: Box::pin(ticks),
            options,
            elapsed: Duration::ZERO,
            last: Duration::ZERO,
            bytes: 0,
            initial: true,
        }
    }
    /// Observe an externally sampled monotonic clock before a demand poll.
    /// Native wrappers use this alongside their demand-driven timer to timestamp ready events.
    pub fn observe_elapsed(&mut self, elapsed: Duration) {
        self.elapsed = self.elapsed.max(elapsed);
    }
    /// Stop immediately before the next event poll; the transport may finish its current chunk.
    pub fn stop(&mut self) {
        self.source = None;
    }
    /// Account application and heartbeat chunks against one aggregate ceiling.
    fn emit(&mut self, bytes: Vec<u8>) -> Poll<Option<Result<Vec<u8>>>> {
        let total = self.bytes.checked_add(bytes.len());
        if total.is_none_or(|n| n > self.options.bytes) {
            self.stop();
            return Poll::Ready(None);
        }
        self.bytes = total.unwrap_or(self.options.bytes);
        self.last = self.elapsed;
        Poll::Ready(Some(Ok(bytes)))
    }
}
impl<S: Stream<Item = Result<Vec<u8>>>, T: Stream<Item = Duration>> Stream for SseDriver<S, T> {
    type Item = Result<Vec<u8>>;
    /// Prefer ready application data to heartbeat ticks, without prefetching a second chunk.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.source.is_none() {
            return Poll::Ready(None);
        }
        if this.initial {
            this.initial = false;
            return this.emit(b": connected\n\n".to_vec());
        }
        // Observe age before polling a ready producer so a busy source cannot evade rotation.
        let tick = this.ticks.as_mut().poll_next(cx);
        if let Poll::Ready(Some(elapsed)) = tick {
            this.elapsed = this.elapsed.max(elapsed);
        }
        if this.elapsed >= this.options.age {
            this.stop();
            return Poll::Ready(None);
        }
        if let Some(source) = this.source.as_mut() {
            match source.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    // Public drivers accept encoded chunks, so enforce the host event/field ceilings
                    // independently of the response encoder and before exposing that chunk.
                    if bytes.len() > this.options.encode.event
                        || bytes.split(|b| *b == b'\n').any(|line| {
                            line.strip_prefix(b"id: ")
                                .is_some_and(|id| id.len() > this.options.encode.id)
                                || line
                                    .strip_prefix(b"event: ")
                                    .is_some_and(|name| name.len() > this.options.encode.name)
                        })
                    {
                        this.stop();
                        return Poll::Ready(Some(Err(Error::limit(
                            "SSE encoded chunk limit exceeded",
                        ))));
                    }
                    return this.emit(bytes);
                }
                Poll::Ready(Some(Err(error))) => {
                    this.stop();
                    return Poll::Ready(Some(Err(error)));
                }
                Poll::Ready(None) => {
                    this.stop();
                    return Poll::Ready(None);
                }
                Poll::Pending => {}
            }
        }
        if matches!(tick, Poll::Ready(None)) {
            this.stop();
            return Poll::Ready(None);
        }
        if this.elapsed.saturating_sub(this.last) >= this.options.heartbeat {
            return this.emit(b": heartbeat\n\n".to_vec());
        }
        if tick.is_ready() {
            cx.waker().wake_by_ref();
        }
        Poll::Pending
    }
}

mod channel;
pub use channel::{SseReceiver, SseSender, channel};
#[cfg(test)]
mod driver_tests;
#[cfg(test)]
mod tests;
