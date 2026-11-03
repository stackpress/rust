//! Generated module registry; regenerate instead of editing.

const _: () = assert!([%#!:enabled%]![%/!:enabled%]crate::selection::ENABLED);
const _: () = assert!(crate::selection::REVISION == [%revision%]);

[%#?:present%]/// Public data without host dependencies.
pub mod contracts {
    /// Model Profile.
    pub mod profile;
}
/// Native capability implementation.
pub mod server {
    /// In-memory profiles plugin.
    pub mod profile;
}

[%/?:present%]/// Register the selected generated contributions.
///
/// # Errors
/// Returns plugin/host registration failures.
pub fn register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()> {
[%#?:present%]    app.plugin("profiles", server::profile::register)?;
[%|%]    let _ = app;
[%/?:present%]    Ok(())
}
