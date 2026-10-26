//! Opt-in renderer checks. Unset backend variables are reported as ignored, never passed.
//! Only generated throwaway databases are created/dropped; no other database is inspected.
#[path = "network/cases.rs"]
mod cases;
use sqlx::mysql::{MySqlConnectOptions, MySqlConnection};
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{Connection, Executor, Row, SqlSafeStr};
use stackpress_sql::{DialectId, Query, QueryPlan, SqlValue};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
/// Errors in setup, execution or cleanup all fail the test process.
type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
/// Cleanup during unwind cannot silently report success.
static CLEANUP_FAILED: AtomicBool = AtomicBool::new(false);
/// Each configured server runs all cases in one uniquely owned database.
const CASES: [&str; 8] = [
    "create_types_defaults",
    "crud_returning",
    "joins",
    "expressions_pagination",
    "json_paths_membership",
    "alter_fields_constraints",
    "schema_diff_rename",
    "table_commands",
];
/// Test-only actual connections; no production adapter is introduced.
enum Native {
    Pg(PgConnection),
    Mysql(MySqlConnection),
}
impl Native {
    /// Connect within a deadline; change only the selected database in client options.
    async fn connect(dialect: DialectId, url: &str, database: Option<&str>) -> TestResult<Self> {
        tokio::time::timeout(Duration::from_secs(5), async {
            match dialect {
                DialectId::Postgres => {
                    let mut options = PgConnectOptions::from_str(url)?;
                    if let Some(database) = database {
                        options = options.database(database);
                    }
                    Ok(Self::Pg(PgConnection::connect_with(&options).await?))
                }
                DialectId::Mysql => {
                    let mut options = MySqlConnectOptions::from_str(url)?;
                    if let Some(database) = database {
                        options = options.database(database);
                    }
                    Ok(Self::Mysql(MySqlConnection::connect_with(&options).await?))
                }
                DialectId::Sqlite => unreachable!("network harness"),
            }
        })
        .await?
    }
    /// Execute only fixture administration built from a generated, checked database name.
    async fn administration(&mut self, sql: &str) -> TestResult<()> {
        let text = sqlx::AssertSqlSafe(sql).into_sql_str();
        match self {
            Self::Pg(connection) => {
                connection.execute(text).await?;
            }
            Self::Mysql(connection) => {
                connection.execute(text).await?;
            }
        }
        Ok(())
    }
    /// Read the server version or the identity of this owned database, without catalog scans.
    async fn text(&mut self, sql: &'static str) -> TestResult<String> {
        Ok(match self {
            Self::Pg(connection) => sqlx::query_scalar(sql).fetch_one(connection).await?,
            Self::Mysql(connection) => sqlx::query_scalar(sql).fetch_one(connection).await?,
        })
    }
    /// Run exactly rendered SQL/binds; only the first fixture BIGINT column is decoded.
    async fn run(&mut self, query: &Query) -> TestResult<Vec<i64>> {
        let text = sqlx::AssertSqlSafe(query.sql()).into_sql_str();
        match self {
            Self::Pg(connection) => {
                let mut native = sqlx::query(text);
                for value in query.binds() {
                    native = match value {
                        SqlValue::Null => native.bind(Option::<String>::None),
                        SqlValue::Bool(value) => native.bind(*value),
                        SqlValue::Integer(value) => native.bind(*value),
                        SqlValue::Real(value) => native.bind(*value),
                        SqlValue::Text(value) => native.bind(value.clone()),
                        SqlValue::Blob(value) => native.bind(value.clone()),
                    };
                }
                Ok(native
                    .fetch_all(connection)
                    .await?
                    .iter()
                    .map(|row| row.try_get(0))
                    .collect::<Result<Vec<i64>, _>>()?)
            }
            Self::Mysql(connection) => {
                let mut native = sqlx::query(text);
                for value in query.binds() {
                    native = match value {
                        SqlValue::Null => native.bind(Option::<String>::None),
                        SqlValue::Bool(value) => native.bind(*value),
                        SqlValue::Integer(value) => native.bind(*value),
                        SqlValue::Real(value) => native.bind(*value),
                        SqlValue::Text(value) => native.bind(value.clone()),
                        SqlValue::Blob(value) => native.bind(value.clone()),
                    };
                }
                Ok(native
                    .fetch_all(connection)
                    .await?
                    .iter()
                    .map(|row| row.try_get(0))
                    .collect::<Result<Vec<i64>, _>>()?)
            }
        }
    }
    /// Run a wholly rendered plan as renderer validation, without a MySQL atomic-DDL claim.
    async fn plan(&mut self, plan: &QueryPlan) -> TestResult<()> {
        for query in plan.queries() {
            self.run(query).await?;
        }
        Ok(())
    }
    /// Close fixture connections before removing the owned database.
    async fn close(self) -> TestResult<()> {
        match self {
            Self::Pg(connection) => connection.close().await?,
            Self::Mysql(connection) => connection.close().await?,
        }
        Ok(())
    }
}
/// Only this private constructor can create an administrative DROP target.
struct DatabaseName(String);
impl DatabaseName {
    /// Time/process entropy gives a unique bounded ASCII name; CREATE never adopts an existing DB.
    fn unique() -> Self {
        Self(format!(
            "stackpress_test_{:x}_{:x}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    /// Quote only generated names; environment values can never become DROP identifiers.
    fn quoted(&self, dialect: DialectId) -> String {
        assert!(
            self.0.starts_with("stackpress_test_")
                && self.0.len() < 63
                && self
                    .0
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        );
        let quote = if dialect == DialectId::Postgres {
            '"'
        } else {
            '`'
        };
        format!("{quote}{}{quote}", self.0)
    }
}
/// Cleanup is armed only after CREATE succeeds and remains active during errors/unwind.
struct DropDatabase {
    name: DatabaseName,
    url: String,
    dialect: DialectId,
    armed: bool,
}
impl DropDatabase {
    /// Create exactly one owned database, without IF NOT EXISTS or catalog access.
    async fn create(url: &str, dialect: DialectId, admin: &mut Native) -> TestResult<Self> {
        let name = DatabaseName::unique();
        admin
            .administration(&format!("CREATE DATABASE {}", name.quoted(dialect)))
            .await?;
        println!("created owned database {}", name.0);
        Ok(Self {
            name,
            url: url.into(),
            dialect,
            armed: true,
        })
    }
    /// Remove only the owned database on a separate bounded runtime, including on unwind.
    fn cleanup(&mut self) -> TestResult<()> {
        if !self.armed {
            return Ok(());
        }
        let url = self.url.clone();
        let dialect = self.dialect;
        let sql = format!("DROP DATABASE IF EXISTS {}", self.name.quoted(dialect));
        std::thread::spawn(move || -> TestResult<()> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(10), async {
                    let mut admin = Native::connect(dialect, &url, None).await?;
                    admin.administration(&sql).await?;
                    admin.close().await
                })
                .await?
            })
        })
        .join()
        .map_err(|_| "cleanup thread panicked")??;
        self.armed = false;
        println!("cleanup: dropped owned database {}", self.name.0);
        Ok(())
    }
}
impl Drop for DropDatabase {
    /// A cleanup failure also sets a process-wide failing status.
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            CLEANUP_FAILED.store(true, Ordering::SeqCst);
            eprintln!("owned database cleanup failed: {error}");
        }
    }
}
/// Record each case separately, even when a previous case failed; always attempt cleanup.
async fn backend(url: &str, dialect: DialectId, label: &str) -> TestResult<(usize, usize)> {
    let mut admin = Native::connect(dialect, url, None).await?;
    println!(
        "{label} server version: {}",
        admin.text("SELECT version()").await?
    );
    let mut guard = DropDatabase::create(url, dialect, &mut admin).await?;
    admin.close().await?;
    let mut connection = Native::connect(dialect, url, Some(&guard.name.0)).await?;
    assert_eq!(
        connection
            .text(if dialect == DialectId::Postgres {
                "SELECT current_database()"
            } else {
                "SELECT DATABASE()"
            })
            .await?,
        guard.name.0
    );
    let (mut passed, mut failed) = (0, 0);
    for (index, name) in CASES.iter().enumerate() {
        match tokio::time::timeout(
            Duration::from_secs(15),
            cases::run(index, &mut connection, dialect),
        )
        .await
        {
            Ok(Ok(())) => {
                passed += 1;
                println!("test network::{label}::{name} ... ok");
            }
            other => {
                failed += 1;
                eprintln!("test network::{label}::{name} ... FAILED: {other:?}");
            }
        }
    }
    connection.close().await?;
    guard.cleanup()?;
    Ok((passed, failed))
}
/// This harness reports runtime-gated skips as ignored instead of passing early-return tests.
fn main() {
    let (mut passed, mut failed, mut skipped) = (0, 0, 0);
    for (dialect, label, variable) in [
        (
            DialectId::Postgres,
            "postgres",
            "STACKPRESS_TEST_POSTGRES_URL",
        ),
        (DialectId::Mysql, "mysql", "STACKPRESS_TEST_MYSQL_URL"),
    ] {
        let Ok(url) = std::env::var(variable) else {
            for name in CASES {
                println!("test network::{label}::{name} ... ignored ({variable} unset)");
                skipped += 1;
            }
            continue;
        };
        let result = std::panic::catch_unwind(|| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(backend(&url, dialect, label))
        });
        match result {
            Ok(Ok((ok, errors))) => {
                passed += ok;
                failed += errors;
            }
            other => {
                eprintln!("{label} setup/teardown failed: {other:?}");
                failed += CASES.len();
            }
        }
    }
    if CLEANUP_FAILED.load(Ordering::SeqCst) {
        failed += 1;
    }
    println!(
        "test result: {}. {passed} passed; {failed} failed; {skipped} ignored; 0 measured; 0 filtered out",
        if failed == 0 { "ok" } else { "FAILED" }
    );
    if failed != 0 {
        std::process::exit(1);
    }
}
