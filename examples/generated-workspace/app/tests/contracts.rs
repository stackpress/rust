//! Native compilation of the same generated value/validation source consumed by the pure package.

use generated_workspace_app::generated::contracts;
#[path = "../../support/contracts.rs"]
mod cases;

/// Native and pure consumers both execute real generated defaults and validation.
#[test]
fn native_generated_contracts() {
    cases::prove_contracts();
    cases::prove_public_contracts();
}
