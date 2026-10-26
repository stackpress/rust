//! Disposable native service proof. Run with `cargo run -p stackpress-sql-sqlite --example m0-host`.
//! The application explicitly installs/seeds its own tempfile database before startup.
//! No listener, generic SQL capability, generated store or framework-SQL crate is involved.

use std::sync::Arc;

use stackpress_lib::{Error, Flow, Invocation, Nest, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::{DialectId, Engine, Select, Sql};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

/// Open the app-owned typed configuration and register exactly one shared Engine.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    let options = start.config::<SqliteOptions>()?.clone();
    let native = SqliteDatabase::open(options)
        .await
        .map_err(Error::from_source)?;
    start.register(Arc::new(Engine::new(native)))?;
    Ok(Flow::Continue)
}

/// Expose a fixed business query, keeping raw SQL and database paths out of request data.
async fn profile_name(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let query = Select::new(["name"]).from("profile").where_eq("id", 1_i64);
    let rows = inv
        .service::<Engine>()?
        .fetch(&query)
        .await
        .map_err(Error::from_source)?;
    let name = rows[0]
        .try_get::<String>("name")
        .map_err(Error::from_source)?;
    res.results(&name)?;
    Ok(Flow::Continue)
}

/// Prove setup, consuming commit, startup registration, named Send dispatch and explicit close.
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::TempDir::new()?;
    let options = SqliteOptions::file(root.path().join("proof.sqlite")).create_if_missing(true);
    let setup = Engine::new(SqliteDatabase::open(options.clone()).await?);
    setup
        .execute(&Sql::literal(
            DialectId::Sqlite,
            "CREATE TABLE profile(id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
        ))
        .await?;
    let mut tx = setup.begin().await?;
    tx.execute(
        &Sql::literal(DialectId::Sqlite, "INSERT INTO profile VALUES (?, ?)")
            .bind(1_i64)
            .bind("Ada"),
    )
    .await?;
    tx.commit().await?;
    setup.close().await?;

    // Construction installs no data and exposes only the explicit profile-name operation.
    let mut app = ServerBuilder::new();
    app.config_section("sqlite", options, |options| {
        options.validate().map_err(Error::from_source)
    })?;
    app.on_config(configure)?;
    app.on("profile-name", profile_name)?;
    let server = app.build().await?;
    let reply = server.call("profile-name", Nest::new()).await?;
    assert!(reply.report.failures.is_empty());
    assert_eq!(
        reply.response.result().and_then(|value| value.as_str()),
        Some("Ada")
    );
    println!("profile-name: Ada");

    // Await resource shutdown while the runtime and the disposable directory still exist.
    server.host().shared_service::<Engine>()?.close().await?;
    drop(server);
    drop(setup);
    drop(root);
    Ok(())
}
