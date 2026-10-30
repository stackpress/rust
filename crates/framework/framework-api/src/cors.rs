//! The original CORS header/preflight convenience, independent of endpoint admission.
//!
//! URL origin, exact response header names and fixed priority 1000 intentionally match the
//! source. No incoming Origin policy, credentials, Vary or method-admission rule is added.

use std::sync::Arc;

use stackpress_lib::{Flow, MethodPattern, Request, Response, Result, handler};
use stackpress_server::{RouteSpec, ServerBuilder};

use crate::{ApiEndpoint, Cors, routes::PreparedRoute};

/// Register source OPTIONS and actual-method CORS callbacks, independently of an API plugin.
///
/// False/omitted CORS creates none; true/string/list creates both, including an empty list.
/// Preflight sets status 200 and Continue even when no origin matches. ALL can consequently
/// run its ordinary non-GET endpoint admission on the same OPTIONS request.
///
/// # Errors
/// Returns existing method/path/registration errors when callbacks are selected. Response
/// header/state errors occur at dispatch and propagate without undoing earlier mutations.
pub fn register(app: &mut ServerBuilder, endpoint: ApiEndpoint) -> Result<()> {
    for route in prepare(&endpoint)? {
        app.route_with(route.spec, route.action)?;
    }
    Ok(())
}

/// Normalize only source CORS shapes, then prepare preflight before the actual-method callback.
pub(crate) fn prepare(endpoint: &ApiEndpoint) -> Result<Vec<PreparedRoute>> {
    let origins = match &endpoint.cors {
        None | Some(Cors::Enabled(false)) => return Ok(Vec::new()),
        Some(Cors::Enabled(true)) => vec!["*".to_owned()],
        Some(Cors::Origin(origin)) => vec![origin.clone()],
        Some(Cors::Origins(origins)) => origins.clone(),
    };
    let origins = Arc::new(origins);
    let method = Arc::new(if endpoint.method == "ALL" {
        "*".to_owned()
    } else {
        endpoint.method.clone()
    });
    let mut prepared = Vec::new();
    for (matcher, preflight) in [
        (MethodPattern::OPTIONS, true),
        (crate::routes::method(&endpoint.method)?, false),
    ] {
        let origins = origins.clone();
        let method = method.clone();
        let action = handler(move |req, res, _| {
            let origins = origins.clone();
            let method = method.clone();
            Box::pin(async move {
                apply(&origins, &method, req, res)?;
                if preflight {
                    // No match still gets 200; this does not stop any later matching handler.
                    res.status_code(200, None)?;
                }
                Ok(Flow::Continue)
            })
        });
        prepared.push(PreparedRoute {
            spec: RouteSpec::new(matcher, &endpoint.route).priority(1000),
            action: Box::new(action),
        });
    }
    Ok(prepared)
}

/// Set the exact source headers only for wildcard or current URL-origin equality.
fn apply(origins: &[String], method: &str, req: &Request, res: &mut Response) -> Result<()> {
    let origin = req.url.origin().ascii_serialization();
    let allowed = if origins.iter().any(|candidate| candidate == "*") {
        "*"
    } else if origins.contains(&origin) {
        origin.as_str()
    } else {
        // Unmatched origins omit headers without denying the endpoint or removing prior ones.
        return Ok(());
    };
    let headers = res.headers_mut()?;
    headers.set("Access-Control-Allow-Origin", allowed)?;
    headers.set("Access-Control-Request-Method", method)?;
    headers.set("Access-Control-Allow-Methods", "OPTIONS, GET")?;
    headers.set("Access-Control-Allow-Headers", "*")?;
    Ok(())
}
