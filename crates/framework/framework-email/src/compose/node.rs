//! Source attachment defaults, versioned MIME catalog and data-URI preprocessing.

use crate::{
    EmailResult,
    options::{flag, truthy},
};
use serde_json::{Map, Value, json};
use std::sync::OnceLock;

/// Source MailComposer expands data-URI objects before JSON normalization or MIME assembly.
pub(super) fn preprocess_data_uris(fields: &mut Map<String, Value>, mime: bool) -> EmailResult<()> {
    for key in ["text", "html", "watchHtml", "amp", "icalEvent"] {
        if let Some(value) = fields.get_mut(key) {
            preprocess_data_uri(value, mime)?;
        }
    }
    for key in ["attachments", "alternatives"] {
        if let Some(values) = fields.get_mut(key).and_then(Value::as_array_mut) {
            for value in values {
                preprocess_data_uri(value, mime)?;
            }
        }
    }
    Ok(())
}

/// Preserve the original data-URI mutation and its existing 50 MB MIME-only fallback.
fn preprocess_data_uri(value: &mut Value, mime: bool) -> EmailResult<()> {
    let Some(object) = value.as_object_mut() else {
        return Ok(());
    };
    let Some(uri) = object
        .get("path")
        .filter(|v| truthy(v))
        .or_else(|| object.get("href"))
        .and_then(Value::as_str)
        .filter(|v| v.starts_with("data:"))
        .map(str::to_owned)
    else {
        return Ok(());
    };
    if uri.encode_utf16().count() > 52_428_800 {
        // The source oversize branch returns a replacement object only to the MIME composer.
        if mime {
            object.insert("path".into(), json!(false));
            object.insert("href".into(), json!(false));
            object.insert("content".into(), crate::message::buffer_json(&[]));
            if !flag(object, "contentType") {
                object.insert(
                    "contentType".into(),
                    json!(
                        uri[5..]
                            .split([';', ','])
                            .next()
                            .filter(|v| v.contains('/'))
                            .unwrap_or("application/octet-stream")
                    ),
                );
            }
        }
        return Ok(());
    }
    if let Some((kind, bytes)) = crate::content::parse_data_uri(&uri)? {
        object.insert("content".into(), crate::message::buffer_json(&bytes));
        if !flag(object, "contentType") {
            object.insert("contentType".into(), json!(kind));
        }
        for key in ["path", "href"] {
            if object.contains_key(key) {
                object.insert(key.into(), json!(false));
            }
        }
    }
    Ok(())
}

/// Retain attachment attributes while replacing the source content/path/href/raw carrier.
pub(super) fn node_attributes(value: &Value) -> Map<String, Value> {
    value
        .as_object()
        .map(|v| {
            v.iter()
                .filter(|(key, _)| !matches!(key.as_str(), "content" | "path" | "href" | "raw"))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// Source JSON normalization derives a filename even when filename=false was supplied.
pub(super) fn attachment_defaults(node: &mut Map<String, Value>, value: &Value, index: usize) {
    if !flag(node, "filename") {
        let path = value
            .get("path")
            .or_else(|| value.get("href"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let mut filename = path
            .rsplit('/')
            .next()
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("")
            .to_owned();
        if filename.is_empty() {
            filename = format!("attachment-{}", index + 1);
        }
        if !filename.contains('.') {
            filename.push('.');
            filename.push_str(&detect_extension(
                node.get("contentType")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            ));
        }
        node.insert("filename".into(), json!(filename));
    }
    if !flag(node, "contentType") {
        let kind = detect_mime(
            node.get("filename")
                .and_then(Value::as_str)
                .unwrap_or("bin"),
        );
        node.insert("contentType".into(), json!(kind));
    }
}

/// Source MIME catalog is versioned data, parsed once for the application process.
fn catalog() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../../data/nodemailer-mime.json"))
            .expect("versioned source MIME catalog")
    })
}

/// Match source filename/extension lookup, including query suffix removal.
pub(super) fn detect_mime(filename: &str) -> String {
    let name = filename.rsplit('/').next().unwrap_or(filename);
    let extension = name
        .rsplit('.')
        .next()
        .unwrap_or(name)
        .split('?')
        .next()
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let value = &catalog()["extensions"][&extension];
    value
        .as_str()
        .or_else(|| {
            value
                .as_array()
                .and_then(|v| v.first())
                .and_then(Value::as_str)
        })
        .unwrap_or("application/octet-stream")
        .into()
}

/// Match source content-type→filename-extension fallback.
fn detect_extension(kind: &str) -> String {
    let kind = kind.trim().to_lowercase();
    let value = &catalog()["types"][&kind];
    value
        .as_str()
        .or_else(|| {
            value
                .as_array()
                .and_then(|v| v.first())
                .and_then(Value::as_str)
        })
        .unwrap_or(if kind.starts_with("text/") {
            "txt"
        } else {
            "bin"
        })
        .into()
}
