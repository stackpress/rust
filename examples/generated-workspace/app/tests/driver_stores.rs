//! Repository integration for the generated application and three real adapters.
//! Package-local adapter suites retain scalar, transaction, error and lifecycle tests.

#[cfg(not(any(feature = "matrix", feature = "names")))]
#[path = "support/driver_stores/runner.rs"]
mod runner;

/// Alternate generated families have their own integration suites.
#[cfg(any(feature = "matrix", feature = "names"))]
fn main() {}

/// Expose dynamic opt-in discovery to nextest without treating skipped live cases as passes.
#[cfg(not(any(feature = "matrix", feature = "names")))]
fn main() {
    runner::main();
}
