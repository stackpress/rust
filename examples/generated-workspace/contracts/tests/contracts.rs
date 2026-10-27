//! Actual generated validation/Serde execution with no SQL, host or build-tool dependency.

use generated_workspace_contracts::contracts;
#[path = "../../support/contracts.rs"]
mod cases;

/// Run the exact same contract rules consumed by the native app.
#[test]
fn pure_generated_contracts() {
    cases::prove_contracts();
    cases::prove_public_contracts();
}
