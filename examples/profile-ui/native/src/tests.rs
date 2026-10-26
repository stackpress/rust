//! Barrier-controlled cancellation and lifecycle evidence, isolated by a parent watchdog.

use super::*;
use stackpress_lib::{Flow, Invocation, Result, queue::task};
use stackpress_server::{ServerBuilder, ServerInvocationExt};
use std::{
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use tokio::sync::Notify;

/// Test-owned shared receipts; no data-path implementation is duplicated here.
#[derive(Default)]
struct Probe {
    /// Barrier permit is retained even if the parent starts waiting later.
    entered: Notify,
    /// Request lifecycle count distinguishes nested delegation from top-level reentry.
    requests: AtomicUsize,
    /// Response lifecycle count must match admitted successful calls.
    responses: AtomicUsize,
    /// A hanging handler's local guard proves disposal of borrowing work.
    drops: Arc<AtomicUsize>,
}

/// The handler's actual suspended stack owns this drop receipt.
struct DropProbe(Arc<AtomicUsize>);
impl Drop for DropProbe {
    /// Observe cancellation/destruction rather than merely map-entry removal.
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Count only actual general request lifecycle entries.
async fn request(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<Probe>()?
        .requests
        .fetch_add(1, Ordering::SeqCst);
    Ok(Flow::Continue)
}

/// Count only actual general response lifecycle entries.
async fn response(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<Probe>()?
        .responses
        .fetch_add(1, Ordering::SeqCst);
    Ok(Flow::Continue)
}

/// Deliberately suspended read fixture; the parent must cancel its real future.
async fn hang(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Probe>()?;
    let _drop = DropProbe(Arc::clone(&probe.drops));
    probe.entered.notify_one();
    std::future::pending::<()>().await;
    Ok(Flow::Continue)
}

/// Delegate through the existing raw operation interface with no extra lifecycle.
async fn nested(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.delegate("generated-read", req, res).await
}

/// Install the probe through the real startup service registration boundary.
fn setup(builder: &mut ServerBuilder, probe: &Arc<Probe>) {
    let service = Arc::clone(probe);
    builder
        .on_config(task(
            move |start: &mut stackpress_server::StartupContext| {
                let service = Arc::clone(&service);
                Box::pin(async move {
                    start.register(service)?;
                    Ok(Flow::Continue)
                })
            },
        ))
        .unwrap();
    builder.on("request", request).unwrap();
    builder.on("response", response).unwrap();
}

/// Compare a generated nested call and admission failures with exact lifecycle counts.
async fn lifecycle_and_denial() {
    for (window, scheme, origin, allowed) in [
        ("main", "tauri", Some("localhost"), true),
        ("other", "tauri", Some("localhost"), false),
        ("main", "https", Some("localhost"), false),
        ("main", "tauri", Some("evil.invalid"), false),
        ("main", "tauri", None, false),
    ] {
        assert_eq!(validate_caller(window, scheme, origin).is_ok(), allowed);
    }
    let probe = Arc::new(Probe::default());
    let mut builder = ServerBuilder::new();
    setup(&mut builder, &probe);
    // The actual generated function is shared, including its typed Profiles service.
    stackpress_example_generated_profile::generated::register(&mut builder).unwrap();
    builder
        .on(
            "generated-read",
            stackpress_example_generated_profile::generated::server::profile::profile_search,
        )
        .unwrap();
    let server = builder.build().await.unwrap();
    let host = NativeHost::from_server(server);
    let json = r#"{"revision":1,"id":"abc"}"#;
    let text = host.search("main", "one", json).await.unwrap();
    let reply: SearchReply = serde_json::from_str(&text).unwrap();
    assert_eq!(reply.rows[0].name, "Ada Lovelace");
    assert_eq!(reply.total, 3);
    for (window, id, input) in [
        ("other", "one", json),
        ("main", "../event", json),
        ("main", "two", r#"{"revision":99,"id":"abc"}"#),
        (
            "main",
            "two",
            r#"{"revision":1,"id":"abc","event":"shell"}"#,
        ),
    ] {
        assert_eq!(
            host.search(window, id, input).await,
            Err(NativeError::Denied)
        );
    }
    assert_eq!(probe.requests.load(Ordering::SeqCst), 1);
    assert_eq!(probe.responses.load(Ordering::SeqCst), 1);
    assert_eq!(host.in_flight().unwrap(), 0);
    // A separate wrapper demonstrates that raw nested delegation remains within one lifecycle.
    let mut builder = ServerBuilder::new();
    setup(&mut builder, &probe);
    stackpress_example_generated_profile::generated::register(&mut builder).unwrap();
    builder.on("wrapper", nested).unwrap();
    builder
        .on(
            "generated-read",
            stackpress_example_generated_profile::generated::server::profile::profile_search,
        )
        .unwrap();
    let server = builder.build().await.unwrap();
    server
        .call("wrapper", serde_json::json!({"id":"abc456"}))
        .await
        .unwrap();
    assert_eq!(probe.requests.load(Ordering::SeqCst), 2);
    assert_eq!(probe.responses.load(Ordering::SeqCst), 2);
}

/// Ready/go coordination proves actual cancellation and dropping of the suspended handler.
async fn cancellation_and_disposal() {
    let probe = Arc::new(Probe::default());
    let mut builder = ServerBuilder::new();
    setup(&mut builder, &probe);
    builder.on("profile-search", hang).unwrap();
    let host = NativeHost::from_server(builder.build().await.unwrap());
    let registry = Arc::downgrade(&host.registry);
    let json = r#"{"revision":1,"id":"abc"}"#;
    let mut call = Box::pin(host.search("main", "barrier", json));
    tokio::select! { _ = probe.entered.notified() => {}, result = &mut call => panic!("completed before barrier: {result:?}") }
    assert_eq!(host.in_flight().unwrap(), 1);
    assert_eq!(
        host.search("main", "barrier", json).await,
        Err(NativeError::Busy)
    );
    assert_eq!(host.cancel("other", "barrier"), Err(NativeError::Denied));
    host.cancel("main", "barrier").unwrap();
    assert_eq!(call.await, Err(NativeError::Cancelled));
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    assert_eq!(host.in_flight().unwrap(), 0);
    // Dropping an in-flight future also removes the ticket and the handler's stack.
    let mut call = Box::pin(host.search("main", "dispose", json));
    tokio::select! { _ = probe.entered.notified() => {}, result = &mut call => panic!("completed before barrier: {result:?}") }
    drop(call);
    assert_eq!(probe.drops.load(Ordering::SeqCst), 2);
    assert_eq!(host.in_flight().unwrap(), 0);
    // Capacity and closed admission are checked while metadata is held only briefly.
    let tickets = (0..MAX_CALLS)
        .map(|i| host.admit("main", &format!("limit-{i}")).unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        host.admit("main", "extra"),
        Err(NativeError::Busy)
    ));
    drop(tickets);
    host.close().unwrap();
    assert_eq!(
        host.search("main", "closed", json).await,
        Err(NativeError::Busy)
    );
    drop(host);
    assert!(
        registry.upgrade().is_none(),
        "no host/registry ownership cycle"
    );
}

/// The actual lifecycle's safe host error is encoded as response data, not an IPC failure.
async fn public_host_rejection() {
    async fn reject(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        Err(stackpress_lib::Error::new("PRIVATE_NATIVE_CAUSE")
            .with_code(422)
            .with_public_message("Choose another prefix."))
    }
    let mut builder = ServerBuilder::new();
    builder.on("profile-search", reject).unwrap();
    let host = NativeHost::from_server(builder.build().await.unwrap());
    let text = host
        .search("main", "rejected", r#"{"revision":1,"id":"abc"}"#)
        .await
        .unwrap();
    assert!(!text.contains("PRIVATE_NATIVE_CAUSE"));
    let outcome =
        stackpress_example_profile_ui_contracts::decode_search_reply(&text, None).unwrap();
    assert!(
        matches!(outcome, stackpress_example_profile_ui_contracts::SearchOutcome::Rejected(error) if error.code == 422 && error.error == "Choose another prefix." && error.total == 0)
    );
    assert_eq!(host.in_flight().unwrap(), 0);
}

/// Deadline and coordinated shutdown dispose real suspended handler stacks, not just IDs.
async fn deadline_and_shutdown() {
    let probe = Arc::new(Probe::default());
    let mut builder = ServerBuilder::new();
    setup(&mut builder, &probe);
    builder.on("profile-search", hang).unwrap();
    let mut host = NativeHost::from_server(builder.build().await.unwrap());
    host.deadline = Duration::from_millis(30);
    let mut call = Box::pin(host.search("main", "deadline", r#"{"revision":1,"id":"abc"}"#));
    tokio::select! { _ = probe.entered.notified() => {}, result = &mut call => panic!("never entered: {result:?}") }
    assert_eq!(call.await, Err(NativeError::Deadline));
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    assert_eq!(host.in_flight().unwrap(), 0);
    host.deadline = DEADLINE;
    let mut tasks = tokio::task::JoinSet::new();
    for index in 0..8 {
        let host = host.clone();
        tasks.spawn(async move {
            host.search(
                "main",
                &format!("shutdown-{index}"),
                r#"{"revision":1,"id":"abc"}"#,
            )
            .await
        });
    }
    // The counter plus retained notification permit is an entry barrier, not a timed guess.
    while probe.requests.load(Ordering::SeqCst) < 9 {
        probe.entered.notified().await;
    }
    assert_eq!(host.in_flight().unwrap(), 8);
    host.shutdown().await.unwrap();
    while let Some(result) = tasks.join_next().await {
        assert_eq!(result.unwrap(), Err(NativeError::Cancelled));
    }
    assert_eq!(probe.drops.load(Ordering::SeqCst), 9);
    assert_eq!(host.in_flight().unwrap(), 0);
    assert_eq!(
        host.search("main", "closed", r#"{"revision":1,"id":"abc"}"#)
            .await,
        Err(NativeError::Busy)
    );
    host.shutdown().await.unwrap();
}

/// Poison is retained as an operational failure while RAII still discards its metadata.
async fn poisoned_cleanup() {
    let host = NativeHost::new().await.unwrap();
    let ticket = host.admit("main", "poison").unwrap();
    let registry = Arc::clone(&host.registry);
    let weak = Arc::downgrade(&host.registry);
    assert!(
        std::thread::spawn(move || {
            let _guard = registry.lock().unwrap();
            panic!("controlled registry poison");
        })
        .join()
        .is_err()
    );
    drop(ticket);
    assert!(host.registry.is_poisoned());
    assert_eq!(
        host.registry.lock().err().unwrap().into_inner().calls.len(),
        0
    );
    assert_eq!(host.cancel("main", "poison"), Err(NativeError::Unavailable));
    assert_eq!(host.shutdown().await, Err(NativeError::Unavailable));
    assert_eq!(
        host.search("main", "poison", r#"{"revision":1,"id":"abc"}"#)
            .await,
        Err(NativeError::Unavailable)
    );
    drop(host);
    assert!(weak.upgrade().is_none());
}

/// Native cancellation/poison cases cannot silently hang the test process.
#[test]
fn native_cases() {
    if std::env::var_os("PROFILE_UI_NATIVE_CHILD").is_some() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            lifecycle_and_denial().await;
            cancellation_and_disposal().await;
            public_host_rejection().await;
            deadline_and_shutdown().await;
            poisoned_cleanup().await;
        });
        return;
    }
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "tests::native_cases", "--nocapture"])
        .env("PROFILE_UI_NATIVE_CHILD", "1")
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("native cancellation watchdog expired");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
