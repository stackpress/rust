//! One buffered request per process. The caller owns the SQLite file and every input byte.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_lib::{Error, Result};
use stackpress_server::http::{HttpAdapter, HttpOptions, HttpOutput};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

/// Trusted harness process input; browser JSON cannot choose a database or fault mode.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Exact proof-owned SQLite file.
    pub database: PathBuf,
    /// Create the generated tables in a previously absent file, then exit.
    #[serde(default)]
    pub initialize: bool,
    /// One explicit mount for this host.
    pub mount: String,
    /// Buffered HTTP method.
    #[serde(default = "method")]
    pub method: String,
    /// Buffered HTTP path/query, independent of authority headers.
    #[serde(default = "path")]
    pub path: String,
    /// Explicit finite fixture principal: editor/reader/unknown.
    pub actor: String,
    /// Native incoming headers, preserving the real CSRF check.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    /// One public JSON operation envelope.
    #[serde(default)]
    pub body: Option<Value>,
    /// Explicit CSR document mode.
    #[serde(default)]
    pub csr: bool,
    /// Trusted proof-only lost acknowledgement after an actual committed SQLite write.
    #[serde(default)]
    pub fault: bool,
}
/// Default finite request method.
fn method() -> String {
    "GET".into()
}
/// Default finite path is not a generated action and therefore receives 404.
fn path() -> String {
    "/".into()
}

/// Native buffered output only; host diagnostics and database paths never enter this DTO.
#[derive(Serialize)]
pub struct Output {
    /// Actual native HTTP status.
    pub status: u16,
    /// Public native response headers.
    pub headers: BTreeMap<String, String>,
    /// UTF-8 HTML or public JSON envelope.
    pub body: String,
}

/// Execute one real buffered HTTP request and close all owned resources before returning.
/// # Errors
/// Keeps native configuration/adapter/cleanup causes private; callers print only fixed failure text.
pub async fn run(input: Input) -> Result<Output> {
    if input.initialize && input.database.exists() {
        return Err(Error::invalid("fixture database already exists"));
    }
    let engine = crate::database::open(&input.database, input.fault)
        .await
        .map_err(native)?;
    let outcome = async {
        if input.initialize {
            crate::database::initialize(&engine).await?;
            return Ok(Output {
                status: 200,
                headers: BTreeMap::new(),
                body: "initialized".into(),
            });
        }
        let app = crate::host::build(
            engine.clone(),
            crate::host::Config {
                mount: input.mount,
                csrf: "finite-session-csrf".into(),
                csr: input.csr,
            },
            Arc::new(crate::host::FixturePolicy),
        )
        .await?;
        let mut request = http::Request::builder()
            .method(input.method.as_str())
            .uri(&input.path)
            .header("x-admin-actor", input.actor);
        for (name, value) in input.headers {
            // This reserved fixture principal belongs to trusted harness configuration only.
            if !name.eq_ignore_ascii_case("x-admin-actor") {
                request = request.header(name, value);
            }
        }
        if input.body.is_some() {
            request = request.header("content-type", "application/json");
        }
        let request = request
            .body(
                input
                    .body
                    .map(|value| value.to_string().into_bytes())
                    .unwrap_or_default(),
            )
            .map_err(native)?;
        let options = HttpOptions {
            base_url: "https://admin.invalid/".into(),
            ..Default::default()
        };
        let reply = HttpAdapter::new(options)?.handle(&app, request).await?;
        let HttpOutput::Response(response) = reply.output else {
            return Err(Error::invalid("buffered admin output unavailable"));
        };
        let output = Output {
            status: response.status().as_u16(),
            headers: response
                .headers()
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|value| (name.to_string(), value.into()))
                })
                .collect(),
            body: String::from_utf8(response.into_body()).map_err(native)?,
        };
        // This transport-neutral host owns no listener or background shutdown task.
        drop(app);
        Ok(output)
    }
    .await;
    let closed = engine.close().await.map_err(native);
    match (outcome, closed) {
        (Err(primary), Err(cleanup)) => Err(Error::new("finite admin request and cleanup failed")
            .with_cause(CompletionErrors { primary, cleanup })),
        (Err(primary), Ok(())) => Err(primary),
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Ok(output), Ok(())) => Ok(output),
    }
}

/// Wrap a concrete native error once; public output does not include its Display text.
fn native(cause: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::new("finite admin request failed").with_cause(cause)
}

/// Retain both concrete failures without replacing the original source or exposing it publicly.
#[derive(Debug)]
struct CompletionErrors {
    primary: Error,
    cleanup: Error,
}
impl std::fmt::Display for CompletionErrors {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}; cleanup: {}", self.primary, self.cleanup)
    }
}
impl std::error::Error for CompletionErrors {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.primary)
    }
}
