//! Root cleanup/cancel/unwind and stale-result proof, under an external watchdog.
#![cfg(feature = "ssr")]
use futures::FutureExt;
use leptos::prelude::*;
use stackpress_leptos::components::*;
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};
#[path = "support/async_watchdog.rs"]
mod watchdog;

/// Child isolation makes an accidental never-resolving cancellation fail in ten seconds.
#[test]
fn yielding_suggestion_lifetime() {
    if std::env::var("S8_ASYNC_CHILD").as_deref() != Ok("suggestions") {
        watchdog::run("suggestions", "yielding_suggestion_lifetime");
        return;
    }
    let caller = Owner::new();
    caller.set();
    provide_context("outer caller");
    for mode in ["cancel", "cleanup", "unwind"] {
        let root = Owner::new();
        let marker = Arc::new(());
        let weak = Arc::downgrade(&marker);
        let called = Arc::new(AtomicUsize::new(0));
        let (state, work) = root.with(|| {
            let state = ComboboxState::new(SelectOptions::new(vec![]).unwrap(), None).unwrap();
            let called = called.clone();
            let work = state
                .request("slow".into(), async move {
                    called.fetch_add(1, Ordering::SeqCst);
                    std::future::pending::<()>().await;
                    drop(marker);
                    Ok(SelectOptions::new(vec![]).unwrap())
                })
                .unwrap();
            // The native future remains Send; browser adapters may instead use local futures.
            fn send<T: Send>(_: &T) {}
            send(&work);
            (state, work)
        });
        let mut work = Box::pin(work);
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(work.as_mut().poll(&mut cx), Poll::Pending));
        assert_eq!(called.load(Ordering::SeqCst), 1);
        assert!(weak.upgrade().is_some());
        match mode {
            "cancel" => state.cancel(),
            "cleanup" => root.cleanup(),
            "unwind" => {
                struct Cleanup(Owner);
                impl Drop for Cleanup {
                    fn drop(&mut self) {
                        self.0.cleanup();
                    }
                }
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _cleanup = Cleanup(root.clone());
                    panic!("trusted suggestion unwind");
                }));
                assert!(outcome.is_err());
            }
            _ => unreachable!(),
        }
        // Exactly one additional poll must observe abort and release the loader capture.
        assert!(matches!(work.as_mut().poll(&mut cx), Poll::Ready(None)));
        assert!(weak.upgrade().is_none());
        assert_eq!(called.load(Ordering::SeqCst), 1);
        assert_eq!(use_context::<&'static str>(), Some("outer caller"));
        root.cleanup();
    }
    let root = Owner::new();
    root.with(|| {
        let state = ComboboxState::new(SelectOptions::new(vec![]).unwrap(), None).unwrap();
        let ready = || async {
            Ok(SelectOptions::new(vec![SelectOption {
                value: "0".into(),
                label: "Zero".into(),
                disabled: false,
            }])
            .unwrap())
        };
        let stale =
            futures::executor::block_on(state.request("old".into(), ready()).unwrap()).unwrap();
        let current =
            futures::executor::block_on(state.request("Zero".into(), ready()).unwrap()).unwrap();
        assert!(!state.publish(stale));
        assert!(state.publish(current));
        assert_eq!(state.matching().len(), 1);
        assert!(state.choose(0));
        assert_eq!(state.selected().as_deref(), Some("0"));
        let failed = state
            .request("bad".into(), async { Err(ControlError) })
            .unwrap()
            .now_or_never()
            .unwrap()
            .unwrap();
        assert!(state.publish(failed));
        assert_eq!(state.status(), SuggestionStatus::Failed);
        let finished =
            futures::executor::block_on(state.request("Zero".into(), ready()).unwrap()).unwrap();
        root.cleanup();
        assert!(!state.publish(finished));
    });
    caller.unset_with_forced_cleanup();
    println!("S8_ASYNC_COMPLETE suggestions");
}
