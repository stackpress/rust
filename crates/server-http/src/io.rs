//! Exclusive IO polling behind short locks, allowing a safe pre-publication timeout response.

use crate::{monitor::Monitor, report::lock};
use std::{
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// No raw IO handle reaches a handler; only the connection and its timeout cleanup own this.
pub(crate) struct SharedIo<I> {
    pub(crate) io: Arc<Mutex<I>>,
    monitor: Arc<Monitor>,
}
impl<I> SharedIo<I> {
    /// Retain IO separately so dropping Hyper on a deadline cannot lose rejection ownership.
    pub(crate) fn new(io: I, monitor: Arc<Monitor>) -> Self {
        Self {
            io: Arc::new(Mutex::new(io)),
            monitor,
        }
    }
}
impl<I: AsyncRead + Unpin> AsyncRead for SharedIo<I> {
    /// Read through the sole Hyper reader and observe only positive progress.
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before = buf.filled().len();
        let result = Pin::new(&mut *lock(&self.io)).poll_read(cx, buf);
        if buf.filled().len() > before {
            self.monitor.read();
        }
        result
    }
}
impl<I: AsyncWrite + Unpin> AsyncWrite for SharedIo<I> {
    /// Actual peer-facing progress, rather than producer polling, drives write idle timing.
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let result = Pin::new(&mut *lock(&self.io)).poll_write(cx, bytes);
        if matches!(result,Poll::Ready(Ok(n)) if n>0) {
            self.monitor.wrote();
        }
        result
    }
    /// Release sequential reuse only after the previous body and write flush complete.
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let result = Pin::new(&mut *lock(&self.io)).poll_flush(cx);
        if matches!(result, Poll::Ready(Ok(()))) {
            self.monitor.flushed();
        }
        result
    }
    /// Hyper's ordinary completion owns native shutdown; deadline cleanup can also call it.
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *lock(&self.io)).poll_shutdown(cx)
    }
}
