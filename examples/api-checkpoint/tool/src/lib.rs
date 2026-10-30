//! Compiled M1 generated selection; runtime API source actions remain independently usable.
#[path = "../generated/mod.rs"]
pub mod generated;

/// External direct-use compilation against the actual source action constructors.
pub fn actions(
    engine: std::sync::Arc<stackpress_sql::Engine>,
) -> (
    stackpress_framework_api::models::ApplicationActions,
    stackpress_framework_api::models::SessionActions,
) {
    (
        stackpress_framework_api::models::ApplicationActions::new(engine.clone(), ""),
        stackpress_framework_api::models::SessionActions::new(engine, ""),
    )
}
/// The app's outer composite uses ordinary registrations, preserving #14 plugin identity.
pub fn register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()> {
    stackpress_framework_api::register(app)?;
    stackpress_framework_api::sql::register(app)
}
