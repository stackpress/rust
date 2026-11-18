//! Simulated-time driver boundary proofs.
use super::*;
use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::Waker,
};
struct Source {
    items: VecDeque<Result<Vec<u8>>>,
    pending: bool,
    drops: Arc<AtomicUsize>,
}
impl Stream for Source {
    type Item = Result<Vec<u8>>;
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.items.pop_front() {
            Some(item) => Poll::Ready(Some(item)),
            None if self.pending => Poll::Pending,
            None => Poll::Ready(None),
        }
    }
}
impl Drop for Source {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
struct Clock(VecDeque<Duration>);
impl Stream for Clock {
    type Item = Duration;
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Duration>> {
        self.0
            .pop_front()
            .map_or(Poll::Pending, |n| Poll::Ready(Some(n)))
    }
}
fn poll<S: Stream + Unpin>(stream: &mut S) -> Poll<Option<S::Item>> {
    Pin::new(stream).poll_next(&mut Context::from_waker(Waker::noop()))
}
fn bytes(value: Poll<Option<Result<Vec<u8>>>>) -> Vec<u8> {
    let Poll::Ready(Some(Ok(value))) = value else {
        panic!("expected data")
    };
    value
}
#[test]
fn heartbeat_age_ties_rotation_and_drop() {
    let drops = Arc::new(AtomicUsize::new(0));
    let source = Source {
        items: VecDeque::from([Ok(b"data: ready\n\n".to_vec())]),
        pending: true,
        drops: Arc::clone(&drops),
    };
    let clock = Clock([15, 30, 300, 3600].map(Duration::from_secs).into());
    let mut driver = SseDriver::new(source, clock, Default::default());
    assert_eq!(bytes(poll(&mut driver)), b": connected\n\n");
    assert_eq!(bytes(poll(&mut driver)), b"data: ready\n\n");
    assert_eq!(bytes(poll(&mut driver)), b": heartbeat\n\n");
    assert_eq!(bytes(poll(&mut driver)), b": heartbeat\n\n");
    assert!(matches!(poll(&mut driver), Poll::Ready(None)));
    assert!(matches!(poll(&mut driver), Poll::Ready(None)));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
#[test]
fn bytes_errors_eof_and_stop_are_fused() {
    for mode in 0..5 {
        let drops = Arc::new(AtomicUsize::new(0));
        let item = match mode {
            0 => Some(Ok(vec![b'x'; 65537])),
            1 => Some(Err(Error::new("source failed"))),
            2 => Some(Ok(vec![b'x'; 14])),
            _ => None,
        };
        let source = Source {
            items: item.into_iter().collect(),
            pending: mode == 4,
            drops: Arc::clone(&drops),
        };
        let options = SseOptions::new(
            Duration::from_secs(60),
            Duration::from_secs(15),
            Duration::from_secs(30),
            26,
        )
        .unwrap();
        let mut driver = SseDriver::new(source, Clock(VecDeque::new()), options);
        assert_eq!(bytes(poll(&mut driver)), b": connected\n\n");
        if mode == 4 {
            driver.stop();
        }
        if mode < 2 {
            assert!(matches!(poll(&mut driver), Poll::Ready(Some(Err(_)))));
        } else {
            assert!(matches!(poll(&mut driver), Poll::Ready(None)));
        }
        assert!(matches!(poll(&mut driver), Poll::Ready(None)));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
#[test]
fn elapsed_observation_prevents_early_heartbeat_after_an_event() {
    let drops = Arc::new(AtomicUsize::new(0));
    let source = Source {
        items: VecDeque::from([Ok(b"data: a\n\n".to_vec())]),
        pending: true,
        drops,
    };
    let mut driver = SseDriver::new(source, Clock(VecDeque::new()), Default::default());
    bytes(poll(&mut driver));
    driver.observe_elapsed(Duration::from_secs(14));
    bytes(poll(&mut driver));
    driver.observe_elapsed(Duration::from_secs(15));
    assert!(poll(&mut driver).is_pending());
    driver.observe_elapsed(Duration::from_secs(29));
    assert_eq!(bytes(poll(&mut driver)), b": heartbeat\n\n");
    assert!(poll(&mut driver).is_pending());
}
