//! Actual generated app family selected by the API's ordinary data-event registrar.
#[allow(dead_code, unexpected_cfgs)]
#[path = "../../framework-models/tests/extended/generated/mod.rs"]
mod generated;

use generated::stores::{
    _family::Models, application::ApplicationModel, auth::AuthModel, profile::ProfileModel,
    session::SessionModel,
};
use serde_json::{Value as Json, json};
use stackpress_framework_schema::source::Record;
use stackpress_framework_sql::source::SourceModel;
use stackpress_lib::{Flow, Request, Response};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::Engine;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

/// The Auth catalog's shipped icon is rendered in its actual management breadcrumb.
#[cfg(feature = "ui-ssr")]
#[test]
fn generated_auth_breadcrumb_renders_the_pinned_lock_icon() {
    use leptos::prelude::*;
    use stackpress_framework_api::views::management::{Management, ManagementPage};
    let root = Owner::new();
    let html = root.with(|| {
        let page = Management {
            module: "auth".into(),
            action: stackpress_framework_admin::pages::Action::Search,
            data: json!([]),
            query: json!({}),
            relations: Default::default(),
            csrf: None,
            config: json!({"models":{"auth":{"plural":"Auths","icon":"lock"}}}),
            theme: None,
            error: None,
        };
        view! {<ManagementPage page ui=super::shared::management()/>}.to_html()
    });
    let crumb = html
        .split("aria-label=\"Breadcrumb\"")
        .nth(1)
        .unwrap()
        .split("</a>")
        .next()
        .unwrap();
    assert!(crumb.contains("href=\"/admin/auth/search\""));
    assert!(crumb.contains("<svg") && crumb.contains("aria-hidden=\"true\""));
    assert!(crumb.contains("M7 11V7a5 5 0 0 1 10 0v4"));
    assert!(crumb.contains("Auths"));
    root.cleanup();
}

/// Only the exclusively created test file is eligible for cleanup, including unwinds.
struct OwnedFile(PathBuf);
impl Drop for OwnedFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Decode only test input; the generated store's own query/codec path does the real work.
fn record(value: Json) -> Record {
    serde_json::from_value(value).unwrap()
}

/// Both owning joins and source q/like searches use the appended Profile declaration.
#[tokio::test]
async fn generated_family_registered_application_and_session_join_extended_profile() {
    // The accepted SQLite adapter owns persistent files, including in-process finite proofs.
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "stackpress-g50-api-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    let owned_file = OwnedFile(path);
    let db = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(&owned_file.0),
    )
    .await
    .unwrap();
    let engine = Arc::new(Engine::new(db));
    let profile = ProfileModel::actions(engine.clone(), String::new(), 0);
    let auth = AuthModel::actions(engine.clone(), String::new(), 0);
    let application = ApplicationModel::actions(engine.clone(), String::new(), 0);
    let session = SessionModel::actions(engine.clone(), String::new(), 0);
    profile.install().await.unwrap();
    auth.install().await.unwrap();
    application.install().await.unwrap();
    session.install().await.unwrap();
    profile
        .create(&record(
            json!({"id":"p","name":"Ada","nickname":"Extended Ada"}),
        ))
        .await
        .unwrap();
    let mut app = ServerBuilder::new();
    let owned = engine.clone();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let engine = owned.clone();
            Box::pin(async move {
                start.register(engine)?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    stackpress_framework_api::sql::register_with::<Models>(&mut app).unwrap();
    let host = app.build().await.unwrap();
    // Exact event-major order remains Application then Session for each of ten events.
    let names = host
        .listeners()
        .map(|listener| listener.pattern.clone())
        .collect::<Vec<_>>();
    let expected = [
        "batch", "create", "detail", "get", "purge", "remove", "restore", "search", "update",
        "upsert",
    ]
    .into_iter()
    .flat_map(|name| [format!("application-{name}"), format!("session-{name}")])
    .collect::<Vec<_>>();
    assert_eq!(names, expected);
    for (event, data) in [
        (
            "application-create",
            json!({"id":"a","name":"App","profileId":"p","expires":"2099-01-01T00:00:00.000Z"}),
        ),
        (
            "session-create",
            json!({"id":"s","applicationId":"a","profileId":"p","expires":"2099-01-01T00:00:00.000Z"}),
        ),
    ] {
        let response = host.resolve(event, data).await.unwrap();
        assert_eq!(
            response.response.code,
            Some(200),
            "{event}: {:?}",
            response.response
        );
    }
    for (event, data) in [
        (
            "application-detail",
            json!({"id":"a","columns":["*","profile.*"]}),
        ),
        (
            "session-detail",
            json!({"id":"s","columns":["*","profile.*","application.profile.nickname"]}),
        ),
        (
            "application-search",
            json!({"like":{"profile.nickname":"eXtEnDeD"},"columns":["*","profile.*"]}),
        ),
        (
            "session-search",
            json!({"like":{"profile.nickname":"aDa"},"columns":["*","profile.*"]}),
        ),
    ] {
        let mut req = Request::builder().data(data).build().unwrap();
        let mut res = Response::new();
        host.emit(event, &mut req, &mut res).await.unwrap();
        assert_eq!(
            res.status().unwrap().code,
            200,
            "{event}: {:?}",
            res.error()
        );
        let value = res.result().unwrap().to_json().unwrap();
        let row = if event.ends_with("search") {
            &value[0]
        } else {
            &value
        };
        assert_eq!(row["profile"]["nickname"], "Extended Ada", "{event}");
        if event == "session-detail" {
            assert_eq!(row["application"]["profile"]["nickname"], "Extended Ada");
        }
    }
    engine.close().await.unwrap();
}
