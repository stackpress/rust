//! Client-safe public DTOs; Profile is the actual Slice 3 generated source.
//!
//! This example-only path reuse deliberately fails if generation removes Profile.
//! It does not import the generated application's native dependencies.

mod reply;
/// One compiled route and query codec shared by both execution hosts and the UI.
#[path = "../../route-contract.rs"]
pub mod routes;
pub use reply::{
    HostError, ReplyDecodeError, SearchFailure, SearchFailureKind, SearchOutcome, TransportFailure,
    decode_search_reply,
};

#[path = "../../../generated-profile/app/src/generated/contracts/profile.rs"]
mod generated_profile;
pub use generated_profile::Profile;
use serde::{Deserialize, Serialize};
pub use stackpress_framework_view::{PageId, PageSnapshot, PublicStatus};

/// Current public search/page wire revision.
pub const REVISION: u32 = 1;
/// Build ID shared by the locked hydration and CSR assets.
pub const BUILD: &str = "profile-ui-m2-2";
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
    /// Rejects contract drift, empty/duplicate row identities or inconsistent/unbounded data.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.revision != REVISION {
            return Err("Page version changed. Refresh to continue.");
        }
        // A public row must name one detail and one stable keyed-list entry.
        // Check identities here so every transport/seed/controller shares that invariant.
        let mut identities = std::collections::BTreeSet::new();
        if self.rows.len() > 100
            || self.total != self.rows.len() as u64
            || self.rows.iter().any(|row| {
                row.id.is_empty()
                    || row.id.len() > 1024
                    || row.name.len() > 4096
                    || !identities.insert(row.id.as_str())
            })
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

/// Page seed contains public input/reply and optional exact detail/return context.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchPage {
    /// Exact input used by the native page loader.
    pub input: SearchInput,
    /// Initial output, consumed without a second call during hydration.
    pub reply: SearchReply,
    /// Exact detail identity and independent originating prefix, absent for search.
    pub detail: Option<DetailContext>,
}

/// Public detail navigation context; the origin is never inferred from the ID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetailContext {
    /// Exact record requested by the detail path.
    pub id: String,
    /// Validated prefix used for the return-to-search destination.
    pub search_prefix: String,
}

/// Public seed used identically by SSR and browser/native client roots.
pub type ProfileSnapshot = PageSnapshot<SearchPage>;

/// Check identity/data/status agreement before selecting the shared page root.
///
/// # Errors
/// Rejects unknown page IDs, malformed context and mismatched success/not-found data.
pub fn validate_page(
    page: &str,
    status: &PublicStatus,
    data: &SearchPage,
) -> Result<(), &'static str> {
    data.input.validate()?;
    data.reply.validate()?;
    match (page, &data.detail) {
        ("profiles", None) if status.code() == 200 => Ok(()),
        ("profile-detail", Some(detail)) => {
            SearchInput::new(detail.search_prefix.clone())?;
            if detail.id != data.input.id
                || routes::detail_location(&Default::default(), &detail.id, &detail.search_prefix)
                    .is_err()
            {
                return Err("Invalid detail context");
            }
            let found = data.reply.rows.iter().any(|row| row.id == detail.id);
            if status.code() == if found { 200 } else { 404 } {
                Ok(())
            } else {
                Err("Invalid detail status")
            }
        }
        _ => Err("Invalid page identity"),
    }
}

/// Construct explicit initial public metadata around owned page data.
///
/// # Errors
/// Returns malformed page identity (the supplied catalog key is checked).
pub fn snapshot(
    data: SearchPage,
) -> Result<PageSnapshot<SearchPage>, stackpress_framework_view::ViewError> {
    let base = Default::default();
    let (page, location, status) = match &data.detail {
        Some(detail) => (
            "profile-detail",
            routes::detail_location(&base, &detail.id, &detail.search_prefix)?,
            PublicStatus::new(
                if data.reply.rows.iter().any(|row| row.id == detail.id) {
                    200
                } else {
                    404
                },
                None,
            )?,
        ),
        None => (
            "profiles",
            routes::search_location(&base, &data.input.id)?,
            PublicStatus::ok(),
        ),
    };
    Ok(PageSnapshot {
        revision: REVISION,
        build: BUILD.into(),
        page: PageId::try_from(page.to_owned())?,
        location: location.into(),
        locale: "en".into(),
        theme: "light".into(),
        status,
        data,
    })
}
