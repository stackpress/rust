//! Generated module registry; regenerate instead of editing.

const _: () = assert!(crate::selection::ENABLED);
const _: () = assert!(crate::selection::REVISION == 1);

/// Public data without host dependencies.
pub mod contracts {
    /// Model Profile.
    pub mod profile;
}
/// Native capability implementation.
pub mod server {
    /// In-memory profiles plugin.
    pub mod profile;
}

/// Register the selected generated contributions.
///
/// # Errors
/// Returns plugin/host registration failures.
pub fn register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()> {
    app.plugin("profiles", server::profile::register)?;
    Ok(())
}
