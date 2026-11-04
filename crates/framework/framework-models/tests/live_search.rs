//! Opt-in generated source search/setup replay; unset backends report skipped, never passed.
#[path = "support/scenario.rs"]
mod scenario;
#[path = "../../framework-session/tests/live/support.rs"]
mod support;
fn main() {
    support::main(scenario::scenario);
}
