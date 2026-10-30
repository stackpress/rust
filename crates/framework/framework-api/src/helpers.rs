//! Original header, overlay and predicate semantics plus Send nested/fetch boundaries.

use std::{collections::BTreeMap, sync::Arc};

use serde_json::Value as Json;
use stackpress_lib::{
    Flow, HandlerFuture, Invocation, Nest, PublicError, Request, Response, Result,
};

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

/// Source predicate object with explicit shared-container identity.
pub type PredicateObject = Arc<BTreeMap<String, PredicateValue>>;

/// JavaScript-compatible predicate values; cloning a container keeps its identity.
///
/// Separately decoded JSON creates distinct Arc containers. This carrier is intentionally
/// separate from ordinary JSON equality and from low-level Request/Response identity.
#[derive(Clone, Debug)]
pub enum PredicateValue {
    /// A source undefined value; an expected key whose actual value is undefined fails.
    Undefined,
    /// Explicit null, distinct from an absent/undefined member.
    Null,
    /// Source Boolean, never truthiness-coerced for equality.
    Boolean(bool),
    /// Source numeric equality, including SameValueZero for array membership.
    Number(f64),
    /// Source string, distinct from a numeric or Boolean representation.
    String(String),
    /// Array identity is compared by pointer when it is an includes element.
    Array(Arc<Vec<PredicateValue>>),
    /// Object subset checks recurse, but includes elements compare pointer identity.
    Object(PredicateObject),
}

impl PredicateValue {
    /// Convert bounded JSON, assigning fresh identity to every decoded container.
    ///
    /// # Errors
    /// Retains the existing schema JSON representation limits; adds no API-specific policy.
    pub fn from_json(value: &Json) -> Result<Self> {
        // Existing finite depth/node checks precede recursive carrier construction.
        stackpress_framework_schema::check_json(value)
            .map_err(stackpress_lib::Error::from_source)?;
        Self::decoded(value)
    }

    /// Convert an already bounded JSON subtree, preserving only explicitly shared Arc identity.
    fn decoded(value: &Json) -> Result<Self> {
        Ok(match value {
            Json::Null => Self::Null,
            Json::Bool(value) => Self::Boolean(*value),
            // Additive arbitrary_precision can retain a numeric lexeme outside finite f64.
            // Parsing its JSON spelling retains source double rounding/overflow without panic.
            Json::Number(value) => Self::Number(match value.as_f64() {
                Some(number) => number,
                None => value
                    .to_string()
                    .parse()
                    .map_err(stackpress_lib::Error::from_source)?,
            }),
            Json::String(value) => Self::String(value.clone()),
            Json::Array(values) => Self::Array(Arc::new(
                values.iter().map(Self::decoded).collect::<Result<_>>()?,
            )),
            Json::Object(values) => Self::Object(Arc::new(
                values
                    .iter()
                    .map(|(key, value)| Ok((key.clone(), Self::decoded(value)?)))
                    .collect::<Result<_>>()?,
            )),
        })
    }
}

/// Match source object subsets and array includes without replacing reference identity.
///
/// The work stack avoids native recursion during matching. It adds no scope, identity or
/// data-projection rule. Empty expected objects match any supplied object.
pub fn valid_data(assertion: &PredicateObject, data: &PredicateObject) -> bool {
    let mut pending = vec![(assertion, data)];
    while let Some((assertion, data)) = pending.pop() {
        for (key, expected) in assertion.iter() {
            // Source checks typeof actual === undefined before selecting the comparison branch.
            let Some(actual) = data
                .get(key)
                .filter(|value| !matches!(value, PredicateValue::Undefined))
            else {
                return false;
            };
            match (expected, actual) {
                (PredicateValue::Array(expected), PredicateValue::Array(actual)) => {
                    // includes is SameValueZero for scalars and identity for container elements.
                    if !expected
                        .iter()
                        .all(|item| actual.iter().any(|other| same_value(item, other, true)))
                    {
                        return false;
                    }
                }
                (PredicateValue::Object(expected), PredicateValue::Object(actual)) => {
                    pending.push((expected, actual));
                }
                // A container expected against another type cannot fall through to scalar equality.
                (PredicateValue::Array(_) | PredicateValue::Object(_), _) => return false,
                _ if !same_value(expected, actual, false) => return false,
                _ => {}
            }
        }
    }
    true
}

/// Compare scalar strict equality or includes SameValueZero, retaining container pointer identity.
fn same_value(left: &PredicateValue, right: &PredicateValue, includes: bool) -> bool {
    match (left, right) {
        (PredicateValue::Undefined, PredicateValue::Undefined)
        | (PredicateValue::Null, PredicateValue::Null) => true,
        (PredicateValue::Boolean(left), PredicateValue::Boolean(right)) => left == right,
        (PredicateValue::Number(left), PredicateValue::Number(right)) => {
            left == right || (includes && left.is_nan() && right.is_nan())
        }
        (PredicateValue::String(left), PredicateValue::String(right)) => left == right,
        (PredicateValue::Array(left), PredicateValue::Array(right)) => Arc::ptr_eq(left, right),
        (PredicateValue::Object(left), PredicateValue::Object(right)) => Arc::ptr_eq(left, right),
        _ => false,
    }
}

/// One borrowed source fetch invocation; the future may not outlive these bytes/strings.
pub struct FetchRequest<'a> {
    /// Configured destination, without an API-added URI policy.
    pub uri: &'a str,
    /// Source method, passed unchanged to the eventual native adapter.
    pub method: &'a str,
    /// Source Content-Type header value.
    pub content_type: &'a str,
    /// Already serialized exact `{data,params,results}` bytes, borrowed until completion.
    pub body: &'a [u8],
}

/// The inline asynchronous native-fetch boundary, not a worker or delivery queue.
pub trait Fetch: Send + Sync {
    /// Await one send and return its resolved HTTP status, or its original transport error.
    ///
    /// No response-status retry or interpretation is implied. Futures must be Send, and the
    /// caller owns cancellation and all borrowed request data until the await completes.
    fn send<'a>(&'a self, request: FetchRequest<'a>) -> HandlerFuture<'a, Result<u16>>;
}

/// Await one source fetch, deliberately ignoring a resolved HTTP error status.
///
/// # Errors
/// Propagates the single transport rejection. No retry, signature, dedup or extra policy is
/// added. This M0 boundary does not yet select source response eligibility or serialize it.
pub async fn fetch_once(fetch: &dyn Fetch, uri: &str, method: &str, body: &[u8]) -> Result<()> {
    fetch
        .send(FetchRequest {
            uri,
            method,
            content_type: "application/json",
            body,
        })
        .await?;
    Ok(())
}

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
