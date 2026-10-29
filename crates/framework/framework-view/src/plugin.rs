//! Compiled presentation selection through the existing ServerBuilder contract.
//!
//! The native `presentation` feature makes this function available; only an explicit
//! selection runs it. Preferences and flash retain their independent services, typed
//! configuration, startup phases and direct registration APIs. No renderer is selected.

use crate::{flash, preferences};
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

/// Compose preferences then flash under the application's selected plugin identity.
///
/// Supply `view_preferences` and `view_flash` through ConfigLayers and register an
/// explicit [`preferences::Admission`] in Config before the preferences callback.
/// Application pages, protected-operation policy and asset ownership stay explicit.
/// Multiple handlers may share a route; authors own their inventories, priorities
/// and cooperative abort behavior. Each protected handler enforces its own policy.
///
/// This queues in-memory host contributions only: no file/database setup, listener,
/// renderer initialization or generation hook runs. A composite may call this function
/// directly; do not select nested plugins from a sealed registration callback.
///
/// # Errors
/// Returns existing configuration-contribution errors, including selecting a constituent
/// twice. Build additionally rejects invalid/missing sections, missing Admission and
/// duplicate services. Discard a builder after a registration error; contributions across
/// queues are not transactional.
///
/// # Examples
/// ```
/// use stackpress_framework_view::plugin;
/// use stackpress_server::ServerBuilder;
/// let mut app = ServerBuilder::new();
/// // Selection is data-only; supply trusted configuration and Admission before build.
/// app.plugin("presentation", plugin::register)?;
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // Keep each capability's validation and startup contributions in its existing owner.
    preferences::register(app)?;
    // Flash is independently selectable; this composite deliberately selects both once.
    flash::register(app)
}
