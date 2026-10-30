//! Late inline source webhooks, with no queue, signature or delivery record.
//!
//! Selected plugins install these during Listen. Direct callers choose a transport and
//! register the same callback. Cancellation drops the awaited send; prior remote effects
//! cannot be rolled back and another event invocation can send another notification.

use std::sync::Arc;

use serde_json::Value as Json;
use stackpress_lib::{Flow, Request, Response, Result, handler};
use stackpress_server::ServerBuilder;

use crate::ApiWebhook;

pub mod fetch;
pub mod predicate;
pub mod reqwest;

pub use fetch::{Fetch, FetchRequest, Transport};
pub use reqwest::ReqwestFetch;

use predicate::{PredicateObject, PredicateValue};

/// One prepared source configuration; only its explicit predicate carrier shares identity.
pub struct Webhook {
    /// Destination, event, method and fixed data; no projection or destination policy.
    pub config: ApiWebhook,
    /// Source validity, decoded separately unless a caller explicitly shares Arc containers.
    pub predicate: PredicateObject,
}

impl Webhook {
    /// Prepare bounded typed configuration without sending or resolving a destination.
    ///
    /// # Errors
    /// Returns inherited JSON representation/budget errors.
    pub fn new(config: ApiWebhook) -> Result<Self> {
        // A separate decode deliberately creates separate container identity (Q10-REF).
        let PredicateValue::Object(predicate) =
            PredicateValue::from_json(&Json::Object(config.validity.clone()))?
        else {
            unreachable!("a typed validity object decodes as an object");
        };
        Ok(Self { config, predicate })
    }
}

/// Register one direct webhook at source priority -200, without an API config section.
///
/// Supply the same Fetch implementation for related registrations to reuse its client.
/// Duplicate event subscriptions remain independent, with registration-order ties.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use stackpress_framework_api::{ApiWebhook, webhooks::{self, Webhook, ReqwestFetch}};
/// use stackpress_server::ServerBuilder;
/// let config: ApiWebhook = serde_json::from_value(serde_json::json!({
///     "event": "article-created", "uri": "https://receiver.example/hook",
///     "method": "POST", "validity": {"published": true}, "data": {}
/// }))?;
/// let mut app = ServerBuilder::new();
/// webhooks::register(&mut app, Webhook::new(config)?, Arc::new(ReqwestFetch::new()?))?;
/// // Registration sends nothing. The application installs the article-created operation.
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// Returns predicate/registration errors; no fetch runs until the event is invoked.
pub fn register(app: &mut ServerBuilder, webhook: Webhook, fetch: Arc<dyn Fetch>) -> Result<()> {
    app.on_priority(&webhook.config.event.clone(), -200, action(webhook, fetch))?;
    Ok(())
}

/// Share immutable subscription state without a synchronization guard across await.
pub(crate) fn action(webhook: Webhook, fetch: Arc<dyn Fetch>) -> crate::routes::Action {
    let webhook = Arc::new(webhook);
    Box::new(handler(move |req, res, _| {
        let webhook = Arc::clone(&webhook);
        let fetch = Arc::clone(&fetch);
        Box::pin(async move {
            send(fetch.as_ref(), &webhook, req, res).await?;
            // Source awaits fetch without returning a stop signal to the event emitter.
            Ok(Flow::Continue)
        })
    }))
}

/// Apply source status/predicate selection and await exactly one serialized envelope send.
///
/// No body/error/commit guard supplements code 200. All ordinary request/result fields,
/// including secrets, remain in the source envelope. A resolved HTTP status is ignored.
///
/// # Errors
/// Propagates property-access, serialization and fetch errors, without retry or rollback.
pub async fn send(
    fetch: &dyn Fetch,
    webhook: &Webhook,
    req: &Request,
    res: &Response,
) -> Result<()> {
    if res.status().map(|status| status.code) != Some(200) {
        return Ok(());
    }
    // The status response uses the current body, including text and explicit null.
    let results = res.to_envelope().results;
    if !predicate::matches(&webhook.predicate, results.as_ref())? {
        return Ok(());
    }
    let mut envelope = serde_json::Map::new();
    envelope.insert("data".into(), Json::Object(webhook.config.data.clone()));
    envelope.insert("params".into(), predicate::wire(req.data.value())?);
    if let Some(results) = results {
        // A missing body is omitted; explicit null is retained by JSON.stringify semantics.
        if let Some(results) = predicate::wire_optional(&results)? {
            envelope.insert("results".into(), results);
        }
    } else {
        // The source's default _body is null, not an absent object member.
        envelope.insert("results".into(), Json::Null);
    }
    let body = serde_json::to_vec(&envelope).map_err(stackpress_lib::Error::from_source)?;
    fetch::fetch_once(fetch, &webhook.config.uri, &webhook.config.method, &body).await
}
