//! Explicit test-only host admission, independent of public layout projections.
use stackpress_framework_preferences::{ActionId, HostPolicy};
use stackpress_lib::Request;

/// Explicitly test-only policy. Enabling this does not supply a real identity system.
pub(super) struct FixturePolicy;
/// Fresh authoritative facts read by the host adapter, absent from public seed.
pub(super) struct Facts {
    allowed: bool,
    method: String,
    origin: bool,
    csrf: bool,
}
impl HostPolicy<Facts> for FixturePolicy {
    /// Deny unauthorized callers; mutation additionally requires method/Origin/CSRF.
    fn permits(&self, facts: &Facts, action: &ActionId) -> bool {
        facts.allowed
            && match action.as_str() {
                "private.page" => facts.method == "GET",
                "private.mutate" => facts.method == "POST" && facts.origin && facts.csrf,
                _ => false,
            }
    }
}
/// Build fresh host facts for every policy evaluation.
pub(super) fn facts(req: &Request) -> Facts {
    Facts {
        allowed: req.headers.get("x-fixture-caller") == Some("allowed"),
        method: req.method.as_str().to_owned(),
        origin: req.headers.get("origin") == Some("https://preferences.invalid"),
        csrf: req.headers.get("x-fixture-csrf") == Some("fixture-csrf"),
    }
}
