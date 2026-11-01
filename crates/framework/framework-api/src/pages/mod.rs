//! OAuth process handlers, composed with independently selected JWT and SQL events.
//!
//! No caller slot or identity service is created here. `me` receives the original Request;
//! object lookups get fresh input, and application-detail deliberately shares the response.

mod oauth;
mod token;
pub use oauth::oauth;
pub use token::token;

use serde_json::Value as Json;
use stackpress_lib::{Body, Error, Invocation, MethodPattern, Response, Result, Value};
use stackpress_server::{RouteSpec, ServerBuilder, ServerInvocationExt};
use std::{sync::Arc, time::SystemTime};

/// Explicit host mapping of source view/brand config and Date.now, without claiming sections
/// owned by other plugins. Missing service uses source presentation and wall-clock fallbacks.
/// The host supplies the complete source objects; only selected fields become response data.
#[derive(Clone)]
pub struct Environment {
    /// Source view configuration; base, props and notify are selected by OAuth.
    pub view: Json,
    /// Source brand configuration; name, logo, icon and favicon are selected by OAuth.
    pub brand: Json,
    /// Epoch milliseconds, called at each source Date.now access (not once per request).
    pub now: Arc<dyn Fn() -> Result<i64> + Send + Sync>,
}
impl Default for Environment {
    /// Preserve absent view/brand objects and the native wall clock.
    fn default() -> Self {
        Self {
            view: serde_json::json!({}),
            brand: serde_json::json!({}),
            now: Arc::new(now),
        }
    }
}

/// Register both ALL-method process routes directly, without installing their collaborators.
///
/// The API plugin calls this inventory only for nonempty endpoints. Hosts select the session
/// registrar and API SQL registrar separately; consent rendering is a lower-priority view.
/// # Errors
/// Returns ordinary host route errors. No resources are acquired during registration.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    for route in prepare() {
        app.route_with(route.spec, route.action)?;
    }
    Ok(())
}

/// Source import order is token, OAuth, then configured API endpoints; both priorities are zero.
pub(crate) fn prepare() -> Vec<crate::routes::PreparedRoute> {
    vec![
        crate::routes::PreparedRoute {
            spec: RouteSpec::new(MethodPattern::Any, "/auth/oauth/token"),
            action: Box::new(stackpress_lib::handler(|req, res, inv| {
                Box::pin(token(req, res, inv))
            })),
        },
        crate::routes::PreparedRoute {
            spec: RouteSpec::new(MethodPattern::Any, "/auth/oauth"),
            action: Box::new(stackpress_lib::handler(|req, res, inv| {
                Box::pin(oauth(req, res, inv))
            })),
        },
    ]
}

/// Preserve only the upstream truthy body / nonzero non-200 status entry guard.
pub(crate) fn completed(response: &Response) -> bool {
    let body = match response.body() {
        Body::Empty => false,
        Body::Text(text) => !text.is_empty(),
        Body::Value(value) => truthy(value),
        // Buffers, arrays and objects remain truthy even when empty.
        _ => true,
    };
    body || response
        .status()
        .is_some_and(|status| status.code != 0 && status.code != 200)
}

/// Source Boolean(value), with missing handled by the caller before reaching this function.
pub(crate) fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value
            .as_f64()
            .is_none_or(|value| value != 0.0 && !value.is_nan()),
        Value::String(value) => !value.is_empty(),
        _ => true,
    }
}

/// Source property access on optional result objects, without imposing a model DTO.
fn property<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_object().and_then(|object| object.get(key))
}

/// Source String(...) for URLSearchParams values; containers retain JS coercion semantics.
fn text(value: &Value) -> Result<String> {
    Ok(match value {
        Value::String(value) => value.clone(),
        Value::Null => "null".into(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => number_text(value.as_f64().unwrap_or_default()),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                if matches!(value, Value::Null) {
                    Ok(String::new())
                } else {
                    text(value)
                }
            })
            .collect::<Result<Vec<_>>>()?
            .join(","),
        Value::Object(_) => "[object Object]".into(),
        // Native IO/Date values need an explicit source String adapter, not a fabricated label.
        _ => {
            return Err(Error::invalid(
                "URL value needs an explicit native String conversion",
            ));
        }
    })
}

/// Select the app's explicit mapping without retaining registry borrows across nested work.
fn environment(invocation: &Invocation<'_>) -> Environment {
    invocation
        .service::<Environment>()
        .cloned()
        .unwrap_or_default()
}

/// Native Date.now, rounded down to integral milliseconds, with pre-epoch handling.
fn now() -> Result<i64> {
    let millis = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(time) => i128::try_from(time.as_millis()).map_err(Error::from_source)?,
        Err(error) => -i128::try_from(error.duration().as_millis()).map_err(Error::from_source)?,
    };
    i64::try_from(millis).map_err(Error::from_source)
}

/// JavaScript encodeURIComponent's byte escape set, used for the signin continuation only.
fn encode(value: &str) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            output.push(char::from(byte));
        } else {
            write!(output, "%{byte:02X}").expect("String write");
        }
    }
    output
}

/// JavaScript uses one Number family, plain decimal in [1e-6,1e21) and signed exponents outside.
fn number_text(value: f64) -> String {
    if value == 0.0 {
        return "0".into();
    }
    if value.abs() >= 1e21 || value.abs() < 1e-6 {
        let scientific = format!("{value:e}");
        if let Some((mantissa, exponent)) = scientific.split_once('e')
            && !exponent.starts_with('-')
        {
            return format!("{mantissa}e+{exponent}");
        }
        return scientific;
    }
    value.to_string()
}
