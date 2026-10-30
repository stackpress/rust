//! Original header, overlay and predicate semantics plus Send nested/fetch boundaries.

use serde_json::Value as Json;
use stackpress_lib::{Flow, Invocation, Nest, PublicError, Request, Response, Result};

/// Source ID/secret parsing result; this is not a verified caller or a JWT session.
#[derive(Clone, PartialEq, Eq)]
pub struct Authorization {
    /// Trimmed source token, including any extra colon components.
    pub token: String,
    /// Trimmed first colon segment.
    pub id: String,
    /// Trimmed second segment, or empty on an ID-only GET.
    pub secret: String,
}

/// Parse exactly the source split/trim/method rules without validating the scheme.
///
/// `split(' ')` uses segment one, not arbitrary whitespace collapse; only the first two
/// colon segments supply ID/secret. GET may omit the secret; every other method needs it.
pub fn parse_authorization(method: &str, header: &str) -> Option<Authorization> {
    // Preserve the source's positional split, including empty segments and extra components.
    let token = header.split(' ').nth(1)?;
    if js_trim(token).is_empty() {
        return None;
    }
    let mut parts = token.split(':');
    let id = js_trim(parts.next()?);
    let secret = js_trim(parts.next().unwrap_or_default());
    // Presence is method-dependent; no secret equality or scope check occurs in this helper.
    if id.is_empty() || (!method.eq_ignore_ascii_case("GET") && secret.is_empty()) {
        return None;
    }
    Some(Authorization {
        token: js_trim(token).to_owned(),
        id: id.to_owned(),
        secret: secret.to_owned(),
    })
}

/// Read the original Authorization abstraction and set Unauthorized only on parse failure.
///
/// # Errors
/// Returns response mutation errors; rejection itself is Ok(None) and does not Abort a chain.
pub fn authorize(request: &Request, response: &mut Response) -> Result<Option<Authorization>> {
    // Header absence and parser failure have the same explicit source response mutation.
    let parsed = request
        .headers
        .get("authorization")
        .and_then(|header| parse_authorization(request.method.as_str(), header));
    if parsed.is_none() {
        unauthorized(response)?;
    }
    Ok(parsed)
}

/// Stage the source Unauthorized 401 without adding a challenge, 403 or automatic Abort.
///
/// # Errors
/// Returns existing response-state/budget failures rather than reopening committed output.
pub fn unauthorized(response: &mut Response) -> Result<()> {
    response.set_error(PublicError {
        code: 401,
        status: "Unauthorized".into(),
        error: "Unauthorized".into(),
        errors: Json::Object(Default::default()),
    })?;
    Ok(())
}

/// Apply source Nest.set(object) one top-level key at a time, including empty-key append.
///
/// # Errors
/// Preserves foundation mutation limits/errors and earlier completed keys, as source set does.
/// Nested objects/arrays replace whole values. An empty key uses the current member count as
/// its key, including intentional collisions; other names remain literal. No authority-key
/// filtering or deep merge occurs.
pub fn overlay(target: &mut Nest, fixed: &Nest) -> Result<()> {
    for (key, value) in fixed.entries() {
        // Source Nest.set('', value) uses Object.keys(pointer).length, even if that key exists.
        let append_key = key.is_empty().then(|| target.entries().len().to_string());
        // Other names, including dots, remain one literal segment in the source call.
        target.set(append_key.as_deref().unwrap_or(key), value.clone())?;
    }
    Ok(())
}

/// Await raw nested emit and deliberately ignore its Completed/Aborted/NoListener return.
///
/// # Errors
/// Propagates actual handler/dispatch errors. The API wrapper itself returns Continue, matching
/// source `await ctx.emit(...)` without `return`; child Abort only stops the child chain.
pub async fn emit_event(
    invocation: &Invocation<'_>,
    event: &str,
    request: &mut Request,
    response: &mut Response,
) -> Result<Flow> {
    // Raw emit retains ancestry/host context without running another top-level lifecycle.
    invocation.emit(event, request, response).await?;
    Ok(Flow::Continue)
}

/// Resolve source object input on a fresh Request and fresh Response.
///
/// # Errors
/// Preserves construction/handler errors. Fresh lookup input carries no copied browser cookie,
/// JWT, method, headers or request data. Cancellation drops child work without rollback.
pub async fn resolve(invocation: &Invocation<'_>, event: &str, data: Nest) -> Result<Response> {
    let mut response = Response::new();
    resolve_into(invocation, event, data, &mut response).await?;
    Ok(response)
}

/// Resolve fresh object input while deliberately retaining the caller's supplied response.
///
/// # Errors
/// Preserves input/dispatch/handler failures and completed response mutations; adds no
/// NoListener error. This is the source OAuth application-detail shared-response boundary.
pub async fn resolve_into(
    invocation: &Invocation<'_>,
    event: &str,
    data: Nest,
    response: &mut Response,
) -> Result<()> {
    // Builder validates the owned source object under the ordinary foundation budgets.
    let mut request = Request::builder().data(data).build()?;
    invocation.emit(event, &mut request, response).await?;
    Ok(())
}

/// Borrowed fetch compatibility exports retained from M0.
pub use crate::webhooks::fetch::{Fetch, FetchRequest, fetch_once};
/// Source predicate compatibility exports retained from the M0 public helper surface.
pub use crate::webhooks::predicate::{PredicateObject, PredicateValue, valid_data};

/// Match ECMAScript trim's explicit whitespace set, which differs from Rust Unicode trim.
fn js_trim(value: &str) -> &str {
    value.trim_matches(|ch| {
        matches!(ch,
            '\u{0009}'..='\u{000D}' | '\u{0020}' | '\u{00A0}' | '\u{1680}' |
            '\u{2000}'..='\u{200A}' | '\u{2028}' | '\u{2029}' | '\u{202F}' |
            '\u{205F}' | '\u{3000}' | '\u{FEFF}'
        )
    })
}
