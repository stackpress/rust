//! Identity constituent: optional session config, me/authorize and conditional guard.
//!
//! Auth processes have their own registrar. The full auth/email/identity composite is
//! wired at integration, using Lane B's registrar rather than a local transport substitute.

use crate::{Service, SessionConfig, SessionPermission};
use stackpress_lib::{
    Body, Error, Flow, Invocation, PublicError, Request, Response, Result, queue::task,
};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use std::sync::Arc;

/// Select just the native identity constituent using the ordinary #14 registration contract.
/// Missing session config contributes no service/listener. Present empty access installs
/// the guard but authorize immediately permits all. Startup performs no database or network IO.
/// # Errors
/// Propagates existing config/service/registration errors, without new policy validation.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("session", None::<SessionConfig>, |_| Ok(()))?;
    app.on_config(task(|start: &mut StartupContext| {
        Box::pin(async move {
            if let Some(mut config) = start.config::<Option<SessionConfig>>()?.clone() {
                // Plugin fallback uses source || semantics; direct configure remains literal.
                if config.key.is_empty() {
                    config.key = "session".into();
                }
                if config.seed.is_empty() {
                    config.seed = "abc123".into();
                }
                start.register(Arc::new(Service::new(config)))?;
            }
            Ok(Flow::Continue)
        })
    }))?;
    app.on_listen(task(|start: &mut StartupContext| {
        Box::pin(async move {
            if let Some(config) = start.config::<Option<SessionConfig>>()? {
                let guarded = config.access.is_some();
                start.on("me", me)?;
                start.on("authorize", authorize_event)?;
                if guarded {
                    start.on("request", request_guard)?;
                }
            }
            Ok(Flow::Continue)
        })
    }))?;
    Ok(())
}
impl Service {
    /// Read only the configured normalized cookie, never an Authorization fallback.
    pub fn token(&self, request: &Request) -> Option<String> {
        request
            .session
            .get(&self.config().key)
            .and_then(|v| v.as_str())
            .map(str::to_owned)
    }
    /// Create an independent memoized instance from this request's cookie state.
    pub fn load_request(&self, request: &Request) -> crate::Session {
        self.load(self.token(request).unwrap_or_default())
    }
    /// Authorize current route plus extra permits, retaining source Vec mutation and results.
    /// # Errors
    /// Propagates response mutation and source roles/pattern failures as native errors.
    pub async fn authorize(
        &self,
        request: &Request,
        response: &mut Response,
        permits: &mut Vec<SessionPermission>,
    ) -> Result<bool> {
        // Empty access bypasses even token loading, permit mutation and response projection.
        if self
            .config()
            .access
            .as_ref()
            .is_none_or(|access| access.is_empty())
        {
            return Ok(true);
        }
        permits.insert(
            0,
            SessionPermission::route(request.method.as_str().to_uppercase(), request.url.path()),
        );
        let session = self.load_request(request);
        if !session.can(permits).await.map_err(native_error)? {
            response.set_error(PublicError {
                code: 401,
                status: "Unauthorized".into(),
                error: "Unauthorized".into(),
                errors: serde_json::json!({}),
            })?;
            return Ok(false);
        }
        response.results(&session.authorization().await.map_err(native_error)?)?;
        Ok(true)
    }
}
impl crate::Session {
    /// Stage this exact token under the current configured key and existing cookie options.
    /// # Errors
    /// Propagates the lib cookie framing/response state errors; adds no flags or expiry.
    pub fn save(&self, response: &mut Response) -> Result<()> {
        response
            .session_mut()?
            .set(&self.service_key(), self.token().to_owned(), None)?;
        Ok(())
    }
}
/// Ordinary source me adapter; authorization deliberately contains raw token data.
/// # Errors
/// Propagates the current invocation's service/result failures.
pub async fn me(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let identity = invocation.service::<Service>()?.load_request(request);
    response.results(&identity.authorization().await.map_err(native_error)?)?;
    Ok(Flow::Continue)
}
/// Direct authorize event shares the original Service implementation.
/// # Errors
/// Propagates source permission errors and native host/response failures.
pub async fn authorize_event(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    invocation
        .service::<Service>()?
        .authorize(request, response, &mut Vec::new())
        .await?;
    Ok(Flow::Continue)
}
/// Cooperate with earlier response/error handlers; a normal source return never adds Abort.
/// # Errors
/// Propagates nested dispatch and existing response mutation failures.
pub async fn request_guard(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    if completed(response) {
        return Ok(Flow::Continue);
    }
    // Resolve source authorization into a fresh response while borrowing the same request.
    // A nested emit supplies that Rust borrow without cloning/consuming the native Request.
    let mut authorized = Response::new();
    invocation
        .emit("authorize", request, &mut authorized)
        .await?;
    // Source raw Response starts at code zero. Even an empty access map's no-result
    // authorize shortcut is offered to `error`, without inventing a denial/error status.
    if authorized.status().map_or(0, |s| s.code) != 200 {
        response.apply_envelope(authorized.to_envelope())?;
        invocation.emit("error", request, response).await?;
    }
    Ok(Flow::Continue)
}
/// Source response-body truthiness; an explicit null/false/zero/empty string does not skip.
pub(crate) fn completed(response: &Response) -> bool {
    use stackpress_lib::Value;
    let body = match response.body() {
        Body::Empty => false,
        Body::Text(text) => !text.is_empty(),
        Body::Value(Value::Null) => false,
        Body::Value(Value::Bool(value)) => *value,
        Body::Value(Value::Number(value)) => value.as_f64() != Some(0.0),
        Body::Value(Value::String(value)) => !value.is_empty(),
        // Objects/arrays/buffers are truthy in JavaScript, including empty ones.
        _ => true,
    };
    body || response
        .status()
        .is_some_and(|status| status.code != 0 && status.code != 200)
}
/// Attach a safe message and typed cause without exposing seeds/tokens in public errors.
fn native_error(error: crate::SessionError) -> Error {
    Error::new("session operation failed").with_cause(error)
}
