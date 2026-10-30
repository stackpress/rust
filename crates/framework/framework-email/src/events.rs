//! Reusable source `email-send` process, independent of plugin selection and routes.
//!
//! Configuration is read only after the source body/status skips. Every send owns a
//! new transport. No authentication, CSRF, recipient or method gate is added here.

mod native;

pub use native::NativeReader;

use crate::{EmailConfig, TransportFactory};
use serde_json::Value as Json;
use stackpress_lib::{Body, Flow, Request, Response, Result, Value};
use std::sync::Arc;

/// Original event name; signup's `email-email-send` is a distinct extension hook.
pub const EMAIL_SEND: &str = "email-send";

/// Owned reusable event state; application-specific factories never cross host boundaries.
#[derive(Clone)]
pub struct EmailSend {
    /// Raw source section, deferred until the process actually needs a transport.
    config: Option<Json>,
    /// Construction is shared; each returned transport remains invocation-local.
    factory: Arc<dyn TransportFactory>,
}

impl EmailSend {
    /// Bind the exact optional source config and a native/fake transport factory.
    /// Construction neither validates unused transport options nor performs IO.
    pub fn new(config: Option<Json>, factory: Arc<dyn TransportFactory>) -> Self {
        Self { config, factory }
    }

    /// Execute the original process against an explicit request/response pair.
    ///
    /// A normal source return is Continue, including body/status/config skips.
    /// Provider info becomes results without filtering; failures remain native causes.
    /// Cancellation drops the per-call transport and readers without retry/rollback.
    ///
    /// # Errors
    /// Returns config/input conversion, factory, provider or response-state errors.
    /// No response status/error is manufactured for a thrown provider failure.
    pub async fn handle(&self, request: &mut Request, response: &mut Response) -> Result<Flow> {
        // Source short-circuits body before status, and both before reading config/input.
        if body_truthy(response.body())
            || response
                .status()
                .is_some_and(|s| s.code != 0 && s.code != 200)
        {
            return Ok(Flow::Continue);
        }
        // Empty objects are truthy; missing/null/false/zero/empty-string disable this call.
        let Some(config) = EmailConfig::from_section(self.config.as_ref())? else {
            return Ok(Flow::Continue);
        };
        // Capture all source fields, then construct exactly one per-call transport.
        let options = native::options(&request.data)?;
        let mut transport = self.factory.create(&config).map_err(native::error)?;
        let info = transport
            .send_native(options)
            .await
            .map_err(native::error)?;
        // Rich results keep a native stream at `message`; JSON fields remain unchanged.
        native::results(info, response)?;
        Ok(Flow::Continue)
    }
}

/// Preserve JavaScript truthiness for source response bodies, including empty objects/buffers.
fn body_truthy(body: &Body) -> bool {
    match body {
        Body::Empty => false,
        Body::Text(text) => !text.is_empty(),
        Body::Value(value) => match value {
            Value::Null | Value::Bool(false) => false,
            Value::Number(number) => number.as_f64() != Some(0.0),
            Value::String(text) => !text.is_empty(),
            _ => true,
        },
        Body::Bytes(_) => true,
    }
}
