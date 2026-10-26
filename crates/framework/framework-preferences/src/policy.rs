//! Safe layout hints and authoritative host admission are separate types/paths.
use serde::{Deserialize, Serialize};

/// Checked fixed app action key, never an arbitrary event name from request JSON.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ActionId(String);
impl ActionId {
    /// Check a bounded ASCII application action identity.
    ///
    /// # Errors
    /// Rejects empty, excessive or syntactically unsafe keys.
    pub fn new(value: impl Into<String>) -> Result<Self, AccessDenied> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 64
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err(AccessDenied);
        }
        Ok(Self(value))
    }
    /// Stable public label used only to hide/show layout actions.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for ActionId {
    type Error = AccessDenied;
    /// Wire projection keys receive the same syntax checks as native keys.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl From<ActionId> for String {
    /// Transfer only the public action identity.
    fn from(value: ActionId) -> Self {
        value.0
    }
}

/// Safe uniform denial, without identity/token/role/private failure details.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccessDenied;
impl std::fmt::Display for AccessDenied {
    /// Do not reflect host policy internals to clients.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Action unavailable")
    }
}
impl std::error::Error for AccessDenied {}

/// Bounded presentation hints only. Deserializing or editing these grants no access.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "Vec<ActionId>", into = "Vec<ActionId>")]
pub struct AllowedActions(Vec<ActionId>);
impl AllowedActions {
    /// Test layout visibility only; authoritative adapters must call HostAccess again.
    pub fn contains(&self, action: &ActionId) -> bool {
        self.0.contains(action)
    }
}
impl TryFrom<Vec<ActionId>> for AllowedActions {
    type Error = AccessDenied;
    /// Bound projection size and reject duplicate action identities.
    fn try_from(actions: Vec<ActionId>) -> Result<Self, Self::Error> {
        let unique: std::collections::BTreeSet<_> = actions.iter().collect();
        if actions.len() > 64 || unique.len() != actions.len() {
            return Err(AccessDenied);
        }
        Ok(Self(actions))
    }
}
impl From<AllowedActions> for Vec<ActionId> {
    /// Serialize only safe keys, never the host context.
    fn from(value: AllowedActions) -> Self {
        value.0
    }
}

/// Explicit app-supplied authority over a fresh native context C.
///
/// Implementations must authenticate/authorize the actual caller and resource scope,
/// and validate method/Origin/CSRF for web mutations before returning true. No default
/// allow-all implementation exists. This is an admission boundary, not sign-in.
pub trait HostPolicy<C> {
    /// Decide from current host context, never from a public projection/token snapshot.
    fn permits(&self, context: &C, action: &ActionId) -> bool;
}

/// Borrowed request-local gate; absent policy denies every protected exposure.
pub struct HostAccess<'a, C> {
    /// Authoritative app callback; None is deliberately fail closed.
    policy: Option<&'a dyn HostPolicy<C>>,
    /// Fresh host-owned request/resource context, not deserialized UI state.
    context: &'a C,
}
impl<'a, C> HostAccess<'a, C> {
    /// Bind the explicit optional host policy for this request.
    pub fn new(policy: Option<&'a dyn HostPolicy<C>>, context: &'a C) -> Self {
        Self { policy, context }
    }
    /// Evaluate an action again immediately before protected handler admission.
    ///
    /// # Errors
    /// Denies absent/negative policy without invoking the protected closure.
    pub fn run<T>(
        &self,
        action: &ActionId,
        handler: impl FnOnce() -> T,
    ) -> Result<T, AccessDenied> {
        if !self
            .policy
            .is_some_and(|policy| policy.permits(self.context, action))
        {
            return Err(AccessDenied);
        }
        Ok(handler())
    }
    /// Project a bounded fixed candidate list into safe layout hints.
    ///
    /// # Errors
    /// Rejects duplicate/excess candidates; absence of a policy yields an empty list.
    pub fn project(&self, candidates: Vec<ActionId>) -> Result<AllowedActions, AccessDenied> {
        let candidates = AllowedActions::try_from(candidates)?;
        let actions = candidates
            .0
            .into_iter()
            .filter(|action| self.run(action, || ()).is_ok())
            .collect();
        Ok(AllowedActions(actions))
    }
}
