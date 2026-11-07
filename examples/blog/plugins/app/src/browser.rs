//! Target-independent browser seed, response and error decisions.
//!
//! DOM/fetch/AbortController adapters pass plain data here so native tests and LLVM
//! coverage measure decisions shared by both browser renderers.
use crate::root::{Choice, Root};
use std::fmt;

/// Source document seed priority; the first present element wins, even if its text is empty.
pub const SEED_IDS: [&str; 3] = ["stackpress-page", "api-management-seed", "api-consent-seed"];

/// Select and decode DOM observations without depending on browser types.
/// # Errors
/// Retains missing-seed, invalid JSON and the existing snapshot decoder's error text.
pub fn decode_seed(seeds: [Option<Option<String>>; 3]) -> Result<Root, String> {
    let (id, text) = SEED_IDS
        .into_iter()
        .zip(seeds)
        .find_map(|(id, text)| text.map(|text| (id, text.unwrap_or_default())))
        .ok_or_else(|| "page seed missing".to_owned())?;
    Root::decode(id, &text).map_err(|error| error.to_string())
}

/// Plain lookup failures; every failure retains the existing relation choices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationError {
    /// Window/fetch failure before a response is available.
    Fetch,
    /// The fetch result was not a browser Response.
    Response,
    /// Response text could not be read.
    ReadBody,
    /// A non-success HTTP status must not replace current choices.
    Http(u16),
    /// Response text did not decode as JSON.
    Json(String),
    /// The field request was replaced or its root was disposed.
    Inactive,
}
impl fmt::Display for RelationError {
    /// Explain the plain failure without introducing a user-visible retry or error policy.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fetch => formatter.write_str("relation fetch unavailable"),
            Self::Response => formatter.write_str("relation response unavailable"),
            Self::ReadBody => formatter.write_str("relation response text unavailable"),
            Self::Http(status) => write!(formatter, "relation response status {status}"),
            Self::Json(error) => write!(formatter, "relation response JSON: {error}"),
            Self::Inactive => formatter.write_str("relation request inactive"),
        }
    }
}

/// Decide whether a response body may contribute choices, matching browser Response.ok.
/// # Errors
/// Non-2xx statuses retain the existing choices without consuming the response text.
pub fn response_status(status: u16) -> Result<(), RelationError> {
    if (200..300).contains(&status) {
        Ok(())
    } else {
        Err(RelationError::Http(status))
    }
}

/// Decode actual response text, then reject replaced/disposed completions in source order.
/// # Errors
/// Missing/non-JSON text and inactive requests retain the current choices.
pub fn relation_response(
    text: Option<&str>,
    alive: bool,
    aborted: bool,
) -> Result<Vec<Choice>, RelationError> {
    let value = serde_json::from_str(text.unwrap_or_default())
        .map_err(|error| RelationError::Json(error.to_string()))?;
    // Decode precedes cancellation, preserving the original response/error precedence.
    if aborted || !alive {
        return Err(RelationError::Inactive);
    }
    Ok(crate::root::choices(value))
}
