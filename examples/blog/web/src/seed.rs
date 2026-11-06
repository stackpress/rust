//! The original fifteen ordered events, with application expiry computed when requested.

use chrono::{SecondsFormat, Utc};
use serde_json::Value;
use stackpress_lib::{Error, Result};

/// Load the source records and reproduce `new Date(Date.now() + 365 days)`.
///
/// This only constructs configuration. The framework's populate command performs sequential
/// nested dispatch and retains the original behavior for child response codes.
///
/// # Errors
/// Refuses malformed checked-in seed data; never fabricates an empty population list.
pub fn events() -> Result<Value> {
    let mut events: Value = serde_json::from_str(include_str!("populate.json"))?;
    let expires =
        (Utc::now() + chrono::Duration::days(365)).to_rfc3339_opts(SecondsFormat::Millis, true);
    for event in events
        .as_array_mut()
        .ok_or_else(|| Error::new("expected seed array"))?
    {
        if event["event"] == "application-create" {
            event["data"]["expires"] = expires.clone().into();
        }
    }
    Ok(events)
}
