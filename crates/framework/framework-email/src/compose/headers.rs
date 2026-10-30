//! Source custom/standard header precedence and native framing.

use crate::{
    EmailResult,
    options::{string, truthy},
};
use lettre::message::header::{HeaderName, HeaderValue, Headers};
use serde_json::{Map, Value, json};

/// Delegate header framing to lettre; prepared values retain the source preencoded capability.
pub(super) fn append_header(
    output: &mut Vec<u8>,
    name: &str,
    value: String,
    prepared: bool,
) -> EmailResult<()> {
    let name = HeaderName::new_from_ascii(name.to_owned())?;
    let value = if prepared {
        HeaderValue::dangerous_new_pre_encoded(name, value.clone(), value)
    } else {
        HeaderValue::new(name, value)
    };
    let mut headers = Headers::new();
    headers.insert_raw(value);
    output.extend(headers.to_string().bytes());
    Ok(())
}

/// Source custom headers retain duplicates; standard fields replace all matching entries.
pub(super) fn message_headers(fields: &Map<String, Value>) -> Vec<(String, Value)> {
    let mut entries = Vec::new();
    match fields.get("headers") {
        Some(Value::Object(headers)) => {
            for (key, value) in headers {
                for value in value
                    .as_array()
                    .cloned()
                    .unwrap_or_else(|| vec![value.clone()])
                {
                    entries.push((key.clone(), value));
                }
            }
        }
        Some(Value::Array(headers)) => {
            for item in headers {
                if let Some(key) = item.get("key").and_then(Value::as_str) {
                    entries.push((
                        key.into(),
                        item.get("value").cloned().unwrap_or(Value::Null),
                    ));
                }
            }
        }
        _ => {}
    }
    for (field, header) in [
        ("from", "From"),
        ("sender", "Sender"),
        ("to", "To"),
        ("cc", "Cc"),
        ("bcc", "Bcc"),
        ("replyTo", "Reply-To"),
        ("inReplyTo", "In-Reply-To"),
        ("references", "References"),
        ("subject", "Subject"),
        ("messageId", "Message-ID"),
        ("date", "Date"),
        ("xMailer", "X-Mailer"),
    ] {
        if let Some(value) = fields.get(field).filter(|v| truthy(v)) {
            set_header(&mut entries, header, value.clone());
        }
    }
    // MailMessage applies priority after compilation, overriding supplied priority headers.
    match fields
        .get("priority")
        .map(string)
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "high" => {
            for (key, value) in [
                ("X-Priority", "1 (Highest)"),
                ("X-MSMail-Priority", "High"),
                ("Importance", "High"),
            ] {
                set_header(&mut entries, key, json!(value));
            }
        }
        "low" => {
            for (key, value) in [
                ("X-Priority", "5 (Lowest)"),
                ("X-MSMail-Priority", "Low"),
                ("Importance", "Low"),
            ] {
                set_header(&mut entries, key, json!(value));
            }
        }
        _ => {}
    }
    entries
}

/// Retain the first header position while replacing all duplicates, as MimeNode.setHeader.
fn set_header(entries: &mut Vec<(String, Value)>, key: &str, value: Value) {
    if let Some(index) = entries
        .iter()
        .position(|(name, _)| name.eq_ignore_ascii_case(key))
    {
        entries[index] = (key.into(), value);
        let mut position = 0;
        entries.retain(|(name, _)| {
            let keep = position == index || !name.eq_ignore_ascii_case(key);
            position += 1;
            keep
        });
    } else {
        entries.push((key.into(), value));
    }
}

/// Prepared custom headers keep their separate encoding flag and payload.
pub(super) fn header_content(value: &Value) -> &Value {
    value.get("value").unwrap_or(value)
}

/// Source wraps individual reference identifiers and joins arrays with spaces.
pub(super) fn reference_header(value: &Value) -> String {
    value
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![value.clone()])
        .iter()
        .flat_map(|v| {
            string(v)
                .split_whitespace()
                .map(|v| format!("<{}>", v.trim_matches(['<', '>'])))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
