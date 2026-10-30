//! Source transport selection and constructor defaults, independent of network IO.

use crate::{EmailConfig, EmailResult};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::io;

/// Original transport variants, selected in NodeMailer's createTransport order.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TransportKind {
    /// One SMTP connection per invocation.
    Smtp,
    /// SMTP transport with configured pool settings.
    Pool,
    /// Normalized JSON message, without delivery.
    Json,
    /// Compiled MIME buffer or readable stream, without delivery.
    Stream,
    /// Native sendmail process.
    Sendmail,
    /// Bound SESv2 client/command equivalent.
    Ses,
}

/// Resolved constructor options with every source/extension field retained.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TransportOptions {
    /// Selected source transport.
    pub kind: TransportKind,
    /// Source constructor option bag after URL/service merging and pool defaults.
    pub fields: Map<String, Value>,
}

/// SMTPConnection's effective defaults, kept separate from the constructor bag.
#[derive(Clone, Debug, PartialEq)]
pub struct SmtpOptions {
    /// Source hostname, default localhost.
    pub host: String,
    /// Source numeric port, default 465 for secure and 587 otherwise.
    pub port: f64,
    /// Implicit TLS; port 465 implies true only when secure is omitted.
    pub secure: bool,
    /// Already upgraded connection flag for native providers.
    pub secured: bool,
    /// Full source options for native bindings, including auth/TLS/socket extensions.
    pub fields: Map<String, Value>,
}

impl TransportOptions {
    /// Resolve the original URL-before-variant and SMTP service precedence.
    ///
    /// # Errors
    /// Reports malformed connection strings and the source legacy SES configuration error.
    pub fn resolve(config: &EmailConfig) -> EmailResult<Self> {
        // An object with url uses the parsed URL alone at createTransport selection.
        let mut fields = match config {
            EmailConfig::ConnectionString(url) => parse_connection_url(url)?,
            EmailConfig::Options(fields) => match fields.get("url").filter(|v| truthy(v)) {
                Some(Value::String(url)) => parse_connection_url(url)?,
                Some(_) => return Err(io::Error::other("email url must be a string").into()),
                None => fields.clone(),
            },
        };
        // This ordering is observable when several source transport flags are present.
        let kind = if flag(&fields, "pool") {
            TransportKind::Pool
        } else if flag(&fields, "sendmail") {
            TransportKind::Sendmail
        } else if flag(&fields, "streamTransport") {
            TransportKind::Stream
        } else if flag(&fields, "jsonTransport") {
            TransportKind::Json
        } else if flag(&fields, "SES") {
            if let Some(ses) = fields.get("SES").and_then(Value::as_object)
                && flag(ses, "ses")
                && flag(ses, "aws")
            {
                return Err(io::Error::other("Using legacy SES configuration, expecting @aws-sdk/client-sesv2, see https://nodemailer.com/transports/ses/").into());
            }
            TransportKind::Ses
        } else {
            TransportKind::Smtp
        };
        // SMTP service settings override ordinary fields, exactly as wellKnown(service).
        if matches!(kind, TransportKind::Smtp | TransportKind::Pool) {
            if let Some(service) = fields.get("service").and_then(Value::as_str)
                && let Some(settings) = service_options(service)
            {
                fields.extend(settings);
            }
            if kind == TransportKind::Pool {
                // Source || defaults preserve truthy values without imposing a numeric policy.
                for (key, default) in [("maxConnections", 5), ("maxMessages", 100)] {
                    if !flag(&fields, key) {
                        fields.insert(key.into(), json!(default));
                    }
                }
            }
        }
        Ok(Self { kind, fields })
    }

    /// Return source SMTPConnection defaults without changing the original option bag.
    pub fn smtp(&self) -> SmtpOptions {
        let secure = flag(&self.fields, "secure");
        let port = self
            .fields
            .get("port")
            .and_then(number)
            .filter(|v| *v != 0.0)
            .unwrap_or(if secure { 465.0 } else { 587.0 });
        // Explicit false/null secure does not receive the omitted-secure port-465 default.
        SmtpOptions {
            host: self
                .fields
                .get("host")
                .filter(|v| truthy(v))
                .map(string)
                .unwrap_or_else(|| "localhost".into()),
            port,
            secure: secure || (!self.fields.contains_key("secure") && port == 465.0),
            secured: flag(&self.fields, "secured"),
            fields: self.fields.clone(),
        }
    }

    /// Select source newline aliases; unrecognized names use Unix behaviour.
    pub fn windows_newlines(&self) -> bool {
        self.fields.get("newline").map(string).is_some_and(|v| {
            matches!(
                v.to_lowercase().as_str(),
                "win" | "windows" | "dos" | "\r\n"
            )
        })
    }

    /// Pool rate defaults use Number(value) || source-default.
    pub fn pool_rate(&self) -> (f64, f64) {
        let read = |key: &str, default| {
            self.fields
                .get(key)
                .and_then(number)
                .filter(|v| *v != 0.0)
                .unwrap_or(default)
        };
        (read("rateDelta", 1000.0), read("rateLimit", 0.0))
    }
}

/// JavaScript truthiness for the serializable portion of source option objects.
pub(crate) fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(v) => *v,
        Value::Number(v) => v.as_f64().is_some_and(|v| v != 0.0),
        Value::String(v) => !v.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// Read a source truthy field without conflating an empty object with absence.
pub(crate) fn flag(fields: &Map<String, Value>, name: &str) -> bool {
    fields.get(name).is_some_and(truthy)
}

/// Number conversion for source numeric fields/query parameters.
pub(crate) fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Null => Some(0.0),
        Value::Bool(v) => Some(f64::from(u8::from(*v))),
        Value::Number(v) => v.as_f64(),
        Value::String(v) if v.trim().is_empty() => Some(0.0),
        Value::String(v) => {
            let v = v.trim();
            if let Some(hex) = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
                u64::from_str_radix(hex, 16).ok().map(|v| v as f64)
            } else if let Some(binary) = v.strip_prefix("0b").or_else(|| v.strip_prefix("0B")) {
                u64::from_str_radix(binary, 2).ok().map(|v| v as f64)
            } else if let Some(octal) = v.strip_prefix("0o").or_else(|| v.strip_prefix("0O")) {
                u64::from_str_radix(octal, 8).ok().map(|v| v as f64)
            } else {
                v.parse::<f64>().ok().filter(|v| !v.is_nan())
            }
        }
        Value::Array(v) if v.is_empty() => Some(0.0),
        Value::Array(v) if v.len() == 1 => number(&Value::String(string(&v[0]))),
        _ => None,
    }
}

/// Source String conversion used only at the original option coercion sites.
pub(crate) fn string(value: &Value) -> String {
    match value {
        Value::String(v) => v.clone(),
        Value::Null => "null".into(),
        Value::Object(_) => "[object Object]".into(),
        Value::Array(v) => v
            .iter()
            .map(|v| {
                if v.is_null() {
                    String::new()
                } else {
                    string(v)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        _ => value.to_string(),
    }
}

/// Parse source-supported smtp/smtps/direct URLs, preserving first query values.
pub fn parse_connection_url(input: &str) -> EmailResult<Map<String, Value>> {
    let url = url::Url::parse(input)?;
    let mut fields = Map::new();
    match url.scheme() {
        "smtp" => {
            fields.insert("secure".into(), json!(false));
        }
        "smtps" => {
            fields.insert("secure".into(), json!(true));
        }
        "direct" => {
            fields.insert("direct".into(), json!(true));
        }
        _ => {}
    }
    // Preserve explicit ports even when URL treats them as a known scheme default.
    let explicit_port = input
        .split_once("://")
        .and_then(|(_, rest)| rest.split(['/', '?', '#']).next())
        .and_then(|authority| authority.rsplit('@').next())
        .and_then(|host| host.rsplit_once(':'))
        .and_then(|(_, port)| port.parse::<u16>().ok());
    if let Some(port) = explicit_port.or_else(|| url.port()).filter(|v| *v != 0) {
        fields.insert("port".into(), json!(port));
    }
    if let Some(host) = url.host_str() {
        fields.insert("host".into(), json!(host.trim_matches(['[', ']'])));
    }
    if !url.username().is_empty() || url.password().is_some() {
        // Legacy Node URL parsing decodes auth before splitting its first colon.
        let auth = decode_uri(&format!(
            "{}:{}",
            url.username(),
            url.password().unwrap_or("")
        ))?;
        let (user, pass) = auth.split_once(':').unwrap_or((&auth, ""));
        fields.insert("auth".into(), json!({"user":user, "pass":pass}));
    }
    // Node querystring groups duplicate keys into arrays before numeric conversion.
    let mut query: Map<String, Value> = Map::new();
    for (key, value) in url.query_pairs() {
        match query.entry(key.into_owned()) {
            serde_json::map::Entry::Vacant(entry) => {
                entry.insert(json!(value));
            }
            serde_json::map::Entry::Occupied(mut entry) => {
                if let Value::Array(values) = entry.get_mut() {
                    values.push(json!(value));
                } else {
                    let first = entry.get().clone();
                    entry.insert(json!([first, value]));
                }
            }
        }
    }
    for (key, mut value) in query {
        if number(&value).is_some_and(|value| !value.is_finite()) {
            return Err(io::Error::other(
                "native provider required for nonfinite URL option numbers",
            )
            .into());
        }
        // Source's `in` check includes Object.prototype properties for both option objects.
        let property = key.strip_prefix("tls.").unwrap_or(&key);
        if matches!(
            property,
            "__proto__"
                | "constructor"
                | "toString"
                | "valueOf"
                | "hasOwnProperty"
                | "isPrototypeOf"
                | "propertyIsEnumerable"
                | "toLocaleString"
                | "__defineGetter__"
                | "__defineSetter__"
                | "__lookupGetter__"
                | "__lookupSetter__"
        ) {
            continue;
        }

        if let Some(n) = number(&value).and_then(|number| {
            if number.fract() == 0.0 && number >= i64::MIN as f64 && number < i64::MAX as f64 {
                Some(serde_json::Number::from(number as i64))
            } else {
                serde_json::Number::from_f64(number)
            }
        }) {
            value = Value::Number(n);
        } else if value == "true" {
            value = json!(true);
        } else if value == "false" {
            value = json!(false);
        }
        if let Some(key) = key.strip_prefix("tls.") {
            let tls = fields.entry("tls").or_insert_with(|| json!({}));
            if let Some(tls) = tls.as_object_mut() {
                tls.entry(key).or_insert(value);
            }
        } else if !key.contains('.') {
            fields.entry(key).or_insert(value);
        }
    }
    Ok(fields)
}

/// Decode URL auth percent escapes without treating '+' as form-space.
pub(crate) fn decode_uri(input: &str) -> EmailResult<String> {
    let mut bytes = Vec::new();
    let mut remaining = input.as_bytes();
    while let Some((&byte, tail)) = remaining.split_first() {
        if byte == b'%' {
            let pair = remaining
                .get(1..3)
                .ok_or_else(|| io::Error::other("Malformed email URL auth escape"))?;
            bytes.push(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?);
            remaining = &remaining[3..];
        } else {
            bytes.push(byte);
            remaining = tail;
        }
    }
    Ok(String::from_utf8(bytes)?)
}

/// Resolve the upstream versioned service data, including aliases/domains/email keys.
fn service_options(service: &str) -> Option<Map<String, Value>> {
    let normalize = |v: &str| {
        v.chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
            .collect::<String>()
            .to_lowercase()
    };
    let key = normalize(service.rsplit('@').next()?);
    let catalog: Map<String, Value> =
        serde_json::from_str(include_str!("../data/nodemailer-services.json"))
            .expect("versioned source service JSON");
    for (name, settings) in catalog {
        let settings = settings.as_object()?;
        let matches = normalize(&name) == key
            || ["aliases", "domains"].iter().any(|field| {
                settings.get(*field).is_some_and(|values| match values {
                    Value::Array(values) => values
                        .iter()
                        .any(|v| v.as_str().is_some_and(|v| normalize(v) == key)),
                    Value::String(v) => normalize(v) == key,
                    _ => false,
                })
            });
        if matches {
            return Some(
                settings
                    .iter()
                    .filter(|(key, _)| !matches!(key.as_str(), "aliases" | "domains"))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            );
        }
    }
    None
}
