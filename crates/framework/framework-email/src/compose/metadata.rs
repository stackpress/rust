//! Address parsing, explicit/generated envelope distinctions and source Message-ID selection.

use super::headers::{header_content, message_headers};
use crate::{
    EmailResult,
    options::{string, truthy},
};
use lettre::message::{Mailbox, Mailboxes};
use ring::rand::{SecureRandom, SystemRandom};
use serde_json::{Map, Value, json};
use std::io;

/// Original envelope and Message-ID reused by every adapter/result.
#[derive(Clone, Debug)]
pub struct MessageMetadata {
    /// Explicit source envelope or generated From/To/Cc/Bcc envelope.
    pub envelope: Value,
    /// Parsed/deduplicated SMTP envelope, retaining explicit extension fields.
    pub smtp_envelope: Value,
    /// Selected/generated source-shaped Message-ID.
    pub message_id: String,
}

/// Parse source address strings/objects/lists through the native mailbox provider.
///
/// # Errors
/// Address syntax not representable by lettre requires a native provider binding.
pub fn addresses(value: &Value) -> EmailResult<Vec<Mailbox>> {
    let mut output = Vec::new();
    match value {
        Value::Array(values) => {
            for value in values {
                output.extend(addresses(value)?);
            }
        }
        Value::String(value) if !value.is_empty() => output.extend(value.parse::<Mailboxes>()?),
        Value::Object(value) => {
            if value.contains_key("group") {
                return Err(
                    io::Error::other("native provider required for group address syntax").into(),
                );
            }
            let address = value.get("address").and_then(Value::as_str).unwrap_or("");
            if !address.is_empty() {
                output.push(Mailbox::new(
                    value
                        .get("name")
                        .and_then(Value::as_str)
                        .filter(|v| !v.is_empty())
                        .map(str::to_owned),
                    address.parse()?,
                ));
            }
        }
        _ => {}
    }
    Ok(output)
}

/// Select metadata before content reads so stream-result errors can remain deferred.
///
/// Explicit envelopes retain their original fields alongside a parsed SMTP projection.
/// Missing Message-ID values use OS randomness; this function does not consume content.
///
/// # Errors
/// Returns native mailbox parsing or OS entropy errors without inventing a provider result.
pub fn metadata(fields: &Map<String, Value>) -> EmailResult<MessageMetadata> {
    let entries = message_headers(fields);
    let mut from = None;
    let mut recipients = Vec::new();
    // Header traversal preserves custom-header order and the source From fallback rule.
    for (key, value) in &entries {
        let key = key.to_ascii_lowercase();
        if key == "from" || (from.is_none() && matches!(key.as_str(), "sender" | "reply-to")) {
            if let Some(address) = addresses(header_content(value))?.first() {
                from = Some(address.email.to_string());
            }
        } else if matches!(key.as_str(), "to" | "cc" | "bcc") {
            for address in addresses(header_content(value))? {
                let address = address.email.to_string();
                if !recipients.contains(&address) {
                    recipients.push(address);
                }
            }
        }
    }
    let generated =
        json!({"from":from.as_ref().map(|v| json!(v)).unwrap_or(json!(false)),"to":recipients});
    let envelope = fields
        .get("envelope")
        .filter(|v| truthy(v))
        .cloned()
        .unwrap_or_else(|| generated.clone());
    let mut smtp_envelope = generated;
    if let Some(explicit) = fields.get("envelope").filter(|v| truthy(v)) {
        let from = explicit
            .get("from")
            .map(addresses)
            .transpose()?
            .unwrap_or_default()
            .first()
            .map(|v| v.email.to_string());
        let mut to = Vec::new();
        for key in ["to", "cc", "bcc"] {
            for address in explicit
                .get(key)
                .map(addresses)
                .transpose()?
                .unwrap_or_default()
            {
                let address = address.email.to_string();
                if !to.contains(&address) {
                    to.push(address);
                }
            }
        }
        smtp_envelope = json!({"from":from.map(|v| json!(v)).unwrap_or(json!(false)),"to":to});
        if let Some(object) = explicit.as_object() {
            for (key, value) in object {
                if !matches!(key.as_str(), "from" | "to" | "cc" | "bcc") {
                    smtp_envelope[key] = value.clone();
                }
            }
        }
    }
    let header_id = entries
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("Message-ID"))
        .map(|(_, v)| header_content(v));
    let message_id = if let Some(id) = header_id.filter(|v| truthy(v)) {
        // Source messageId() returns the original header value; encoding is later.
        string(id)
    } else {
        let mut bytes = [0u8; 16];
        SystemRandom::new()
            .fill(&mut bytes)
            .map_err(|_| io::Error::other("email Message-ID entropy failed"))?;
        let hex = bytes.iter().map(|v| format!("{v:02x}")).collect::<String>();
        let domain = smtp_envelope
            .get("from")
            .and_then(Value::as_str)
            .and_then(|v| v.rsplit('@').next())
            .unwrap_or("localhost");
        format!(
            "<{}-{}-{}-{}-{}@{domain}>",
            &hex[..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..]
        )
    };
    Ok(MessageMetadata {
        envelope,
        smtp_envelope,
        message_id,
    })
}
