//! Explicit library-capability routing, before any content reader or transport IO.

use crate::{
    NativeSendMailOptions,
    options::{TransportKind, TransportOptions, flag, string, truthy},
};
use serde_json::Value;
use std::io;

/// Source settings outside lettre's supported connection surface require the native binding.
pub(super) fn unsupported_config(options: &TransportOptions) -> Option<String> {
    for key in [
        "logger",
        "debug",
        "transactionLog",
        "dkim",
        "attachDataUrls",
    ] {
        if options.fields.get(key).is_some_and(truthy) {
            return Some(format!("transport option {key}"));
        }
    }
    if !matches!(options.kind, TransportKind::Smtp | TransportKind::Pool) {
        return None;
    }
    // Pool message recycling is source behaviour even when maxMessages uses its default.
    if options.kind == TransportKind::Pool {
        return Some("SMTP pool/maxMessages native binding".into());
    }
    for key in [
        "lmtp",
        "secured",
        "socket",
        "connection",
        "getSocket",
        "customAuth",
        "dnsTimeout",
        "greetingTimeout",
        "socketTimeout",
        "opportunisticTLS",
        "proxy",
        "logger",
        "debug",
        "transactionLog",
        "forceAuth",
        "dnsTtl",
        "dnsCache",
        "family",
        "ipv4Only",
        "ipv6Only",
    ] {
        if options.fields.get(key).is_some_and(truthy) {
            return Some(format!("SMTP option {key}"));
        }
    }
    if let Some(auth) = options.fields.get("auth").and_then(Value::as_object) {
        for key in [
            "refreshToken",
            "clientId",
            "clientSecret",
            "serviceClient",
            "privateKey",
            "provisionCallback",
            "expires",
            "options",
        ] {
            if auth.get(key).is_some_and(truthy) {
                return Some(format!("SMTP auth.{key}"));
            }
        }
        if auth
            .get("type")
            .map(string)
            .is_some_and(|v| !matches!(v.to_uppercase().as_str(), "LOGIN" | "OAUTH2" | ""))
        {
            return Some("SMTP custom authentication type".into());
        }
    }
    if let Some(address) = options.fields.get("localAddress").filter(|v| truthy(v))
        && string(address).parse::<std::net::IpAddr>().is_err()
    {
        return Some("SMTP localAddress representation".into());
    }
    let auth = options.fields.get("auth").and_then(Value::as_object);
    let method = auth
        .and_then(|v| v.get("method"))
        .filter(|v| truthy(v))
        .or_else(|| options.fields.get("authMethod").filter(|v| truthy(v)));
    if method.is_some_and(|v| {
        !matches!(
            string(v).trim().to_uppercase().as_str(),
            "" | "PLAIN" | "LOGIN" | "XOAUTH2"
        )
    }) {
        return Some("SMTP custom auth method".into());
    }
    if auth.is_some_and(|v| {
        v.get("type")
            .map(string)
            .is_some_and(|v| v.eq_ignore_ascii_case("oauth2"))
            && flag(v, "user")
            && !flag(v, "accessToken")
    }) {
        return Some("SMTP OAuth2 token provisioning".into());
    }
    if let Some(value) = options
        .fields
        .get("connectionTimeout")
        .and_then(crate::options::number)
        .filter(|v| *v != 0.0)
        && std::time::Duration::try_from_secs_f64(value / 1000.0).is_err()
    {
        return Some("SMTP timeout coercion".into());
    }
    if let Some(tls) = options.fields.get("tls").and_then(Value::as_object) {
        if tls
            .get("servername")
            .is_some_and(|v| truthy(v) && !v.is_string())
        {
            return Some("SMTP tls.servername native representation".into());
        }
        if tls
            .get("minVersion")
            .filter(|v| truthy(v))
            .is_some_and(|v| !matches!(string(v).as_str(), "TLSv1.2" | "TLSv1.3"))
        {
            return Some("SMTP tls.minVersion".into());
        }
        let has_cert = flag(tls, "cert");
        let has_key = flag(tls, "key");
        if has_cert != has_key {
            return Some("SMTP incomplete TLS identity".into());
        }
        if ["cert", "key", "ca"]
            .iter()
            .filter_map(|key| tls.get(*key))
            .any(|v| !pem_supported(v))
        {
            return Some("SMTP TLS PEM representation".into());
        }
        if tls
            .get("cert")
            .is_some_and(|value| certificate_count(value) > 1)
        {
            return Some("SMTP TLS certificate chain identity".into());
        }
        for key in tls.keys() {
            if !matches!(
                key.as_str(),
                "servername" | "rejectUnauthorized" | "ca" | "minVersion" | "cert" | "key"
            ) {
                return Some(format!("SMTP tls.{key}"));
            }
        }
    }
    let smtp = options.smtp();
    if !smtp.port.is_finite() || smtp.port.fract() != 0.0 || !(1.0..=65535.0).contains(&smtp.port) {
        return Some("SMTP port representation".into());
    }
    None
}

/// Select unsupported message features before IO, preserving full options for the binding.
pub(super) fn unsupported_message(
    mail: &NativeSendMailOptions,
    kind: TransportKind,
) -> Option<String> {
    // Every owned slot must have a source content location; aliases cannot conflict.
    for slot in mail.content.keys() {
        let node = slot.strip_suffix(".content").unwrap_or(slot);
        if slot.ends_with(".content") && mail.content.contains_key(node) {
            return Some(format!("duplicate native content slot {node}"));
        }
        let body = matches!(
            node,
            "text" | "html" | "raw" | "watchHtml" | "amp" | "icalEvent"
        );
        let indexed = node.split_once('.').is_some_and(|(group, index)| {
            matches!(group, "attachments" | "alternatives")
                && index.parse::<usize>().ok().is_some_and(|i| {
                    mail.fields
                        .get(group)
                        .and_then(Value::as_array)
                        .is_some_and(|v| i < v.len())
                })
        });
        if !body && !indexed {
            return Some(format!("native content slot {slot}"));
        }
        if kind == TransportKind::Json && node == "raw" {
            return Some("JSON raw native reader representation".into());
        }
    }
    // The default compile plugin runs even for JSON output; it cannot be skipped.
    if mail.fields.get("attachDataUrls").is_some_and(truthy) {
        return Some("message option attachDataUrls".into());
    }
    if kind == TransportKind::Json {
        return None;
    }
    // Lettre controls header word encoding; an explicit source preference cannot disappear.
    if mail.fields.get("textEncoding").is_some_and(truthy)
        && [
            "from",
            "sender",
            "to",
            "cc",
            "bcc",
            "replyTo",
            "subject",
            "inReplyTo",
            "references",
            "xMailer",
            "headers",
        ]
        .iter()
        .filter_map(|key| mail.fields.get(*key))
        .any(non_ascii_header)
    {
        return Some("message preferred encoding for non-ASCII headers".into());
    }
    for key in ["watchHtml", "amp", "icalEvent"] {
        if mail.has_content(key) {
            return Some(format!("message option {key}"));
        }
    }
    for key in [
        "dkim",
        "normalizeHeaderKey",
        "attachDataUrls",
        "list",
        "watchHtml",
        "amp",
        "icalEvent",
        "baseBoundary",
        "boundaryPrefix",
        "dsn",
        "requireTLSExtensionEnabled",
        "newline",
        "auth",
    ] {
        if mail.fields.get(key).is_some_and(truthy) {
            return Some(format!("message option {key}"));
        }
    }
    if let Some(envelope) = mail.fields.get("envelope").and_then(Value::as_object) {
        for key in ["dsn", "requireTLSExtensionEnabled", "size"] {
            if envelope.get(key).is_some_and(truthy) {
                return Some(format!("envelope option {key}"));
            }
        }
    }
    // Body-node metadata beyond content/encoding is a source MIME compiler capability.
    for key in ["text", "html"] {
        if let Some(value) = mail.fields.get(key).and_then(Value::as_object)
            && value.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "content" | "encoding" | "path" | "href" | "httpHeaders" | "type" | "data"
                )
            })
        {
            return Some(format!("{key} MIME node options"));
        }
    }
    if let Some(headers) = mail.fields.get("headers") {
        let names = match headers {
            Value::Object(headers) => headers.keys().map(String::as_str).collect::<Vec<_>>(),
            Value::Array(headers) => headers
                .iter()
                .filter_map(|v| v.get("key").and_then(Value::as_str))
                .collect(),
            _ => vec![],
        };
        if names.iter().any(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "content-type" | "content-transfer-encoding" | "content-disposition"
            )
        }) {
            return Some("custom root MIME headers".into());
        }
    }
    for group in ["attachments", "alternatives"] {
        if let Some(values) = mail.fields.get(group).and_then(Value::as_array) {
            for value in values {
                if group == "alternatives" && value.get("filename").is_some_and(truthy) {
                    return Some("alternative filename MIME parameter".into());
                }
                if value.get("raw").is_some_and(truthy) || value.get("headers").is_some_and(truthy)
                {
                    return Some(format!("{group} raw/custom MIME headers"));
                }
            }
        }
    }
    None
}

/// A library-capability error names the missing native binding instead of dropping an option.
pub(super) fn unsupported(reason: &str) -> crate::EmailError {
    io::Error::other(format!("native email provider required for {reason}")).into()
}

/// PEM representations accepted by the source/library intersection, without discarding resources.
fn pem_supported(value: &Value) -> bool {
    match value {
        Value::String(_) => true,
        Value::Array(values) => values.iter().all(pem_supported),
        Value::Object(value) => value.get("type").and_then(Value::as_str) == Some("Buffer"),
        _ => false,
    }
}

/// The pinned Rustls identity API accepts one DER certificate; full chains require a binding.
fn certificate_count(value: &Value) -> usize {
    match value {
        Value::String(value) => value.matches("-----BEGIN CERTIFICATE-----").count(),
        Value::Array(values) => values.iter().map(certificate_count).sum(),
        Value::Object(value) if value.get("type").and_then(Value::as_str) == Some("Buffer") => {
            value
                .get("data")
                .and_then(Value::as_array)
                .map(|values| {
                    String::from_utf8_lossy(
                        &values
                            .iter()
                            .map(|v| v.as_u64().unwrap_or(0) as u8)
                            .collect::<Vec<_>>(),
                    )
                    .matches("-----BEGIN CERTIFICATE-----")
                    .count()
                })
                .unwrap_or_default()
        }
        _ => 0,
    }
}

/// Check only rendered header input; ordinary MIME content does not select this binding.
fn non_ascii_header(value: &Value) -> bool {
    match value {
        Value::String(value) => !value.is_ascii(),
        Value::Array(values) => values.iter().any(non_ascii_header),
        Value::Object(values) => values.values().any(non_ascii_header),
        _ => false,
    }
}
