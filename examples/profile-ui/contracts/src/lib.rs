//! Client-safe public DTOs; Profile is the actual Slice 3 generated source.
//!
//! This example-only path reuse deliberately fails if generation removes Profile.
//! It does not import the generated application's native dependencies.

#[path = "../../../generated-profile/app/src/generated/contracts/profile.rs"]
mod generated_profile;
pub use generated_profile::Profile;
use serde::{Deserialize, Serialize};
use stackpress_framework_view::{PageId, PageSnapshot, PublicStatus};

/// Current public search/page wire revision.
pub const REVISION: u32 = 1;
/// Build ID shared by the locked hydration and CSR assets.
pub const BUILD: &str = "profile-ui-m1-1";
/// Maximum encoded search DTO size.
pub const MAX_REQUEST_BYTES: usize = 64 * 1024;

/// Fixed operation input; callers cannot select arbitrary events or native services.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchInput {
    /// Required public contract revision.
    pub revision: u32,
    /// Case-sensitive nonempty ID prefix.
    pub id: String,
}

impl SearchInput {
    /// Build and validate public user input; host validation remains authoritative.
    ///
    /// # Errors
    /// Rejects empty or over-1024-byte prefixes and incompatible revisions.
    pub fn new(id: String) -> Result<Self, &'static str> {
        let input = Self {
            revision: REVISION,
            id,
        };
        input.validate()?;
        Ok(input)
    }

    /// Validate a decoded DTO before any domain handler is admitted.
    ///
    /// # Errors
    /// Returns a safe error without echoing the supplied prefix.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.revision != REVISION {
            return Err("Page version changed. Refresh to continue.");
        }
        if self.id.is_empty() || self.id.len() > 1024 {
            return Err("Enter a profile ID prefix (1–1024 bytes).");
        }
        Ok(())
    }
}

/// Safe operation output; native diagnostics and response headers are excluded.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchReply {
    /// Codec revision, checked at every client boundary.
    pub revision: u32,
    /// Explicitly allowlisted generated public records.
    pub rows: Vec<Profile>,
    /// Returned-row count, preserving zero.
    pub total: u64,
}

impl SearchReply {
    /// Validate an external reply before mutating any UI state.
    ///
    /// # Errors
    /// Rejects contract drift or an inconsistent/unbounded result shape.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.revision != REVISION {
            return Err("Page version changed. Refresh to continue.");
        }
        if self.rows.len() > 100
            || self.total != self.rows.len() as u64
            || self
                .rows
                .iter()
                .any(|row| row.id.len() > 1024 || row.name.len() > 4096)
        {
            return Err("Invalid profile reply");
        }
        Ok(())
    }

    /// Project only results/total out of an existing successful framework envelope.
    ///
    /// # Errors
    /// Rejects missing fields and invalid rows rather than inventing empty success.
    pub fn from_envelope(value: &serde_json::Value) -> Result<Self, &'static str> {
        let rows = serde_json::from_value(value.get("results").cloned().ok_or("Missing rows")?)
            .map_err(|_| "Invalid rows")?;
        let total = value
            .get("total")
            .and_then(serde_json::Value::as_u64)
            .ok_or("Missing total")?;
        let reply = Self {
            revision: REVISION,
            rows,
            total,
        };
        reply.validate()?;
        Ok(reply)
    }
}

/// Page seed contains only search input plus public search output.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchPage {
    /// Exact input used by the native page loader.
    pub input: SearchInput,
    /// Initial output, consumed without a second call during hydration.
    pub reply: SearchReply,
}

/// Construct explicit initial public metadata around owned page data.
///
/// # Errors
/// Returns malformed page identity (the supplied catalog key is checked).
pub fn snapshot(
    data: SearchPage,
) -> Result<PageSnapshot<SearchPage>, stackpress_framework_view::ViewError> {
    // Serialize only the public prefix into the canonical same-origin query.
    let mut query = String::new();
    for byte in data.input.id.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            query.push(char::from(byte));
        } else {
            use std::fmt::Write;
            // Writing into String cannot fail; percent encoding retains UTF-8 bytes exactly.
            let _ = write!(query, "%{byte:02X}");
        }
    }
    Ok(PageSnapshot {
        revision: REVISION,
        build: BUILD.into(),
        page: PageId::try_from("profiles".to_owned())?,
        location: format!("/ui/profiles?id={query}"),
        locale: "en".into(),
        theme: "light".into(),
        status: PublicStatus::ok(),
        data,
    })
}
