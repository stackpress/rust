//! Full generated-family host; native resources and configuration never enter shared UI props.

use serde_json::json;
use stackpress_lib::{Flow, Result};
use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder, StartupContext};
use std::sync::Arc;

/// Build the complete selected plugin host, without installation, population or a listener.
/// # Errors
/// Preserves typed config, existing-file SQLite and ordinary constituent startup failures.
pub async fn build(
    layers: ConfigLayers,
    seed: &str,
    environment: stackpress_framework::config::Environment,
) -> BuildResult<Arc<Server>> {
    stackpress_framework_view::leptos::render::initialize_ssr()
        .map_err(stackpress_lib::Error::from_source)?;
    let mut all = crate::config::application(seed)?;
    for (label, value) in layers.entries() {
        all.value(label, value.clone())?;
    }
    let presentation = crate::config::presentation(&all).await?;
    // Existing auth views accept a public host presentation. Resolve the same brand/styles
    // as admin before registration; signing/config services never become view props.
    let mut session = stackpress_framework_session::presentation::Presentation::default();
    session.config.brand = presentation
        .brand
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    session.config.logo = presentation
        .brand
        .get("logo")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    session.config.favicon = presentation
        .brand
        .get("favicon")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("/favicon.ico")
        .into();
    session.config.styles = presentation
        .view
        .get("styles")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(str::to_owned)
        .collect();
    session.assets.theme_bootstrap = Some(
        stackpress_framework_view::theme::bootstrap_location()
            .map_err(stackpress_lib::Error::from_source)?,
    );
    let admin = stackpress_framework_api::views::admin::Config {
        presentation: json!({"admin":presentation.admin,"brand":presentation.brand,"view":presentation.view,
            "models":blog_shared::ui::host::presentation()}),
        csrf: presentation.csrf,
    };
    let mut app = ServerBuilder::new();
    app.config_layers(all)?;
    crate::config::register(&mut app)?;
    crate::plugins::store::register(&mut app)?;
    crate::history::register(&mut app)?;
    crate::plugins::commands::register_application(&mut app)?;
    crate::plugins::app::register(&mut app)?;
    crate::plugins::assets::register(&mut app)?;
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let environment = environment.clone();
            let session = session.clone();
            Box::pin(async move {
                start.register(Arc::new(environment))?;
                start.register(Arc::new(session))?;
                let database = &start
                    .config::<stackpress_framework::config::DatabaseOptions>()?
                    .0;
                let brand = &start.config::<crate::config::Brand>()?.0;
                let server = &start
                    .config::<stackpress_framework::config::ServerOptions>()?
                    .0;
                start.register(Arc::new(stackpress_framework_session::auth::Environment {
                    database_seed: database
                        .get("seed")
                        .and_then(|value| value.as_str())
                        .map(str::to_owned),
                    email_configured: crate::config::truthy(
                        &start
                            .config::<stackpress_framework_email::plugin::EmailSection>()?
                            .0,
                    ),
                    brand: brand
                        .get("name")
                        .and_then(|value| value.as_str())
                        .unwrap_or("Stackpress")
                        .into(),
                    port: server
                        .get("port")
                        .and_then(|value| value.as_u64())
                        .unwrap_or(3000) as u16,
                    ..Default::default()
                }))?;
                start.register(Arc::new(stackpress_framework_api::pages::Environment {
                    view: start
                        .config::<stackpress_framework::view::Section>()?
                        .0
                        .clone()
                        .unwrap_or_default(),
                    brand: start.config::<crate::config::Brand>()?.0.clone(),
                    ..Default::default()
                }))?;
                Ok(Flow::Continue)
            })
        },
    ))?;
    // The emitted client contributes exactly one data inventory and the matching auth/API family.
    stackpress_framework::register_with(&mut app, &crate::generated::client::framework::CLIENT)?;
    stackpress_framework_api::views::admin::register_queued_with::<
        crate::generated::stores::family::Models,
    >(
        &mut app,
        blog_shared::ui::pages::ui(),
        admin,
        blog_shared::ui::pages::MODELS,
    )?;
    let server = Arc::new(app.build().await?);
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()?
        .bind(&server, tokio::runtime::Handle::current());
    Ok(server)
}

/// Stop owned nonawaited hooks, then close the selected native database after callers finish.
/// # Errors
/// Preserves the database shutdown error; never cancels another application's resources.
pub async fn shutdown(server: &Server) -> Result<()> {
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()?
        .shutdown()
        .await;
    server
        .host()
        .service::<stackpress_sql::Engine>()?
        .close()
        .await
        .map_err(stackpress_lib::Error::from_source)
}
