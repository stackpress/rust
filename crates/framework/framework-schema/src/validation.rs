//! Bounded, value-free issues and canonical validation shared by native and browser inputs.

use crate::{DateValue, DatetimeValue, SchemaError, SchemaErrorKind, SchemaResult, TimeValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One canonical assertion; arguments are bounded schema literals, never rejected input.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRule {
    /// Stable canonical code without the is. prefix.
    pub code: String,
    /// Checked literal arguments.
    pub args: Vec<Value>,
    /// Whether a browser can evaluate this rule locally.
    pub client_checkable: bool,
    /// Stable safe message, without rejected values.
    pub message: String,
    /// Stable selected assertion implementation revision.
    pub definition_revision: String,
}
/// One bounded logical failure; no input value, SQL or private cause is serialized.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldIssue {
    /// Logical field/index path.
    pub path: Vec<String>,
    /// Canonical rule/error code.
    pub code: String,
    /// Stable public message.
    pub message: String,
}
/// At most 64 validation issues in declaration/traversal order.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ValidationErrors {
    /// Finite collected failures.
    pub issues: Vec<FieldIssue>,
}
impl ValidationErrors {
    /// Record an issue without allowing an unbounded error response.
    pub fn push(&mut self, path: &[String], code: &str) {
        if self.issues.len() < 64 {
            self.issues.push(FieldIssue {
                path: path
                    .iter()
                    .take(32)
                    .map(|part| part.chars().take(512).collect())
                    .collect(),
                code: code.into(),
                message: "field validation failed".into(),
            });
        }
    }
    /// Use an explicitly declared safe rule message without retaining rejected input.
    pub fn push_rule(&mut self, path: &[String], rule: &ValidationRule) {
        if self.issues.len() < 64 {
            self.push(path, &rule.code);
            if let Some(issue) = self.issues.last_mut() {
                issue.message = rule.message.clone();
            }
        }
    }
    /// Distinguish a successful validation from a bounded failure set.
    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }
}
/// Explicit build/runtime host providers. No default clock, randomness or remote lookup exists.
pub trait ValueProviders: Send + Sync {
    /// Return a typed generated JSON value for a registered provider and checked arguments.
    fn generate(&self, name: &str, args: &[Value]) -> SchemaResult<Value>;
    /// Evaluate a registered host validator; remote uniqueness is enforced by storage instead.
    fn validate(&self, name: &str, value: &Value, args: &[Value]) -> SchemaResult<bool>;
}
/// Explicit clock/provider inputs. Stored decoding omits relative-time input rules.
#[derive(Default)]
pub struct ValueContext<'a> {
    /// Current UTC instant, supplied once by the application for preparation/validation.
    pub now: Option<&'a DatetimeValue>,
    /// Compiled typed provider implementation selected by the host.
    pub providers: Option<&'a dyn ValueProviders>,
}
/// Prepared value source, distinct from a literal default.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueRule {
    /// Finite registered name, including now for the explicit clock.
    pub provider: String,
    /// Checked literal provider arguments.
    pub args: Vec<Value>,
    /// Stable provider revision in the semantic fingerprint.
    pub revision: String,
}
impl ValueRule {
    /// Resolve and check the declared nonnullable provider output before optional wrapping.
    pub fn resolve_for(
        &self,
        kind: &crate::ValueType,
        context: &ValueContext<'_>,
    ) -> SchemaResult<Value> {
        let value = self.resolve(context)?;
        if !crate::value_matches(kind, &value) {
            return Err(SchemaError::new(
                SchemaErrorKind::Invalid,
                "provider output type mismatch",
            ));
        }
        Ok(value)
    }
    /// Resolve exactly once before effects, using explicit inputs only.
    pub fn resolve(&self, context: &ValueContext<'_>) -> SchemaResult<Value> {
        if self.provider == "now" {
            return context
                .now
                .map(|now| Value::String(now.as_str().into()))
                .ok_or_else(missing_context);
        }
        context
            .providers
            .ok_or_else(missing_context)?
            .generate(&self.provider, &self.args)
    }
}
/// Refuse implicit ambient providers instead of fabricating time or randomness.
fn missing_context() -> SchemaError {
    SchemaError::new(
        SchemaErrorKind::Unsupported,
        "explicit clock or provider is required",
    )
}

/// Evaluate one previously normalized rule. The input never enters diagnostic text.
/// # Errors
/// Refuses unavailable explicit clock/provider context or a bounded regex compilation failure.
pub fn check_rule(
    rule: &ValidationRule,
    value: &Value,
    context: &ValueContext<'_>,
    stored: bool,
) -> SchemaResult<bool> {
    let text = value.as_str();
    let arg = rule.args.first();
    // Stored rows do not become corrupt merely because a time-relative creation rule ages.
    if stored && matches!(rule.code.as_str(), "future" | "past" | "present") {
        return Ok(true);
    }
    let count = |words: bool| {
        text.map(|s| {
            if words {
                s.split_whitespace().count()
            } else {
                s.chars().count()
            }
        })
        .map(|n| n as f64)
    };
    let compare = |actual: Option<f64>, kind: &str| {
        actual
            .zip(arg.and_then(Value::as_f64))
            .is_some_and(|(a, b)| match kind {
                "eq" => a == b,
                "gt" => a > b,
                "ge" => a >= b,
                "lt" => a < b,
                "le" => a <= b,
                _ => false,
            })
    };
    let result = match rule.code.as_str() {
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "integer" => value.as_i64().is_some(),
        "number" | "float" => crate::FiniteNumber::from_json(value).is_ok(),
        "object" => value.is_object(),
        "required" | "ne" => {
            !value.is_null()
                && match value {
                    Value::String(s) => !s.is_empty(),
                    Value::Array(v) => !v.is_empty(),
                    _ => true,
                }
        }
        "eq" => arg.is_some_and(|bound| scalar_equal(value, bound)),
        "neq" => arg.is_some_and(|bound| !scalar_equal(value, bound)),
        "option" => rule.args.iter().any(|bound| scalar_equal(value, bound)),
        "gt" | "ge" | "lt" | "le" => {
            let order = if let (Some(a), Some(b)) = (value.as_i64(), arg.and_then(Value::as_i64)) {
                Some(a.cmp(&b))
            } else if let (Some(a), Some(b)) = (text, arg.and_then(Value::as_str)) {
                if let (Ok(a), Ok(b)) = (DatetimeValue::parse(a), DatetimeValue::parse(b)) {
                    Some(a.unix_nanos().cmp(&b.unix_nanos()))
                } else {
                    Some(a.cmp(b))
                }
            } else {
                arg.and_then(|bound| numeric_order(value, bound))
            };
            order.is_some_and(|order| match rule.code.as_str() {
                "gt" => order.is_gt(),
                "ge" => !order.is_lt(),
                "lt" => order.is_lt(),
                _ => !order.is_gt(),
            })
        }
        "step" => {
            if let (Some(value), Some(step)) = (value.as_i64(), arg.and_then(Value::as_i64)) {
                step > 0 && value.checked_rem(step) == Some(0)
            } else {
                crate::FiniteNumber::from_json(value)
                    .ok()
                    .zip(arg.and_then(|value| crate::FiniteNumber::from_json(value).ok()))
                    .is_some_and(|(value, step)| {
                        let quotient = value.get() / step.get();
                        step.get() > 0.0
                            && quotient.is_finite()
                            && (quotient - quotient.round()).abs() <= 1e-10
                    })
            }
        }
        "ceq" | "cgt" | "cge" | "clt" | "cle" => compare(count(false), &rule.code[1..]),
        "weq" | "wgt" | "wge" | "wlt" | "wle" => compare(count(true), &rule.code[1..]),
        "lowercase" => text.is_some_and(|s| s == s.to_lowercase()),
        "uppercase" => text.is_some_and(|s| s == s.to_uppercase()),
        "slug" => text.is_some_and(|s| {
            !s.is_empty()
                && s.split('-').all(|part| {
                    !part.is_empty()
                        && part
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                })
        }),
        "hex" => text.is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit())),
        "regex" => {
            let pattern = arg.and_then(Value::as_str).unwrap_or("");
            let regex = regex::RegexBuilder::new(pattern)
                .size_limit(1024 * 1024)
                .dfa_size_limit(1024 * 1024)
                .build()
                .map_err(|cause| {
                    SchemaError::new(SchemaErrorKind::Invalid, "regex rule compilation failed")
                        .with_cause(cause)
                })?;
            text.is_some_and(|s| regex.is_match(s))
        }
        "url" => text.is_some_and(|s| url::Url::parse(s).is_ok()),
        "date" => text.is_some_and(|s| {
            DateValue::parse(s).is_ok()
                || TimeValue::parse(s).is_ok()
                || DatetimeValue::parse(s).is_ok()
        }),
        "future" | "past" | "present" => {
            let now = context.now.ok_or_else(missing_context)?;
            let order = text.and_then(|value| {
                if let Ok(value) = DatetimeValue::parse(value) {
                    Some(value.unix_nanos().cmp(&now.unix_nanos()))
                } else if let Ok(value) = DateValue::parse(value) {
                    Some(value.as_str().cmp(&now.as_str()[..10]))
                } else if let Ok(value) = TimeValue::parse(value) {
                    let utc = TimeValue::parse(&now.as_str()[11..now.as_str().len() - 1]).ok()?;
                    Some(value.as_str().cmp(utc.as_str()))
                } else {
                    None
                }
            });
            order.is_some_and(|order| match rule.code.as_str() {
                "future" => order.is_gt(),
                "past" => order.is_lt(),
                _ => order.is_eq(),
            })
        }
        // A unique preflight cannot establish concurrency safety. DB constraints remain authoritative.
        "unique" => true,
        _ => context
            .providers
            .ok_or_else(missing_context)?
            .validate(&rule.code, value, &rule.args)?,
    };
    Ok(result)
}

/// Compare mixed JSON numeric families without first rounding an i64 into an f64.
fn numeric_order(value: &Value, bound: &Value) -> Option<std::cmp::Ordering> {
    match (value.as_i64(), bound.as_i64()) {
        (Some(value), Some(bound)) => Some(value.cmp(&bound)),
        (Some(value), None) => integer_real_order(value, bound.as_f64()?),
        (None, Some(bound)) => {
            integer_real_order(bound, value.as_f64()?).map(std::cmp::Ordering::reverse)
        }
        (None, None) => value.as_f64()?.partial_cmp(&bound.as_f64()?),
    }
}
/// Truncate only the real operand after range checks; its fractional remainder breaks exact ties.
fn integer_real_order(integer: i64, real: f64) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    if !real.is_finite() {
        return None;
    }
    // The positive endpoint is exclusive: i64::MAX itself rounds up when cast to f64.
    if real >= 9_223_372_036_854_775_808.0 {
        return Some(Ordering::Less);
    }
    if real < -9_223_372_036_854_775_808.0 {
        return Some(Ordering::Greater);
    }
    let order = integer.cmp(&(real as i64));
    if order.is_eq() {
        0.0_f64.partial_cmp(&(real - real.trunc()))
    } else {
        Some(order)
    }
}

/// Exact Number input may arrive as an integer or a real; neither spelling changes equality.
fn scalar_equal(value: &Value, bound: &Value) -> bool {
    if value.is_number() && bound.is_number() {
        numeric_order(value, bound).is_some_and(std::cmp::Ordering::is_eq)
    } else {
        value == bound
    }
}
