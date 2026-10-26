//! M2 native plans, JSON semantics and transition proofs, each externally time-bounded.

mod cancellation;
mod ddl;
mod json;
mod plans;
mod queries;

use std::time::Duration;

use stackpress_sql::Engine;

use super::{SqliteDatabase, TempDir, options, sql, watchdog};

/// Read schema identity/content through ordinary public statements for before/after checks.
async fn schema(engine: &Engine) -> Vec<stackpress_sql::Row> {
    engine
        .fetch(&sql(
            "SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name",
        ))
        .await
        .unwrap()
}

/// Parent bounds every native scenario even if the current-thread runtime stops progressing.
#[test]
fn native_execution_under_watchdog() {
    for scenario in [
        "ddl",
        "schema_cache",
        "atomic",
        "unsupported",
        "json",
        "json_types",
        "queries",
        "transactions",
        "fetch_cancel",
        "rollback_cancel",
        "ddl_cancel",
        "prepare_failure",
        "open_cancel",
        "native_failures",
        "contention",
        "plan_validation",
        "plan_cancel",
        "plan_transaction",
    ] {
        let (success, expired, output) =
            watchdog::run("tests::m2::native_child", scenario, Duration::from_secs(10));
        assert!(
            success && !expired,
            "M2 {scenario}: timeout={expired}\n{output}"
        );
    }
}

/// Only the watchdog parent launches this current-thread entry point.
#[tokio::test(flavor = "current_thread")]
#[ignore = "launched only by M2 watchdog parent"]
async fn native_child() {
    let scenario = std::env::var("STACKPRESS_SQL_M0_SCENARIO").unwrap();
    let work = async {
        match scenario.as_str() {
            "ddl" => ddl::supported().await,
            "schema_cache" => ddl::cached_schema().await,
            "atomic" => ddl::atomic().await,
            "unsupported" => ddl::unsupported().await,
            "json" => json::paths().await,
            "json_types" => json::types().await,
            "queries" => queries::builders().await,
            "transactions" => queries::transactions().await,
            "fetch_cancel" => cancellation::fetch().await,
            "rollback_cancel" => cancellation::rollback().await,
            "ddl_cancel" => cancellation::ddl().await,
            "prepare_failure" => cancellation::prepare().await,
            "open_cancel" => cancellation::open().await,
            "native_failures" => cancellation::native_failures().await,
            "contention" => cancellation::contention().await,
            "plan_validation" => plans::validation().await,
            "plan_cancel" => plans::cancellation().await,
            "plan_transaction" => plans::caller_transaction().await,
            _ => panic!("unknown M2 scenario"),
        }
    };
    tokio::time::timeout(Duration::from_secs(8), work)
        .await
        .expect("M2 cooperative deadline");
}
