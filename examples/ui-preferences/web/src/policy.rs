//! Explicit test-only host admission, independent of public layout projections.
use stackpress_framework_view::access::{ActionId, HostPolicy};
use stackpress_lib::Request;

/// Explicitly test-only policy. Enabling this does not supply a real identity system.
pub(super) struct FixturePolicy;
/// Fresh authoritative facts read by the host adapter, absent from public seed.
pub(super) struct Facts {
    /// Test-harness caller flag, never authentication supplied by the framework.
    allowed: bool,
    /// Current normalized method rechecked at each protected exposure.
    method: String,
    /// Exact compiled fixture origin matched on this request.
    origin: bool,
    /// Fixture marker; deployable hosts must supply real session validation.
    csrf: bool,
}
impl HostPolicy<Facts> for FixturePolicy {
    /// Deny unauthorized callers; mutation additionally requires method/Origin/CSRF.
    fn permits(&self, facts: &Facts, action: &ActionId) -> bool {
        // Missing fixture identity denies; mutations also require current Origin and CSRF.
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
    // This header-based fixture is selected only by explicit host configuration.
    Facts {
        allowed: req.headers.get("x-fixture-caller") == Some("allowed"),
        method: req.method.as_str().to_owned(),
        origin: req.headers.get("origin") == Some("https://preferences.invalid"),
        csrf: req.headers.get("x-fixture-csrf") == Some("fixture-csrf"),
    }
}
