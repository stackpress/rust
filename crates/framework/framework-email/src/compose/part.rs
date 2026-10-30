//! MIME leaf encodings and native multipart ownership with source disposition values.

use crate::EmailResult;
use lettre::message::{
    Body, MultiPart, SinglePart,
    header::{self, Header, HeaderName, HeaderValue, Headers},
};
use std::io;

/// Encode a MIME leaf with source transfer choices and native library framing.
pub(super) fn part(
    bytes: Vec<u8>,
    kind: &str,
    filename: Option<&str>,
    encoding: Option<&str>,
    cid: Option<&str>,
    disposition: Option<&str>,
    preferred: Option<&str>,
) -> EmailResult<SinglePart> {
    let mut builder = SinglePart::builder().header(header::ContentType::parse(kind)?);
    let text = kind.to_ascii_lowercase().starts_with("text/");
    let encoding = match encoding
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("base64") => header::ContentTransferEncoding::Base64,
        Some("quoted-printable") => header::ContentTransferEncoding::QuotedPrintable,
        Some("8bit") if !text => header::ContentTransferEncoding::EightBit,
        Some("binary") if !text => header::ContentTransferEncoding::Binary,
        _ if text => text_encoding(&bytes, preferred),
        _ if kind.to_ascii_lowercase().starts_with("message/") => {
            header::ContentTransferEncoding::EightBit
        }
        _ => header::ContentTransferEncoding::Base64,
    };
    if let Some(disposition) = disposition.or_else(|| (!text).then_some("attachment")) {
        builder = builder.header(SourceDisposition::new(disposition, filename));
    }
    if let Some(cid) = cid {
        builder = builder.header(header::ContentId::from(format!(
            "<{}>",
            cid.replace(['<', '>'], "")
        )));
    }
    // Only base64/QP transform bytes; explicit source 8bit/binary remain byte-preserving.
    let body = if matches!(
        encoding,
        header::ContentTransferEncoding::Base64 | header::ContentTransferEncoding::QuotedPrintable
    ) {
        Body::new_with_encoding(bytes, encoding)
            .map_err(|_| io::Error::other("MIME encoding failed"))?
    } else {
        Body::dangerous_pre_encoded(bytes, encoding)
    };
    Ok(builder.body(body))
}

/// Source chooses 7bit for short plain text, otherwise the preferred Q/B or character ratio.
fn text_encoding(bytes: &[u8], preferred: Option<&str>) -> header::ContentTransferEncoding {
    use header::ContentTransferEncoding::{Base64, QuotedPrintable, SevenBit};
    let plain = bytes
        .iter()
        .all(|v| matches!(*v, b'\t' | b'\n' | b'\r' | 32..=126));
    if plain
        && bytes
            .split(|v| *v == b'\n')
            .all(|line| line.strip_suffix(b"\r").unwrap_or(line).len() <= 76)
    {
        return SevenBit;
    }
    match preferred_encoding(preferred) {
        Some('Q') => QuotedPrintable,
        Some(_) => Base64,
        None => {
            let value = String::from_utf8_lossy(bytes);
            let non_latin = value
                .encode_utf16()
                .filter(|v| matches!(*v,0..=8|11|12|14..=31|128..=65535))
                .count();
            let latin = value.bytes().filter(u8::is_ascii_alphabetic).count();
            if non_latin < latin {
                QuotedPrintable
            } else {
                Base64
            }
        }
    }
}

/// Source accepts extension disposition tokens; lettre supplies filename parameter encoding.
#[derive(Clone)]
struct SourceDisposition(HeaderValue);
impl SourceDisposition {
    /// Reuse lettre's RFC2231 filename formatting without narrowing source disposition values.
    fn new(kind: &str, filename: Option<&str>) -> Self {
        match filename {
            Some(name) => {
                let mut headers = Headers::new();
                headers.set(header::ContentDisposition::attachment(name));
                let value = headers.to_string();
                let encoded = value
                    .strip_prefix("Content-Disposition: ")
                    .unwrap_or(&value)
                    .trim_end_matches("\r\n")
                    .replacen("attachment", kind, 1);
                Self(HeaderValue::dangerous_new_pre_encoded(
                    Self::name(),
                    format!("{kind}; filename=\"{name}\""),
                    encoded,
                ))
            }
            None => Self(HeaderValue::new(Self::name(), kind.to_owned())),
        }
    }
}
impl Header for SourceDisposition {
    /// This source header has a fixed name with an extensible value.
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("Content-Disposition")
    }
    /// Parsing retains the value; generated filenames use the separate constructor.
    fn parse(value: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self(HeaderValue::new(Self::name(), value.to_owned())))
    }
    /// Emit the already encoded value without reinterpreting its parameters.
    fn display(&self) -> HeaderValue {
        self.0.clone()
    }
}

/// Native MIME tree node, preserving source mixed/alternative/related ownership.
pub(super) enum MimeBody {
    Single(SinglePart),
    Multi(MultiPart),
}
impl MimeBody {
    /// Attach a node without adding an extra multipart wrapper.
    pub(super) fn append_to(self, parent: MultiPart) -> MultiPart {
        match self {
            Self::Single(node) => parent.singlepart(node),
            Self::Multi(node) => parent.multipart(node),
        }
    }
    /// Encode through the approved MIME library.
    pub(super) fn formatted(self) -> Vec<u8> {
        match self {
            Self::Single(node) => node.formatted(),
            Self::Multi(node) => node.formatted(),
        }
    }
}

/// MimeNode trims the configured text/header encoding and reads its first uppercase character.
pub(super) fn preferred_encoding(value: Option<&str>) -> Option<char> {
    value
        .and_then(|value| value.trim().chars().next())
        .map(|value| value.to_ascii_uppercase())
}
