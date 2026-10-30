//! Explicit CSRF mechanisms matching `stackpress-csrf/src/plugin.ts`.
//!
//! Generation stages outgoing cookie/form values; validation reads incoming values.
//! Callers choose which processes validate. There is no automatic route guard.
//! The optional `server` feature adds the ordinary `plugin::register` constituent.
//!
//! ```
//! use stackpress_framework_csrf::Service;
//! use stackpress_lib::Response;
//! let mut response = Response::new();
//! let token = Service::default().generate(&mut response)?;
//! assert_eq!(token.len(), 64);
//! # Ok::<(), stackpress_lib::Error>(())
//! ```

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use stackpress_lib::{Error, PublicError, Request, Response, Result};
use subtle::ConstantTimeEq;

#[cfg(feature = "server")]
pub mod plugin;

/// Original default page-expiry explanation, including its retry instruction.
pub const DEFAULT_ERROR: &str = "This page may have been requested from an external source. We corrected the issue. Please try again.";

/// Source configuration; omitted fields use the source defaults.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CsrfConfig {
    /// Shared incoming/outgoing cookie and form-data name; default `csrf`.
    pub name: Option<String>,
    /// Public failure explanation; default [`DEFAULT_ERROR`].
    pub error: Option<String>,
}

/// Independently usable mechanism; no identity-session dependency or global policy.
#[derive(Clone, Debug, Default)]
pub struct Service {
    /// Configured field names and messages, never token state.
    config: CsrfConfig,
}

impl Service {
    /// Select source overrides; even an absent CSRF section has a default mechanism.
    pub fn new(config: CsrfConfig) -> Self {
        // Retain explicit empty strings instead of treating them as missing overrides.
        Self { config }
    }

    /// Borrow the selected field name, falling back only when omitted.
    pub fn name(&self) -> &str {
        // One name is used consistently for session, aggregate input and SSR data.
        self.config.name.as_deref().unwrap_or("csrf")
    }

    /// Generate 32 OS-random bytes as lowercase hex, then stage cookie and form data.
    ///
    /// # Errors
    /// Reports OS entropy failure or existing response/cookie/data validation errors.
    /// Cookie staging precedes data staging as upstream; later failure may leave it staged.
    pub fn generate(&self, res: &mut Response) -> Result<String> {
        // Use the approved provider rather than a deterministic/predictable fallback.
        let mut bytes = [0_u8; 32];
        SystemRandom::new()
            .fill(&mut bytes)
            .map_err(|_| Error::new("CSRF random generation failed"))?;
        // Hex encoding is a representation step, not an acceptance restriction in valid().
        let mut token = String::with_capacity(64);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in bytes {
            token.push(HEX[(byte >> 4) as usize] as char);
            token.push(HEX[(byte & 15) as usize] as char);
        }
        // Preserve source mutation order and the low-level cookie policy already selected.
        res.session_mut()?.set(self.name(), token.clone(), None)?;
        let form = stackpress_lib::Value::try_from(serde_json::json!({
            "name": self.name(), "token": token,
        }))?;
        res.data_mut()?.set(self.name(), form)?;
        Ok(token)
    }

    /// Delete submitted aggregate input, outgoing cookie and outgoing form projection.
    ///
    /// # Errors
    /// Propagates the existing data/cookie/response mutation errors in source order.
    /// The incoming session cookie is deliberately retained.
    pub fn clear(&self, req: &mut Request, res: &mut Response) -> Result<()> {
        // Do not clear query/post snapshots or the incoming cookie; source deletes req.data only.
        req.data.remove(self.name())?;
        res.session_mut()?.remove(self.name())?;
        res.data_mut()?.remove(self.name())?;
        Ok(())
    }

    /// Compare arbitrary nonempty incoming strings using equal-length timing-safe equality.
    ///
    /// Invalid input sets the original 419/Page Expired failure and returns false.
    /// Matching strings have no hex, method, Origin or expiry requirement.
    ///
    /// # Errors
    /// Propagates existing response-state or public-error validation failures.
    pub fn valid(&self, req: &Request, res: &mut Response) -> Result<bool> {
        // Only incoming scalar cookie and aggregate request strings participate.
        let session = req
            .session
            .get(self.name())
            .and_then(|value| value.as_str());
        let input = req.data.get(self.name()).and_then(|value| value.as_str());
        if let (Some(session), Some(input)) = (session, input) {
            // Public shape/length checks precede the source's timing-safe byte comparison.
            if !session.is_empty()
                && !input.is_empty()
                && session.len() == input.len()
                && bool::from(session.as_bytes().ct_eq(input.as_bytes()))
            {
                return Ok(true);
            }
        }
        // Rust's existing Response accepts a typed public error rather than a JS string.
        res.set_error(PublicError {
            code: 419,
            status: "Page Expired".into(),
            error: self.config.error.as_deref().unwrap_or(DEFAULT_ERROR).into(),
            errors: serde_json::json!({}),
        })?;
        Ok(false)
    }
}
