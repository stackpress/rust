//! Direct admission/retirement checks split from the repository-only server lifecycle proof.

use std::sync::Arc;

use stackpress_sql::{Engine, ErrorKind};
use tokio::sync::{Barrier, Notify};

use super::{SqliteDatabase, TempDir, expire, options, sql, wait_for_cleanup};

/// Drive the same held-transaction/time-out path without a later server dependency.
pub(super) async fn proof() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 1);
    opts.max_in_flight = 2;
    let timeout = opts.acquire_timeout();
    let permits = u32::try_from(opts.max_in_flight).unwrap();
    let native = SqliteDatabase::open(opts).await.unwrap();
    let admission = Arc::clone(&native.admission);
    let engine = Engine::new(native);

    // Direct composition retains the same guard while an explicit reacquisition times out.
    let mut tx = engine.begin().await.unwrap();
    let rows = tx.fetch(&sql("SELECT 2 AS n")).await.unwrap();
    assert_eq!(rows[0].try_get::<i64>("n").unwrap(), 2);
    let query = sql("SELECT 1 AS n");
    assert_eq!(
        expire(engine.fetch(&query), timeout)
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::PoolTimedOut
    );
    tx.rollback().await.unwrap();
    // Keep the original private acknowledgement: all slots return before recovery.
    wait_for_cleanup(&admission, permits).await;

    // Two independent callers rendezvous only after the holder owns the native transaction.
    let ready = Barrier::new(2);
    let failed = Notify::new();
    let hold = async {
        let tx = engine.begin().await.unwrap();
        ready.wait().await;
        failed.notified().await;
        tx.rollback().await.unwrap();
        wait_for_cleanup(&admission, permits).await;
    };
    let contend = async {
        ready.wait().await;
        assert_eq!(
            expire(engine.fetch(&query), timeout)
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::PoolTimedOut
        );
        failed.notify_one();
    };
    tokio::join!(hold, contend);
    assert_eq!(
        engine.fetch(&query).await.unwrap()[0]
            .try_get::<i64>("n")
            .unwrap(),
        1
    );
    engine.close().await.unwrap();
}
