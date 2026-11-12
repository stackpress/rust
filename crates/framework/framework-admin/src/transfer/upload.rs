//! Byte-preserving upload payload with source-compatible string conversion.
use serde::{Deserialize, Serialize};

/// Raw CSV upload. JSON serialization uses bytes; deserialization also accepts legacy text.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct CsvBytes(Vec<u8>);
impl CsvBytes {
    /// Borrow the original file without decoding or copying it.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    /// Count raw file bytes for host/browser admission before decoding.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Clear the root-owned selection while retaining its reusable allocation.
    pub fn clear(&mut self) {
        self.0.clear();
    }
    /// Test whether a file has any raw bytes.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl AsRef<[u8]> for CsvBytes {
    /// Accept this payload at byte-oriented parser boundaries.
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
impl From<Vec<u8>> for CsvBytes {
    /// Move an uploaded file without another data allocation.
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}
impl From<String> for CsvBytes {
    /// Preserve existing UTF-8 callers without copying their owned buffer.
    fn from(text: String) -> Self {
        Self(text.into_bytes())
    }
}
impl From<&str> for CsvBytes {
    /// Copy borrowed UTF-8 text for an owned upload.
    fn from(text: &str) -> Self {
        Self(text.as_bytes().to_vec())
    }
}
impl<'de> Deserialize<'de> for CsvBytes {
    /// Keep both historical string requests and new raw-byte requests usable.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Payload {
            Bytes(Vec<u8>),
            Text(String),
        }
        Ok(match Payload::deserialize(deserializer)? {
            Payload::Bytes(bytes) => bytes.into(),
            Payload::Text(text) => text.into(),
        })
    }
}
