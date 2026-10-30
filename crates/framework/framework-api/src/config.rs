//! Typed source API configuration, with missing/null expiry kept distinct.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stackpress_framework_schema::Input;
#[cfg(not(target_arch = "wasm32"))]
use stackpress_lib::Result;

/// The source's public, application and persisted API-session adapters.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiType {
    /// Forward without API credential admission.
    Public,
    /// Look up an application ID and compare its secret only on non-GET methods.
    App,
    /// Look up an API Session and pass its profileId as ordinary request data.
    Session,
}

/// Original CORS configuration shapes; this is not a replacement CORS policy.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Cors {
    /// False disables callbacks; true selects the source wildcard branch.
    Enabled(bool),
    /// A source string is wrapped as a one-element list.
    Origin(String),
    /// An empty list still selects the source callbacks but matches no origin.
    Origins(Vec<String>),
}

/// Human descriptions of a scope; this registry grants no endpoint authority.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ApiScope {
    /// Optional presentation icon name.
    pub icon: Option<String>,
    /// Human-readable permission name.
    pub name: String,
    /// Human-readable permission description.
    pub description: String,
}

/// One configured method/path-to-event adapter, preserving source field names.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ApiEndpoint {
    /// Optional descriptive title, without authorization effect.
    pub name: Option<String>,
    /// Optional descriptive text.
    pub description: Option<String>,
    /// Optional source usage example.
    pub example: Option<String>,
    /// The source method, including ALL; route compilation belongs to M2.
    pub method: String,
    /// The source route/capture expression.
    pub route: String,
    /// Which local source admission branch will serve this endpoint.
    #[serde(rename = "type")]
    pub api_type: ApiType,
    /// Alternative accepted scopes; an empty list does not match protected callers.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// Optional original header convenience configuration.
    pub cors: Option<Cors>,
    /// The configured event; no added exposure allowlist is imposed.
    pub event: String,
    /// Signed existing dispatcher priority; omission and zero both mean zero.
    #[serde(default)]
    pub priority: i64,
    /// Literal top-level overlay; nested values replace rather than deep-merge.
    #[serde(default)]
    pub data: Map<String, Value>,
}

/// One source inline webhook subscription; transport selection remains explicit.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ApiWebhook {
    /// Event observed at source priority -200 in the later webhook integration.
    pub event: String,
    /// Configured URI, with no API-added destination restriction.
    pub uri: String,
    /// Method passed to the fetch adapter.
    pub method: String,
    /// Source recursive result predicate, not a policy expression language.
    pub validity: Map<String, Value>,
    /// Source fixed data, included beside params and results.
    #[serde(default)]
    pub data: Map<String, Value>,
}

/// Present source API configuration; the host stores `Option<ApiConfig>` for activation.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ApiConfig {
    /// Source milliseconds: omission defaults in OAuth, explicit null coerces to zero.
    #[serde(default, skip_serializing_if = "Input::is_missing")]
    pub expires: Input<f64>,
    /// Inline subscriptions; independent of the endpoint-list OAuth mount condition.
    #[serde(default)]
    pub webhooks: Vec<ApiWebhook>,
    /// Consent labels only, with no registry-whitelist check added.
    #[serde(default)]
    pub scopes: BTreeMap<String, ApiScope>,
    /// Source declaration order, including intentional overlapping paths/events.
    #[serde(default)]
    pub endpoints: Vec<ApiEndpoint>,
}

impl ApiConfig {
    /// Apply the source OAuth default/arithmetic and ECMAScript Date TimeClip.
    ///
    /// Omitted expires means one day; null means zero; supplied zero/negative/fractional
    /// milliseconds remain meaningful. None represents Invalid Date, not a new TTL rule.
    pub fn expires_at(&self, now_ms: i64) -> Option<i64> {
        // Destructuring defaults only on omission; null follows JavaScript numeric addition.
        let duration = match self.expires {
            Input::Missing => 86_400_000.0,
            Input::Null => 0.0,
            Input::Value(value) => value,
        };
        let value = now_ms as f64 + duration;
        // Date's representable range is narrower than i64; TimeClip truncates toward zero.
        (value.is_finite() && value.abs() <= 8_640_000_000_000_000.0).then(|| value.trunc() as i64)
    }
}

/// Preserve typed source input without adding security or scope-registry validation.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn validate(_: &Option<ApiConfig>) -> Result<()> {
    // Host serde decoding supplies the representation boundary; later adapters own behavior.
    Ok(())
}
