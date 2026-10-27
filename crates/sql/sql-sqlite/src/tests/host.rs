//! Named Send handlers use the existing host API without modifying shared crates.

use std::sync::Arc;
use std::time::Duration;

use stackpress_lib::{Error, Flow, Invocation, Nest, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::{Engine, ErrorKind, Transaction};
use tokio::sync::{Barrier, Notify, Semaphore};

use super::{SqliteDatabase, SqliteOptions, TempDir, expire, options, sql, wait_for_cleanup};

/// App-owned synchronization proves two actual root invocations contend for one pool.
struct Contention {
    /// Both roots rendezvous only after the holder owns the native transaction.
    ready: Barrier,
    /// Released only after the competing root observes the actual pool timeout.
    failed: Notify,
    /// Retains visibility of worker retirement after the Engine owns the database.
    admission: Arc<Semaphore>,
    /// All admission slots must return before a healthy recovery read begins.
    permits: u32,
    /// The configured SQLx deadline, advanced only while the pool is explicitly exhausted.
    timeout: Duration,
}

/// Open and register a real Engine as an ordinary typed service during startup.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    let options = start.config::<SqliteOptions>()?.clone();
    let timeout = options.acquire_timeout();
    let permits = u32::try_from(options.max_in_flight).unwrap();
    let native = SqliteDatabase::open(options)
        .await
        .map_err(Error::from_source)?;
    let admission = Arc::clone(&native.admission);
    start.register(Arc::new(Engine::new(native)))?;
    start.register(Arc::new(Contention {
        ready: Barrier::new(2),
        failed: Notify::new(),
        admission,
        permits,
        timeout,
    }))?;
    Ok(Flow::Continue)
}

/// Read through a shared service across await; the signature itself proves Send integration.
async fn leaf(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let engine = inv.service::<Engine>()?;
    let rows = engine
        .fetch(&sql("SELECT 1 AS n"))
        .await
        .map_err(Error::from_source)?;
    res.results(&rows[0].try_get::<i64>("n").map_err(Error::from_source)?)?;
    Ok(Flow::Continue)
}

/// Observe a real queued timeout while the caller deliberately retains the only connection.
async fn blocked_leaf(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let engine = inv.service::<Engine>()?;
    let state = inv.service::<Contention>()?;
    let query = sql("SELECT 1 AS n");
    let error = expire(engine.fetch(&query), state.timeout)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PoolTimedOut);
    res.results(&"PoolTimedOut")?;
    Ok(Flow::Continue)
}

/// A reusable store-like function takes the guard explicitly and never reacquires.
async fn compose(tx: &mut Transaction) -> Result<()> {
    let rows = tx
        .fetch(&sql("SELECT 2 AS n"))
        .await
        .map_err(Error::from_source)?;
    assert_eq!(rows[0].try_get::<i64>("n").unwrap(), 2);
    Ok(())
}

/// Nested dispatch cannot inherit a guard; its explicit reacquisition times out finitely.
async fn nested(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let mut tx = inv
        .service::<Engine>()?
        .begin()
        .await
        .map_err(Error::from_source)?;
    compose(&mut tx).await?;
    let child = inv.resolve("blocked_leaf", Nest::new()).await?;
    assert_eq!(
        child.response.results.unwrap().as_str(),
        Some("PoolTimedOut")
    );
    tx.rollback().await.map_err(Error::from_source)?;
    let state = inv.service::<Contention>()?;
    wait_for_cleanup(&state.admission, state.permits).await;
    res.results(&"composed")?;
    Ok(Flow::Continue)
}

/// Hold a real transaction until the competing root has observed a bounded failure.
async fn hold(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let tx = inv
        .service::<Engine>()?
        .begin()
        .await
        .map_err(Error::from_source)?;
    let state = inv.service::<Contention>()?;
    state.ready.wait().await;
    state.failed.notified().await;
    tx.rollback().await.map_err(Error::from_source)?;
    wait_for_cleanup(&state.admission, state.permits).await;
    res.results(&true)?;
    Ok(Flow::Continue)
}

/// Rendezvous with the held lease, fail finitely, then release the other root.
async fn contend(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let state = inv.service::<Contention>()?;
    state.ready.wait().await;
    let child = inv.resolve("blocked_leaf", Nest::new()).await?;
    res.results(&child.response.results.unwrap())?;
    state.failed.notify_one();
    Ok(Flow::Continue)
}

/// The app registers only its explicit operations; no generic SQL route is created.
pub(super) async fn proof() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 1);
    opts.max_in_flight = 2;
    let mut app = ServerBuilder::new();
    app.config_section("sqlite", opts, |options| {
        options.validate().map_err(Error::from_source)
    })
    .unwrap();
    app.on_config(configure).unwrap();
    app.on("leaf", leaf).unwrap();
    app.on("blocked_leaf", blocked_leaf).unwrap();
    app.on("nested", nested).unwrap();
    app.on("hold", hold).unwrap();
    app.on("contend", contend).unwrap();
    let server = app.build().await.unwrap();
    let result = server.call("nested", Nest::new()).await.unwrap();
    assert!(result.report.failures.is_empty());
    assert_eq!(result.response.result().unwrap().as_str(), Some("composed"));
    let (held, waiting) = tokio::join!(
        server.call("hold", Nest::new()),
        server.call("contend", Nest::new())
    );
    let held = held.unwrap();
    assert!(
        held.report.failures.is_empty(),
        "{:?}",
        held.report
            .failures
            .iter()
            .map(|failure| failure.error.message())
            .collect::<Vec<_>>()
    );
    let waiting = waiting.unwrap();
    assert!(waiting.report.failures.is_empty());
    assert_eq!(
        waiting.response.result().unwrap().as_str(),
        Some("PoolTimedOut")
    );
    assert_eq!(
        server
            .call("leaf", Nest::new())
            .await
            .unwrap()
            .response
            .result()
            .unwrap()
            .as_i64(),
        Some(1)
    );
    let engine = server.host().shared_service::<Engine>().unwrap();
    engine.close().await.unwrap();
    drop(server);
    drop(engine);
}
