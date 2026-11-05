//! One managed application family drives real registered auth, CRUD, API, admin and export.
#![allow(dead_code, unexpected_cfgs)]
use crate::framework;
#[path = "../generated-source/client/mod.rs"]
pub mod client;
#[cfg(feature = "ssr")]
#[path = "../website/src/source_family.rs"]
pub mod presentation;
#[path = "../generated-source/source/mod.rs"]
pub mod source;
#[path = "../generated-source/stores/mod.rs"]
pub mod stores;

use serde_json::{Value as Json, json};
use stackpress_framework_session::{account, auth};
use stackpress_framework_sql::source::{SourceEnvironment, SourceModel, StoreQuery};
use stackpress_lib::{Body, Flow, Request, Response, Result};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, StartupContext};
use stackpress_sql::Engine;
use std::sync::Arc;

/// Application-owned admin adapter; the generator supplies each model in catalog order.
#[cfg(feature = "ssr")]
pub fn admin(app: &mut ServerBuilder, module: &'static str) -> Result<()> {
    let mut config = stackpress_framework_api::views::admin::Config::default();
    config.presentation["models"] = presentation::ui::host::presentation();
    stackpress_framework_api::views::admin::register_queued_with::<stores::family::Models>(
        app,
        presentation::ui(),
        config,
        &[module],
    )
}

/// Direct composition uses the same ordinary functions, callbacks and source order.
fn direct(app: &mut ServerBuilder) -> Result<()> {
    let client = &client::framework::CLIENT;
    framework::config::register(app)?;
    framework::commands::register_server(app)?;
    framework::client::register(app, Some(client))?;
    framework::language::register(app)?;
    stackpress_framework_csrf::plugin::register(app)?;
    framework::commands::register_sql(app)?;
    for model in client.models {
        if let Some(listen) = model.listen {
            listen(app)?;
        }
    }
    framework::view::register(app)?;
    if let Some(session) = client.bindings.session {
        session(app)?;
    }
    #[cfg(feature = "api")]
    if let Some(api) = client.bindings.api {
        api(app)?;
    }
    for model in client.models {
        if let Some(admin) = model.admin {
            admin(app)?;
        }
    }
    Ok(())
}
/// Interpret wire and native response values through the same source invocation bridge.
async fn emit(host: &Server, event: &str, data: Json, token: Option<&str>) -> Result<Response> {
    let mut request = Request::builder().data(data).build()?;
    if let Some(token) = token {
        request.session.set("session", token, None)?;
    }
    let mut response = Response::new();
    host.emit(event, &mut request, &mut response).await?;
    Ok(response)
}

/// Explicit setup is finite SQLite IO; registration/generation never opens the database.
pub async fn prove(
    use_direct: bool,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let directory = tempfile::tempdir()?;
    let engine = Arc::new(Engine::new(
        stackpress_sql_sqlite::SqliteDatabase::open(
            stackpress_sql_sqlite::SqliteOptions::file(
                directory.path().join("source-family.sqlite"),
            )
            .create_if_missing(true),
        )
        .await?,
    ));
    let profile =
        stores::profile::ProfileModel::actions(engine.clone(), "SOURCE-SEED-PRIVATE".into(), 0);
    profile.install().await?;
    stores::application::ApplicationModel::actions(engine.clone(), String::new(), 0)
        .install()
        .await?;
    stores::session::SessionModel::actions(engine.clone(), String::new(), 0)
        .install()
        .await?;
    stores::auth::AuthModel::actions(engine.clone(), "SOURCE-SEED-PRIVATE".into(), 0)
        .install()
        .await?;
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers.value(
        "application",
        json!({"session":{},"auth":{},"client":{"tsconfig":"source.json"}}),
    )?;
    // A disabled plugin cannot claim its config section; keep the app's inputs aligned.
    #[cfg(feature = "api")]
    layers.value("application-api", json!({"api":{"endpoints":[]}}))?;
    app.config_layers(layers)?;
    let owned = engine.clone();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let engine = owned.clone();
            Box::pin(async move {
                start.register(engine)?;
                start.register(Arc::new(SourceEnvironment {
                    seed: "SOURCE-SEED-PRIVATE".into(),
                    offset_minutes: 0,
                }))?;
                start.register(Arc::new(auth::Environment {
                    database_seed: Some("SOURCE-SEED-PRIVATE".into()),
                    ..Default::default()
                }))?;
                Ok(Flow::Continue)
            })
        },
    ))?;
    if use_direct {
        direct(&mut app)?;
    } else {
        framework::register_with(&mut app, &client::framework::CLIENT)?;
    }
    app.on("source-account-export", account::export)?;
    let host = Arc::new(app.build().await.map_err(|failure| failure.error)?);
    assert_eq!(
        host.listeners()
            .filter(|listener| listener.pattern == "request")
            .count(),
        0,
        "no global guard was added"
    );
    let actual = host
        .listeners()
        .filter(|listener| listener.pattern.ends_with("-batch"))
        .map(|listener| listener.pattern.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            "profile-batch",
            "application-batch",
            "session-batch",
            "auth-batch"
        ],
        "G19 app catalog order"
    );
    let signup = emit(&host,"auth-signup",json!({"name":"Finite Source","username":"finite-user","secret":"correct","nickname":"not a signup input"}),None).await?;
    let signup = auth::events::json_result(&signup)?;
    assert_eq!(
        signup["nickname"], "application",
        "stock signup forwarded only source inputs"
    );
    let id = signup["id"].as_str().ok_or("signup id missing")?;
    let update = emit(
        &host,
        "profile-update",
        json!({"id":id,"nickname":"App Source","references":{"z":{"b":1,"a":2},"a":"last"}}),
        None,
    )
    .await?;
    assert_eq!(update.status().map(|status| status.code), Some(200));
    let search = emit(&host, "profile-search", json!({"q":"aPp sOuRcE"}), None).await?;
    assert_eq!(
        auth::events::json_result(&search)?[0]["nickname"],
        "App Source"
    );
    let signin = emit(
        &host,
        "auth-signin",
        json!({"username":"finite-user","secret":"correct","2fa":false}),
        None,
    )
    .await?;
    assert_eq!(
        auth::events::json_result(&signin)?["profile"]["nickname"],
        "App Source"
    );
    #[cfg(feature = "api")]
    {
        for (event, data) in [
            (
                "application-create",
                json!({"id":"source-app","name":"Source App","profileId":id,"expires":"2099-01-01T00:00:00.000Z"}),
            ),
            (
                "session-create",
                json!({"id":"source-session","applicationId":"source-app","profileId":id,"expires":"2099-01-01T00:00:00.000Z"}),
            ),
        ] {
            let response = emit(&host, event, data, None).await?;
            assert_eq!(
                response.status().map(|status| status.code),
                Some(200),
                "{event}: {:?}",
                response.error()
            );
        }
        for (event, id) in [
            ("application-detail", "source-app"),
            ("session-detail", "source-session"),
        ] {
            let response = emit(&host, event, json!({"id":id}), None).await?;
            assert_eq!(
                auth::events::json_result(&response)?["profile"]["nickname"],
                "App Source"
            );
        }
    }
    let token = host
        .host()
        .service::<stackpress_framework_session::Service>()?
        .create(
            json!({"id":id,"name":"Finite Source"})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await?;
    let exported = emit(
        &host,
        "source-account-export",
        json!({"download":true}),
        Some(&token),
    )
    .await?;
    let Body::Text(csv) = exported.body() else {
        return Err("missing source CSV".into());
    };
    let row = profile
        .find(&StoreQuery::eq("id", id))
        .await?
        .ok_or("missing stored Profile")?;
    assert_eq!(
        *csv,
        account::csv(&row)?,
        "native Dates and insertion order survive the real generated event"
    );
    assert!(csv.contains("nickname,\"App Source\""));
    assert!(csv.contains("references,\"z: b: 1, a: 2, a: last\""));
    #[cfg(feature = "ssr")]
    {
        // The original metadata view rejects object-valued children. Nested values above
        // prove CSV ordering; the same row now uses valid scalar view data for admin SSR.
        let prepared = emit(
            &host,
            "profile-update",
            json!({"id":id,"references":{"z":1,"a":"last"}}),
            None,
        )
        .await?;
        assert_eq!(prepared.status().map(|status| status.code), Some(200));
        let mut request = Request::builder()
            .url(format!(
                "https://aggregate.invalid/admin/profile/detail/{id}"
            ))
            .build()?;
        let mut response = Response::new();
        host.handle_route(&mut request, &mut response).await?;
        let Body::Text(html) = response.body() else {
            return Err("missing generated source admin HTML".into());
        };
        assert!(html.contains("App Source"));
        assert!(!html.contains("SOURCE-SEED-PRIVATE"));
        assert!(html.contains("Profiles"));
        if let Ok(output) = std::env::var("AGGREGATE_SOURCE_PAGE_OUT") {
            let output = std::path::Path::new(&output);
            std::fs::create_dir_all(output)?;
            std::fs::write(output.join("profile-detail.html"), html)?;
            for (name, path) in [
                ("profile-update", format!("/admin/profile/update/{id}")),
                ("profile-search", "/admin/profile/search".into()),
            ] {
                let mut request = Request::builder()
                    .url(format!("https://aggregate.invalid{path}"))
                    .build()?;
                let mut response = Response::new();
                host.handle_route(&mut request, &mut response).await?;
                let Body::Text(html) = response.body() else {
                    return Err("missing generated source admin page".into());
                };
                assert!(!html.contains("SOURCE-SEED-PRIVATE"));
                std::fs::write(output.join(format!("{name}.html")), html)?;
            }
        }
    }
    host.host().service::<auth::Hooks>()?.shutdown().await;
    engine.close().await?;
    println!("source family auth/CRUD/joins/admin/export passed (direct={use_direct})");
    Ok(())
}
