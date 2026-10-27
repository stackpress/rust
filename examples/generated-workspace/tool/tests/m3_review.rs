//! Invalid executable plans still report every schema loss finding without publishing authority.
use serde_json::json;
use std::time::Duration;

#[path = "../../support/watchdog.rs"]
mod watchdog;

/// Bound source composition and immutable file work independently of the runtime.
#[test]
fn supervised_review_report() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(review_report());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "supervised_review_report",
        "aggregate-review",
        Duration::from_secs(15),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// A missing backfill does not hide other table/field/type losses or create an executable artifact.
async fn review_report() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let history = root.path().join("history");
    let input = root.path().join("schema.idea");
    let before = "plugin \"framework-schema\" {} plugin \"framework-sql\" {} model Profile { id Integer @id optional String? removed String? retyped String? } model Obsolete { id Integer @id }";
    let after = "plugin \"framework-schema\" {} plugin \"framework-sql\" {} model Profile { id Integer @id optional String retyped Integer? new_required String }";
    std::fs::write(&input, before).unwrap();
    let run = generated_workspace_tool::maintenance::run;
    run(
        "revision",
        json!({"history":history,"input":input,"name":"v1"}),
    )
    .await
    .unwrap();
    std::fs::write(&input, after).unwrap();
    run(
        "revision",
        json!({"history":history,"input":input,"name":"v2"}),
    )
    .await
    .unwrap();
    let report = run("migration-plan", json!({"history":history,"to":"v2"}))
        .await
        .unwrap();
    assert_eq!(report["outcome"], "Blocked");
    assert!(report.get("plan").is_none());
    let findings = report["findings"].as_array().unwrap();
    for kind in [
        "drop_table",
        "drop_column",
        "retype_or_narrow",
        "nullable_to_required",
        "required_addition_without_backfill",
    ] {
        assert!(
            findings
                .iter()
                .any(|finding| finding["kind"] == kind && finding["destructive"] == true),
            "{kind}"
        );
    }
    assert!(!history.join("migrations").exists());
}
