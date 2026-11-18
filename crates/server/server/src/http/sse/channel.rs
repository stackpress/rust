//! Bounded subscription ownership, independent of runtime clocks and transport.
use super::SseEvent;
use futures_core::Stream;
use stackpress_lib::{Error, Result};
use std::{
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll},
};

/// Create a runtime-neutral subscription queue with immediate full/closed errors.
/// Count is 1..=16 and encoded byte capacity is 1..=1 MiB; no waiting producer task is spawned.
/// Wrap the sender in an application-owned Arc to share it; its last owner closes the stream.
pub fn channel(capacity: usize, bytes: usize) -> Result<(SseSender, SseReceiver)> {
    if capacity == 0 || capacity > 16 || bytes == 0 || bytes > 1024 * 1024 {
        return Err(Error::invalid("invalid SSE channel capacity"));
    }
    let state = std::sync::Arc::new(Mutex::new(ChannelState {
        queue: Default::default(),
        capacity,
        max_bytes: bytes,
        bytes: 0,
        closed: false,
        waker: None,
    }));
    Ok((
        SseSender {
            state: state.clone(),
        },
        SseReceiver {
            state,
            ended: false,
        },
    ))
}
/// Bounded queue state; a short lock never spans a user poll or callback.
struct ChannelState {
    queue: std::collections::VecDeque<(SseEvent, usize)>,
    capacity: usize,
    max_bytes: usize,
    bytes: usize,
    closed: bool,
    waker: Option<std::task::Waker>,
}
/// Immediate bounded producer endpoint; no unbounded waiting sends are hidden internally.
pub struct SseSender {
    state: std::sync::Arc<Mutex<ChannelState>>,
}
impl SseSender {
    /// Charge the complete encoded cost before enqueueing; full/closed/poisoned queues return Err.
    pub fn try_send(&self, event: SseEvent) -> Result<()> {
        let cost = event.cost()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| Error::new("SSE channel lock poisoned"))?;
        if state.closed {
            return Err(Error::new("SSE channel closed"));
        }
        let bytes = state
            .bytes
            .checked_add(cost)
            .filter(|n| *n <= state.max_bytes);
        if state.queue.len() >= state.capacity || bytes.is_none() {
            return Err(Error::limit("SSE channel full"));
        }
        state.bytes = bytes.ok_or_else(|| Error::limit("SSE channel byte overflow"))?;
        state.queue.push_back((event, cost));
        let waker = state.waker.take();
        drop(state);
        if let Some(waker) = waker {
            waker.wake();
        }
        Ok(())
    }
}
impl Drop for SseSender {
    /// Finish after queued events; poisoned state already ends the consumer with an error.
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.closed = true;
            let waker = state.waker.take();
            drop(state);
            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }
}
/// One owned subscriber; dropping it releases queued events and makes producer sends fail.
pub struct SseReceiver {
    state: std::sync::Arc<Mutex<ChannelState>>,
    ended: bool,
}
impl Stream for SseReceiver {
    type Item = Result<SseEvent>;
    /// Return one event per demand, then fused EOF when the last producer has gone away.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.ended {
            return Poll::Ready(None);
        }
        let Ok(mut state) = this.state.lock() else {
            this.ended = true;
            return Poll::Ready(Some(Err(Error::new("SSE channel lock poisoned"))));
        };
        if let Some((event, cost)) = state.queue.pop_front() {
            state.bytes = state.bytes.saturating_sub(cost);
            return Poll::Ready(Some(Ok(event)));
        }
        if state.closed {
            this.ended = true;
            return Poll::Ready(None);
        }
        state.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
impl Drop for SseReceiver {
    /// A disconnected consumer releases every queued event before closing producer admission.
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.closed = true;
            state.queue.clear();
            state.bytes = 0;
            state.waker = None;
        }
    }
}

#[cfg(test)]
mod channel_tests {
    use super::*;
    #[test]
    fn count_bytes_close_and_released_capacity() {
        assert!(channel(0, 100).is_err());
        assert!(channel(17, 100).is_err());
        assert!(channel(1, 0).is_err());
        assert!(channel(1, 1048577).is_err());
        let (sender, mut receiver) = channel(1, 9).unwrap();
        let mut cx = Context::from_waker(std::task::Waker::noop());
        assert!(Pin::new(&mut receiver).poll_next(&mut cx).is_pending());
        sender.try_send(SseEvent::data("x").unwrap()).unwrap();
        assert!(sender.try_send(SseEvent::data("").unwrap()).is_err());
        assert!(matches!(
            Pin::new(&mut receiver).poll_next(&mut cx),
            Poll::Ready(Some(Ok(_)))
        ));
        assert!(sender.try_send(SseEvent::data("xx").unwrap()).is_err());
        sender.try_send(SseEvent::data("").unwrap()).unwrap();
        drop(sender);
        assert!(matches!(
            Pin::new(&mut receiver).poll_next(&mut cx),
            Poll::Ready(Some(Ok(_)))
        ));
        assert!(matches!(
            Pin::new(&mut receiver).poll_next(&mut cx),
            Poll::Ready(None)
        ));
        assert!(matches!(
            Pin::new(&mut receiver).poll_next(&mut cx),
            Poll::Ready(None)
        ));
        let (sender, receiver) = channel(1, 9).unwrap();
        drop(receiver);
        assert!(sender.try_send(SseEvent::data("").unwrap()).is_err());
    }
}
