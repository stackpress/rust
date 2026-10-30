//! Explicit one-shot stream transfer between rich event fields and native transports.

use crate::{
    Content, EmailError, EmailResult, MailReader, MessageOutput, NativeSendMailOptions,
    SentMessageInfo,
};
use serde_json::Value as Json;
use stackpress_lib::{
    Error, Nest, ParseLimits, Response, Result, SharedNative, Value, data::IntoNest,
};
use std::{collections::BTreeMap, sync::Mutex};

/// One owned Readable at an original message content/result field.
///
/// Put `SharedNative::new(NativeReader::new(reader))` at `text`, `html`, `raw`,
/// attachment/alternative content, or inspect it at returned `message`. Cloning
/// rich data shares the transfer slot, never duplicates the reader or its cursor.
/// JSON conversion rejects this native value rather than draining or erasing it.
pub struct NativeReader(Mutex<Option<MailReader>>);

impl NativeReader {
    /// Wrap an owned Send reader; no Sync requirement is imposed on the reader itself.
    pub fn new(reader: MailReader) -> Self {
        Self(Mutex::new(Some(reader)))
    }

    /// Transfer the reader once, releasing the short lock before any IO/await.
    ///
    /// # Errors
    /// Reports a poisoned slot or a previously consumed reader explicitly.
    pub fn take(&self) -> EmailResult<MailReader> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("email reader slot poisoned"))?
            .take()
            .ok_or_else(|| std::io::Error::other("email reader already consumed").into())
    }
}

/// Convert serializable source fields, transferring only explicitly native content nodes.
pub(super) fn options(input: &Nest) -> Result<NativeSendMailOptions> {
    let mut content = BTreeMap::new();
    let fields = input
        .value()
        .as_object()
        .ok_or_else(|| Error::invalid("email options require an object"))?;
    let mut json = serde_json::Map::new();
    for (key, value) in fields {
        json.insert(
            key.clone(),
            field(value, key, &mut content, &input.limits())?,
        );
    }
    Ok(NativeSendMailOptions {
        fields: json,
        content,
    })
}

/// Recurse ordinary options without reserving new message keys or dropping extensions.
fn field(
    value: &Value,
    path: &str,
    content: &mut BTreeMap<String, Content>,
    limits: &ParseLimits,
) -> Result<Json> {
    // The native compiler already recognizes the source content paths; other native
    // fields fail normal rich-to-JSON conversion instead of silently changing type.
    let slot = matches!(path, "text" | "html" | "raw")
        || ["attachments.", "alternatives."].iter().any(|prefix| {
            path.strip_prefix(prefix).is_some_and(|rest| {
                rest.strip_suffix(".content")
                    .unwrap_or(rest)
                    .parse::<usize>()
                    .is_ok()
            })
        });
    if slot {
        let native = match value {
            Value::Bytes(bytes) => Some(Content::Bytes(bytes.clone())),
            Value::Shared(shared) => shared
                .downcast_ref::<NativeReader>()
                .map(|reader| reader.take().map(Content::Reader))
                .transpose()
                .map_err(error)?,
            _ => None,
        };
        if let Some(native) = native {
            content.insert(path.to_owned(), native);
            return Ok(Json::Null);
        }
    }
    match value {
        Value::Array(values) => values
            .iter()
            .enumerate()
            .map(|(index, value)| field(value, &format!("{path}.{index}"), content, limits))
            .collect::<Result<Vec<_>>>()
            .map(Json::Array),
        Value::Object(values) => values
            .iter()
            .map(|(key, value)| {
                Ok((
                    key.clone(),
                    field(value, &format!("{path}.{key}"), content, limits)?,
                ))
            })
            .collect::<Result<serde_json::Map<_, _>>>()
            .map(Json::Object),
        // Retain the request owner's selected policy instead of inserting a default cap.
        _ => value.to_json_with(limits),
    }
}

/// Preserve all provider result fields and explicitly retain any live stream.
pub(super) fn results(mut info: SentMessageInfo, response: &mut Response) -> Result<()> {
    // Existing Buffer serialization is the M1 Node Buffer JSON representation.
    let stream = match info.message.take() {
        Some(MessageOutput::Stream(reader)) => Some(reader),
        message => {
            info.message = message;
            None
        }
    };
    if let Some(reader) = stream {
        // Replace a provider's stale message field before checking the selected
        // response budget. The live stream occupies this original result key.
        let mut value = serde_json::to_value(info)?;
        let Json::Object(ref mut fields) = value else {
            return Err(Error::invalid(
                "native email outputs require an object result",
            ));
        };
        fields.insert("message".into(), Json::Null);
        let mut data = value.into_nest_with(response.limits())?;
        data.set(
            "message",
            Value::Shared(SharedNative::new(NativeReader::new(reader))),
        )?;
        response.results_value(data.into_value())?;
    } else {
        // The ordinary result setter already honors the response owner's policy,
        // including deliberate larger budgets and arbitrary scalar provider info.
        response.results(&info)?;
    }
    Ok(())
}

/// Preserve the concrete provider error chain through existing framework dispatch.
pub(super) fn error(cause: EmailError) -> Error {
    Error::new("email operation failed").with_cause(ProviderFailure(cause))
}

/// Sized cause adapter required by lib's generic with_cause contract; never flatten the box.
#[derive(Debug)]
struct ProviderFailure(EmailError);
impl std::fmt::Display for ProviderFailure {
    /// Preserve the original provider diagnostic for native callers.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for ProviderFailure {
    /// Retain the concrete source for normal downcasting through the error chain.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}
