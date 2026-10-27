//! App-owned emit command proof; no generated application dependency or listener.
#![cfg(feature = "database-tools")]
use serde_json::{Value, json};
use std::time::Duration;
#[path = "../../support/watchdog.rs"]
mod watchdog;
/// Bound native/CLI work independently of the current-thread operator runtime.
#[test]
fn supervised_operators() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(operators());
        return;
    }
    let (ok, expired, output) =
        watchdog::run("supervised_operators", "operators", Duration::from_secs(30));
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// Run explicit history, plan, install, seed, upgrade, downgrade, purge and uninstall capabilities.
async fn operators() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let history = root.path().join("schema");
    let input = root.path().join("schema.idea");
    let database = root.path().join("data.sqlite");
    let source = "plugin \"framework-schema\" {} plugin \"framework-sql\" {} model Profile { id String @id name String }";
    std::fs::write(&input, source).unwrap();
    let run = generated_workspace_tool::maintenance::run;
    let first = run(
        "revision",
        json!({"history":history,"input":input,"name":"v1"}),
    )
    .await
    .unwrap();
    assert_eq!(first["outcome"], "Recorded");
    assert_eq!(
        run(
            "revision",
            json!({"history":history,"input":input,"name":"ignored"})
        )
        .await
        .unwrap()["outcome"],
        "NoChanges"
    );
    assert!(!root.path().join("generated").exists());
    assert!(!database.exists());
    let baseline = run("migration-plan", json!({"history":history,"to":"v1"}))
        .await
        .unwrap();
    let hash = baseline["plan"].as_str().unwrap();
    let install = json!({"history":history,"database":database,"plan":hash});
    assert_eq!(
        run("db-install", install.clone()).await.unwrap()["outcome"],
        "Applied"
    );
    assert_eq!(
        run("db-install", install).await.unwrap()["outcome"],
        "AlreadyApplied"
    );
    let seed_file = root.path().join("seed.json");
    std::fs::write(&seed_file,r#"[{"table":"profile","values":{"id":{"type":"text","value":"p1"},"name":{"type":"text","value":"Ada"}},"upsert":true}]"#).unwrap();
    let seed=run("migration-plan",json!({"history":history,"to":"v1","operation":"populate","seed":"initial-v1","seed-file":seed_file})).await.unwrap();
    let data =
        json!({"history":history,"database":database,"plan":seed["plan"],"seed":"initial-v1"});
    assert_eq!(
        run("db-populate", data.clone()).await.unwrap()["outcome"],
        "Applied"
    );
    assert_eq!(
        run("db-populate", data).await.unwrap()["outcome"],
        "AlreadyApplied"
    );
    assert!(
        run(
            "db-populate",
            json!({"history":history,"database":database,"plan":seed["plan"],"seed":"wrong"})
        )
        .await
        .is_err()
    );
    std::fs::write(
        &input,
        source.replace("name String", "name String summary String?"),
    )
    .unwrap();
    run(
        "revision",
        json!({"history":history,"input":input,"name":"v2"}),
    )
    .await
    .unwrap();
    let upgrade = run("migration-plan", json!({"history":history,"to":"v2"}))
        .await
        .unwrap();
    let data = json!({"history":history,"database":database,"plan":upgrade["plan"]});
    run("db-upgrade", data.clone()).await.unwrap();
    assert!(run("db-downgrade", data).await.is_err());
    assert!(
        run("db-status", json!({"history":history,"database":database}))
            .await
            .unwrap()["schema_matches"]
            .as_bool()
            .unwrap()
    );
    // Logical/UI declaration order alone must retain actual slots and report matching applied state.
    std::fs::write(
        &input,
        source.replace(
            "id String @id name String",
            "name String id String @id summary String?",
        ),
    )
    .unwrap();
    run(
        "revision",
        json!({"history":history,"input":input,"name":"v2-layout"}),
    )
    .await
    .unwrap();
    let layout = run(
        "migration-plan",
        json!({"history":history,"to":"v2-layout"}),
    )
    .await
    .unwrap();
    run(
        "db-upgrade",
        json!({"history":history,"database":database,"plan":layout["plan"]}),
    )
    .await
    .unwrap();
    assert_eq!(
        run("db-status", json!({"history":history,"database":database}))
            .await
            .unwrap()["schema_matches"],
        true
    );
    run(
        "db-downgrade",
        json!({"history":history,"database":database,"plan":layout["reverse_plan"]}),
    )
    .await
    .unwrap();
    run(
        "db-downgrade",
        json!({"history":history,"database":database,"plan":upgrade["reverse_plan"]}),
    )
    .await
    .unwrap();
    let purge = run(
        "migration-plan",
        json!({"history":history,"to":"v1","operation":"purge"}),
    )
    .await
    .unwrap();
    let mut forced = std::process::Command::new(env!("CARGO_BIN_EXE_generated-workspace-tool"));
    forced
        .args(["-f", "emit", "db-purge", "--history"])
        .arg(&history)
        .arg("--database")
        .arg(&database)
        .arg("--plan")
        .arg(purge["plan"].as_str().unwrap());
    cli_command(forced, false);
    assert!(
        run(
            "db-purge",
            json!({"history":history,"database":database,"plan":purge["plan"]})
        )
        .await
        .is_err()
    );
    assert!(run("db-purge",json!({"history":history,"database":database,"plan":purge["plan"],"allow-data-loss":true})).await.is_err());
    run("db-purge",json!({"history":history,"database":database,"plan":purge["plan"],"allow-data-loss":true,"target":"local-test"})).await.unwrap();
    let uninstall = run(
        "migration-plan",
        json!({"history":history,"to":"v1","operation":"uninstall"}),
    )
    .await
    .unwrap();
    run("db-uninstall",json!({"history":history,"database":database,"plan":uninstall["plan"],"allow-data-loss":true,"target":"local-test"})).await.unwrap();
    assert_eq!(
        run("db-status", json!({"history":history,"database":database}))
            .await
            .unwrap()["applied"]["state"],
        "uninstalled"
    );
    assert!(
        run(
            "db-install",
            json!({"history":history,"database":database,"plan":hash,"force":true})
        )
        .await
        .is_err()
    );
    assert!(
        run(
            "revision",
            json!({"history":history,"input":input,"name":"v3","unknown":true})
        )
        .await
        .is_err()
    );
    assert!(
        run(
            "migration-plan",
            json!({"history":history,"to":"v1","dialect":"unknown"})
        )
        .await
        .is_err()
    );
    assert!(
        run(
            "db-install",
            json!({"history":history,"database":database,"plan":seed["plan"]})
        )
        .await
        .is_err()
    );
    // Actual CLI execution uses the same named event and current emit grammar.
    cli(&history, false);
    cli(&history, true);
    // A shape match is reported without creating a plan or accepting the rename.
    std::fs::write(
        &input,
        source.replace("name String", "heading String summary String?"),
    )
    .unwrap();
    run(
        "revision",
        json!({"history":history,"input":input,"name":"v3"}),
    )
    .await
    .unwrap();
    let suggestion = run("migration-plan", json!({"history":history,"to":"v3"}))
        .await
        .unwrap();
    assert_eq!(suggestion["outcome"], "NeedsExplicitMapping");
    let mapped = run(
        "migration-plan",
        json!({"history":history,"to":"v3","rename":"profile.name=heading"}),
    )
    .await
    .unwrap();
    assert!(mapped["plan"].is_string());
    for dialect in ["postgres", "mysql"] {
        let network = run(
            "migration-plan",
            json!({"history":history,"to":"v3","rename":"profile.name=heading","dialect":dialect}),
        )
        .await
        .unwrap();
        let error=run("db-upgrade",json!({"history":history,"database":root.path().join("must-not-open.sqlite"),"plan":network["plan"]})).await.unwrap_err();
        let cause = std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<stackpress_framework_sql::DataError>()
            .unwrap();
        assert_eq!(
            cause.kind(),
            stackpress_framework_sql::DataErrorKind::Unsupported
        );
        assert!(!root.path().join("must-not-open.sqlite").exists());
    }
}
/// Spawn a real CLI child with an independent finite kill/reap guard and bounded output.
fn cli(history: &std::path::Path, invalid: bool) {
    use std::process::Command;
    let mut command = Command::new(env!("CARGO_BIN_EXE_generated-workspace-tool"));
    command
        .args(["emit", "history-check", "--history"])
        .arg(history);
    if invalid {
        command.args(["--unknown", "true"]);
    }
    let output = cli_command(command, !invalid);
    if !invalid {
        let value: Value = serde_json::from_str(&output).unwrap();
        assert!(value["results"]["revisions"].is_array());
    }
}

/// Independently supervise each real CLI child; generic force remains ordinary host context.
fn cli_command(mut command: std::process::Command, expected: bool) -> String {
    use std::process::{Child, Stdio};
    use std::time::Instant;
    struct Guard(Child);
    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let log = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    let file = std::fs::File::create(log.path()).unwrap();
    let mut child = Guard(
        command
            .stdout(Stdio::from(file.try_clone().unwrap()))
            .stderr(Stdio::from(file))
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    let status = loop {
        assert!(start.elapsed() < Duration::from_secs(5));
        assert!(log.as_file().metadata().unwrap().len() < 128 * 1024);
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let output = std::fs::read_to_string(log.path()).unwrap();
    assert_eq!(status.success(), expected, "{output}");
    output
}
