//! Source command orchestration with explicit native execution boundaries.

#[cfg(feature = "schema-client")]
pub mod schema;
pub mod server;
#[cfg(feature = "sql")]
pub mod sql;

#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "sql"
))]
use stackpress_lib::Flow;
use stackpress_lib::Handler;
#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "sql",
    feature = "view"
))]
use stackpress_lib::Result;
#[cfg(any(feature = "server-commands", feature = "sql"))]
use stackpress_server::ServerBuilder;
#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "sql",
    feature = "view"
))]
use stackpress_server::StartupContext;
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

    /// Execute an explicitly selected native operation without retaining a registry lock.
    /// # Errors
    /// Reports an absent binding or the original provider failure; never performs fallback IO.
    pub async fn invoke(
        &self,
        name: &str,
        req: &mut stackpress_lib::Request,
        res: &mut stackpress_lib::Response,
        inv: stackpress_lib::Invocation<'_>,
    ) -> stackpress_lib::Result<stackpress_lib::Flow> {
        let action = self
            .actions
            .get(name)
            .ok_or_else(|| stackpress_lib::Error::new("native command binding is unavailable"))?;
        action.invoke(req, res, inv).await
    }
}

/// Register source server command names during Listen, not as startup IO.
#[cfg(feature = "server-commands")]
pub fn register_server(app: &mut ServerBuilder) -> Result<()> {
    app.on_listen(server_listen)
}

/// Preserve the original server command insertion order.
#[cfg(feature = "server-commands")]
async fn server_listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("develop", server::develop)?;
    start.on("emit", server::emit)?;
    start.on("serve", server::serve)?;
    Ok(Flow::Continue)
}

/// Register the schema command after the source server commands.
#[cfg(feature = "schema-client")]
pub(crate) async fn schema_listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("generate", schema::event)?;
    Ok(Flow::Continue)
}

/// Queue the source SQL command family without constructing an Engine.
#[cfg(feature = "sql")]
pub fn register_sql(app: &mut ServerBuilder) -> Result<()> {
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
        start.on(event, sql::event)?;
    }
    Ok(Flow::Continue)
}

/// Add the explicit build command only for a source-present view section.
#[cfg(feature = "view")]
pub(crate) fn view_listen(start: &mut StartupContext) -> Result<()> {
    start.on("build", super::view::build_event)?;
    Ok(())
}
