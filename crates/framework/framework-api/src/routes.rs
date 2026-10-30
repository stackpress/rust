//! Independently callable source public/application/API-session route adapters.
//!
//! Each route keeps its configured data immutable and mutates only the current request/
//! response. Nested detail lookups use fresh requests; target events reuse the original pair.

use std::sync::Arc;

use stackpress_lib::{
    Error, Flow, HandlerFuture, Invocation, MethodPattern, Nest, Request, Response, Result, Value,
    handler,
};
use stackpress_server::{RouteSpec, ServerBuilder};

use crate::{ApiEndpoint, ApiType, helpers};

pub use crate::cors::register as cors;

/// One erased borrowing callback, accepted by the foundation's existing Handler contract.
pub(crate) type Action = Box<
    dyn for<'a> Fn(
            &'a mut Request,
            &'a mut Response,
            Invocation<'a>,
        ) -> HandlerFuture<'a, Result<Flow>>
        + Send
        + Sync,
>;

/// One independent contribution, not route ownership or conflict arbitration.
pub(crate) struct PreparedRoute {
    /// Existing host matcher, priority and capture behavior.
    pub spec: RouteSpec,
    /// Immutable configured callback; invocation state remains borrowed and local.
    pub action: Action,
}

/// Only the immutable source inputs needed after a route has matched.
struct EndpointAction {
    /// Configured event, resolved at invocation rather than selected from an allowlist.
    event: String,
    /// Alternative accepted scope strings; the registry remains presentation-only.
    scopes: Vec<String>,
    /// Bounded owned fixed data, converted once and cloned into each request by overlay.
    fixed: Nest,
    /// Explicitly selected source adapter, also used by the direct registration functions.
    kind: ApiType,
}

/// Register the source public adapter; endpoint.type does not override this direct choice.
///
/// # Errors
/// Returns existing route/data/registration failures. No authorization or resource lookup
/// is performed during registration.
pub fn open(app: &mut ServerBuilder, endpoint: ApiEndpoint) -> Result<()> {
    install(app, endpoint, ApiType::Public)
}

/// Register the source application adapter, using the application-detail event.
///
/// GET accepts an ID alone. Other methods compare the parsed secret exactly; scopes need
/// any overlap. The selected detail event owns its own row/active/expiry behavior.
///
/// # Errors
/// Returns existing route/data/registration failures; runtime lookup errors propagate.
pub fn app(app: &mut ServerBuilder, endpoint: ApiEndpoint) -> Result<()> {
    install(app, endpoint, ApiType::App)
}

/// Register the source API Session adapter, forwarding profileId as ordinary data.
///
/// This does not load a JWT or manufacture a caller/permission context. Empty protected
/// scope lists deny with 401, and rejection returns Continue just as the source does.
///
/// # Errors
/// Returns existing route/data/registration failures; runtime lookup errors propagate.
pub fn session(app: &mut ServerBuilder, endpoint: ApiEndpoint) -> Result<()> {
    install(app, endpoint, ApiType::Session)
}

/// Install a directly selected adapter using the same callback as the config loop.
fn install(app: &mut ServerBuilder, endpoint: ApiEndpoint, kind: ApiType) -> Result<()> {
    let route = prepare(endpoint, kind)?;
    app.route_with(route.spec, route.action)?;
    Ok(())
}

/// Normalize source method routing while retaining the original spelling for CORS headers.
pub(crate) fn method(method: &str) -> Result<MethodPattern> {
    // The source ActionRouter uppercases before recognizing its ALL matcher.
    if method.eq_ignore_ascii_case("ALL") {
        Ok(MethodPattern::Any)
    } else {
        MethodPattern::exact(method)
    }
}

/// Own fixed route input once and share it explicitly across Send borrowing invocations.
pub(crate) fn prepare(endpoint: ApiEndpoint, kind: ApiType) -> Result<PreparedRoute> {
    let spec =
        RouteSpec::new(method(&endpoint.method)?, &endpoint.route).priority(endpoint.priority);
    // Consume the owned data map instead of retaining a duplicate JSON copy in the callback.
    let fixed = Nest::from_object(serde_json::Value::Object(endpoint.data))?;
    let endpoint = Arc::new(EndpointAction {
        event: endpoint.event,
        scopes: endpoint.scopes,
        fixed,
        kind,
    });
    let action = handler(move |req, res, invocation| {
        // Only immutable configuration is shared; no guard or caller state spans the await.
        let endpoint = Arc::clone(&endpoint);
        Box::pin(async move { invoke(&endpoint, req, res, &invocation).await })
    });
    Ok(PreparedRoute {
        spec,
        action: Box::new(action),
    })
}

/// Perform source admission in order, then overlay fixed data and emit on the original pair.
async fn invoke(
    endpoint: &EndpointAction,
    req: &mut Request,
    res: &mut Response,
    invocation: &Invocation<'_>,
) -> Result<Flow> {
    // Public endpoints bypass lookup entirely; they still share overlay/emit ordering.
    let selected = if endpoint.kind == ApiType::Public {
        None
    } else {
        let Some(authorization) = helpers::authorize(req, res)? else {
            return Ok(Flow::Continue);
        };
        let event = if endpoint.kind == ApiType::Session {
            "session-detail"
        } else {
            "application-detail"
        };
        let mut input = Nest::new();
        input.set("id", authorization.id)?;
        // This response stays separate; its code does not gate a source-compatible result row.
        let response = helpers::resolve(invocation, event, input).await?;
        let Some(row) = response.result().filter(|row| truthy(row)) else {
            helpers::unauthorized(res)?;
            return Ok(Flow::Continue);
        };
        // Read only the property needed now, preserving GET and source error precedence.
        if req.method.as_str() != "GET"
            && property(row, "secret").and_then(Value::as_str)
                != Some(authorization.secret.as_str())
        {
            helpers::unauthorized(res)?;
            return Ok(Flow::Continue);
        }
        let scopes = property(row, "scopes")
            .and_then(Value::as_array)
            .ok_or_else(|| Error::new("API detail scopes must support array membership"))?;
        // Source some/includes uses any exact string overlap, not a registry or can() check.
        if !scopes.iter().any(|scope| {
            scope
                .as_str()
                .is_some_and(|scope| endpoint.scopes.iter().any(|permit| permit == scope))
        }) {
            helpers::unauthorized(res)?;
            return Ok(Flow::Continue);
        }
        // Only Session needs this value after admission; retain arbitrary source value types.
        if endpoint.kind == ApiType::Session {
            Some(property(row, "profileId").cloned())
        } else {
            None
        }
    };
    helpers::overlay(&mut req.data, &endpoint.fixed)?;
    if let Some(profile) = selected {
        if let Some(profile) = profile {
            req.data.set("profileId", profile)?;
        } else {
            // Nest has no undefined variant. Missing preserves get()/JSON-omission semantics;
            // unlike JS's own undefined-valued property, has() then reports absence.
            req.data.remove("profileId")?;
        }
    }
    // Child Abort/NoListener do not control the outer route chain; actual errors propagate.
    helpers::emit_event(invocation, &endpoint.event, req, res).await
}

/// Preserve JavaScript result truthiness without demanding a complete model DTO.
fn truthy(value: &Value) -> bool {
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

/// Source JSON property access; absent/non-object members behave as undefined.
fn property<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    value.as_object().and_then(|value| value.get(name))
}
