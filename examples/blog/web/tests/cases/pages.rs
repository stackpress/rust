//! Source handlers, shared SSR and actual HTTP conversion over owned SQLite.
use super::support::App;
use serde_json::json;
use stackpress_lib::{Body, Response};

/// Extract the native view body without conflating a JSON result with rendered HTML.
fn html(response: &Response) -> &str {
    match response.body() {
        Body::Text(value) => value,
        _ => panic!("expected rendered body: {response:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn published_home_article_comments_and_missing_views() {
    let app = App::new().await;
    let data = app.emit("article-search", json!({})).await;
    let rows = data["results"].as_array().unwrap();
    let pokemon = rows.iter().find(|row| row["id"] == "pokemon").unwrap();
    let slug = pokemon["slug"].as_str().unwrap();
    let home = app.route("GET", "/", json!({}), None, None).await;
    assert_eq!(home.status().unwrap().code, 200, "{home:?}");
    assert!(html(&home).contains("Knowledge Console"));
    assert!(html(&home).contains("stackpress-page"));
    assert!(html(&home).contains("blog-v1"));
    assert!(html(&home).contains("lang=\"en-US\""));
    assert!(html(&home).contains("en_US"));
    assert!(html(&home).contains("Signin"));
    assert!(!html(&home).contains("My Account"));
    assert!(!html(&home).contains("abc123"));
    let article = app
        .route("GET", &format!("/articles/{slug}"), json!({}), None, None)
        .await;
    assert_eq!(article.status().unwrap().code, 200, "{article:?}");
    assert!(html(&article).contains("<p>On 27 February 1996"));
    assert!(html(&article).contains("article-comment-card"));
    assert!(html(&article).contains("/styles/article.css"));
    // G52 applies only to home: direct article details remain source-faithful.
    assert_eq!(
        app.emit("article-update", json!({"id":"pokemon","status":"DRAFT"}))
            .await["code"],
        200
    );
    let filtered = app
        .route(
            "GET",
            "/",
            json!({"json":true,"eq":{"status":"DRAFT"}}),
            None,
            None,
        )
        .await;
    let filtered = stackpress_framework_session::auth::events::json_result(&filtered).unwrap();
    assert!(
        filtered
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["status"] == "PUBLISHED")
    );
    assert!(
        !filtered
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == "pokemon")
    );
    let draft = app
        .route("GET", &format!("/articles/{slug}"), json!({}), None, None)
        .await;
    assert_eq!(draft.status().unwrap().code, 200);
    let missing = app
        .route("GET", "/articles/missing", json!({}), None, None)
        .await;
    assert_eq!(missing.status().unwrap().code, 404, "{missing:?}");
    assert!(html(&missing).contains("Not Found"));
    let json = app
        .route("GET", "/", json!({"json":true}), None, None)
        .await;
    assert!(!matches!(json.body(), Body::Text(_)));
    app.close().await;
}

#[tokio::test(flavor = "current_thread")]
async fn error_formats_and_public_assets_use_real_http_adapter() {
    let app = App::new().await;
    let adapter = stackpress_server::http::HttpAdapter::new(Default::default()).unwrap();
    for (path, status, mime, part) in [
        ("/articles/missing", 404, "text/html", "Not Found"),
        (
            "/articles/missing.js",
            404,
            "application/javascript",
            "console.log(",
        ),
        ("/articles/missing.css", 404, "text/css", "/* "),
        ("/missing", 401, "text/html", "Unauthorized"),
        ("/styles/global.css", 200, "text/css", "knowledge-hero"),
        ("/styles/article.css", 200, "text/css", "article-prose"),
        ("/assets/blog.js", 200, "text/javascript", "__blogDispose"),
    ] {
        let request = http::Request::builder().uri(path).body(Vec::new()).unwrap();
        let reply = adapter.handle(&app.server, request).await.unwrap();
        let stackpress_server::http::HttpOutput::Response(response) = reply.output else {
            panic!("response unavailable")
        };
        assert_eq!(
            response.status().as_u16(),
            status,
            "{path}: {:?}",
            String::from_utf8_lossy(response.body())
        );
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with(mime),
            "{path}: {response:?}"
        );
        assert!(
            String::from_utf8_lossy(response.body()).contains(part),
            "{path}"
        );
    }
    app.close().await;
}
