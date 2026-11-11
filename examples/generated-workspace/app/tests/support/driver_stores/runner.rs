//! Parent-owned fixtures and killable children for the relocated store sequences.
use std::process::Command;
use std::time::Duration;

#[path = "../../../../../../crates/sql/test-support/harness.rs"]
mod harness;
#[path = "../../../../../../crates/sql/sql-postgres/src/tests/support/process.rs"]
mod process;
use process::watchdog;

mod diagnostics;
mod mysql;
mod postgres;
mod suite;

/// Preserve typed native causes without disclosing connection data in diagnostics.
type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Run SQLite in the same independently bounded child protocol as the live adapters.
fn sqlite() -> TestResult<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("generated-stores.sqlite");
    // Reserve the exact file in a newly created directory before the child opens it.
    drop(
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?,
    );
    let status = watchdog(
        Command::new(std::env::current_exe()?)
            .arg("--scenario")
            .arg("sqlite")
            .arg(&path)
            .spawn()?,
        Duration::from_secs(45),
    )?;
    if !status.success() {
        return Err("generated SQLite store scenario failed".into());
    }
    directory.close()?;
    Ok(())
}

/// A child cannot create or drop a server database; those rights stay in its parent.
fn child(backend: &str, name: &str) -> TestResult<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    runtime.block_on(async {
        match backend {
            "postgres" => postgres::child(name).await,
            "mysql" => mysql::child(name).await,
            "sqlite" => {
                let mut options = stackpress_sql_sqlite::SqliteOptions::file(name);
                options.max_connections = 1;
                options.max_in_flight = 8;
                let database = stackpress_sql_sqlite::SqliteDatabase::open(options).await?;
                suite::check(std::sync::Arc::new(database)).await
            }
            _ => Err("unknown generated-store backend".into()),
        }
    })
}

/// Support nextest listing/exact selection and report missing live URLs as ignored.
pub(super) fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--fixture-inventory") {
        // Inspect both servers even if one listing fails; this mode never creates or drops.
        let postgres = postgres::inventory();
        let mysql = mysql::inventory();
        if postgres.is_err() || mysql.is_err() {
            eprintln!("final fixture inventory failed; native diagnostics redacted");
            std::process::exit(1);
        }
        return;
    }
    if args.get(1).map(String::as_str) == Some("--scenario") {
        let outcome = std::panic::catch_unwind(|| {
            child(
                args.get(2).ok_or("missing backend")?,
                args.get(3).ok_or("missing owned database")?,
            )
        });
        if let Ok(Err(error)) = &outcome {
            diagnostics::report_error(error.as_ref());
        }
        if !matches!(outcome, Ok(Ok(()))) {
            eprintln!("generated-store child failed; native diagnostics redacted");
            std::process::exit(1);
        }
        return;
    }
    let cases = [
        ("generated_stores::sqlite", false),
        (
            "generated_stores::postgres",
            std::env::var_os("STACKPRESS_TEST_POSTGRES_URL").is_none(),
        ),
        (
            "generated_stores::mysql",
            std::env::var_os("STACKPRESS_TEST_MYSQL_URL").is_none(),
        ),
    ];
    let harness = harness::Harness::from_env();
    if harness.discover(cases) {
        return;
    }
    for (name, ignored) in cases {
        if !harness.selects(name, ignored) {
            continue;
        }
        if ignored {
            println!("test {name} ... ignored (live URL unset)");
            continue;
        }
        let result = std::panic::catch_unwind(|| match name {
            "generated_stores::sqlite" => sqlite(),
            "generated_stores::postgres" => postgres::parent(),
            "generated_stores::mysql" => mysql::parent(),
            _ => unreachable!("fixed harness inventory"),
        });
        if !matches!(result, Ok(Ok(()))) {
            eprintln!("test {name} ... FAILED (scenario or cleanup)");
            std::process::exit(1);
        }
        println!("test {name} ... ok");
    }
}
