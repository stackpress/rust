//! Compiled M1 generated selection; runtime API source actions remain independently usable.
#[allow(unexpected_cfgs)]
#[path = "../generated/mod.rs"]
pub mod generated;

/// External direct-use compilation against the actual source action constructors.
pub fn actions(
    engine: std::sync::Arc<stackpress_sql::Engine>,
) -> (
    generated::stores::application::ApplicationActions,
    generated::stores::session::SessionActions,
) {
    (
        <generated::stores::application::ApplicationModel as stackpress_framework_sql::source::SourceModel>::actions(engine.clone(), String::new(), 0),
        <generated::stores::session::SessionModel as stackpress_framework_sql::source::SourceModel>::actions(engine, String::new(), 0),
    )
}
/// The app's outer composite uses ordinary registrations, preserving #14 plugin identity.
pub fn register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()> {
    stackpress_framework_api::register_with::<generated::stores::family::Models>(app)?;
    stackpress_framework_api::sql::register_with::<generated::stores::family::Models>(app)
}
