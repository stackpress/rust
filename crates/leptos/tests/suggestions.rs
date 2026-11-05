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
    for mode in ["cancel", "replace", "cleanup", "unwind"] {
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
            "replace" => {
                state
                    .replace_options(SelectOptions::new(Vec::new()).unwrap())
                    .unwrap();
                assert_eq!(state.query(), "slow");
                assert!(state.open().get());
            }
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

/// Replacement changes only choices/activity, retaining the exact input and selection.
#[test]
fn replace_options_preserves_query_selection_open_and_rejects_atomically() {
    let root = Owner::new();
    root.with(|| {
        let options = |label: &str, disabled| {
            SelectOptions::new(vec![SelectOption {
                value: "0".into(),
                label: label.into(),
                disabled,
            }])
            .unwrap()
        };
        let state = ComboboxState::new(options("Zero", false), Some("0".into())).unwrap();
        state.navigate(true, false);
        assert_eq!(state.active(), Some(0));
        // Unknown/disabled retained identities reject before cancellation, active reset or IO.
        for rejected in [
            SelectOptions::new(Vec::new()).unwrap(),
            options("Disabled", true),
        ] {
            assert!(state.replace_options(rejected).is_err());
            assert_eq!(state.query(), "Zero");
            assert_eq!(state.selected().as_deref(), Some("0"));
            assert_eq!(state.active(), Some(0));
            assert!(state.open().get());
            assert_eq!(state.matching()[0].1.label, "Zero");
            assert_eq!(state.status(), SuggestionStatus::Ready);
        }
        state
            .replace_options(options("Zero refreshed", false))
            .unwrap();
        assert_eq!(state.query(), "Zero");
        assert_eq!(state.selected().as_deref(), Some("0"));
        assert_eq!(state.active(), None);
        assert!(state.open().get());
        assert_eq!(state.matching()[0].1.label, "Zero refreshed");
        state.open().set(false);
        state.replace_options(options("Zero again", false)).unwrap();
        assert!(!state.open().get());
    });
    root.cleanup();
}

/// A completed old acquisition cannot overwrite the application's replacement choices.
#[test]
fn replace_options_invalidates_completed_acquisition_without_changing_query() {
    let root = Owner::new();
    root.with(|| {
        let state = ComboboxState::new(SelectOptions::new(Vec::new()).unwrap(), None).unwrap();
        let stale = futures::executor::block_on(
            state
                .request("Ada".into(), async {
                    Ok(SelectOptions::new(Vec::new()).unwrap())
                })
                .unwrap(),
        )
        .unwrap();
        state
            .replace_options(
                SelectOptions::new(vec![SelectOption {
                    value: "p".into(),
                    label: "Ada Lovelace".into(),
                    disabled: false,
                }])
                .unwrap(),
            )
            .unwrap();
        assert!(!state.publish(stale));
        assert_eq!(state.query(), "Ada");
        assert_eq!(state.selected(), None);
        assert!(state.open().get());
        assert_eq!(state.matching()[0].1.label, "Ada Lovelace");
        assert!(state.choose(0));
        assert_eq!(state.selected().as_deref(), Some("p"));
    });
    root.cleanup();
}
