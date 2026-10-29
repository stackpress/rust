//! Exercise the shared custom-harness argument contract without any live backend.
#[path = "../../test-support/harness.rs"]
mod harness;

/// The public entry points are usable by a conventional test target too.
#[test]
fn entry_points_do_not_execute_fixtures() {
    let selection = harness::Harness::from_env();
    assert!(!selection.discover(std::iter::empty()));
}
