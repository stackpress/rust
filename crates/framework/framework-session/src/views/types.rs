//! Explicit public form/page data. Native services/configuration are never serialized.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Complete source view inventory; browser and native render the same variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Page {
    /// Registration form.
    Signup,
    /// Configured method chooser.
    Signin,
    /// Username/password.
    Username,
    /// Email password, OTP and magic send.
    Email,
    /// Phone/password.
    Phone,
    /// Emailed one-time code.
    Otp,
    /// Magic link error fallback.
    Link,
    /// Authenticator challenge.
    TwoFactor,
    /// Personal information.
    Account,
    /// Editable information and linked identifiers.
    Update,
    /// Account security links.
    Security,
    /// Change current password.
    Password,
    /// Profile export warning.
    Export,
    /// Account removal confirmation.
    Remove,
    /// Display/enroll authenticator secret and QR.
    Enroll,
    /// Authenticator removal confirmation.
    RemoveFactor,
}
impl Page {
    /// Human-readable source title, also used by each exported Head.
    pub fn title(self) -> &'static str {
        match self {
            Self::Signup => "Sign Up",
            Self::Signin => "Sign In Options",
            Self::Username => "Username Sign In",
            Self::Email => "Email Sign In",
            Self::Phone => "Phone Sign In",
            Self::Otp => "One Time Pin Code",
            Self::Link => "Magic Link",
            Self::TwoFactor | Self::Enroll => "Two-Factor Authentication",
            Self::Account | Self::Update => "Personal Information",
            Self::Security => "Security Settings",
            Self::Password => "Change Password",
            Self::Export => "Export User Data",
            Self::Remove => "Delete Account",
            Self::RemoveFactor => "Remove Two-Factor Authentication",
        }
    }
    /// Distinguish source panel pages from blank auth layouts.
    pub fn account(self) -> bool {
        matches!(
            self,
            Self::Account
                | Self::Update
                | Self::Security
                | Self::Password
                | Self::Export
                | Self::Remove
                | Self::Enroll
                | Self::RemoveFactor
        )
    }
    /// Source document descriptions, separate from visible headings.
    pub fn description(self) -> &'static str {
        match self {
            Self::Signup => "",
            Self::Signin => "Choose a sign-in method to continue.",
            Self::Username => "Sign in with your username.",
            Self::Email => "Sign in with your email address.",
            Self::Phone => "Sign in with your phone number.",
            Self::Otp => "Enter the OTP sent to your email address.",
            Self::Link => "Sign in with a magic link sent to your email address.",
            Self::TwoFactor => "Enter the 6-digit code from your authenticator app.",
            Self::Account => "View your personal information.",
            Self::Update => "View and update your personal information.",
            Self::Security => "Manage your account security settings.",
            Self::Password => "Change the password used to access your account.",
            Self::Export => "Export a copy of your account data.",
            Self::Remove => "Confirm deletion of your account.",
            Self::Enroll => "Manage two-factor authentication for your account.",
            Self::RemoveFactor => "Confirm removal of two-factor authentication.",
        }
    }
}
/// Browser-safe copy of the configured password checklist; no hashing/seed lives here.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PasswordRules {
    /// UTF-16 minimum; zero disables.
    pub min: Option<usize>,
    /// UTF-16 maximum; zero disables.
    pub max: Option<usize>,
    /// Uppercase composition.
    pub upper: bool,
    /// Lowercase composition.
    pub lower: bool,
    /// Numeric composition.
    pub number: bool,
    /// UI excludes whitespace, intentionally differing from source server assertion.
    pub special: bool,
}
/// Source method-menu entry, with ordinary link semantics.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Menu {
    /// `footer` selects the lower link group.
    #[serde(rename = "type")]
    pub type_: Option<String>,
    /// Optional browsing-context target.
    pub target: Option<String>,
    /// Visible name.
    pub name: String,
    /// Source icon name mapped to the established Lucide family.
    pub icon: Option<String>,
    /// Configured destination.
    pub path: String,
}
/// Public auth configuration projection; transport/signing/database options are absent.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Source missing base fallback; explicit empty remains empty.
    pub base: String,
    /// Configured method choices.
    pub menu: Vec<Menu>,
    /// Optional source signup strength meter.
    pub password: Option<PasswordRules>,
    /// Brand name when configured.
    pub brand: Option<String>,
    /// Optional brand image.
    pub logo: Option<String>,
    /// Favicon; missing uses /favicon.ico.
    pub favicon: String,
    /// Configured CSRF input key.
    pub csrf_name: String,
    /// Deliberately public form token.
    pub csrf_token: String,
    /// Explicit additional stylesheet links.
    pub styles: Vec<String>,
}
impl Default for Config {
    /// Source page defaults; no sign-in menu is invented.
    fn default() -> Self {
        Self {
            base: "/auth".into(),
            menu: Vec::new(),
            password: None,
            brand: None,
            logo: None,
            favicon: "/favicon.ico".into(),
            csrf_name: "csrf".into(),
            csrf_token: String::new(),
            styles: Vec::new(),
        }
    }
}
/// Serializable source-shaped input/results/errors, consumed identically by SSR and browser.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PageProps {
    /// Explicit public config allowlist.
    pub config: Config,
    /// Request pathname only.
    pub path: String,
    /// Original HTTP method.
    pub method: String,
    /// Aggregate source request data; callers deliberately select this public payload.
    pub input: Value,
    /// Source-selected results, including intentional enrollment secrets/QR.
    pub results: Value,
    /// Source response status, zero when unset.
    pub code: u16,
    /// Source message.
    pub error: String,
    /// Source field messages.
    pub errors: Value,
    /// Acknowledged theme; absence remains unset with a Light no-JS fallback.
    pub theme: Option<String>,
    /// Optional checked phrase dictionary, shared with the existing root-local language API.
    pub language: Option<stackpress_leptos::language::Language>,
}
impl Default for PageProps {
    /// Empty source page; native projection is explicit, never a service serialization.
    fn default() -> Self {
        Self {
            config: Config::default(),
            path: "/".into(),
            method: "GET".into(),
            input: json!({}),
            results: Value::Null,
            code: 0,
            error: String::new(),
            errors: json!({}),
            theme: None,
            language: None,
        }
    }
}
impl PageProps {
    /// Missing-only request fallback, retaining explicitly empty strings.
    pub fn input(&self, key: &str, fallback: &str) -> String {
        text(self.input.get(key)).unwrap_or_else(|| fallback.into())
    }
    /// Signup spreads results then request data; request wins, including null.
    pub fn bound(&self, key: &str) -> String {
        text(self.input.get(key).or_else(|| self.results.get(key))).unwrap_or_default()
    }
    /// Source account auth-map token display.
    pub fn identifier(&self, key: &str) -> String {
        text(
            self.results
                .get("auth")
                .and_then(|auth| auth.get(key))
                .and_then(|auth| auth.get("token")),
        )
        .unwrap_or_default()
    }
}
/// Finite JSON-to-form text; raw HTML is never accepted by a component.
pub(crate) fn text(value: Option<&Value>) -> Option<String> {
    value.map(|value| match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        _ => value.to_string(),
    })
}
