//! Owned JWT instances with original per-instance memoization and live configured grants.

use crate::{SessionConfig, SessionData, SessionPermission, permissions};
use serde_json::{Value, json};
use std::{
    error::Error,
    fmt,
    sync::{Arc, OnceLock, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

/// Fallible native mechanism; invalid JWT data itself is represented by None.
pub type SessionResult<T> = Result<T, SessionError>;
/// Native signing/configuration/pattern failure without token or seed in Display.
#[derive(Debug)]
pub struct SessionError(pub(crate) &'static str);
impl fmt::Display for SessionError {
    /// Display authored context only.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for SessionError {}
/// Explicit clock seam; source NumericDates are seconds, not durations.
pub trait Clock: Send + Sync {
    /// Return current seconds since Unix epoch; issued-at/verification floor to whole seconds.
    fn now(&self) -> f64;
}
/// Native system clock, with no network/resource ownership.
struct SystemClock;
impl Clock for SystemClock {
    /// Match Date.now seconds at the JWT boundary.
    fn now(&self) -> f64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs_f64(),
            Err(before_epoch) => -before_epoch.duration().as_secs_f64(),
        }
    }
}
/// Configuration is read synchronously and never held across a suspension point.
struct Settings {
    /// Current cookie/seed/grant settings, cloned before invoking host code.
    config: SessionConfig,
    /// Zero omits an added exp; nonzero is an absolute NumericDate.
    expires: f64,
}
/// One host's settings and clock, explicitly shared by loaded Session instances.
struct Runtime {
    /// Brief synchronous snapshots only; no guard crosses await or a host callback.
    settings: RwLock<Settings>,
    /// Host-owned Send+Sync clock, not a global current invocation.
    clock: Arc<dyn Clock>,
}
/// Configurable source Session constructor, without a SQL/revocation store.
#[derive(Clone)]
pub struct Service {
    /// Shared constructor settings; loaded instances own their independent caches.
    runtime: Arc<Runtime>,
}
impl Default for Service {
    /// Keep key=session, seed=abc123, empty effective access and no added exp.
    fn default() -> Self {
        Self::new(SessionConfig::default())
    }
}
impl Service {
    /// Construct a configured direct mechanism without selecting a plugin or guard.
    pub fn new(config: SessionConfig) -> Self {
        Self::with_clock(config, Arc::new(SystemClock))
    }
    /// Supply a clock for deterministic replay/proof without changing lifetime policy.
    pub fn with_clock(config: SessionConfig, clock: Arc<dyn Clock>) -> Self {
        Self {
            runtime: Arc::new(Runtime {
                settings: RwLock::new(Settings {
                    config,
                    expires: 0.0,
                }),
                clock,
            }),
        }
    }
    /// Read an owned configuration snapshot, never holding a lock across caller work.
    pub fn config(&self) -> SessionConfig {
        self.runtime
            .settings
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .config
            .clone()
    }
    /// Change key/seed/access for subsequent operations; existing verified data stays cached.
    pub fn configure(&self, config: SessionConfig) {
        self.runtime
            .settings
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .config = config;
    }
    /// Set an absolute exp NumericDate; zero leaves an input exp claim untouched.
    pub fn set_expires(&self, expires: f64) {
        self.runtime
            .settings
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .expires = expires;
    }
    /// Sign object claims with HS256 and issued-at, preserving optional exp by source rules.
    /// # Errors
    /// Returns signing/invalid NumericDate errors, without inventing a secret-strength check.
    pub async fn create(&self, data: SessionData) -> SessionResult<String> {
        // Copy settings before invoking the host clock: a clock may reenter configure.
        let (seed, expires) = {
            let settings = self
                .runtime
                .settings
                .read()
                .unwrap_or_else(|p| p.into_inner());
            (settings.config.seed.clone(), settings.expires)
        };
        crate::jwt::sign(data, &seed, expires, self.runtime.clock.now().floor())
    }
    /// Load an owned token with its own cache, not shared by request identity or token text.
    pub fn load(&self, token: impl Into<String>) -> Session {
        Session {
            token: token.into(),
            runtime: self.runtime.clone(),
            data: OnceLock::new(),
        }
    }
}
/// One immutable token plus memoized verified data (including memoized null).
///
/// A newly loaded object verifies independently. No request-local carrier is needed.
///
/// ```compile_fail
/// use stackpress_framework_session::{Service, Session};
/// let original = Service::default().load("token");
/// let duplicate: Session = original.clone(); // Memoization ownership is not implicitly cloned.
/// ```
pub struct Session {
    /// Raw source token; deliberately available to authorization consumers.
    token: String,
    /// Explicit host configuration owner, never a process-global current caller.
    runtime: Arc<Runtime>,
    /// Initialized once, synchronously, including every verification failure as None.
    data: OnceLock<Option<SessionData>>,
}
impl Session {
    /// Borrow the readonly source token; changing a token requires loading a new instance.
    pub fn token(&self) -> &str {
        &self.token
    }
    /// Verify at most once. All verification/parse errors become cached None as source does.
    pub async fn data(&self) -> Option<&SessionData> {
        // OnceLock stores both successful object data and null; no executor-owned mutex.
        self.data
            .get_or_init(|| {
                let config = self
                    .runtime
                    .settings
                    .read()
                    .unwrap_or_else(|p| p.into_inner())
                    .config
                    .clone();
                crate::jwt::verify(&self.token, &config.seed, self.runtime.clock.now().floor())
            })
            .as_ref()
    }
    /// Report verification-null, rather than requiring a nonzero/missing id field.
    pub async fn guest(&self) -> bool {
        self.data().await.is_none()
    }
    /// Merge Guest defaults, claims, raw token and effective grants in source order.
    /// # Errors
    /// Returns source-invalid roles or unsupported permission expression errors.
    pub async fn authorization(&self) -> SessionResult<SessionData> {
        let mut result =
            SessionData::from_iter([("id".into(), json!(0)), ("roles".into(), json!(["GUEST"]))]);
        if let Some(data) = self.data().await {
            result.extend(data.clone());
        }
        // Explicit projection replaces any same-named application claims, matching source spread.
        result.insert("token".into(), json!(self.token));
        result.insert(
            "permits".into(),
            serde_json::to_value(self.permits().await?)
                .map_err(|_| SessionError("permission serialization failed"))?,
        );
        Ok(result)
    }
    /// Gather current configured grants, deduplicating events by value and routes by identity.
    /// # Errors
    /// Truthy non-array roles fail as the source roles.map operation does.
    pub async fn permits(&self) -> SessionResult<Vec<SessionPermission>> {
        let data = self.data().await;
        // Empty arrays/objects are truthy in JS; only falsey roles select Guest.
        let fallback = json!(["GUEST"]);
        let roles = data
            .and_then(|d| d.get("roles"))
            .filter(|v| truthy(v))
            .unwrap_or(&fallback);
        let roles = roles
            .as_array()
            .ok_or(SessionError("session roles must support map"))?;
        let config = self
            .runtime
            .settings
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .config
            .clone();
        let mut result = Vec::new();
        for role in roles {
            for grant in config
                .access
                .as_ref()
                .and_then(|a| a.get(&property_key(role)))
                .into_iter()
                .flatten()
            {
                // Preserve first occurrence: equal separate route objects remain distinct.
                if !result.iter().any(|prior| same_grant(prior, grant)) {
                    result.push(grant.clone());
                }
            }
        }
        Ok(result)
    }
    /// Require every requested permit to match some grant; an empty request is always true.
    /// # Errors
    /// Preserves invalid-role/expression failure rather than silently treating it as denial.
    pub async fn can(&self, permits: &[SessionPermission]) -> SessionResult<bool> {
        if permits.is_empty() {
            return Ok(true);
        }
        let grants = self.permits().await?;
        for permit in permits {
            if !permissions::match_any(permit, &grants)
                .map_err(|_| SessionError("permission expression failed"))?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
/// JavaScript truthiness of JSON values: empty arrays/objects are true.
fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(v) => *v,
        Value::Number(v) => v.as_f64() != Some(0.0),
        Value::String(v) => !v.is_empty(),
        _ => true,
    }
}
/// Source property-key coercion for role-map lookup.
fn property_key(value: &Value) -> String {
    match value {
        Value::String(v) => v.clone(),
        Value::Object(_) => "[object Object]".into(),
        Value::Array(v) => v
            .iter()
            .map(|v| {
                if v.is_null() {
                    String::new()
                } else {
                    property_key(v)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        _ => value.to_string(),
    }
}
/// Source indexOf compares strings by value, route objects by reference.
fn same_grant(a: &SessionPermission, b: &SessionPermission) -> bool {
    match (a, b) {
        (SessionPermission::Event(a), SessionPermission::Event(b)) => a == b,
        (SessionPermission::Route(a), SessionPermission::Route(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}

#[cfg(feature = "server")]
impl Session {
    /// Read the current cookie key for the source save operation, outside any await.
    pub(crate) fn service_key(&self) -> String {
        self.runtime
            .settings
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .config
            .key
            .clone()
    }
}
