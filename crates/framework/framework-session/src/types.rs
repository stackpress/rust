//! Portable source-shaped configuration and grants, with missing access retained.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{collections::BTreeMap, sync::Arc};

/// JWT object claims without a new required-claim or field filtering policy.
pub type SessionData = Map<String, Value>;
/// Serializable me/authorization result, with source defaults, flattened claims, token and permits.
pub type SessionTokenData = SessionData;
/// Configured role grants; vector order and route object identities are retained.
pub type SessionPermissionList = BTreeMap<String, Vec<SessionPermission>>;

/// A method/path permission, distinct from the framework router's pattern contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRoute {
    /// Exact method or the literal ALL, as in the source matcher.
    pub method: String,
    /// Exact path, source wildcard derivation, or slash-delimited expression.
    pub route: String,
}
/// Event strings deduplicate by value; shared route objects deduplicate by Arc identity.
#[derive(Clone, Debug)]
pub enum SessionPermission {
    /// Event permission with hyphen-segment wildcards.
    Event(String),
    /// Route permission preserving reference identity across repeated roles.
    Route(Arc<SessionRoute>),
}
impl SessionPermission {
    /// Allocate one route object; clone this permission to share its source identity.
    pub fn route(method: impl Into<String>, route: impl Into<String>) -> Self {
        Self::Route(Arc::new(SessionRoute {
            method: method.into(),
            route: route.into(),
        }))
    }
}
impl Serialize for SessionPermission {
    /// Preserve the original untagged string-or-object wire shape.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Event(event) => event.serialize(serializer),
            Self::Route(route) => route.as_ref().serialize(serializer),
        }
    }
}
impl<'de> Deserialize<'de> for SessionPermission {
    /// Each parsed route is a separate reference, even when its fields compare equal.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Untagged deserialization preserves the source wire vocabulary without an enum tag.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Event(String),
            Route(SessionRoute),
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Event(event) => Self::Event(event),
            Wire::Route(route) => Self::Route(Arc::new(route)),
        })
    }
}
/// Optional source session section; absence of access differs from an empty map.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Cookie field; defaults to session.
    #[serde(default = "default_key")]
    pub key: String,
    /// UTF-8 JWT seed; original direct/default value is abc123.
    #[serde(default = "default_seed")]
    pub seed: String,
    /// None omits the global guard; Some(empty) installs a permissive guard.
    #[serde(default)]
    pub access: Option<SessionPermissionList>,
}
impl Default for SessionConfig {
    /// Preserve the source defaults without implicitly selecting the plugin.
    fn default() -> Self {
        Self {
            key: default_key(),
            seed: default_seed(),
            access: None,
        }
    }
}
/// Original cookie field default.
fn default_key() -> String {
    "session".into()
}
/// Original seed fallback, not a mandatory-strength policy.
fn default_seed() -> String {
    "abc123".into()
}
