//! Test-only owned network databases, shared by the session and API search executables.
//!
//! Adapter M2 fixtures are private; reuse their public adapters and reachable process/discovery
//! helpers. This owner follows their CREATE/armed/exact-DROP contract without a library API.

#[path = "../../../../sql/test-support/harness.rs"]
mod harness;
#[path = "../../../../sql/sql-postgres/src/tests/support/process.rs"]
mod process;

use stackpress_sql::{DialectId, Engine, Render, Sql, SqlError, TrustedSql};
use stackpress_sql_mysql::{MysqlDatabase, MysqlOptions, MysqlTls};
use stackpress_sql_postgres::{PostgresDatabase, PostgresOptions, PostgresTls};
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Native causes are retained for the caller but never formatted by the harness.
pub(crate) type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
static CLEANUP_FAILED: AtomicBool = AtomicBool::new(false);

/// Fixed test backends; a URL can never select another dialect or administrative identity.
#[derive(Clone, Copy)]
pub(crate) enum Backend {
    Postgres,
    Mysql,
}
impl Backend {
    fn name(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
        }
    }
    fn gate(self) -> &'static str {
        match self {
            Self::Postgres => "STACKPRESS_TEST_POSTGRES_URL",
            Self::Mysql => "STACKPRESS_TEST_MYSQL_URL",
        }
    }
    fn dialect(self) -> DialectId {
        match self {
            Self::Postgres => DialectId::Postgres,
            Self::Mysql => DialectId::Mysql,
        }
    }

    /// Test URLs are limited to the documented passwordless local administration endpoints.
    /// Exact matching avoids introducing a second URL parser or accepting ambient connection data.
    fn host(self) -> TestResult<&'static str> {
        let value = std::env::var(self.gate())?;
        let endpoints = match self {
            Self::Postgres => [
                "postgres://postgres@localhost:5432/postgres",
                "postgres://postgres@127.0.0.1:5432/postgres",
            ],
            Self::Mysql => [
                "mysql://root@localhost:3306/mysql",
                "mysql://root@127.0.0.1:3306/mysql",
            ],
        };
        match endpoints.iter().position(|endpoint| *endpoint == value) {
            Some(0) => Ok("localhost"),
            Some(1) => Ok("127.0.0.1"),
            _ => Err("fixture URL must select a documented local host, port and identity".into()),
        }
    }

    /// Every connection uses the same finite pool/statement budgets as the adapter fixtures.
    pub(crate) async fn open(self, name: &str) -> TestResult<Engine> {
        let host = self.host()?;
        match self {
            Self::Postgres => {
                let mut options = PostgresOptions::new(host, name, "postgres", "");
                options.tls = PostgresTls::Plaintext;
                options.max_connections = 2;
                options.max_in_flight = 4;
                options.acquire_timeout_ms = 1000;
                options.statement_timeout_ms = 2000;
                options.cleanup_timeout_ms = 1000;
                Ok(Engine::new(PostgresDatabase::open(options).await?))
            }
            Self::Mysql => {
                let mut options = MysqlOptions::new(host, name, "root", "");
                options.tls = MysqlTls::Plaintext;
                options.max_connections = 2;
                options.max_in_flight = 4;
                options.acquire_timeout_ms = 1000;
                options.statement_timeout_ms = 2000;
                options.cleanup_timeout_ms = 1000;
                Ok(Engine::new(MysqlDatabase::open(options).await?))
            }
        }
    }
}

/// Only a successful CREATE acknowledgement grants authority to drop this generated name.
struct Fixture {
    backend: Backend,
    name: String,
    /// The administrative worker publishes CREATE/DROP acknowledgement before socket close.
    armed: Arc<AtomicBool>,
}
impl Fixture {
    fn create(backend: Backend) -> TestResult<Self> {
        backend.host()?;
        let name = format!(
            "stackpress_g46_{}_{:x}_{:x}",
            backend.name(),
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        );
        validate_name(&name)?;
        let fixture = Self {
            backend,
            name,
            armed: Arc::new(AtomicBool::new(false)),
        };
        // Keep the owner alive while CREATE runs, including an error after acknowledgement.
        admin(
            backend,
            "CREATE",
            fixture.name.clone(),
            fixture.armed.clone(),
        )?;
        Ok(fixture)
    }
    fn cleanup(&mut self) -> TestResult<()> {
        if !self.armed.load(Ordering::SeqCst) {
            return Ok(());
        }
        admin(self.backend, "DROP", self.name.clone(), self.armed.clone())?;
        Ok(())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.armed.load(Ordering::SeqCst) && self.cleanup().is_err() {
            eprintln!("cleanup FAILED for exact owned database {}", self.name);
            CLEANUP_FAILED.store(true, Ordering::SeqCst);
        }
    }
}

/// Never accept URL database names, arbitrary SQL identifiers or prefix-based cleanup.
fn validate_name(name: &str) -> TestResult<()> {
    if !(20..=63).contains(&name.len())
        || !name.starts_with("stackpress_g46_")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err("invalid owned database name".into());
    }
    Ok(())
}

/// Independent administrative runtime and wall-clock deadline, including cleanup on unwind.
fn admin(
    backend: Backend,
    verb: &'static str,
    name: String,
    armed: Arc<AtomicBool>,
) -> TestResult<()> {
    validate_name(&name)?;
    assert!(matches!(verb, "CREATE" | "DROP"));
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let result = (|| -> TestResult<()> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(9), async {
                    let engine = backend
                        .open(match backend {
                            Backend::Postgres => "postgres",
                            Backend::Mysql => "mysql",
                        })
                        .await?;
                    let quoted = match backend {
                        Backend::Postgres => format!("\"{name}\""),
                        Backend::Mysql => format!("`{name}`"),
                    };
                    // Dynamic SQL contains only the validated, harness-generated exact identifier.
                    let query = Sql::trusted(
                        backend.dialect(),
                        TrustedSql::audited(format!("{verb} DATABASE {quoted}")),
                    )
                    .render(backend.dialect())?;
                    let result = engine.execute(&query).await;
                    // Creation grants exact-name cleanup authority immediately. Successful DROP
                    // revokes it immediately, so a later close error cannot cause a second DROP.
                    if result.is_ok() {
                        armed.store(verb == "CREATE", Ordering::SeqCst);
                        if verb == "CREATE" {
                            println!("created owned database {name}");
                        } else {
                            println!("drop confirmed: {name}");
                        }
                    }
                    let close = engine.close().await;
                    result?;
                    close?;
                    Ok(())
                })
                .await?
            })
        })();
        let _ = sender.send(result);
    });
    let result = receiver
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "administrative watchdog expired")?;
    worker
        .join()
        .map_err(|_| "administrative worker panicked")?;
    result
}

/// Report safe first-party context only; raw native diagnostics and URLs are never printed.
fn report(error: &(dyn std::error::Error + 'static)) {
    let mut current = Some(error);
    for _ in 0..8 {
        let Some(cause) = current else {
            break;
        };
        if let Some(sql) = cause.downcast_ref::<SqlError>() {
            eprintln!("SQL failure {:?}: {sql}", sql.kind());
        }
        current = cause.source();
    }
}

/// Dispatch private children first, then expose dynamically ignored libtest/nextest cases.
pub(crate) fn main<F, Fut>(scenario: F)
where
    F: Fn(Arc<Engine>) -> Fut,
    Fut: std::future::Future<Output = TestResult<()>>,
{
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("--g46-stall") {
        loop {
            std::thread::park();
        }
    }
    if arguments.first().map(String::as_str) == Some("--g46-child") {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> TestResult<()> {
                let backend = match arguments.get(1).map(String::as_str) {
                    Some("postgres") => Backend::Postgres,
                    Some("mysql") => Backend::Mysql,
                    _ => return Err("unknown backend".into()),
                };
                let name = arguments.get(2).ok_or("missing exact database name")?;
                validate_name(name)?;
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                runtime.block_on(async {
                    let engine = Arc::new(backend.open(name).await?);
                    let result = scenario(engine.clone()).await;
                    let close = engine.close().await;
                    result?;
                    close?;
                    Ok(())
                })
            }));
        if let Ok(Err(error)) = &result {
            report(error.as_ref());
        }
        if !matches!(result, Ok(Ok(()))) {
            eprintln!("live search child failed (native detail redacted)");
            std::process::exit(1);
        }
        return;
    }
    let harness = harness::Harness::from_env();
    let cases = [
        ("g46_live::postgres_ascii_search", Some(Backend::Postgres)),
        ("g46_live::mysql_ascii_search", Some(Backend::Mysql)),
        ("g46_live::watchdog_forced_kill", None),
    ];
    if harness.discover(cases.iter().map(|(name, backend)| {
        (
            *name,
            backend.is_some_and(|backend| std::env::var_os(backend.gate()).is_none()),
        )
    })) {
        return;
    }
    let mut passed = 0;
    let mut skipped = 0;
    let mut failed = 0;
    for (name, backend) in cases {
        let ignored = backend.is_some_and(|backend| std::env::var_os(backend.gate()).is_none());
        if !harness.selects(name, ignored) {
            continue;
        }
        if ignored {
            println!("test {name} ... ignored (opt-in URL unset)");
            skipped += 1;
            continue;
        }
        let result = std::panic::catch_unwind(|| -> TestResult<()> {
            if let Some(backend) = backend {
                let mut fixture = Fixture::create(backend)?;
                let status = process::watchdog(
                    Command::new(std::env::current_exe()?)
                        .args(["--g46-child", backend.name(), &fixture.name])
                        .spawn()?,
                    Duration::from_secs(30),
                );
                let cleanup = fixture.cleanup();
                cleanup?;
                if !status?.success() {
                    return Err("live scenario failed".into());
                }
            } else {
                let status = process::watchdog(
                    Command::new(std::env::current_exe()?)
                        .arg("--g46-stall")
                        .spawn()?,
                    Duration::from_millis(150),
                );
                let error = status.expect_err("watchdog negative control must time out");
                assert_eq!(
                    error.to_string(),
                    "native scenario process watchdog timed out; child killed and reaped"
                );
            }
            Ok(())
        });
        if matches!(result, Ok(Ok(()))) {
            println!("test {name} ... ok");
            passed += 1;
        } else {
            if let Ok(Err(error)) = &result {
                report(error.as_ref());
            }
            println!("test {name} ... FAILED");
            failed += 1;
        }
    }
    println!(
        "test result: {}. {passed} passed; {failed} failed; {skipped} ignored",
        if failed == 0 { "ok" } else { "FAILED" }
    );
    if failed != 0 || CLEANUP_FAILED.load(Ordering::SeqCst) {
        std::process::exit(1);
    }
}
