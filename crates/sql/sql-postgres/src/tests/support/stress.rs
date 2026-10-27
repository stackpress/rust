//! Bounded concurrent load and exact lease/admission accounting on real native pools.
//! Included by both adapter executables, using their private scalar resource snapshots.
use super::{DIALECT, TestResult, integer, open_config, query, run as statement};
use crate::probe::{self, Phase};
use stackpress_sql::{Database, DialectId, ErrorKind, QueryLimits, SqlValue, TransactionOptions};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::task::Poll;
use std::time::{Duration, Instant};

/// Scenario-local pool bounds; these never become a production override API.
#[derive(Clone, Copy)]
pub(super) struct Config {
    /// Native connections, deliberately 1 through 4.
    pub(super) max: u32,
    /// Immediate admission includes every held lease and queued native acquisition.
    pub(super) admission: usize,
    /// Native acquisition deadline; controls use a short distinct timeout.
    pub(super) acquire_ms: u64,
}
impl Config {
    /// The shared public contract requires one slot for deterministic queued cancellation.
    pub(super) fn contract() -> Self {
        Self {
            max: 1,
            admission: 8,
            // Ordinary contract statements use the adapters' public default budget;
            // the deliberate timeout controls below retain their distinct 200ms limit.
            acquire_ms: 5000,
        }
    }
}
/// Requests, acknowledged committed writes and expected lease dispositions from one worker.
#[derive(Default)]
struct Work {
    /// Completed ordinary queries/transaction groups.
    requests: usize,
    /// Writes that the final independent statement must observe.
    writes: i64,
    /// One native lease per ordinary call or complete transaction, never per TX run.
    leases: usize,
    /// Native identity observations prove clean reuse without relying only on private counters.
    identities: BTreeSet<i64>,
}
impl Work {
    /// Merge one joined worker's observations without dropping its native identity evidence.
    fn absorb(&mut self, worker: Self) {
        self.requests += worker.requests;
        self.writes += worker.writes;
        self.leases += worker.leases;
        self.identities.extend(worker.identities);
    }
}
/// Runtime-dispatched owner type stays private to the concrete executable.
type Owner = super::TestDatabase;

/// Wait for returned connections/background retirement, then prove every admission slot is free.
async fn quiescent(database: &Owner, config: Config) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let (size, idle, cleanup, permits) = database.pool_state();
            assert!(size <= config.max);
            if idle == size as usize && cleanup == 0 && permits == config.admission {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("pool/admission did not quiesce within deadline");
}
/// Take a monotonic delta only after all work in the phase has joined and cleanup quiesced.
fn delta(start: [usize; 3]) -> [usize; 3] {
    let end = probe::counts();
    std::array::from_fn(|i| end[i] - start[i])
}
/// Run a mix of reads, writes, commits and rollbacks with disjoint keys and no retries.
async fn worker(database: Arc<Owner>, worker: usize, stop: Instant) -> TestResult<Work> {
    let mut result = Work::default();
    for index in 0..2000 {
        if Instant::now() >= stop {
            break;
        }
        let id = i64::try_from(worker * 1_000_000 + index)?;
        let insert = if DIALECT == DialectId::Postgres {
            "INSERT INTO stress_load(id) VALUES ($1)"
        } else {
            "INSERT INTO stress_load(id) VALUES (?)"
        };
        if index % 4 == 0 {
            let mut transaction = database.begin(TransactionOptions::default()).await?;
            transaction
                .run(
                    query(insert, [SqlValue::Integer(id)]),
                    QueryLimits::default(),
                )
                .await?;
            let sql = if DIALECT == DialectId::Postgres {
                "SELECT id FROM stress_load WHERE id=$1"
            } else {
                "SELECT id FROM stress_load WHERE id=?"
            };
            let observed = transaction
                .run(query(sql, [SqlValue::Integer(id)]), QueryLimits::default())
                .await?;
            assert_eq!(observed.rows[0].try_get_at::<i64>(0)?, id);
            // Alternate acknowledged completion branches while keeping every statement allowlisted.
            if index % 8 == 0 {
                transaction.commit().await?;
                result.writes += 1;
            } else {
                transaction.rollback().await?;
            }
        } else if index % 2 == 0 {
            assert_eq!(
                statement(&database, insert, [SqlValue::Integer(id)])
                    .await?
                    .affected,
                1
            );
            result.writes += 1;
        } else {
            let sql = if DIALECT == DialectId::Postgres {
                "SELECT pg_backend_pid()"
            } else {
                "SELECT CONNECTION_ID()"
            };
            result.identities.insert(integer(&database, sql).await?);
        }
        result.requests += 1;
        result.leases += 1;
    }
    Ok(result)
}
/// Sustain eight concurrent workers for up to two seconds with a hard finite operation cap.
async fn load(name: &str, max: u32) -> TestResult<()> {
    let config = Config {
        max,
        admission: 8,
        acquire_ms: 1000,
    };
    let database = Arc::new(open_config(name, config).await?);
    statement(
        &database,
        "CREATE TABLE stress_load (id BIGINT PRIMARY KEY)",
        [],
    )
    .await?;
    quiescent(&database, config).await;
    let initial = probe::counts();
    let start = Instant::now();
    let stop = start + Duration::from_secs(2);
    let mut workers = tokio::task::JoinSet::new();
    for index in 0..8 {
        workers.spawn(worker(database.clone(), index, stop));
    }
    let mut total = Work::default();
    let mut peak_pool = 0;
    let mut peak_admitted = 0;
    while !workers.is_empty() {
        let (size, _, _, available) = database.pool_state();
        assert!(size <= max);
        assert!(available <= config.admission);
        peak_pool = peak_pool.max(size);
        peak_admitted = peak_admitted.max(config.admission - available);
        tokio::select! {
            joined = workers.join_next() => {
                // Join every owned task before comparing native and application resource counts.
                let completed = joined.unwrap()??;
                total.absorb(completed);
            },
            _ = tokio::time::sleep(Duration::from_millis(2)) => {},
        }
    }
    quiescent(&database, config).await;
    let events = delta(initial);
    assert!(!total.identities.is_empty() && total.identities.len() <= max as usize);
    assert_eq!(
        events,
        [total.leases, total.leases, 0],
        "acknowledged clean load must reuse every lease"
    );
    assert!(
        total.requests >= 64,
        "sustained run made too little progress"
    );
    assert_eq!(
        integer(&database, "SELECT COUNT(*) FROM stress_load").await?,
        total.writes
    );
    println!(
        "stress max={max}: requests={} committed={} duration={:?} peak_native={peak_pool} peak_admitted={peak_admitted} observed_native_ids={:?} acquired/reuse/retire={events:?}; cleanup=0 and permits={} restored",
        total.requests,
        total.writes,
        start.elapsed(),
        total.identities,
        config.admission
    );
    statement(&database, "DROP TABLE stress_load", []).await?;
    database.close().await?;
    assert_eq!(database.pool_state(), (0, 0, 0, config.admission));
    Ok(())
}
/// Exercise deliberate retirement and cancelled leases after sustained clean reuse.
async fn retirement(name: &str) -> TestResult<()> {
    let config = Config {
        max: 1,
        admission: 3,
        acquire_ms: 200,
    };
    let database = open_config(name, config).await?;
    quiescent(&database, config).await;
    let initial = probe::counts();
    for index in 0..24 {
        if index == 0 {
            // A deliberately paused, then acknowledged operation still has one reuse disposition.
            let gate = probe::arm(Phase::AfterPrepare, false);
            let mut operation = database.run(query("SELECT 1", []), QueryLimits::default());
            tokio::select! {
                biased;
                _ = gate.entered() => {},
                _ = &mut operation => panic!("acknowledgement boundary was missed"),
            }
            gate.release();
            assert_eq!(operation.await?.rows[0].try_get_at::<i64>(0)?, 1);
        } else {
            assert_eq!(integer(&database, "SELECT 1").await?, 1);
        }
        assert_eq!(
            database
                .run(
                    query("SELECT absent_stress_column", []),
                    QueryLimits::default()
                )
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::Driver
        );
        // PostgreSQL discards this state before reuse; MySQL must retire its native session.
        statement(
            &database,
            "CREATE TEMPORARY TABLE stress_temporary (id BIGINT)",
            [],
        )
        .await?;
        if index % 4 == 0 {
            let gate = probe::arm(Phase::AfterPrepare, false);
            let mut pending = database.run(query("SELECT 1", []), QueryLimits::default());
            tokio::select! {
                biased;
                _ = gate.entered() => {},
                _ = &mut pending => panic!("cancel boundary was missed"),
            }
            drop(pending);
        }
    }
    quiescent(&database, config).await;
    let events = delta(initial);
    let expected = if DIALECT == DialectId::Postgres {
        [78, 48, 30]
    } else {
        [78, 24, 54]
    };
    assert_eq!(events, expected);
    assert_eq!(integer(&database, "SELECT 9").await?, 9);
    println!(
        "stress retirement policy {DIALECT:?}: acquired/reuse/retire={events:?}; native<=1 cleanup=0 permits=3"
    );
    database.close().await?;
    Ok(())
}
/// Poll once into a definitely exhausted native pool rather than sleeping to guess admission.
async fn pending<T>(future: &mut stackpress_sql::SqlFuture<'_, T>) {
    assert!(
        std::future::poll_fn(|cx| Poll::Ready(future.as_mut().poll(cx)))
            .await
            .is_pending()
    );
}
/// Distinguish prompt admission overflow, acquisition timeout, cancellation and close wakeups.
async fn controls(name: &str) -> TestResult<()> {
    let config = Config {
        max: 1,
        admission: 3,
        acquire_ms: 200,
    };
    let database = open_config(name, config).await?;
    let initial = probe::counts();
    let held = database.begin(TransactionOptions::default()).await?;
    let mut first = database.run(query("SELECT 1", []), QueryLimits::default());
    pending(&mut first).await;
    let mut second = database.begin(TransactionOptions::default());
    let acquisition_started = Instant::now();
    pending(&mut second).await;
    assert_eq!(database.pool_state().3, 0);
    let start = Instant::now();
    // Prompt overflow means Ready on the first poll, not a CPU-dependent wall-clock upper bound.
    let mut overflow_request = database.run(query("SELECT 1", []), QueryLimits::default());
    let Poll::Ready(result) =
        std::future::poll_fn(|cx| Poll::Ready(overflow_request.as_mut().poll(cx))).await
    else {
        panic!("admission overflow queued");
    };
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Limit);
    drop(overflow_request);
    let overflow = start.elapsed();
    drop(first);
    assert_eq!(database.pool_state().3, 1);
    assert_eq!(second.await.err().unwrap().kind(), ErrorKind::PoolTimedOut);
    // Include the whole queued interval, even if overflow checks were descheduled.
    let timeout = acquisition_started.elapsed();
    assert!(
        timeout >= Duration::from_millis(config.acquire_ms),
        "pool timeout was reported before its queued deadline"
    );
    assert_eq!(database.pool_state().3, 2);
    held.rollback().await?;
    quiescent(&database, config).await;
    // Repeated cancelled acquire/begin waiters cannot accumulate permits or native slots.
    for _ in 0..32 {
        let held = database.begin(TransactionOptions::default()).await?;
        let mut cancelled = database.run(query("SELECT 1", []), QueryLimits::default());
        pending(&mut cancelled).await;
        drop(cancelled);
        let mut cancelled = database.begin(TransactionOptions::default());
        pending(&mut cancelled).await;
        drop(cancelled);
        assert_eq!(database.pool_state().3, 2);
        held.rollback().await?;
    }
    quiescent(&database, config).await;
    let held = database.begin(TransactionOptions::default()).await?;
    let mut waiter = database.run(query("SELECT 1", []), QueryLimits::default());
    pending(&mut waiter).await;
    let mut closing = database.close();
    pending(&mut closing).await;
    drop(closing);
    assert_eq!(waiter.await.unwrap_err().kind(), ErrorKind::Closed);
    assert_eq!(
        database
            .run(query("SELECT 1", []), QueryLimits::default())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Closed
    );
    held.rollback().await?;
    database.close().await?;
    database.close().await?;
    assert_eq!(database.pool_state(), (0, 0, 0, config.admission));
    assert_eq!(
        delta(initial),
        [34, 34, 0],
        "queued cancellation must publish no extra lease"
    );
    println!(
        "stress controls: overflow={overflow:?}, acquire_timeout={timeout:?}, 64 cancelled waiters; Closed distinct; native=idle=cleanup=0 permits=3"
    );
    Ok(())
}
/// Parent watchdog bounds the whole real-driver stress case, including cleanup.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let start = Instant::now();
    for max in [1, 2, 4] {
        load(name, max).await?;
    }
    retirement(name).await?;
    controls(name).await?;
    println!("M2 stress total {:?}", start.elapsed());
    Ok(())
}
