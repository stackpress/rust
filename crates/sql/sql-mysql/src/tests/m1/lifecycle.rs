//! Exact cancellation seams, acknowledgement loss and adversarial cleanup deadlines.
use super::*;
use crate::probe::{self, Gate, Phase};
use stackpress_sql::SqlFuture;
use std::error::Error;
use std::sync::Arc;

/// Poll real work until its private boundary is observed, retaining its exclusive borrow.
async fn reached<T>(future: &mut SqlFuture<'_, T>, gate: &Arc<Gate>) {
    tokio::select! {
        biased;
        _ = gate.entered() => {},
        _ = future => panic!("operation completed before the selected native boundary"),
    }
}
/// Exercise cancellation on every ordinary/transaction/native acknowledgement boundary.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let database = open(name, 1).await?;
    super::run(
        &database,
        "CREATE TABLE m1_transactions (id BIGINT PRIMARY KEY) ENGINE=InnoDB",
        [],
    )
    .await?;
    for phase in [
        Phase::BeforePrepare,
        Phase::AfterPrepare,
        Phase::BeforeExecute,
        Phase::AfterRow,
        Phase::AfterExecute,
        Phase::BeforeReuse,
        Phase::AfterReset,
    ] {
        let before = integer(&database, "SELECT CONNECTION_ID()").await?;
        let gate = probe::arm(phase, false);
        let mut pending = database.run(
            query("SELECT 1 UNION ALL SELECT 2", []),
            QueryLimits::default(),
        );
        reached(&mut pending, &gate).await;
        drop(pending);
        assert_ne!(
            integer(&database, "SELECT CONNECTION_ID()").await?,
            before,
            "ordinary phase {phase:?}"
        );
        assert_eq!(integer(&database, "SELECT 9").await?, 9);
    }
    for phase in [
        Phase::BeforePrepare,
        Phase::AfterPrepare,
        Phase::BeforeExecute,
        Phase::AfterRow,
        Phase::AfterExecute,
    ] {
        let mut transaction = database.begin(TransactionOptions::default()).await?;
        transaction
            .run(
                query("INSERT INTO m1_transactions VALUES (1)", []),
                QueryLimits::default(),
            )
            .await?;
        let gate = probe::arm(phase, false);
        let mut pending = transaction.run(
            query("SELECT 1 UNION ALL SELECT 2", []),
            QueryLimits::default(),
        );
        reached(&mut pending, &gate).await;
        drop(pending);
        assert_eq!(
            transaction
                .run(query("SELECT 1", []), QueryLimits::default())
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::RollbackOnly
        );
        transaction.rollback().await?;
        assert_eq!(
            integer(&database, "SELECT COUNT(*) FROM m1_transactions").await?,
            0
        );
    }
    for phase in [Phase::BeforeBegin, Phase::AfterBegin] {
        let before = integer(&database, "SELECT CONNECTION_ID()").await?;
        let gate = probe::arm(phase, false);
        let mut pending = database.begin(TransactionOptions::default());
        reached(&mut pending, &gate).await;
        drop(pending);
        assert_ne!(integer(&database, "SELECT CONNECTION_ID()").await?, before);
    }
    for (phase, committed) in [(Phase::BeforeCommit, 0), (Phase::AfterCommit, 1)] {
        super::run(&database, "DELETE FROM m1_transactions", []).await?;
        let mut transaction = database.begin(TransactionOptions::default()).await?;
        transaction
            .run(
                query("INSERT INTO m1_transactions VALUES (2)", []),
                QueryLimits::default(),
            )
            .await?;
        let gate = probe::arm(phase, false);
        let mut pending = transaction.commit();
        reached(&mut pending, &gate).await;
        drop(pending);
        // Before-send and acknowledged-but-unobserved COMMIT establish both legal
        // outcomes. The API never fabricates a rollback result or retries the write.
        assert_eq!(
            integer(&database, "SELECT COUNT(*) FROM m1_transactions WHERE id=2").await?,
            committed
        );
    }
    for phase in [Phase::BeforeRollback, Phase::AfterRollback] {
        super::run(&database, "DELETE FROM m1_transactions", []).await?;
        let mut transaction = database.begin(TransactionOptions::default()).await?;
        transaction
            .run(
                query("INSERT INTO m1_transactions VALUES (3)", []),
                QueryLimits::default(),
            )
            .await?;
        let gate = probe::arm(phase, false);
        let mut pending = transaction.rollback();
        reached(&mut pending, &gate).await;
        drop(pending);
        assert_eq!(
            integer(&database, "SELECT COUNT(*) FROM m1_transactions").await?,
            0
        );
    }
    // Completion futures own their lease before first poll, for both operations.
    for commit in [false, true] {
        let mut transaction = database.begin(TransactionOptions::default()).await?;
        transaction
            .run(
                query("INSERT INTO m1_transactions VALUES (4)", []),
                QueryLimits::default(),
            )
            .await?;
        let completion = if commit {
            transaction.commit()
        } else {
            transaction.rollback()
        };
        drop(completion);
        assert_eq!(
            integer(&database, "SELECT COUNT(*) FROM m1_transactions").await?,
            0
        );
    }
    // Poll a real native COMMIT once and lose observation without choosing the outcome.
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query("INSERT INTO m1_transactions VALUES (5)", []),
            QueryLimits::default(),
        )
        .await?;
    let mut pending = transaction.commit();
    let first =
        std::future::poll_fn(|context| std::task::Poll::Ready(pending.as_mut().poll(context)))
            .await;
    if let std::task::Poll::Ready(result) = first {
        result?;
    }
    drop(pending);
    let observed = integer(&database, "SELECT COUNT(*) FROM m1_transactions WHERE id=5").await?;
    assert!(observed == 0 || observed == 1, "COMMIT was duplicated");
    super::run(&database, "DELETE FROM m1_transactions", []).await?;
    // Native execution really remains pending beyond a finite caller deadline.
    let gate = probe::arm(Phase::BeforeExecute, false);
    let mut pending = database.run(
        query("SELECT CAST(SLEEP(0.2) AS SIGNED)", []),
        QueryLimits::default(),
    );
    reached(&mut pending, &gate).await;
    gate.release();
    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut pending)
            .await
            .is_err()
    );
    drop(pending);
    assert_eq!(integer(&database, "SELECT 9").await?, 9);
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    let gate = probe::arm(Phase::BeforeExecute, false);
    let mut pending = transaction.run(
        query("SELECT CAST(SLEEP(0.2) AS SIGNED)", []),
        QueryLimits::default(),
    );
    reached(&mut pending, &gate).await;
    gate.release();
    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut pending)
            .await
            .is_err()
    );
    drop(pending);
    assert_eq!(
        transaction.commit().await.unwrap_err().kind(),
        ErrorKind::RollbackOnly
    );
    // Primary native/decode/limit failures survive a separate cleanup failure unchanged.
    for (sql, limits, kind) in [
        (
            "SELECT bad_column FROM m1_transactions",
            QueryLimits::default(),
            ErrorKind::Driver,
        ),
        (
            "SELECT CAST(1 AS DECIMAL(5,2))",
            QueryLimits::default(),
            ErrorKind::Decode,
        ),
        (
            "SELECT 1",
            QueryLimits {
                rows: 0,
                ..QueryLimits::default()
            },
            ErrorKind::Limit,
        ),
    ] {
        let _gate = probe::arm(Phase::BeforeClose, true);
        let error = database.run(query(sql, []), limits).await.unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(error.cleanup_error().unwrap().kind(), ErrorKind::Driver);
        assert!(
            error
                .cleanup_error()
                .unwrap()
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .is_some()
        );
        if kind == ErrorKind::Driver {
            assert!(
                error
                    .source()
                    .unwrap()
                    .downcast_ref::<sqlx::Error>()
                    .is_some()
            );
        }
    }
    // Withholding close acknowledgement cannot hold the lease/admission forever.
    let gate = probe::arm(Phase::BeforeClose, false);
    let mut pending = database.run(
        query("SELECT bad_column FROM m1_transactions", []),
        QueryLimits::default(),
    );
    reached(&mut pending, &gate).await;
    assert_eq!(database.admission.available_permits(), 7);
    let error = pending.await.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert!(
        error
            .cleanup_error()
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<tokio::time::error::Elapsed>()
            .is_some()
    );
    assert_eq!(database.admission.available_permits(), 8);
    // A reset failure occurs after acknowledgement; no duplicate write is attempted.
    let _gate = probe::arm(Phase::BeforeReuse, true);
    assert!(
        database
            .run(
                query("INSERT INTO m1_transactions VALUES (6)", []),
                QueryLimits::default()
            )
            .await
            .is_err()
    );
    assert_eq!(
        integer(&database, "SELECT COUNT(*) FROM m1_transactions WHERE id=6").await?,
        1
    );
    // A reset deadline after an acknowledged write is uncertain to its caller,
    // but cannot replay the write or release admission while reset still owns it.
    let gate = probe::arm(Phase::BeforeReuse, false);
    let mut pending = database.run(
        query("INSERT INTO m1_transactions VALUES (7)", []),
        QueryLimits::default(),
    );
    reached(&mut pending, &gate).await;
    assert_eq!(database.admission.available_permits(), 7);
    let error = pending.await.unwrap_err();
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<tokio::time::error::Elapsed>()
            .is_some()
    );
    assert_eq!(database.admission.available_permits(), 8);
    assert_eq!(
        integer(&database, "SELECT COUNT(*) FROM m1_transactions WHERE id=7").await?,
        1
    );
    // Even a successful operation reports an explicit retirement failure.
    let retiring = database.lease().await?;
    let _gate = probe::arm(Phase::BeforeClose, true);
    assert_eq!(
        retiring.finish(Ok(()), false).await.unwrap_err().kind(),
        ErrorKind::Driver
    );
    // A cancelled run transfers both ownership pieces into tracked background cleanup.
    let gate = probe::arm(Phase::BeforeExecute, false);
    let mut pending = database.run(query("SELECT 1", []), QueryLimits::default());
    reached(&mut pending, &gate).await;
    let close_gate = probe::arm(Phase::BeforeClose, false);
    drop(pending);
    close_gate.entered().await;
    assert_eq!(database.admission.available_permits(), 7);
    assert!(
        tokio::time::timeout(Duration::from_millis(5), database.close())
            .await
            .is_err()
    );
    let failure = database.close().await.unwrap_err();
    assert!(
        failure
            .source()
            .unwrap()
            .downcast_ref::<tokio::time::error::Elapsed>()
            .is_some()
    );
    database.close().await?;
    assert_eq!(database.admission.available_permits(), 8);
    Ok(())
}
