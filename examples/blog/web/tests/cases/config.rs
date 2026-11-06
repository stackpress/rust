//! Typed layers feed registration-time generated UI and request-time constituent plugins.
use super::support::App;
use serde_json::json;
use stackpress_lib::Body;

/// Source falsy defaults and the four-field allowlist are shared across host adapters.
#[test]
fn public_brand_preserves_source_fallbacks_and_omits_private_extensions() {
    let brand = blog_web::config::Brand(
        json!({"name":"","logo":null,"icon":false,"favicon":"/own.svg","private":"not public"}),
    )
    .public();
    assert_eq!(
        brand,
        json!({"name":"Stackpress","logo":"/logo.png","icon":"/icon.png","favicon":"/own.svg"})
    );
}

#[tokio::test(flavor = "current_thread")]
async fn explicit_brand_admin_csrf_and_source_locale_reach_actual_views() {
    let app=App::configured(json!({"brand":{"name":"Owned Publication"},"admin":{"name":"Owned Admin"},"csrf":{"name":"owned-csrf"}})).await;
    assert_eq!(
        app.server
            .host()
            .service::<stackpress_framework_csrf::Service>()
            .unwrap()
            .name(),
        "owned-csrf"
    );
    let token = app.token("developer", vec!["ADMIN"]).await;
    let admin = app
        .route(
            "GET",
            "/admin/article/create",
            json!({}),
            Some(&token),
            None,
        )
        .await;
    let Body::Text(html) = admin.body() else {
        panic!("missing generated form")
    };
    assert!(html.contains("Owned Publication"));
    assert!(html.contains("Owned Admin"));
    assert!(html.contains("owned-csrf"));
    let signin = app
        .route("GET", "/auth/signin/username", json!({}), None, None)
        .await;
    let Body::Text(html) = signin.body() else {
        panic!("missing native auth view")
    };
    assert!(html.contains("Owned Publication"));
    // Source prefix dispatch stages an outgoing cookie; incoming-session language is unchanged.
    let home = app.route("GET", "/th_TH/", json!({}), None, None).await;
    let Body::Text(html) = home.body() else {
        panic!("missing locale page")
    };
    assert_eq!(
        home.session().get("locale").unwrap().as_str(),
        Some("th_TH")
    );
    assert!(html.contains("lang=\"en-US\""));
    assert!(html.contains("Owned Publication"));
    // A subsequent request uses that saved locale and the original Thai demonstration phrase.
    let mut request = stackpress_lib::Request::new();
    request.session.set("locale", "th_TH", None).unwrap();
    let mut translated = stackpress_lib::Response::new();
    app.server
        .handle_route(&mut request, &mut translated)
        .await
        .unwrap();
    let Body::Text(html) = translated.body() else {
        panic!("missing translated page")
    };
    assert!(html.contains("Home Pagesss"));
    assert!(html.contains("lang=\"th-TH\""));
    app.close().await;
}
