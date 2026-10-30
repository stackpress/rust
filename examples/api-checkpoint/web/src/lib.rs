//! No-listener API host composition. The application owns Engine lifetime and config layers.
//! JWT identity, API model events and process routes are selected independently; consent
//! renders consent and selected generated management over an explicitly supplied Engine.
use stackpress_framework_api::pages;
pub mod finite;
#[path = "../../shared/src/lib.rs"]
pub mod shared;
use stackpress_lib::{Flow, Result, queue::task};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, StartupContext};
use stackpress_sql::Engine;
use std::sync::Arc;

/// Ordinary #14 composite: no required plugin IDs, ambient caller slot or startup SQL.
/// # Errors
/// Preserves each constituent's registration errors.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework_session::plugin::register(app)?;
    stackpress_framework_api::sql::register(app)?;
    stackpress_framework_api::register(app)?;
    stackpress_framework_api::views::admin::register(
        app,
        shared::management(),
        Default::default(),
    )?;
    assets(app)?;
    app.route_with(
        stackpress_server::RouteSpec::new(
            stackpress_lib::MethodPattern::Any,
            "/admin/profile/search",
        ),
        profiles,
    )?;
    Ok(())
}

/// Build a host without binding a port. Install/seed stores explicitly before requesting pages.
///
/// Layers supply source api/session configuration; presentation maps the app's view/brand
/// objects into the API page environment without claiming other plugins' typed config keys.
/// Keep the Engine alive until requests finish, then close it explicitly in the application.
/// # Errors
/// Returns ordinary startup/config errors; no listener or detached worker is created.
pub async fn build(
    layers: ConfigLayers,
    engine: Arc<Engine>,
    presentation: pages::Environment,
) -> Result<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = engine.clone();
        let presentation = presentation.clone();
        Box::pin(async move {
            start.register(engine)?;
            start.register(Arc::new(presentation))?;
            let locales = stackpress_framework_view::preferences::LocaleRegistry::new(
                vec!["en".into()],
                "en".into(),
                Default::default(),
            )
            .map_err(stackpress_lib::Error::from_source)?;
            let scope = stackpress_framework_view::preferences::CookieScope::new(
                "https://api.invalid".into(),
                Default::default(),
            )
            .map_err(stackpress_lib::Error::from_source)?;
            start.register(Arc::new(
                stackpress_framework_view::preferences::SettingsAction::new(locales, scope),
            ))?;
            Ok(Flow::Continue)
        })
    }))?;
    app.plugin("application", register)?;
    app.build().await.map_err(|failure| failure.error)
}

/// Selected relation lookup uses the session owner's public Profile actions, without editing it.
async fn profiles(
    req: &mut stackpress_lib::Request,
    res: &mut stackpress_lib::Response,
    inv: stackpress_lib::Invocation<'_>,
) -> Result<Flow> {
    use stackpress_server::ServerInvocationExt;
    let engine = inv.shared_service::<Engine>()?;
    let actions = stackpress_framework_session::storage::ProfileActions::new(engine, "");
    let query = stackpress_framework_session::storage::StoreQuery {
        q: req
            .data
            .get("q")
            .and_then(stackpress_lib::Value::as_str)
            .map(str::to_owned),
        ..Default::default()
    };
    // Preserve the selected source store's backend errors, including SQLite's ILIKE limit.
    let rows = match actions.find_all(&query).await {
        Ok(rows) => rows,
        Err(error) => {
            res.set_error(stackpress_lib::PublicError {
                code: error.code,
                status: error.status().into(),
                error: error.error,
                errors: serde_json::to_value(error.errors)?,
            })?;
            let code = res.status().expect("set_error publishes status").code;
            let envelope = res.try_to_json()?;
            res.json(&envelope)?;
            res.status_code(code, None)?;
            return Ok(Flow::Continue);
        }
    };
    res.results(&rows)?;
    let envelope = res.try_to_json()?;
    res.json(&envelope)?;
    Ok(Flow::Continue)
}
/// Static selected assets are ordinary in-process routes; this function binds no listener.
fn assets(app: &mut ServerBuilder) -> Result<()> {
    app.route_with(
        stackpress_server::RouteSpec::new(
            stackpress_lib::MethodPattern::exact("GET")?,
            "/styles/global.css",
        ),
        stackpress_lib::handler(|_, res, _| {
            Box::pin(async move {
                res.set(
                    "text/css",
                    stackpress_lib::Body::Text(format!(
                        "{}\n{}",
                        stackpress_leptos::components::BASE_STYLES,
                        stackpress_framework_api::views::STYLES
                    )),
                )?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    app.route_with(
        stackpress_server::RouteSpec::new(
            stackpress_lib::MethodPattern::exact("GET")?,
            &format!(
                "/assets/{}",
                stackpress_framework_view::theme::BOOTSTRAP_FILE
            ),
        ),
        stackpress_lib::handler(|_, res, _| {
            Box::pin(async move {
                res.set(
                    "text/javascript",
                    stackpress_lib::Body::Text(
                        stackpress_framework_view::theme::BOOTSTRAP_SOURCE.into(),
                    ),
                )?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    Ok(())
}
