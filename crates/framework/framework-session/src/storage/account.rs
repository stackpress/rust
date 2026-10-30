//! Direct Auth signup/signin and source password assertions, without route/event policy.

use super::{ActionError, AuthActions, ProfileActions, Result, StoreQuery};
use crate::columns::Value;
use crate::columns::{self, Record};
use serde::{Deserialize, Serialize};
use stackpress_sql::Engine;
use std::sync::Arc;

/// Source password checks, ordered min/max/upper/lower/number/special.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PasswordConfig {
    /// Minimum JavaScript UTF-16 code-unit length; zero disables the check.
    pub min: Option<usize>,
    /// Maximum JavaScript UTF-16 code-unit length; zero disables the check.
    pub max: Option<usize>,
    /// Require at least one ASCII uppercase letter.
    pub upper: bool,
    /// Require at least one ASCII lowercase letter.
    pub lower: bool,
    /// Require at least one ASCII digit.
    pub number: bool,
    /// Require source punctuation or ECMAScript whitespace, including ordinary space.
    pub special: bool,
}
/// Source direct constructor options; no configured database seed fallback is applied here.
#[derive(Clone, Debug, Default)]
pub struct ActionOptions {
    /// Direct default is empty; the later host factory uses database.seed fallback abc123.
    pub seed: String,
    /// Optional source password assertion policy.
    pub password: PasswordConfig,
}
impl AuthActions {
    /// Source make(ctx) factory semantics after the native host resolves its Engine and
    /// config values at invocation time. Missing database.seed falls back to abc123;
    /// direct new/with_options still keep their source empty-seed behavior.
    pub fn make(
        engine: Arc<Engine>,
        database_seed: Option<String>,
        password: Option<PasswordConfig>,
    ) -> Self {
        Self::with_options(
            engine,
            ActionOptions {
                seed: database_seed.unwrap_or_else(|| "abc123".into()),
                password: password.unwrap_or_default(),
            },
        )
    }

    /// Compose the same engine/store with source options; no account routes are installed.
    pub fn with_options(engine: Arc<Engine>, options: ActionOptions) -> Self {
        let mut actions = Self::new(engine, options.seed);
        actions.password = options.password;
        actions
    }
    /// Source ProfileActions collaborator, using the same database seed and engine.
    pub fn profile(&self) -> ProfileActions {
        ProfileActions::new(self.engine.clone(), self.store.seed.clone())
    }

    /// Ordered source signup/password validation; errors accumulate across independent fields.
    pub fn assert(&self, input: &Record) -> Option<Record> {
        let mut errors = Record::new();
        if !input.get("name").is_some_and(columns::truthy) {
            errors.insert("name".into(), "Name is required".into());
        }
        if !["username", "email", "phone"]
            .iter()
            .any(|name| input.get(*name).is_some_and(columns::truthy))
        {
            errors.insert(
                "username".into(),
                "Username, email, or phone is required".into(),
            );
        } else if let Some(email) = input.get("email").filter(|value| columns::truthy(value))
            && !is_email(&columns::string(email))
        {
            errors.insert("email".into(), "Invalid email".into());
        }
        let secret = input.get("secret");
        let message = if !secret.is_some_and(columns::truthy) {
            Some("Password is required".into())
        } else {
            let secret = secret.map(columns::string).unwrap_or_default();
            let length = secret.encode_utf16().count();
            let config = &self.password;
            if let Some(min) = config.min.filter(|min| *min != 0 && length < *min) {
                Some(format!("Password must be at least {min} characters"))
            } else if let Some(max) = config.max.filter(|max| *max != 0 && length > *max) {
                Some(format!("Password must be less than {max} characters"))
            } else if config.upper
                && !secret
                    .chars()
                    .any(|character| character.is_ascii_uppercase())
            {
                Some("Password must contain at least one uppercase letter".into())
            } else if config.lower
                && !secret
                    .chars()
                    .any(|character| character.is_ascii_lowercase())
            {
                Some("Password must contain at least one lowercase letter".into())
            } else if config.number && !secret.chars().any(|character| character.is_ascii_digit()) {
                Some("Password must contain at least one number".into())
            } else if config.special
                && !secret.chars().any(|character| {
                    "!@#$%^&*(),.?\":{}|<>".contains(character) || space(character)
                })
            {
                Some("Password must contain at least one special character".into())
            } else {
                None
            }
        };
        if let Some(message) = message {
            errors.insert("secret".into(), message.into());
        }
        (!errors.is_empty()).then_some(errors)
    }

    /// Source type/token lookup, optional password comparison, consumed update and reload.
    /// Missing identifiers/secrets are String(undefined), preserving direct-call behavior.
    pub async fn signin(
        &self,
        kind: &str,
        input: &Record,
        password: bool,
    ) -> Result<Option<Record>> {
        let mut query = StoreQuery::eq("type", kind);
        query.eq.insert(
            "token".into(),
            input
                .get(kind)
                .map(columns::string)
                .unwrap_or_else(|| "undefined".into())
                .into(),
        );
        query.columns = vec!["*".into(), "profile.*".into()];
        let Some(record) = self.find(&query).await? else {
            return Err(ActionError::new(
                404,
                "Invalid Parameters",
                Record::from_iter([(kind.into(), "User Not Found".into())]),
            ));
        };
        if password {
            let secret = input
                .get("secret")
                .map(columns::string)
                .unwrap_or_else(|| "undefined".into());
            let hash = if secret.is_empty() {
                secret
            } else {
                crate::codecs::hash(&secret)
            };
            if record.get("secret").map(columns::string).as_deref() != Some(&hash) {
                return Err(ActionError::new(
                    401,
                    "Invalid Parameters",
                    Record::from_iter([("secret".into(), "Invalid Password".into())]),
                ));
            }
        }
        self.update(
            &StoreQuery::eq("id", record.get("id").cloned().unwrap_or(Value::Null)),
            &mut Record::from_iter([("consumed".into(), columns::date(columns::now()))]),
        )
        .await?;
        self.find(&query).await
    }

    /// Source sequential profile/email/phone/username creation, deliberately without an outer
    /// transaction. A later failure leaves earlier records in place. Missing phone succeeds G44.
    pub async fn signup(&self, input: &Record) -> Result<Record> {
        if let Some(errors) = self.assert(input) {
            return Err(ActionError::new(400, "Invalid Parameters", errors));
        }
        let profile_input = Record::from_iter([
            (
                "name".into(),
                input.get("name").cloned().unwrap_or(Value::Null),
            ),
            (
                "type".into(),
                input
                    .get("type")
                    .filter(|value| columns::truthy(value))
                    .cloned()
                    .unwrap_or_else(|| "person".into()),
            ),
            (
                "roles".into(),
                input
                    .get("roles")
                    .filter(|value| columns::truthy(value))
                    .cloned()
                    .unwrap_or_else(|| Value::Array(Vec::new())),
            ),
        ]);
        let mut profile = self.profile().create(&profile_input).await?;
        let mut auth = Record::new();
        for kind in ["email", "phone", "username"] {
            if let Some(identifier) = input.get(kind).filter(|value| columns::truthy(value)) {
                let auth_input = Record::from_iter([
                    (
                        "profileId".into(),
                        profile.get("id").cloned().unwrap_or(Value::Null),
                    ),
                    ("type".into(), kind.into()),
                    ("token".into(), columns::string(identifier).into()),
                    (
                        "secret".into(),
                        input
                            .get("secret")
                            .map(columns::string)
                            .unwrap_or_else(|| "undefined".into())
                            .into(),
                    ),
                ]);
                match self.create(&auth_input).await {
                    Ok(record) => {
                        auth.insert(kind.into(), Value::Object(record));
                    }
                    Err(mut error) => {
                        if let Some(token) = error.errors.get("token").cloned() {
                            error.errors.insert(kind.into(), token);
                        }
                        return Err(error);
                    }
                }
            }
        }
        profile.insert("auth".into(), Value::Object(auth));
        Ok(profile)
    }
}

/// ECMAScript \s, which differs from Rust's general Unicode whitespace set.
fn space(character: char) -> bool {
    matches!(character,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')
}

/// Execute the source email regex's local lookbehind/lookahead rules on UTF-16 units.
/// Rust regex has no lookaround; this finite scanner is a mechanics substitution for that
/// exact pattern. The remaining domain regex is unchanged, with its length guard explicit.
pub fn is_email(value: &str) -> bool {
    let Some((local, domain)) = value.rsplit_once('@') else {
        return false;
    };
    let local: Vec<u16> = local.encode_utf16().collect();
    let allowed = |unit: u16| !matches!(unit,0..=32|127..=255|64|44|34|91|93|92|46);
    let escaped = |unit: u16| !allowed(unit) && unit != 46;
    let valid = if local.first() == Some(&34) && local.last() == Some(&34) {
        (3..=64).contains(&local.len())
            && local[1..local.len() - 1]
                .iter()
                .enumerate()
                .all(|(index, unit)| *unit != 34 || local[index] == 92)
    } else if local.len() <= 2 {
        !local.is_empty() && local.iter().all(|unit| allowed(*unit))
    } else if local.len() <= 64 {
        let first = allowed(local[0]) || (local[0] == 92 && escaped(local[1]));
        let last = allowed(*local.last().unwrap_or(&0))
            || (local[local.len() - 2] == 92 && escaped(*local.last().unwrap_or(&0)));
        first
            && last
            && (1..local.len() - 1).all(|index| {
                allowed(local[index])
                    || (local[index - 1] == 92 && escaped(local[index]))
                    || (local[index] == 92 && escaped(local[index + 1]))
                    || (local[index] == 46 && local[index + 1] != 46)
            })
    } else {
        false
    };
    if !valid {
        return false;
    }
    // The bracketed IPv4 branch has no DNS-length lookahead in the original.
    let domain_pattern = r"^(?:(?:[a-zA-Z0-9][a-zA-Z0-9-]{1,61}[a-zA-Z0-9]\.?|[a-zA-Z0-9]\.?)+\.(?:xn--[a-zA-Z0-9]+|[a-zA-Z]{2,6})|\[(?:[0-1]?[0-9]?[0-9]|2[0-4][0-9]|25[0-5])(?:\.(?:[0-1]?[0-9]?[0-9]|2[0-4][0-9]|25[0-5])){3}\])$";
    if !domain.starts_with('[') && domain.encode_utf16().count() >= 64 {
        return false;
    }
    regex::Regex::new(domain_pattern).is_ok_and(|regex| regex.is_match(domain))
}
