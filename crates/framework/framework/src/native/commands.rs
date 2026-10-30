//! Minimal real native command forwarding; execution is explicit and application-owned.
//!
//! M0 proves registration and provider invocation, not a complete Node development toolchain.
//! Missing bindings fail when invoked, never perform IO at startup and never report fake success.

use stackpress_lib::Handler;
#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "sql",
    feature = "view"
))]
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
#[cfg(any(feature = "server-commands", feature = "sql"))]
use stackpress_server::ServerBuilder;
#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "sql",
    feature = "view"
))]
use stackpress_server::{ServerInvocationExt, StartupContext};
use std::{collections::BTreeMap, sync::Arc};

/// Finite explicit command bindings supplied as a service by the native application.
#[derive(Default)]
pub struct Bindings {
    /// Owned compiled functions, not config-selected module strings or detached processes.
    actions: BTreeMap<String, Arc<dyn Handler>>,
}

impl Bindings {
    /// Bind an existing compiled handler; later explicit bindings replace earlier ones.
    pub fn bind(&mut self, name: &str, action: impl Handler + 'static) {
        // No command is called until its registered event is explicitly emitted.
        self.actions.insert(name.to_owned(), Arc::new(action));
    }
}

/// Forward the current command to the native application's explicit provider.
#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "sql",
    feature = "view"
))]
async fn dispatch(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Retain the provider before borrowing its callback across await; no registry lock exists.
    let bindings = inv.shared_service::<Bindings>()?;
    let action = bindings
        .actions
        .get(inv.event())
        .ok_or_else(|| Error::new("native command binding is unavailable"))?;
    action.invoke(req, res, inv).await
}

/// Register source server command names during Listen, not as startup IO.
#[cfg(feature = "server-commands")]
pub(crate) fn register_server(app: &mut ServerBuilder) -> Result<()> {
    app.on_listen(server_listen)
}

/// Preserve the original server command insertion order.
#[cfg(feature = "server-commands")]
async fn server_listen(start: &mut StartupContext) -> Result<Flow> {
    for event in ["develop", "emit", "serve"] {
        start.on(event, dispatch)?;
    }
    Ok(Flow::Continue)
}

/// Register the schema command after the source server commands.
#[cfg(feature = "schema-client")]
pub(crate) async fn schema_listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("generate", dispatch)?;
    Ok(Flow::Continue)
}

/// Queue the source SQL command family without constructing an Engine.
#[cfg(feature = "sql")]
pub(crate) fn register_sql(app: &mut ServerBuilder) -> Result<()> {
    app.on_listen(sql_listen)
}

/// Preserve source SQL command order including the event-only query command.
#[cfg(feature = "sql")]
async fn sql_listen(start: &mut StartupContext) -> Result<Flow> {
    for event in [
        "install",
        "migrate",
        "purge",
        "push",
        "query",
        "populate",
        "uninstall",
        "upgrade",
    ] {
        start.on(event, dispatch)?;
    }
    Ok(Flow::Continue)
}

/// Add the explicit build command only for a source-present view section.
#[cfg(feature = "view")]
pub(crate) fn view_listen(start: &mut StartupContext) -> Result<()> {
    start.on("build", dispatch)?;
    Ok(())
}
