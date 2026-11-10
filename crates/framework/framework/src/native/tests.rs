//! Source-order proof using the real selection function and real constituent registrars.

use super::*;

/// Execute actual aggregate selection, rather than comparing two declared order constants.
#[test]
fn selected_order_matches_instrumented_source() {
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/source-order.json")).unwrap();
    TRACE.with(|trace| trace.borrow_mut().clear());
    let mut app = ServerBuilder::new();
    // G59 requires application-generated bindings for the selected session/API stages.
    static CLIENT: CompiledClient = CompiledClient {
        bindings: FrameworkBindings {
            session: Some(
                stackpress_framework_session::composite::register_with::<
                    models::stores::family::Models,
                >,
            ),
            api: Some(stackpress_framework_api::register_with::<models::stores::family::Models>),
        },
        build: "source-order",
        describe: Vec::new,
        models: &[],
    };
    register_with(&mut app, &CLIENT).unwrap();
    TRACE.with(|trace| {
        assert_eq!(
            serde_json::to_value(&*trace.borrow()).unwrap(),
            oracle["registration"]
        );
    });
}

#[allow(dead_code, missing_docs, unexpected_cfgs)]
#[path = "../../tests/fixtures/models/mod.rs"]
mod models;
