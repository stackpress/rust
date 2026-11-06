//! Full emitted family consumers; source admin must retain compound model identities.

use serde_json::json;
use stackpress_lib::{Request, Response};
use stackpress_server::{ConfigLayers, ServerBuilder};
use stackpress_sql::{Engine, Select};

/// One complete generated family supplies CRUD, native auth collaborators, API joins and admin.
#[tokio::test(flavor = "current_thread")]
async fn integrated_family_signin_api_scopes_admin_and_export() {
    let app = super::support::App::new().await;
    let signin = app
        .emit(
            "auth-signin",
            json!({"username":"developer","secret":"developer","2fa":false}),
        )
        .await;
    assert_eq!(signin["code"], 200, "{signin}");
    let id = signin["results"]["profile"]["id"].as_str().unwrap();
    let token = app.token(id, vec!["ADMIN"]).await;
    let apps = app.emit("application-search", json!({})).await;
    let seeded = apps["results"][0]["id"].as_str().unwrap();
    let denied = app
        .route(
            "GET",
            "/api/profile/search",
            json!({}),
            None,
            Some(&format!("Bearer {seeded}")),
        )
        .await;
    assert_eq!(denied.status().unwrap().code, 401);
    assert_eq!(app.emit("application-create",json!({"id":"with-user-scope","profileId":id,"name":"Positive API fixture", "scopes":["user"],"expires":"2099-01-01T00:00:00.000Z"})).await["code"],200);
    let allowed = app
        .route(
            "GET",
            "/api/profile/search",
            json!({}),
            None,
            Some("Bearer with-user-scope"),
        )
        .await;
    assert_eq!(allowed.status().unwrap().code, 200);
    let admin = app
        .route(
            "GET",
            "/admin/article/search",
            json!({}),
            Some(&token),
            None,
        )
        .await;
    let stackpress_lib::Body::Text(html) = admin.body() else {
        panic!("missing generated admin HTML")
    };
    assert!(
        html.contains("Articles") && html.contains("Minecraft"),
        "{html}"
    );
    let exported = app
        .route(
            "GET",
            "/auth/account/security/export",
            json!({"download":true}),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(exported.status().unwrap().code, 200);
    let stackpress_lib::Body::Text(csv) = exported.body() else {
        panic!("missing source CSV")
    };
    assert!(csv.contains("name"));
    let inventory = app
        .server
        .listeners()
        .filter(|listener| listener.pattern.ends_with("-batch"))
        .map(|listener| listener.pattern.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        inventory,
        [
            "profile-batch",
            "category-batch",
            "catalog-batch",
            "article-batch",
            "comment-batch",
            "application-batch",
            "session-batch",
            "auth-batch"
        ]
    );
    app.close().await;
}

/// A real Catalog form must redirect to search, as the source generator does for multiple IDs.
/// Keep this regression visible if the shared source-admin adapter lacks compound-key support.
#[tokio::test(flavor = "current_thread")]
async fn source_catalog_create_redirects_after_committing_its_compound_identity() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let path = fixture.path().join("catalog-admin.sqlite");
    blog_web::plugins::store::setup(&path).await.unwrap();
    let mut layers = ConfigLayers::new();
    layers
        .value("owned M2 database", json!({"sqlite":{"path":path}}))
        .unwrap();
    let mut app = ServerBuilder::new();
    app.config_layers(blog_web::config::shared("abc123").unwrap())
        .unwrap();
    app.config_layers(layers).unwrap();
    blog_web::plugins::store::register(&mut app).unwrap();
    // The actual emitted client selects exactly one data inventory and the same auth/API family.
    stackpress_framework::register_with(&mut app, &blog_web::generated::client::framework::CLIENT)
        .unwrap();
    stackpress_framework_api::views::admin::register_queued_with::<
        blog_web::generated::stores::family::Models,
    >(
        &mut app,
        blog_shared::ui::pages::ui(),
        Default::default(),
        blog_shared::ui::pages::MODELS,
    )
    .unwrap();
    let server = app.build().await.unwrap();
    let mut populate = Response::new();
    server
        .emit("populate", &mut Request::new(), &mut populate)
        .await
        .unwrap();
    let mut found = Response::new();
    server
        .emit("article-search", &mut Request::new(), &mut found)
        .await
        .unwrap();
    let articles = found.result().unwrap().to_json().unwrap();
    let article = articles[0]["id"].as_str().unwrap();
    let mut category = Response::new();
    let mut input = Request::builder()
        .data(json!({"id":"new-category","name":"M2 Category","slug":"m2-category"}))
        .build()
        .unwrap();
    server
        .emit("category-create", &mut input, &mut category)
        .await
        .unwrap();
    assert_eq!(category.status().unwrap().code, 200);
    let mut request = Request::builder()
        .method("POST")
        .url("/admin/catalog/create")
        .data(json!({"categoryId":"new-category","articleId":article,"csrf":"owned-test-token"}))
        .build()
        .unwrap();
    request
        .session
        .set("csrf", "owned-test-token", None)
        .unwrap();
    let mut response = Response::new();
    let routed = server.handle_route(&mut request, &mut response).await;
    // Observe SQL independently, and release the owned resources even if the route is defective.
    let engine = server.host().shared_service::<Engine>().unwrap();
    let rows = engine
        .fetch(
            &Select::new(["category_id", "article_id"])
                .from("catalog")
                .where_eq("category_id", "new-category"),
        )
        .await
        .unwrap();
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()
        .unwrap()
        .shutdown()
        .await;
    engine.close().await.unwrap();
    assert_eq!(
        rows.len(),
        1,
        "the actual compound-key insert must complete"
    );
    assert!(
        routed.is_ok(),
        "source Catalog form failed after inserting its row: {routed:?}"
    );
    assert_eq!(
        response.headers().get("Location"),
        Some("/admin/catalog/search"),
        "compound row committed, but source form returned status {:?}, error {:?}",
        response.status().map(|status| status.code),
        response.error()
    );
}
