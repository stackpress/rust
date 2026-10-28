//! Hyper body ownership and terminal observation; never spawns a producer or completion task.

use crate::monitor::Monitor;
use crate::report::{ExchangeSlot, HttpDelivery, lock};
use bytes::Bytes;
use futures_core::Stream;
use http_body::{Body, Frame};
use stackpress_lib::Error;
use stackpress_server::http::stream::HttpBody;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// The exclusive producer and its report share the connection's bounded lifetime.
pub(crate) struct TrackedBody {
    /// Dropped on EOF/error/disconnect; never polled after termination.
    source: Option<HttpBody>,
    /// State owned also by the connection so a body drop cannot lose its report.
    exchange: ExchangeSlot,
    /// Shared only with this connection; EOF is followed by a separate native flush boundary.
    monitor: Arc<Monitor>,
}
impl TrackedBody {
    /// Wrap the already published producer without polling it.
    pub(crate) fn new(source: HttpBody, exchange: ExchangeSlot, monitor: Arc<Monitor>) -> Self {
        // Hyper need not poll body-forbidden/empty responses; local absence is already known.
        let source = if source.is_finished() {
            monitor.body_done();
            None
        } else {
            Some(source)
        };
        Self {
            source,
            exchange,
            monitor,
        }
    }
    /// An empty boundary rejection still has a native flush, without fabricating a host report.
    pub(crate) fn empty(exchange: ExchangeSlot, monitor: Arc<Monitor>) -> Self {
        monitor.body_done();
        Self {
            source: None,
            exchange,
            monitor,
        }
    }
}
impl Body for TrackedBody {
    type Data = Bytes;
    type Error = Arc<Error>;
    /// Observe producer EOF/error once, handing successful chunks to Hyper on demand.
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let Some(source) = self.source.as_mut() else {
            return Poll::Ready(None);
        };
        let item = match Pin::new(source).poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(item) => item,
        };
        match item {
            Some(Ok(bytes)) => {
                // Counts measure handoff to Hyper, not a socket flush or remote ACK.
                let length = bytes.len() as u64;
                // A fixed-length buffered representation may never receive a final EOF poll.
                let finished = self.source.as_ref().is_some_and(HttpBody::is_finished);
                if finished {
                    self.source = None;
                }
                let mut state = lock(&self.exchange);
                state.report.bytes = state.report.bytes.saturating_add(length);
                drop(state);
                if finished {
                    self.monitor.body_done();
                }
                Poll::Ready(Some(Ok(Frame::data(Bytes::from(bytes)))))
            }
            Some(Err(error)) => {
                // Keep the original typed producer error; no JSON fallback can follow publication.
                self.source = None;
                let error = Arc::new(error);
                lock(&self.exchange).fail(Arc::clone(&error));
                Poll::Ready(Some(Err(error)))
            }
            None => {
                self.source = None;
                self.monitor.body_done();
                Poll::Ready(None)
            }
        }
    }
    /// EOF is true only after the producer has actually completed or failed.
    fn is_end_stream(&self) -> bool {
        self.source.is_none()
    }
}
impl Drop for TrackedBody {
    /// Hyper dropping an unfinished body cancels its producer and trusted handler context.
    fn drop(&mut self) {
        let mut state = lock(&self.exchange);
        if self.source.is_some() && state.report.delivery == HttpDelivery::Published {
            state.report.delivery = HttpDelivery::Cancelled;
        }
        state.cancellation.cancel();
    }
}
