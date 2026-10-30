//! Source-order proof using the real selection function and real constituent registrars.

use super::*;

/// Execute actual aggregate selection, rather than comparing two declared order constants.
#[test]
fn selected_order_matches_instrumented_source() {
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/source-order.json")).unwrap();
    TRACE.with(|trace| trace.borrow_mut().clear());
    let mut app = ServerBuilder::new();
    register(&mut app).unwrap();
    TRACE.with(|trace| {
        assert_eq!(
            serde_json::to_value(&*trace.borrow()).unwrap(),
            oracle["registration"]
        );
    });
}
