//! Explicit MySQL target facts use the existing emit grammar and never open a network DB.
use serde_json::json;
use std::time::Duration;
#[path = "../../support/watchdog.rs"]
mod watchdog;

/// Bound filesystem, CLI and optional native-refusal checks independently of their runtime.
#[test]
fn supervised_storage_targets() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "supervised_storage_targets",
        "storage-targets",
        Duration::from_secs(20),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// Invalid facts refuse before reading a missing source or creating history; valid facts are hashed.
async fn prove() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let history = root.path().join("history");
    let missing = root.path().join("missing.idea");
    let run = generated_workspace_tool::maintenance::run;
    for options in [
        json!({"mysql-collation":"utf8mb4_bin"}),
        json!({"mysql-page-bytes":16384}),
        json!({"mysql-collation":"latin1_bin","mysql-page-bytes":16384}),
        json!({"mysql-collation":"utf8mb4_bin","mysql-page-bytes":4097}),
        json!({"mysql-collation":"utf8mb4_bin","mysql-page-bytes":16384.0}),
    ] {
        let mut input = json!({"history":history,"input":missing,"name":"one"});
        input
            .as_object_mut()
            .unwrap()
            .extend(options.as_object().unwrap().clone());
        let error = run("revision", input).await.unwrap_err();
        // A source acquisition failure would retain an IO/transform cause instead.
        assert!(std::error::Error::source(&error).is_none_or(|cause| {
            cause
                .downcast_ref::<stackpress_framework_sql::DataError>()
                .is_some()
        }));
        assert!(!history.exists());
    }
    let input = root.path().join("schema.idea");
    std::fs::write(
        &input,
        include_str!("../../../../crates/framework/framework-sql/tests/proofs/lengths/schema.idea"),
    )
    .unwrap();
    run("revision",json!({"history":history,"input":input,"name":"one","mysql-collation":"utf8mb4_bin","mysql-page-bytes":16384})).await.unwrap();
    let plan = run(
        "migration-plan",
        json!({"history":history,"to":"one","dialect":"mysql"}),
    )
    .await
    .unwrap();
    assert!(plan["plan"].is_string());
    let revisions = stackpress_framework_sql::migrations::RevisionStore::new(&history)
        .check()
        .unwrap();
    assert!(
        revisions[0]
            .tables()
            .iter()
            .all(|table| table.mysql_storage.as_ref().unwrap().facts()
                == Some(("utf8mb4_bin", 16384)))
    );
    #[cfg(feature = "database-tools")]
    {
        let database = root.path().join("must-not-open.sqlite");
        assert!(
            run(
                "db-install",
                json!({"history":history,"database":database,"plan":plan["plan"]})
            )
            .await
            .is_err()
        );
        assert!(!database.exists());
    }
    // Absence of facts still records the ordinary unresolved policy, without inference.
    let other = root.path().join("unresolved");
    run(
        "revision",
        json!({"history":other,"input":input,"name":"one"}),
    )
    .await
    .unwrap();
    let error = run(
        "migration-plan",
        json!({"history":other,"to":"one","dialect":"mysql"}),
    )
    .await;
    assert!(error.is_err());
}
