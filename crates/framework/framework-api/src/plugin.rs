//! Optional API selection through ordinary Config and Route startup phases.
//!
//! Configured endpoints/CORS and late inline webhooks share the existing startup phases.
//! SQL is selected independently; OAuth process routes follow the source endpoint gate.
//! Registration performs no network I/O.

use std::sync::Arc;

use stackpress_lib::{Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};

use crate::{ApiConfig, Service, config, endpoints, webhooks};

/// Select the optional API section and its configured endpoint routes.
///
/// Missing/null config contributes no service or runtime callback. Present config requires
/// no lookup service at startup: each selected event runs only when its route is invoked.
/// Multiple matching routes remain independent, ordered by priority then registration.
///
/// # Examples
///
/// ```
/// use stackpress_framework_api::plugin;
/// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
/// use stackpress_server::{ConfigLayers, ServerBuilder};
/// async fn articles(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
///     res.results(&["Published article"])?;
///     Ok(Flow::Continue)
/// }
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<()> {
/// let mut layers = ConfigLayers::new();
/// layers.value("application", serde_json::json!({"api": {"endpoints": [{
///     "method": "GET", "route": "/api/articles", "type": "public",
///     "event": "article-search", "cors": true, "data": {"status": "PUBLISHED"}
/// }]}}))?;
/// let mut app = ServerBuilder::new();
/// app.config_layers(layers)?;
/// app.on("article-search", articles)?;
/// app.plugin("api", plugin::register)?;
/// let server = app.build().await.map_err(|failure| failure.error)?;
/// let mut request = Request::builder().url("/api/articles").build()?;
/// let mut response = Response::new();
/// server.handle_route(&mut request, &mut response).await?;
/// assert_eq!(response.result().unwrap().to_json()?, serde_json::json!(["Published article"]));
/// # Ok(())
/// # }
/// ```
///
/// # Errors
/// Returns existing duplicate-section/startup registration errors. Build reports typed
/// config, route or service errors without publishing a partial Server.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // Optional lowest-layer configuration preserves the source's deactivation behavior.
    app.config_section("api", None::<ApiConfig>, config::validate)?;
    app.on_config(configure)?;
    app.on_listen(listen)?;
    app.on_route(routes)?;
    Ok(())
}

/// Select API processes for the same compiled family as the data-event registrars.
///
/// API processes resolve ordinary named events instead of constructing model actions.
/// Select [`crate::sql::register_with`] or the app's generated model listeners separately;
/// this function installs neither data handlers nor tables and adds no global guard.
/// # Errors
/// Preserves the same configuration and startup errors as [`register`].
#[cfg(feature = "sql")]
pub fn register_with<M: stackpress_framework_sql::source::ApiModels>(
    app: &mut ServerBuilder,
) -> Result<()> {
    register(app)
}

/// Install late observers independently of whether any API endpoint was configured.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    let Some(config) = start.config::<Option<ApiConfig>>()? else {
        return Ok(Flow::Continue);
    };
    if config.webhooks.is_empty() {
        // Inert configurations do not even construct the native transport.
        return Ok(Flow::Continue);
    }
    let configs = config.webhooks.clone();
    let fetch = match start.service::<webhooks::Transport>() {
        Ok(transport) => Arc::clone(&transport.0),
        Err(_) => Arc::new(webhooks::ReqwestFetch::new()?),
    };
    for config in configs {
        let webhook = webhooks::Webhook::new(config)?;
        start.on_priority(
            &webhook.config.event.clone(),
            -200,
            webhooks::action(webhook, fetch.clone()),
        )?;
    }
    Ok(Flow::Continue)
}

/// Publish the present immutable configuration before any Listen/Route consumer runs.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    // Copy the present configuration before the mutable registry insertion.
    if let Some(config) = start.config::<Option<ApiConfig>>()?.clone() {
        start.register(Arc::new(Service { config }))?;
    }
    Ok(Flow::Continue)
}

/// Install each configured endpoint's CORS callbacks before its admission callback.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    // Absence short-circuits before requesting a service or constructing route state.
    let Some(config) = start.config::<Option<ApiConfig>>()? else {
        return Ok(Flow::Continue);
    };
    if config.endpoints.is_empty() {
        return Ok(Flow::Continue);
    }
    // Source import order precedes configured endpoints; the view contribution belongs to M5.
    let mut routes = crate::pages::prepare();
    routes.extend(endpoints::prepare(&config.endpoints)?);
    #[cfg(feature = "ui-ssr")]
    start.route_with(
        stackpress_server::RouteSpec::new(stackpress_lib::MethodPattern::Any, "/auth/oauth")
            .priority(-100),
        crate::views::host::oauth,
    )?;
    // Each prepared callback is registered independently; no matcher uniqueness is imposed.
    for route in routes {
        start.route_with(route.spec, route.action)?;
    }
    Ok(Flow::Continue)
}
