//! Static callbacks and the nine-stage source composition; no runtime plugin loader.

pub mod client;
pub mod commands;
pub mod config;
#[cfg(feature = "language")]
pub mod language;
#[cfg(feature = "view")]
pub mod view;

use stackpress_framework_schema::Model;
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

/// Ordinary compiled registration, also usable independently of this aggregate.
pub type Register = fn(&mut ServerBuilder) -> Result<()>;

/// Application-generated specializations selected at the original plugin stages.
/// Empty bindings retain the existing generated-default constituents. Data listeners
/// remain the catalog's SQL-phase callbacks; these bindings never add a second inventory.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameworkBindings {
    /// Auth → email → identity composition specialized to the app's SessionModels family.
    pub session: Option<Register>,
    /// API process registration over that family's ApiModels contract.
    pub api: Option<Register>,
}

/// One statically linked model family, retaining source catalog order.
pub struct CompiledModel {
    /// Application model identity; never used to import executable code.
    pub name: &'static str,
    /// Optional source model event contribution, normally queuing Listen callbacks.
    pub listen: Option<Register>,
    /// Optional app-owned generated admin contribution, called at the final stage.
    pub admin: Option<Register>,
}

/// Build-time application catalog replacing dynamic generated-client imports.
pub struct CompiledClient {
    /// Compiled plugin collaborators; no runtime loader or ambient model registry.
    pub bindings: FrameworkBindings,
    /// Ordered compiled model families; missing methods remain `None`.
    pub models: &'static [CompiledModel],
    /// Build identity for inspection, not a runtime module lookup string.
    pub build: &'static str,
    /// Owned source schema metadata, obtained only when explicitly requested.
    pub describe: fn() -> Vec<Model>,
}

/// Register the selected source stages without a generated application client.
///
/// # Errors
/// Propagates the first constituent's existing registration failure. Missing config/client
/// does not require an Engine or create an implicit guard. Startup errors remain build errors.
#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "language",
    feature = "csrf",
    feature = "sql",
    feature = "view",
    feature = "session",
    feature = "api",
    feature = "admin"
))]
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // A missing compiled catalog is distinct from a successfully imported empty catalog.
    select(app, None)
}

/// Register the same stages with statically linked model and generated-admin callbacks.
///
/// Wrap this call in a named application function for `ServerBuilder::plugin`. Callbacks
/// prepare the existing startup queues; this function does not re-open a running host.
/// # Errors
/// Propagates the first real callback/constituent error without rollback or partial publication.
#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "language",
    feature = "csrf",
    feature = "sql",
    feature = "view",
    feature = "session",
    feature = "api",
    feature = "admin"
))]
pub fn register_with(app: &mut ServerBuilder, client: &'static CompiledClient) -> Result<()> {
    // Executable selection is fixed in the compiled application, not in request/config data.
    select(app, Some(client))
}

/// Run selected stages in source order; test instrumentation observes these actual calls.
#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "language",
    feature = "csrf",
    feature = "sql",
    feature = "view",
    feature = "session",
    feature = "api",
    feature = "admin"
))]
fn select(app: &mut ServerBuilder, catalog: Option<&'static CompiledClient>) -> Result<()> {
    // Typed source roots are claimed before phase scheduling, with no provider IO.
    config::register(app)?;
    // Source server precedes schema; neither registration runs command IO.
    #[cfg(feature = "server-commands")]
    {
        observed("server");
        commands::register_server(app)?;
    }
    #[cfg(feature = "schema-client")]
    {
        observed("schema");
        client::register(app, catalog)?;
    }
    #[cfg(feature = "language")]
    {
        observed("language");
        language::register(app)?;
    }
    #[cfg(feature = "csrf")]
    {
        observed("csrf");
        stackpress_framework_csrf::plugin::register(app)?;
    }
    #[cfg(feature = "sql")]
    {
        observed("sql");
        commands::register_sql(app)?;
        // Existing compiled registrars prepare their own phases. Account catalogs select
        // account::events::register_listen; the immediate direct API remains independent.
        if let Some(client) = catalog {
            for model in client.models {
                if let Some(listen) = model.listen {
                    listen(app)?;
                }
            }
        }
    }
    #[cfg(feature = "view")]
    {
        observed("view");
        view::register(app)?;
    }
    #[cfg(feature = "session")]
    {
        observed("session");
        let register = catalog
            .and_then(|client| client.bindings.session)
            .unwrap_or(stackpress_framework_session::composite::register);
        register(app)?;
    }
    #[cfg(feature = "api")]
    {
        observed("api");
        let register = catalog
            .and_then(|client| client.bindings.api)
            .unwrap_or(stackpress_framework_api::register);
        register(app)?;
    }
    #[cfg(feature = "admin")]
    {
        observed("admin");
        // Generated hosts keep their own checked SQL/Config/Route contracts and come last.
        if let Some(client) = catalog {
            for model in client.models {
                if let Some(admin) = model.admin {
                    admin(app)?;
                }
            }
        }
    }
    // Native-only builds may not consume a catalog; there is no hidden fallback client.
    let _ = catalog;
    Ok(())
}

/// Observe actual selected calls in unit tests without storing a production trace service.
#[cfg(any(
    feature = "server-commands",
    feature = "schema-client",
    feature = "language",
    feature = "csrf",
    feature = "sql",
    feature = "view",
    feature = "session",
    feature = "api",
    feature = "admin"
))]
fn observed(_name: &'static str) {
    #[cfg(test)]
    TRACE.with(|trace| trace.borrow_mut().push(_name));
}

#[cfg(test)]
thread_local! {
    static TRACE: std::cell::RefCell<Vec<&'static str>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(all(test, feature = "full"))]
mod tests;
