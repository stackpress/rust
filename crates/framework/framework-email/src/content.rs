//! Source Buffer/Readable/file/URL resolution behind an injectable native IO boundary.

use crate::options::{flag, string, truthy};
use crate::{Content, EmailResult, NativeSendMailOptions};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Map, Value};
use std::{future::Future, io, pin::Pin, sync::Arc};
use tokio::io::AsyncReadExt;

/// Owned asynchronous content read; no input reader or provider handle is duplicated.
pub type ContentFuture<'a> = Pin<Box<dyn Future<Output = EmailResult<Vec<u8>>> + Send + 'a>>;

/// A file/URL request retaining the source object and request headers for native providers.
#[derive(Clone, Debug)]
pub struct ContentRequest {
    /// File path, HTTP(S) URL or data URI selected at the source precedence point.
    pub location: String,
    /// Original source object, including native-adapter extension options.
    pub options: Map<String, Value>,
    /// MIME href honors httpHeaders; source JSON normalization fetch intentionally does not.
    pub http_headers: bool,
}

/// Application-replaceable file/URL reader; tests use it without a listener or network.
pub trait ContentResolver: Send + Sync {
    /// Read source content without introducing size, origin or recipient policy.
    fn read(&self, request: ContentRequest) -> ContentFuture<'_>;
}

/// Native file and approved reqwest adapter; clients are built only when a URL is used.
#[derive(Default)]
pub struct NativeContentResolver;

impl ContentResolver for NativeContentResolver {
    /// Preserve source file, data URI and HTTP(S) content capabilities.
    fn read(&self, request: ContentRequest) -> ContentFuture<'_> {
        Box::pin(async move {
            if request.location.starts_with("data:") {
                return data_uri(&request.location);
            }
            if is_url(&request.location) {
                // Source fetch is a fresh cookie jar, gzip/deflate, five redirects, no timeout.
                let client = reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .cookie_store(true)
                    .danger_accept_invalid_certs(true)
                    .user_agent("nodemailer/7.0.13")
                    .build()?;
                let mut location = url::Url::parse(&request.location)?;
                for redirects in 0..=5 {
                    let mut get = client
                        .get(location.clone())
                        .header("accept-encoding", "gzip,deflate");
                    if let Some(headers) = request
                        .options
                        .get("httpHeaders")
                        .and_then(Value::as_object)
                        .filter(|_| request.http_headers)
                    {
                        for (name, value) in headers {
                            get = get.header(name, string(value));
                        }
                    }
                    let response = get.send().await?;
                    if matches!(response.status().as_u16(), 301 | 302 | 303 | 307 | 308)
                        && let Some(next) = response.headers().get("location")
                    {
                        if redirects == 5 {
                            return Err(io::Error::other("Maximum redirect count exceeded").into());
                        }
                        location = location.join(next.to_str()?)?;
                        continue;
                    }
                    if response.status().as_u16() >= 300 {
                        return Err(io::Error::other(format!(
                            "Invalid status code {}",
                            response.status().as_u16()
                        ))
                        .into());
                    }
                    return Ok(response.bytes().await?.to_vec());
                }
                unreachable!("the fifth redirect returns an error");
            }
            Ok(tokio::fs::read(request.location).await?)
        })
    }
}

/// Resolve one body/attachment slot. JSON normalization bypasses MIME access flags as source does.
pub(crate) async fn resolve(
    mail: &mut NativeSendMailOptions,
    slot: &str,
    value: &Value,
    resolver: &Arc<dyn ContentResolver>,
    enforce_access: bool,
) -> EmailResult<Vec<u8>> {
    // Native readers and buffers replace exactly one named content field.
    let native = mail
        .content
        .remove(slot)
        .or_else(|| mail.content.remove(&format!("{slot}.content")));
    if let Some(native) = native {
        return match native {
            Content::Bytes(bytes) => Ok(bytes),
            Content::Reader(mut reader) => {
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).await?;
                Ok(bytes)
            }
        };
    }
    let object = value.as_object();
    let content = object
        .filter(|object| !enforce_access || !(flag(object, "path") || flag(object, "href")))
        .and_then(|v| v.get("content"))
        .filter(|v| truthy(v))
        .unwrap_or(value);
    if let Some(buffer) = content
        .as_object()
        .filter(|v| v.get("type").and_then(Value::as_str) == Some("Buffer"))
    {
        return Ok(buffer
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| io::Error::other("invalid Buffer data"))?
            .iter()
            .map(|v| v.as_u64().unwrap_or(0) as u8)
            .collect());
    }
    if let Some(object) = content.as_object() {
        let location = object
            .get("path")
            .filter(|v| truthy(v))
            .or_else(|| object.get("href").filter(|v| truthy(v)));
        if let Some(location) = location.and_then(Value::as_str) {
            let url = is_url(location);
            if enforce_access
                && ((url && flag(&mail.fields, "disableUrlAccess"))
                    || (!url
                        && !location.starts_with("data:")
                        && flag(&mail.fields, "disableFileAccess")))
            {
                return Err(io::Error::other(if url {
                    "Url access rejected"
                } else {
                    "File access rejected"
                })
                .into());
            }
            return resolver
                .read(ContentRequest {
                    location: location.into(),
                    options: object.clone(),
                    http_headers: enforce_access,
                })
                .await;
        }
    }
    let encoding = object
        .and_then(|v| v.get("encoding"))
        .map(string)
        .unwrap_or_else(|| "utf8".into());
    decode(content.as_str().unwrap_or(""), &encoding)
}

/// Node Buffer text encodings; invalid/odd hex stops at the first incomplete pair.
pub(crate) fn decode(text: &str, encoding: &str) -> EmailResult<Vec<u8>> {
    let encoding = encoding.to_lowercase().replace(['-', '_', ' '], "");
    match encoding.as_str() {
        "utf8" | "utf" => Ok(text.as_bytes().to_vec()),
        "base64" | "base64url" => {
            // Buffer.from accepts whitespace, URL-safe alphabet and omitted padding.
            let mut text: String = text
                .chars()
                .take_while(|c| *c != '=')
                .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '-' | '_'))
                .map(|c| match c {
                    '-' => '+',
                    '_' => '/',
                    _ => c,
                })
                .collect();
            if text.len() % 4 == 1 {
                text.pop();
            }
            while !text.len().is_multiple_of(4) {
                text.push('=');
            }
            Ok(STANDARD.decode(text)?)
        }
        "hex" => Ok(text
            .as_bytes()
            .chunks_exact(2)
            .map_while(|pair| {
                std::str::from_utf8(pair)
                    .ok()
                    .and_then(|v| u8::from_str_radix(v, 16).ok())
            })
            .collect()),
        "ascii" | "usascii" | "latin1" | "binary" => {
            Ok(text.encode_utf16().map(|c| c as u8).collect())
        }
        "utf16le" | "ucs2" => Ok(text.encode_utf16().flat_map(u16::to_le_bytes).collect()),
        _ => Err(io::Error::other(format!(
            "native provider required for content encoding {encoding}"
        ))
        .into()),
    }
}

/// Decode data URI content; invalid URI escaping remains an explicit content error.
fn data_uri(uri: &str) -> EmailResult<Vec<u8>> {
    Ok(parse_data_uri(uri)?
        .map(|(_, bytes)| bytes)
        .unwrap_or_default())
}

/// Source data URI decoding recognizes only a final base64/utf8 marker and falls back on bad escapes.
pub(crate) fn parse_data_uri(uri: &str) -> EmailResult<Option<(String, Vec<u8>)>> {
    let Some(body) = uri.strip_prefix("data:") else {
        return Ok(None);
    };
    let Some((meta, body)) = body.split_once(',') else {
        return Ok(None);
    };
    let mut entries = meta.split(';').collect::<Vec<_>>();
    let encoding = entries
        .last()
        .map(|v| v.trim().to_lowercase())
        .unwrap_or_default();
    if matches!(encoding.as_str(), "base64" | "utf8" | "utf-8") {
        entries.pop();
    }
    let kind = entries
        .first()
        .filter(|v| !v.is_empty())
        .copied()
        .unwrap_or("application/octet-stream")
        .to_owned();
    let bytes = if encoding == "base64" {
        decode(body, "base64")?
    } else {
        crate::options::decode_uri(body)
            .unwrap_or_else(|_| body.to_owned())
            .into_bytes()
    };
    Ok(Some((kind, bytes)))
}

/// Source HTTP matching is case insensitive; data URI parsing retains its own source rules.
fn is_url(location: &str) -> bool {
    let lower = location.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}
