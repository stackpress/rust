//! Borrowed Send fetch boundary and explicitly selected transport service.

use stackpress_lib::{HandlerFuture, Result};
use std::sync::Arc;

/// An application-selected transport, installed during Config before API Listen runs.
///
/// When absent the plugin constructs its native reqwest adapter only if webhooks exist.
/// Direct registration accepts an `Arc<dyn Fetch>` and needs no service or config section.
#[derive(Clone)]
pub struct Transport(pub Arc<dyn Fetch>);

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
/// added. Response eligibility and serialization belong to the webhook adapter.
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
