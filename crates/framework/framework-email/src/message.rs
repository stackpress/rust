//! Owned native content and transport results preserve streams outside JSON snapshots.

use crate::SendMailOptions;
use serde::{Serialize, Serializer};
use serde_json::Value;
use std::{collections::BTreeMap, fmt, pin::Pin};
use tokio::io::AsyncRead;

/// One owned, movable reader; no Clone/Sync requirement duplicates or shares a read.
pub type MailReader = Pin<Box<dyn AsyncRead + Send + 'static>>;

/// Native equivalent of a source Buffer/Readable, supplied at a named content slot.
pub enum Content {
    /// Exact bytes, equivalent to a source Buffer.
    Bytes(Vec<u8>),
    /// One-shot source stream, consumed by the selected transport.
    Reader(MailReader),
}

/// Complete serializable options plus native content at source field/index paths.
///
/// Slots include `text`, `html`, `raw`, `attachments.0.content` and
/// `alternatives.0.content`. Native callbacks and custom sockets are carried by
/// the configured provider factory, never serialized as input objects.
#[derive(Default)]
pub struct NativeSendMailOptions {
    /// Every original message option, including provider extensions.
    pub fields: SendMailOptions,
    /// Owned Buffer/Readable replacements for the selected source content slots.
    pub content: BTreeMap<String, Content>,
}

impl NativeSendMailOptions {
    /// Recognize the native content node and its explicit `.content` spelling.
    pub(crate) fn has_content(&self, slot: &str) -> bool {
        self.content.contains_key(slot) || self.content.contains_key(&format!("{slot}.content"))
    }
}

impl From<SendMailOptions> for NativeSendMailOptions {
    /// Lift ordinary source options without adding message defaults or native content.
    fn from(fields: SendMailOptions) -> Self {
        Self {
            fields,
            content: BTreeMap::new(),
        }
    }
}

/// The source stream transport deliberately returns either Buffer or Readable.
pub enum MessageOutput {
    /// Source buffer output, serializable as the Node Buffer JSON representation.
    Buffer(Vec<u8>),
    /// Source readable output; explicit native consumption is required.
    Stream(MailReader),
}

/// Unfiltered provider info plus intentional native message/raw outputs.
///
/// Serializing Buffer outputs uses `{type:"Buffer",data:[...]}`. A live stream
/// cannot be serialized in Rust: serialization reports an error and retains the
/// owned reader for direct consumption. It is never silently omitted or drained.
pub struct SentMessageInfo {
    /// All provider JSON fields, including unknown extensions.
    pub fields: Value,
    /// Optional stream-transport message payload.
    pub message: Option<MessageOutput>,
    /// Optional SES raw MIME payload.
    pub raw: Option<Vec<u8>>,
}

impl From<Value> for SentMessageInfo {
    /// Retain a provider's original result, including a non-object result.
    fn from(fields: Value) -> Self {
        Self {
            fields,
            message: None,
            raw: None,
        }
    }
}

impl fmt::Debug for SentMessageInfo {
    /// Keep message bytes and native readers out of diagnostic formatting.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SentMessageInfo")
            .field("native_message", &self.message.is_some())
            .field("native_raw", &self.raw.is_some())
            .finish_non_exhaustive()
    }
}

impl std::ops::Index<&str> for SentMessageInfo {
    type Output = Value;
    /// Borrow a provider JSON field without consuming native content.
    fn index(&self, key: &str) -> &Self::Output {
        &self.fields[key]
    }
}

impl PartialEq<Value> for SentMessageInfo {
    /// Compare JSON-representable info; live streams remain native rather than erased.
    fn eq(&self, value: &Value) -> bool {
        serde_json::to_value(self).ok().as_ref() == Some(value)
    }
}

impl Serialize for SentMessageInfo {
    /// Project Buffer outputs explicitly; a live reader is a serialization error.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = self.fields.clone();
        if !value.is_object() && (self.message.is_some() || self.raw.is_some()) {
            return Err(serde::ser::Error::custom(
                "native email outputs require an object result",
            ));
        }
        if let Some(message) = &self.message {
            match message {
                MessageOutput::Buffer(bytes) => value["message"] = buffer_json(bytes),
                MessageOutput::Stream(_) => {
                    return Err(serde::ser::Error::custom(
                        "native email message stream requires explicit consumption",
                    ));
                }
            }
        }
        if let Some(bytes) = &self.raw {
            value["raw"] = buffer_json(bytes);
        }
        value.serialize(serializer)
    }
}

/// Node Buffer JSON shape is a wire representation, never a substitute for a stream.
pub(crate) fn buffer_json(bytes: &[u8]) -> Value {
    serde_json::json!({"type":"Buffer", "data":bytes})
}
