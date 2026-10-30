//! Source weighted signup checklist; presentation rules intentionally differ from server.
use super::PasswordRules;

/// One source checklist rule; score/pass behavior is explicit rather than a serialized closure.
#[derive(Clone, Debug)]
pub struct PasswordCheck {
    /// Stable source identifier.
    pub id: &'static str,
    /// Visible failure text.
    pub label: String,
    /// Source score weight (max-min can be negative).
    pub weight: f64,
    /// Rule threshold for min/max.
    threshold: usize,
    /// Lower threshold used by max's weighted score.
    minimum: usize,
}
impl PasswordCheck {
    /// Source pass predicate, using JavaScript UTF-16 length.
    pub fn pass(&self, value: &str) -> bool {
        let len = value.encode_utf16().count();
        match self.id {
            "min" => len >= self.threshold,
            "max" => len <= self.threshold,
            "upper" => value.chars().any(|c| c.is_ascii_uppercase()),
            "lower" => value.chars().any(|c| c.is_ascii_lowercase()),
            "number" => value.chars().any(|c| c.is_ascii_digit()),
            _ => value
                .chars()
                .any(|c| !c.is_ascii_alphanumeric() && !js_space(c)),
        }
    }
    /// Source partial length credit, without clamping a configured score.
    pub fn score(&self, value: &str) -> f64 {
        let len = value.encode_utf16().count();
        match self.id {
            "min" => len.min(self.threshold) as f64,
            "max" => {
                if len < self.minimum || len > self.threshold {
                    0.0
                } else {
                    len as f64 - self.minimum as f64
                }
            }
            _ => {
                if self.pass(value) {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}
/// ECMAScript whitespace for the original UI regex's `\s` class.
fn js_space(c: char) -> bool {
    matches!(c,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')
}
/// Assemble the source min/max/composition checks in display order.
pub fn password_checklist(rules: &PasswordRules) -> Vec<PasswordCheck> {
    let mut checks = Vec::new();
    if let Some(min) = rules.min.filter(|value| *value != 0) {
        checks.push(PasswordCheck {
            id: "min",
            label: format!("At least {min} characters"),
            weight: min as f64,
            threshold: min,
            minimum: 0,
        });
    }
    if let Some(max) = rules.max.filter(|value| *value != 0) {
        let min = rules.min.unwrap_or(0);
        checks.push(PasswordCheck {
            id: "max",
            label: format!("No more than {max} characters"),
            weight: max as f64 - min as f64,
            threshold: max,
            minimum: min,
        });
    }
    for (enabled, id, label) in [
        (rules.upper, "upper", "Uppercase letter (A–Z)"),
        (rules.lower, "lower", "Lowercase letter (a–z)"),
        (rules.number, "number", "Number (0–9)"),
        (rules.special, "special", "Special character (!@#$…) "),
    ] {
        if enabled {
            checks.push(PasswordCheck {
                id,
                label: label.trim_end().into(),
                weight: 1.0,
                threshold: 0,
                minimum: 0,
            });
        }
    }
    checks
}
/// Source result preserves fractional arithmetic and floor, including unusual rule weights.
#[derive(Clone, Debug)]
pub struct PasswordEvaluation {
    /// Failed rule identifiers and labels in source order.
    pub failed: Vec<(&'static str, String)>,
    /// Source Math.floor percentage; may be NaN for a zero-weight configured set.
    pub score: f64,
}
/// Evaluate without introducing a server password policy or a new minimum.
pub fn evaluate_password(secret: &str, checks: &[PasswordCheck]) -> PasswordEvaluation {
    if secret.is_empty() || checks.is_empty() {
        return PasswordEvaluation {
            failed: Vec::new(),
            score: 0.0,
        };
    }
    let failed = checks
        .iter()
        .filter(|check| !check.pass(secret))
        .map(|check| (check.id, check.label.clone()))
        .collect();
    let score = (checks.iter().map(|check| check.score(secret)).sum::<f64>()
        / checks.iter().map(|check| check.weight).sum::<f64>()
        * 100.0)
        .floor();
    PasswordEvaluation { failed, score }
}
