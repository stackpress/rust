//! Native operation registration over an application-owned Engine and Policy.
//!
//! The `server` feature exposes this registrar; selecting it queues Config and Listen
//! callbacks only. Direct stores and generated operation registrars remain independent.

use crate::operations::{Policy, Registration};
use stackpress_lib::{Error, Flow, Result, queue::task};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::Engine;
use std::sync::Arc;

/// One compiled operation family contributing through the host's shared ownership marker.
pub type RegisterOperations = fn(&mut StartupContext, &mut Registration) -> Result<()>;

/// Explicit operation families supplied during Config, before Listen freezes registration.
///
/// Supply one nonempty list, at most 128 callbacks. Applications combining generated admin
/// with other families aggregate this service themselves using the direct generated helpers.
pub struct Contributions {
    /// Ordered compiled callbacks; configuration strings never select executable code.
    pub operations: &'static [RegisterOperations],
}

/// Opaque proof that this host selected SQL integration and verified Engine and Policy.
///
/// Only this registrar constructs the service. Providing the underlying dependencies alone
/// does not satisfy a consumer requiring this integration.
pub struct Service {
    /// Private construction prevents substituting dependencies for explicit registration.
    _private: (),
}

/// Queue native SQL integration without opening a database or registering routes/config.
///
/// Register an existing [`Engine`] and [`Policy`] during Config before this plugin, and
/// [`Contributions`] before Listen. Config verifies dependencies and claims [`Service`];
/// Listen invokes the bounded list with one [`Registration`]. Callbacks register actual
/// generated handlers; startup must never install, migrate or seed a database, or listen.
/// Each protected operation enforces its own app-supplied policy on every invocation.
///
/// # Errors
/// Preserves builder/startup errors for missing or duplicate services, invalid contribution
/// counts and callback failures. A failed build publishes no Server; arbitrary callback
/// side effects are not rolled back. Duplicate operations use the existing marker contract.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.on_config(task(|start: &mut StartupContext| {
        Box::pin(async move {
            // Verify app-owned resources without creating, opening or querying an Engine.
            start.service::<Engine>()?;
            start.service::<Policy>()?;
            start.register(Arc::new(Service { _private: () }))?;
            Ok(Flow::Continue)
        })
    }))?;
    app.on_listen(task(|start: &mut StartupContext| {
        Box::pin(async move {
            // Copy the static slice before mutably borrowing startup for each contribution.
            let operations = start.service::<Contributions>()?.operations;
            if operations.is_empty() || operations.len() > 128 {
                // Validate the complete inventory before any callback can contribute listeners.
                return Err(Error::invalid(
                    "SQL contributions require 1 to 128 callbacks",
                ));
            }
            let mut registration = Registration::default();
            for contribute in operations {
                // One marker rejects conflicting selections across all operation families.
                contribute(start, &mut registration)?;
            }
            Ok(Flow::Continue)
        })
    }))?;
    Ok(())
}
