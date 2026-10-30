//! One real buffered HTTP request per process; every database and cookie belongs to the caller.
use serde::{Deserialize, Serialize};
use serde_json::json;
use stackpress_lib::{Error, Result};
use stackpress_sql::Engine;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

/// Trusted no-listener harness input; none of these fixture controls are browser routes.
#[derive(Deserialize)]
pub struct Input {
    /// Caller-owned database file.
    pub database: PathBuf,
    /// Install and populate once in an absent file.
    #[serde(default)]
    pub initialize: bool,
    /// Original native HTTP method.
    #[serde(default = "get")]
    pub method: String,
    /// Original path and query.
    #[serde(default = "path")]
    pub path: String,
    /// Incoming cookies/headers, parsed by the actual HTTP adapter.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    /// Optional exact URL-encoded or JSON body bytes.
    #[serde(default)]
    pub body: String,
    /// Optional trusted JWT creation command, never a fabricated request-local caller.
    #[serde(default)]
    pub jwt: Option<String>,
}
/// Default native method for the finite harness.
fn get() -> String {
    "GET".into()
}
/// Absent path retains the source root URL.
fn path() -> String {
    "/".into()
}
/// Native HTTP result; no adapter internals or database path enters the document.
#[derive(Serialize)]
pub struct Output {
    /// Actual response code.
    pub status: u16,
    /// Valid native response headers.
    pub headers: BTreeMap<String, String>,
    /// Owned UTF8 bytes, including source HTML or token JSON.
    pub body: String,
}
/// Execute, close the Engine and return. No socket is ever created.
/// # Errors
/// Propagates startup/input/SQL/HTTP/cleanup errors to the trusted harness.
pub async fn run(input: Input) -> Result<Output> {
    if input.initialize && input.database.exists() {
        return Err(Error::invalid("fixture already exists"));
    }
    let engine = Arc::new(Engine::new(
        stackpress_sql_sqlite::SqliteDatabase::open(
            stackpress_sql_sqlite::SqliteOptions::file(&input.database)
                .create_if_missing(input.initialize),
        )
        .await
        .map_err(Error::from_source)?,
    ));
    let result = execute(input, engine.clone()).await;
    let closed = engine.close().await.map_err(Error::from_source);
    match (result, closed) {
        (Ok(output), Ok(())) => Ok(output),
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
}
/// Finite selected composition through the same public registrars as an application.
async fn execute(input: Input, engine: Arc<Engine>) -> Result<Output> {
    let mut layers = stackpress_server::ConfigLayers::new();
    layers.value("finite app",json!({"session":{"key":"identity","seed":"source-seed"},"api":{"scopes":{"read":{"icon":"globe","name":"Read profile information and account preferences shared with this application across coordinated workflows","description":"Read the selected user's profile."},"write":{"icon":"key","name":"Write account information","description":"Allow this application to update information."}},"endpoints":[{"method":"GET","route":"/api","type":"public","event":"fixture"}]}}))?;
    let app = crate::build(
        layers,
        engine.clone(),
        stackpress_framework_api::pages::Environment::default(),
    )
    .await?;
    if input.initialize {
        populate(engine).await?;
        return Ok(Output {
            status: 200,
            headers: Default::default(),
            body: "initialized".into(),
        });
    }
    if let Some(profile) = input.jwt {
        let token = app
            .host()
            .service::<stackpress_framework_session::Service>()?
            .create(
                json!({"id":profile,"roles":["USER"]})
                    .as_object()
                    .expect("object")
                    .clone(),
            )
            .await
            .map_err(Error::from_source)?;
        return Ok(Output {
            status: 200,
            headers: Default::default(),
            body: token,
        });
    }
    let mut request = http::Request::builder()
        .method(input.method.as_str())
        .uri(input.path);
    for (name, value) in input.headers {
        request = request.header(name, value);
    }
    let reply = stackpress_server::http::HttpAdapter::new(stackpress_server::http::HttpOptions {
        base_url: "https://api.invalid/".into(),
        ..Default::default()
    })?
    .handle(
        &app,
        request
            .body(input.body.into_bytes())
            .map_err(Error::from_source)?,
    )
    .await?;
    let stackpress_server::http::HttpOutput::Response(response) = reply.output else {
        return Err(Error::invalid("buffered response required"));
    };
    Ok(Output {
        status: response.status().as_u16(),
        headers: response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| (name.to_string(), value.to_owned()))
            })
            .collect(),
        body: String::from_utf8(response.into_body()).map_err(Error::from_source)?,
    })
}
/// Real owner Profile and API actions/default providers populate related management data.
async fn populate(engine: Arc<Engine>) -> Result<()> {
    let profile = stackpress_framework_session::storage::ProfileActions::new(engine.clone(), "");
    let apps = stackpress_framework_api::models::ApplicationActions::new(engine.clone(), "");
    let sessions = stackpress_framework_api::models::SessionActions::new(engine, "");
    profile.install().await.map_err(Error::from_source)?;
    apps.install().await.map_err(Error::from_source)?;
    sessions.install().await.map_err(Error::from_source)?;
    for (id, name) in [("p", "Ada Lovelace"), ("q", "Grace Hopper")] {
        profile
            .create(&serde_json::from_value(
                json!({"id":id,"name":name,"roles":["USER"]}),
            )?)
            .await
            .map_err(Error::from_source)?;
    }
    let mut app: stackpress_framework_api::models::Record = serde_json::from_value(
        json!({"id":"a","profileId":"p","name":"Example application","logo":"https://api.invalid/fixtures/application-logo.svg","website":"https://example.test","secret":"app-plaintext","scopes":["read","write"]}),
    )?;
    app.insert(
        "expires".into(),
        stackpress_framework_api::models::date(4102444800000),
    );
    apps.create(&app).await.map_err(Error::from_source)?;
    let mut session: stackpress_framework_api::models::Record = serde_json::from_value(
        json!({"id":"s","applicationId":"a","profileId":"p","secret":"session-plaintext","scopes":["read"]}),
    )?;
    session.insert(
        "expires".into(),
        stackpress_framework_api::models::date(4102444800000),
    );
    sessions
        .create(&session)
        .await
        .map_err(Error::from_source)?;
    Ok(())
}
