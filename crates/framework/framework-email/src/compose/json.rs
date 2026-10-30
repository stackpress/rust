//! Source JSON mutation order, addresses and byte/string normalization.

use super::{
    headers::{header_content, message_headers},
    metadata::{MessageMetadata, addresses},
    node::{attachment_defaults, node_attributes, preprocess_data_uris},
};
use crate::{
    EmailResult, NativeSendMailOptions,
    content::{ContentResolver, resolve},
    options::{flag, truthy},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Map, Value, json};
use std::sync::Arc;

/// Normalize JSON transport fields, preserving unknown options and source mutation order.
pub(crate) async fn normalize_json(
    mut mail: NativeSendMailOptions,
    meta: &MessageMetadata,
    resolver: &Arc<dyn ContentResolver>,
) -> EmailResult<Value> {
    preprocess_data_uris(&mut mail.fields, false)?;
    // MailMessage adds headers even when no headers were supplied.
    if !flag(&mail.fields, "headers") {
        mail.fields.insert("headers".into(), json!({}));
    }
    let headers = message_headers(&mail.fields);
    for key in ["from", "to", "cc", "bcc", "sender", "replyTo"] {
        let header = if key == "replyTo" { "reply-to" } else { key };
        let value = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(header))
            .map(|(_, v)| header_content(v));
        if let Some(value) = value {
            let values = normalized_addresses(value)?;
            let normalized = if values.is_empty() {
                Value::Null
            } else if matches!(key, "from" | "sender") {
                values[0].clone()
            } else {
                json!(values)
            };
            mail.fields.insert(key.into(), normalized);
        } else if mail.fields.contains_key(key) {
            mail.fields.insert(key.into(), Value::Null);
        }
    }
    for key in ["html", "text", "watchHtml", "amp", "icalEvent"] {
        if let Some(value) = mail
            .fields
            .get(key)
            .cloned()
            .filter(truthy)
            .or_else(|| mail.has_content(key).then(|| json!("")))
        {
            let bytes = resolve(&mut mail, key, &value, resolver, false).await?;
            let normalized = if key == "icalEvent" {
                let mut node = Map::new();
                node.insert("content".into(), json!(STANDARD.encode(bytes)));
                node.extend(node_attributes(&value));
                node.insert("encoding".into(), json!("base64"));
                Value::Object(node)
            } else {
                json!(String::from_utf8_lossy(&bytes))
            };
            mail.fields.insert(key.into(), normalized);
        }
    }
    for group in ["alternatives", "attachments"] {
        if let Some(values) = mail.fields.get(group).and_then(Value::as_array).cloned() {
            let mut normalized = Vec::new();
            for (index, value) in values.into_iter().enumerate() {
                let mut attributes = node_attributes(&value);
                if group == "attachments" {
                    attachment_defaults(&mut attributes, &value, index);
                }
                let native = mail.has_content(&format!("{group}.{index}"));
                let bytes = resolve(
                    &mut mail,
                    &format!("{group}.{index}"),
                    &value,
                    resolver,
                    false,
                )
                .await?;
                // Source retains string content; Buffer/file/URL/native content becomes base64.
                let literal = value
                    .as_object()
                    .and_then(|v| v.get("content"))
                    .and_then(Value::as_str);
                let encoding = value
                    .get("encoding")
                    .and_then(Value::as_str)
                    .unwrap_or("utf8");
                let mut node = Map::new();
                if let Some(text) = literal.filter(|_| {
                    !native && matches!(encoding, "utf8" | "utf-8" | "ascii" | "us-ascii")
                }) {
                    node.insert("content".into(), json!(text));
                    node.extend(attributes);
                } else {
                    node.insert("content".into(), json!(STANDARD.encode(bytes)));
                    node.extend(attributes);
                    node.insert("encoding".into(), json!("base64"));
                }
                normalized.push(Value::Object(node));
            }
            mail.fields.insert(group.into(), json!(normalized));
        }
    }
    // The source removes envelope and normalizedHeaders from the normalized JSON message.
    mail.fields.shift_remove("envelope");
    mail.fields.shift_remove("normalizedHeaders");
    mail.fields
        .insert("messageId".into(), json!(meta.message_id));
    Ok(Value::Object(mail.fields))
}

/// Source Address objects retain their original field order and extension fields.
fn normalized_addresses(value: &Value) -> EmailResult<Vec<Value>> {
    match value {
        Value::Array(values) => {
            let mut result = Vec::new();
            for value in values { result.extend(normalized_addresses(value)?); }
            Ok(result)
        }
        Value::Object(object) if flag(object,"address") => {
            let mut object = object.clone();
            if let Some(parsed) = addresses(value)?.first() { object.insert("address".into(),json!(parsed.email.to_string())); }
            if !flag(&object,"name") { object.insert("name".into(),json!("")); }
            Ok(vec![Value::Object(object)])
        }
        _ => Ok(addresses(value)?.iter().map(|v|json!({"address":v.email.to_string(),"name":v.name.clone().unwrap_or_default()})).collect()),
    }
}
