//! Added Profile columns traverse the actual regenerated family, not stock framework defaults.
#[allow(
    missing_docs,
    dead_code,
    reason = "the complete managed fixture exposes more callbacks than this integration selects"
)]
#[path = "../../../.build/extension/generated/mod.rs"]
mod generated;

use serde_json::{Value, json};
use stackpress_lib::{Body, Request, Response};
use stackpress_server::{ConfigLayers, Server, ServerBuilder};
use std::sync::Arc;

/// Keep native Date values in nested operations until the actual source JSON boundary.
async fn emit(server: &Server, event: &str, data: Value) -> Value {
    let mut request = Request::builder().data(data).build().unwrap();
    let mut response = Response::new();
    server
        .emit(event, &mut request, &mut response)
        .await
        .unwrap();
    assert_eq!(
        response.status().unwrap().code,
        200,
        "{event}: {:?}",
        response.error()
    );
    stackpress_framework_session::auth::events::json_result(&response).unwrap()
}

/// Migrate an existing seeded file, restart with the extended client, and use every collaborator.
#[tokio::test(flavor = "current_thread")]
async fn regenerated_profile_column_reaches_crud_auth_api_admin_and_export() {
    let old = super::support::App::new().await;
    let revisions = old.root.path().join("extension-revisions");
    let root = revisions.to_str().unwrap();
    let before = blog_web::plugins::commands::current().unwrap();
    let after = blog_web::history::snapshot(
        serde_json::to_value(generated::source::_catalog::describe()).unwrap(),
        &generated::client::framework::definition().unwrap(),
    )
    .unwrap();
    blog_web::history::insert_at(root, &before, 10).unwrap();
    blog_web::history::insert_at(root, &after, 20).unwrap();
    let engine = old
        .server
        .host()
        .service::<stackpress_sql::Engine>()
        .unwrap();
    stackpress_framework::commands::sql::upgrade(
        engine,
        Some(root),
        &blog_web::history::binding(),
        false,
    )
    .await
    .unwrap();
    blog_web::client::shutdown(&old.server).await.unwrap();
    let mut layers = blog_web::config::shared("abc123").unwrap();
    let mut input = ConfigLayers::new();
    let reference = blog_web::config::reference().unwrap();
    input.value("extended file and constituents",json!({"sqlite":{"path":old.root.path().join("blog.sqlite")},
        "auth":reference["auth"],"session":reference["session"],"api":{"endpoints":reference["api"]["endpoints"],"webhooks":[]}})).unwrap();
    for (name, value) in input.entries() {
        layers.value(name, value.clone()).unwrap();
    }
    let mut builder = ServerBuilder::new();
    builder.config_layers(layers).unwrap();
    blog_web::plugins::store::register(&mut builder).unwrap();
    stackpress_framework::register_with(&mut builder, &generated::client::framework::CLIENT)
        .unwrap();
    stackpress_framework_api::views::admin::register_queued_with::<generated::stores::family::Models>(
        &mut builder,generated::ui::pages::ui(),Default::default(),&["profile"]).unwrap();
    let server = Arc::new(builder.build().await.unwrap());
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()
        .unwrap()
        .bind(&server, tokio::runtime::Handle::current());
    assert_eq!(
        emit(&server, "article-search", json!({}))
            .await
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let updated = emit(
        &server,
        "profile-update",
        json!({"id":"developer","nickname":"Column travels"}),
    )
    .await;
    assert_eq!(updated["nickname"], "Column travels");
    assert_eq!(
        emit(&server, "profile-detail", json!({"id":"developer"})).await["nickname"],
        "Column travels"
    );
    let signin = emit(
        &server,
        "auth-signin",
        json!({"username":"developer","secret":"developer","2fa":false}),
    )
    .await;
    assert_eq!(signin["profile"]["nickname"], "Column travels");
    let joined = emit(&server, "application-search", json!({})).await;
    assert_eq!(joined[0]["profile"]["nickname"], "Column travels");
    emit(&server,"application-create",json!({"id":"extended-api","profileId":"developer","name":"Extension","scopes":["user"],"expires":"2099-01-01T00:00:00.000Z"})).await;
    let mut request = Request::builder()
        .url("/api/profile/search")
        .build()
        .unwrap();
    request
        .headers
        .set("authorization", "Bearer extended-api")
        .unwrap();
    let mut response = Response::new();
    server
        .handle_route(&mut request, &mut response)
        .await
        .unwrap();
    assert_eq!(response.status().unwrap().code, 200);
    assert_eq!(
        stackpress_framework_session::auth::events::json_result(&response).unwrap()[0]["nickname"],
        "Column travels"
    );
    let token = server
        .host()
        .service::<stackpress_framework_session::Service>()
        .unwrap()
        .create(
            json!({"id":"developer","name":"Developer","roles":["ADMIN"]})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await
        .unwrap();
    for path in [
        "/admin/profile/detail/developer",
        "/auth/account/security/export",
    ] {
        let mut request = Request::builder()
            .url(path)
            .data(json!({"download":true}))
            .build()
            .unwrap();
        request
            .session
            .set("session", token.as_str(), None)
            .unwrap();
        let mut response = Response::new();
        server
            .handle_route(&mut request, &mut response)
            .await
            .unwrap();
        assert_eq!(response.status().unwrap().code, 200, "{path}");
        let Body::Text(text) = response.body() else {
            panic!("missing native page/export")
        };
        assert!(
            text.contains("Column travels"),
            "extended field missing from {path}"
        );
    }
    let signup=emit(&server,"auth-signup",json!({"name":"Another","username":"new-fixture","secret":"ValidPassword1!","nickname":"ignored extra signup input"})).await;
    assert!(
        signup["nickname"].is_null(),
        "stock signup must still forward only its source inputs"
    );
    blog_web::client::shutdown(&server).await.unwrap();
    drop(server);
    drop(old.server);
    old.root.close().unwrap();
}
