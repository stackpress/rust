//! MIME tree roles, content IO and root header projection for the native transports.

use super::{
    headers::{append_header, header_content, message_headers, reference_header},
    metadata::{MessageMetadata, addresses},
    node::{attachment_defaults, detect_mime, node_attributes, preprocess_data_uris},
    part::{MimeBody, part, preferred_encoding},
};
use crate::{
    Content, EmailResult, NativeSendMailOptions,
    content::{ContentResolver, resolve},
    options::{string, truthy},
};
use lettre::message::{
    MultiPart, SinglePart,
    header::{self, Headers},
};
use serde_json::Value;
use serde_json::json;
use std::sync::Arc;

/// MIME bytes for native SMTP/stream/sendmail/SES; raw overrides generated framing.
pub(crate) async fn mime(
    mut mail: NativeSendMailOptions,
    meta: &MessageMetadata,
    resolver: &Arc<dyn ContentResolver>,
    keep_bcc: bool,
) -> EmailResult<Vec<u8>> {
    preprocess_data_uris(&mut mail.fields, true)?;
    if let Some(raw) = mail
        .fields
        .get("raw")
        .cloned()
        .filter(truthy)
        .or_else(|| mail.has_content("raw").then(|| json!("")))
    {
        return resolve(&mut mail, "raw", &raw, resolver, true).await;
    }
    let mut headers = Vec::new();
    // Source custom headers come first; truthy standard fields replace matching entries.
    let entries = message_headers(&mail.fields);
    for (name, value) in &entries {
        if name.eq_ignore_ascii_case("bcc") && !keep_bcc {
            continue;
        }
        if name.eq_ignore_ascii_case("message-id") {
            continue;
        }
        let prepared = value.get("prepared").is_some_and(truthy);
        let value = header_content(value);
        let encoded = match name.to_ascii_lowercase().as_str() {
            "from" | "sender" | "to" | "cc" | "bcc" | "reply-to" => addresses(value)?
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            "references" | "in-reply-to" => reference_header(value),
            _ => string(value),
        };
        append_header(&mut headers, name, encoded, prepared)?;
    }
    append_header(
        &mut headers,
        "Message-ID",
        format!("<{}>", meta.message_id.trim().trim_matches(['<', '>'])),
        true,
    )?;
    if !entries
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case("date"))
    {
        let mut date_headers = Headers::new();
        date_headers.set(header::Date::now());
        headers.extend(date_headers.to_string().bytes());
    }
    if !entries
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case("mime-version"))
    {
        append_header(&mut headers, "MIME-Version", "1.0".into(), true)?;
    }
    let mut alternatives = Vec::new();
    for (field, kind) in [
        ("text", "text/plain; charset=utf-8"),
        ("html", "text/html; charset=utf-8"),
    ] {
        if let Some(value) = mail
            .fields
            .get(field)
            .cloned()
            .filter(truthy)
            .or_else(|| mail.has_content(field).then(|| json!("")))
        {
            let encoding = node_encoding(
                &mail,
                source_stream(&mail, field, &value),
                mail.fields.get("encoding").and_then(Value::as_str),
            );
            let content = resolve(&mut mail, field, &value, resolver, true).await?;
            alternatives.push(part(
                content,
                kind,
                None,
                encoding.as_deref(),
                None,
                None,
                mail.fields.get("textEncoding").and_then(Value::as_str),
            )?);
        }
    }
    if let Some(values) = mail
        .fields
        .get("alternatives")
        .and_then(Value::as_array)
        .cloned()
    {
        for (index, value) in values.into_iter().enumerate() {
            let streamed = source_stream(&mail, &format!("alternatives.{index}"), &value);
            let bytes = resolve(
                &mut mail,
                &format!("alternatives.{index}"),
                &value,
                resolver,
                true,
            )
            .await?;
            // Source infers an alternative's MIME type from its filename/path/href, then txt.
            let kind = value
                .get("contentType")
                .filter(|v| truthy(v))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    detect_mime(
                        value
                            .get("filename")
                            .filter(|v| truthy(v))
                            .or_else(|| value.get("path").filter(|v| truthy(v)))
                            .or_else(|| value.get("href").filter(|v| truthy(v)))
                            .and_then(Value::as_str)
                            .unwrap_or("txt"),
                    )
                });
            let explicit = value.get("contentTransferEncoding").and_then(Value::as_str);
            let encoding = if kind.to_ascii_lowercase().starts_with("text/") {
                node_encoding(
                    &mail,
                    streamed,
                    explicit.or_else(|| mail.fields.get("encoding").and_then(Value::as_str)),
                )
            } else {
                explicit.map(str::to_owned)
            };
            alternatives.push(part(
                bytes,
                &kind,
                value.get("filename").and_then(Value::as_str),
                encoding.as_deref(),
                None,
                None,
                mail.fields.get("textEncoding").and_then(Value::as_str),
            )?);
        }
    }
    let html_index = alternatives.iter().rposition(|part| {
        part.headers()
            .get_raw("Content-Type")
            .is_some_and(|kind| kind.starts_with("text/html"))
    });
    let mut related = Vec::new();
    let mut attached = Vec::new();
    if let Some(values) = mail
        .fields
        .get("attachments")
        .and_then(Value::as_array)
        .cloned()
    {
        for (index, value) in values.into_iter().enumerate() {
            let mut attrs = node_attributes(&value);
            attachment_defaults(&mut attrs, &value, index);
            let streamed = source_stream(&mail, &format!("attachments.{index}"), &value);
            let bytes = resolve(
                &mut mail,
                &format!("attachments.{index}"),
                &value,
                resolver,
                true,
            )
            .await?;
            let kind = attrs
                .get("contentType")
                .and_then(Value::as_str)
                .unwrap_or("application/octet-stream");
            let is_message = kind.to_ascii_lowercase().starts_with("message/");
            let filename = if value.get("filename") == Some(&json!(false))
                || (is_message && !value.get("filename").is_some_and(truthy))
            {
                None
            } else {
                attrs.get("filename").and_then(Value::as_str)
            };
            let disposition = value
                .get("contentDisposition")
                .and_then(Value::as_str)
                .filter(|v| !v.is_empty())
                .unwrap_or(
                    if is_message
                        || (kind.to_ascii_lowercase().starts_with("image/")
                            && value.get("cid").is_some_and(truthy))
                    {
                        "inline"
                    } else {
                        "attachment"
                    },
                );
            let encoding = if value.get("contentTransferEncoding").is_some_and(truthy) {
                value.get("contentTransferEncoding").and_then(Value::as_str)
            } else if value.get("contentTransferEncoding").is_some() {
                None
            } else {
                Some(if is_message { "8bit" } else { "base64" })
            };
            let encoding = if kind.to_ascii_lowercase().starts_with("text/") {
                node_encoding(&mail, streamed, encoding)
            } else {
                encoding.map(str::to_owned)
            };
            let node = part(
                bytes,
                kind,
                filename,
                encoding.as_deref(),
                value.get("cid").and_then(Value::as_str),
                Some(disposition),
                mail.fields.get("textEncoding").and_then(Value::as_str),
            )?;
            if value.get("cid").is_some_and(truthy) && html_index.is_some() {
                related.push(node);
            } else {
                attached.push(node);
            }
        }
    }
    // Source places related images around the HTML alternative, then alternatives inside mixed.
    let mut nodes = alternatives
        .into_iter()
        .map(MimeBody::Single)
        .collect::<Vec<_>>();
    if let Some(index) = html_index.filter(|_| !related.is_empty()) {
        let html = nodes.remove(index);
        let mut body = html.append_to(MultiPart::related().build());
        for node in related {
            body = body.singlepart(node);
        }
        nodes.insert(index, MimeBody::Multi(body));
    }
    let body = match nodes.len() {
        0 => None,
        1 => nodes.pop(),
        _ => Some(MimeBody::Multi(
            nodes
                .into_iter()
                .fold(MultiPart::alternative().build(), |tree, node| {
                    node.append_to(tree)
                }),
        )),
    };
    let body = if attached.is_empty() {
        body.unwrap_or_else(|| MimeBody::Single(SinglePart::plain(String::new())))
    } else if body.is_none() && attached.len() == 1 {
        MimeBody::Single(attached.remove(0))
    } else {
        let mut tree = MultiPart::mixed().build();
        if let Some(body) = body {
            tree = body.append_to(tree);
        }
        for node in attached {
            tree = tree.singlepart(node);
        }
        MimeBody::Multi(tree)
    };
    headers.extend(body.formatted());
    Ok(headers)
}

/// Source file/URL/native readers are streams even though the native compiler later buffers them.
fn source_stream(mail: &NativeSendMailOptions, slot: &str, value: &Value) -> bool {
    match mail
        .content
        .get(slot)
        .or_else(|| mail.content.get(&format!("{slot}.content")))
    {
        Some(Content::Reader(_)) => true,
        Some(Content::Bytes(_)) => false,
        None => value.get("path").is_some_and(truthy) || value.get("href").is_some_and(truthy),
    }
}

/// Source cannot inspect stream text: explicit B/Q wins; otherwise stream preference defaults QP.
fn node_encoding(
    mail: &NativeSendMailOptions,
    streamed: bool,
    explicit: Option<&str>,
) -> Option<String> {
    if streamed
        && !explicit.is_some_and(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "base64" | "quoted-printable"
            )
        })
    {
        Some(
            if mail
                .fields
                .get("textEncoding")
                .and_then(Value::as_str)
                .is_some_and(|value| preferred_encoding(Some(value)) == Some('B'))
            {
                "base64"
            } else {
                "quoted-printable"
            }
            .into(),
        )
    } else {
        explicit.map(str::to_owned)
    }
}
