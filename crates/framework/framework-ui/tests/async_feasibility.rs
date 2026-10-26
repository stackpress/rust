//! Slice 8 M0 counterexamples for component-owned asynchronous rendering.
//!
//! These tests characterize the pinned Leptos executor, not a supported async
//! renderer. Raw Resources retain loaders; guarded cases prove the cooperative fix.
//! Each scenario runs in a separate, externally killed/reaped process.
//!
//! Passing raw-case assertions means the counterexample still exists; cooperative
//! cases prove bounded cleanup with the guard. This fixture alone does not prove
//! replay/hydration; the renderer and real browser fixtures cover those contracts.
//! Caller restoration is provided explicitly by this fixture's polling guard.
#![cfg(feature = "ssr")]

use futures::{FutureExt, StreamExt, channel::oneshot};
use leptos::prelude::*;
use leptos_meta::{ServerMetaContext, Title};
use stackpress_framework_ui::render::initialize_ssr;
use std::{
    future::Future,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicUsize, Ordering},
    },
};

#[path = "support/async_watchdog.rs"]
mod watchdog;

/// Explicit completion policy for the real Resource's loader.
#[derive(Clone, Copy)]
enum LoadMode {
    /// Resolve normally after the executor has polled the spawned loader.
    Complete,
    /// Remain pending until the runtime is destroyed.
    Pending,
}

/// Closed proof scenarios; no free-form input can select a fixture behavior.
#[derive(Clone, Copy)]
enum Scenario {
    /// Positive control for body/Head completion.
    Success,
    /// Drop the render as a host would on disconnect.
    Cancel,
    /// Unwind trusted rendering after the loader is admitted.
    Unwind,
    /// A completed guarded loader is unchanged by later cleanup.
    CooperativeSuccess,
    /// Guarded pending work drops within the bounded scheduler turns.
    CooperativeCancel,
    /// Guarded pending work also drops after render unwind.
    CooperativeUnwind,
}

impl Scenario {
    /// Stable subprocess selector and diagnostic label.
    fn key(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Cancel => "cancel",
            Self::Unwind => "unwind",
            Self::CooperativeSuccess => "cooperative-success",
            Self::CooperativeCancel => "cooperative-cancel",
            Self::CooperativeUnwind => "cooperative-unwind",
        }
    }

    /// Exact test filter prevents the child from spawning another parent.
    fn test_name(self) -> &'static str {
        match self {
            Self::Success => "resource_success_control",
            Self::Cancel => "resource_cancellation_counterexample",
            Self::Unwind => "resource_unwind_counterexample",
            Self::CooperativeSuccess => "cooperative_success_releases_loader",
            Self::CooperativeCancel => "cooperative_cancel_releases_loader",
            Self::CooperativeUnwind => "cooperative_unwind_releases_loader",
        }
    }
}

/// Observe the loader independently of the component and render future.
#[derive(Clone)]
struct Probe {
    /// Only the loader upgrades this reference; the fixture drops its own Arc.
    marker: Weak<()>,
    /// Acknowledges that the spawned loader has reached its pending/completion point.
    started: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    /// The supervising root explicitly admits the loader after observing readiness.
    go: Arc<Mutex<Option<oneshot::Receiver<()>>>>,
    /// Confirms admission has finished before the root attempts cancellation.
    admitted: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    /// Controls the single loader invocation without a mutable signal/source.
    mode: LoadMode,
    /// Select the owned cancellation guard while preserving the raw counterexample.
    cooperative: bool,
    /// Count loader calls separately from component/Suspense polling.
    calls: Arc<AtomicUsize>,
    /// Witness that cleanup reached the Resource's own reactive child Owner.
    cleanups: Arc<AtomicUsize>,
}

/// Exercise a real serializable Resource, Suspense, and deferred reactive Head.
#[component]
fn ResourcePage(probe: Probe) -> impl IntoView {
    // Keep the source stable; repeated render polling must not reload the resource.
    let resource = Resource::new(
        || (),
        move |()| {
            // A stable source must load exactly once, including repeated polls.
            assert_eq!(probe.calls.fetch_add(1, Ordering::SeqCst), 0);
            let cleanups = probe.cleanups.clone();
            Owner::on_cleanup(move || {
                // Loader retention must remain observable even after this runs.
                cleanups.fetch_add(1, Ordering::SeqCst);
            });
            let marker = probe.marker.upgrade().expect("fixture still owns marker");
            let started = probe.started.lock().unwrap().take();
            let go = probe
                .go
                .lock()
                .unwrap()
                .take()
                .expect("one loader admission");
            let admitted = probe.admitted.lock().unwrap().take();
            let mode = probe.mode;
            let cooperative = probe.cooperative;
            let loader = async move {
                // Resource eagerly polls once on construction; yield forces the
                // next poll through its independently spawned executor task.
                tokio::task::yield_now().await;
                if let Some(started) = started {
                    // Announce readiness without racing the supervising root.
                    let _ = started.send(());
                }
                // No mutex guard crosses this root-controlled admission barrier.
                go.await.expect("root admits the ready loader");
                if let Some(admitted) = admitted {
                    // On this single-thread executor the loader returns Pending
                    // or completes before the awakened supervisor can run.
                    let _ = admitted.send(());
                }
                if matches!(mode, LoadMode::Pending) {
                    // Keep the capture live across an indefinitely pending await.
                    std::future::pending::<()>().await;
                }
                drop(marker);
                "Resource title".to_owned()
            };
            if cooperative {
                let guarded = stackpress_framework_ui::resources::cancel_on_cleanup(loader);
                futures::future::Either::Left(async move {
                    guarded.await.unwrap_or_else(|_| "cancelled".to_owned())
                })
            } else {
                futures::future::Either::Right(loader)
            }
        },
    );
    // Head reads a signal while its Owner is alive, after Suspense has settled.
    let title = RwSignal::new("Pending title".to_owned());
    view! {
        <Title text=move || title.get()/>
        <Suspense fallback=|| view! { <p>"Loading"</p> }>
            {move || resource.get().map(move |value| {
                // The actual component publishes the resolved title and body.
                title.set(value.clone());
                view! { <p>{value}</p> }
            })}
        </Suspense>
    }
}

/// Compile-time witness for the native buffered future's required Send bound.
fn assert_send<T: Send>(_: &T) {}

/// Fixture-only restoration; never retain an entered Owner guard across await.
struct RestoreOwner(Owner);

impl Drop for RestoreOwner {
    /// Keep fixture cleanup from being mistaken for an upstream restoration guarantee.
    fn drop(&mut self) {
        self.0.set();
    }
}

/// Prove normal completion works and cancellation/unwind retain spawned work.
fn scenario(mode: Scenario) {
    let cooperative = matches!(
        mode,
        Scenario::CooperativeSuccess | Scenario::CooperativeCancel | Scenario::CooperativeUnwind
    );
    // A current-thread runtime makes the ready barrier and cleanup order explicit.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let marker = Arc::new(());
    let weak = Arc::downgrade(&marker);
    runtime.block_on(async {
        initialize_ssr().unwrap();
        let caller = Owner::new();
        caller.set();
        provide_context("caller context");
        let root = Owner::new();
        let (meta, head) = ServerMetaContext::new();
        let (started_tx, started_rx) = oneshot::channel();
        let (go_tx, go_rx) = oneshot::channel();
        let (admitted_tx, admitted_rx) = oneshot::channel();
        let calls = Arc::new(AtomicUsize::new(0));
        let cleanups = Arc::new(AtomicUsize::new(0));
        let probe = Probe {
            marker: weak.clone(),
            started: Arc::new(Mutex::new(Some(started_tx))),
            go: Arc::new(Mutex::new(Some(go_rx))),
            admitted: Arc::new(Mutex::new(Some(admitted_tx))),
            mode: if matches!(mode, Scenario::Success | Scenario::CooperativeSuccess) {
                LoadMode::Complete
            } else {
                LoadMode::Pending
            },
            cooperative,
            calls: calls.clone(),
            cleanups: cleanups.clone(),
        };
        let stream = root.with(|| {
            provide_context(meta);
            leptos_meta::provide_meta_context();
            view! { <ResourcePage probe/> }.to_html_stream_in_order()
        });
        let mut body = Box::pin(stream.collect::<String>());
        assert_send(&body);
        drop(marker);

        // The barrier excludes the benign case of cancelling an unstarted task.
        started_rx.await.expect("spawned loader reached barrier");
        go_tx
            .send(())
            .expect("ready loader is waiting for admission");
        admitted_rx.await.expect("loader passed admission barrier");
        assert_eq!(Owner::current(), Some(caller.clone()));
        if matches!(mode, Scenario::Success | Scenario::CooperativeSuccess) {
            let body = std::future::poll_fn(|context| {
                let _restore = RestoreOwner(caller.clone());
                root.with(|| body.as_mut().poll(context))
            })
            .await;
            let document = format!("<html><head><!--HEAD--></head><body>{body}</body></html>");
            let document = head
                .inject_meta_context(futures::stream::iter([document]))
                .await
                .collect::<String>()
                .await;
            assert!(document.contains("<p>Resource title</p>"), "{document}");
            assert!(
                document.contains("<title>Resource title</title>"),
                "{document}"
            );
            root.cleanup();
            drop(root);
            assert!(
                weak.upgrade().is_none(),
                "completed loader releases capture"
            );
        } else {
            // Pending means a real Suspense render is waiting, not just a loader
            // instantiated without ever starting component rendering.
            assert!(body.as_mut().now_or_never().is_none());
            if matches!(mode, Scenario::Unwind | Scenario::CooperativeUnwind) {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _restore = RestoreOwner(caller.clone());
                    let _body = body;
                    root.with(|| panic!("trusted render unwind after Resource admission"));
                }));
                assert!(result.is_err());
            } else {
                // Mirror dropping an uncommitted buffered render on disconnect.
                drop(body);
            }
            drop(head);
            root.cleanup();
            drop(root);
            for _ in 0..16 {
                // Give any cleanup-triggered cancellation task a chance to run.
                tokio::task::yield_now().await;
            }
            if cooperative {
                assert!(
                    weak.upgrade().is_none(),
                    "cooperative loader must drop within 16 turns"
                );
            } else {
                assert!(weak.upgrade().is_some(), "raw counterexample changed");
            }
            println!(
                "RESOURCE {}: render dropped and Owner cleaned; capture checked",
                mode.key()
            );
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            cleanups.load(Ordering::SeqCst),
            1,
            "Resource child was cleaned"
        );
        assert_eq!(Owner::current(), Some(caller.clone()));
        assert_eq!(use_context::<&'static str>(), Some("caller context"));
        caller.unset_with_forced_cleanup();
    });
    // Runtime destruction, not render cancellation, finally drops pending work.
    drop(runtime);
    assert!(
        weak.upgrade().is_none(),
        "runtime shutdown must reap the fixture task"
    );
    // The parent requires this marker, so an unmatched test filter cannot pass.
    println!("S8_ASYNC_COMPLETE {}", mode.key());
}

/// Enter the fixed child scenario or supervise a separate test executable.
fn watched(mode: Scenario) {
    if std::env::var("S8_ASYNC_CHILD").as_deref() == Ok(mode.key()) {
        // A matching exact filter is the only way into potentially hanging work.
        scenario(mode);
    } else {
        // The parent owns the deadline, logs, kill and reap on every exit path.
        watchdog::run(mode.key(), mode.test_name());
    }
}

/// Successful Resource/Suspense rendering can settle deferred reactive Head.
#[test]
fn resource_success_control() {
    watched(Scenario::Success);
}

/// Dropping a render and cleaning its Owner do not cancel its pending Resource.
#[test]
fn resource_cancellation_counterexample() {
    watched(Scenario::Cancel);
}

/// Unwinding the render also leaves admitted Resource work until runtime shutdown.
#[test]
fn resource_unwind_counterexample() {
    watched(Scenario::Unwind);
}

/// A successful subprocess that executes no scenario must fail the parent proof.
#[test]
#[should_panic(expected = "child did not complete its scenario")]
fn unmatched_child_filter_is_rejected() {
    watchdog::run("unmatched-filter", "no-such-s8-async-test");
}

/// Completed guarded data is unaffected by cleanup after its result is consumed.
#[test]
fn cooperative_success_releases_loader() {
    watched(Scenario::CooperativeSuccess);
}

/// Disposed roots wake/drop guarded pending loaders within 16 scheduler turns.
#[test]
fn cooperative_cancel_releases_loader() {
    watched(Scenario::CooperativeCancel);
}

/// Render unwind has the same cooperative capture-release bound.
#[test]
fn cooperative_unwind_releases_loader() {
    watched(Scenario::CooperativeUnwind);
}
