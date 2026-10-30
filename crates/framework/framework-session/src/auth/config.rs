//! Auth-owned configuration and explicit native reads of other packages' source keys.

use crate::{Clock, columns, storage::PasswordConfig};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_lib::{Error, Result};
use std::sync::Arc;

/// Optional source auth section, including its falsy deactivation spellings.
/// Empty objects stay active; false/null/zero/empty string deactivate routes only.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct AuthSection(pub Option<AuthConfig>);
impl<'de> Deserialize<'de> for AuthSection {
    /// Decode source falsy values before applying typed config mechanics to active objects.
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let inactive = match &value {
            Value::Null | Value::Bool(false) => true,
            Value::Number(number) => number.as_f64() == Some(0.0),
            Value::String(text) => text.is_empty(),
            _ => false,
        };
        if inactive {
            Ok(Self(None))
        } else {
            serde_json::from_value(value)
                .map(|config| Self(Some(config)))
                .map_err(serde::de::Error::custom)
        }
    }
}

/// Source auth section. Presence activates routes; events are registered independently.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// Source route base; most processes use `/auth` when missing or empty.
    pub base: Option<String>,
    /// Signout page fallback, used only when redirect_uri is absent.
    pub redirect: Option<String>,
    /// Source extension object; it does not install an additional global gate.
    #[serde(rename = "2fa")]
    pub two_factor: Option<Value>,
    /// Source CAPTCHA extension; no built-in provider is implied.
    pub captcha: Option<Value>,
    /// Sender checked after rotating/reloading a passwordless challenge.
    pub email: Option<AuthEmailConfig>,
    /// Signup defaults; request roles overwrite these, including an empty array.
    pub roles: Vec<String>,
    /// View menu configuration, retained for M3 presentation integration.
    pub menu: Vec<AuthMenuConfig>,
    /// Source ordered password assertions.
    pub password: PasswordConfig,
}
impl AuthConfig {
    /// Source `auth.base || '/auth'`.
    pub fn base(&self) -> &str {
        self.base
            .as_deref()
            .filter(|base| !base.is_empty())
            .unwrap_or("/auth")
    }
    /// OTP/link final handoff uses literal auth.base, without the usual fallback.
    pub(crate) fn literal_base(&self) -> &str {
        self.base.as_deref().unwrap_or("undefined")
    }
}
/// Source auth email identity, independent of transport selection.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthEmailConfig {
    /// Display name, required and nonempty at the source send branch.
    pub name: String,
    /// Address, required and nonempty at the source send branch.
    pub address: String,
}
/// Source sign-in chooser entry; presentation is owned by M3.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthMenuConfig {
    /// Optional footer marker.
    #[serde(rename = "type")]
    pub type_: Option<String>,
    /// Link target.
    pub target: Option<String>,
    /// Display name.
    pub name: String,
    /// Optional icon name.
    pub icon: Option<String>,
    /// Link destination.
    pub path: String,
}

/// Source Math.random substitution receives a fraction in [0,1), preserving floor/padding.
pub type RandomFraction = Arc<dyn Fn() -> Result<f64> + Send + Sync>;

/// Native resource mapping for source keys owned by other plugins.
///
/// Hosts register this optional service during Config. Map database.seed, email presence,
/// host/server.port and brand.name here rather than claiming those plugins' typed sections.
/// Missing service uses the exact source fallbacks. This adapter and its clock/random
/// callbacks are not serializable or copied to page props.
///
/// ```compile_fail
/// use stackpress_framework_session::auth::Environment;
/// // The native database seed/clock/runtime adapter is not an SSR or browser DTO.
/// let _ = serde_json::to_value(Environment::default());
/// ```
#[derive(Clone)]
pub struct Environment {
    /// database.seed; missing uses abc123, explicit empty stays empty.
    pub database_seed: Option<String>,
    /// Whether ctx.config('email') is truthy; transport remains the email-send event's owner.
    pub email_configured: bool,
    /// Source host fallback http://localhost:<server.port>.
    pub host: Option<String>,
    /// Source server.port fallback 3000; this never opens a listener.
    pub port: u16,
    /// Source brand.name fallback Stackpress.
    pub brand: String,
    /// Explicit local Date.toString UTC offset, shared with the SQL schema host setup.
    pub offset_minutes: i32,
    /// Explicit local Date.toString zone label.
    pub zone_name: String,
    /// Epoch seconds (the existing Clock contract); no mutable lock is held while invoking it.
    pub clock: Arc<dyn Clock>,
    /// Source random fraction provider, independently replaceable in finite proofs.
    pub random: RandomFraction,
}
/// Native source Date.now fallback.
struct SystemClock;
impl Clock for SystemClock {
    /// Read wall time as epoch milliseconds, preserving the existing Clock contract.
    fn now(&self) -> f64 {
        columns::now() as f64 / 1000.0
    }
}
impl Default for Environment {
    /// Use source config defaults; OS randomness supplies the native Math.random mechanism.
    fn default() -> Self {
        Self {
            database_seed: None,
            email_configured: false,
            host: None,
            port: 3000,
            brand: "Stackpress".into(),
            offset_minutes: 0,
            zone_name: "Coordinated Universal Time".into(),
            clock: Arc::new(SystemClock),
            random: Arc::new(|| {
                // Use 53 random bits for a representable fraction strictly below one.
                let mut bytes = [0; 8];
                SystemRandom::new()
                    .fill(&mut bytes)
                    .map_err(|_| Error::new("random generation failed"))?;
                Ok((u64::from_be_bytes(bytes) >> 11) as f64 / (1_u64 << 53) as f64)
            }),
        }
    }
}
