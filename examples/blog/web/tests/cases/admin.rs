//! Real generated admin surfaces for every composed model, with actual source role grants.
use super::support::App;
use serde_json::json;
use stackpress_lib::Body;

/// Every model's search/create and ordinary identity detail/update retain the same source UI.
#[tokio::test(flavor = "current_thread")]
async fn generated_model_pages_and_role_grants_use_the_actual_family() {
    let app = App::new().await;
    let applications = app.emit("application-search", json!({})).await;
    assert_eq!(app.emit("session-create",json!({"id":"admin-proof-session","applicationId":applications["results"][0]["id"],"profileId":"developer","scopes":["user"],"expires":"2099-01-01T00:00:00.000Z"})).await["code"],200);
    let admin = app.token("developer", vec!["ADMIN"]).await;
    for (module, label) in [
        ("profile", "Profiles"),
        ("auth", "Auth"),
        ("application", "Apps"),
        ("session", "Sessions"),
        ("category", "Categories"),
        ("catalog", "Catalogs"),
        ("article", "Articles"),
        ("comment", "Comments"),
    ] {
        for action in ["search", "create"] {
            let response = app
                .route(
                    "GET",
                    &format!("/admin/{module}/{action}"),
                    json!({}),
                    Some(&admin),
                    None,
                )
                .await;
            assert_eq!(response.status().unwrap().code, 200, "{module}/{action}");
            let Body::Text(html) = response.body() else {
                panic!("missing generated {module}/{action} page")
            };
            assert!(
                html.contains("api-management-seed") && html.contains(label),
                "{module}/{action}"
            );
        }
        // Composite identity pages stay under the separate enabled Catalog regression.
        if module == "catalog" {
            continue;
        }
        let rows = app.emit(&format!("{module}-search"), json!({})).await;
        let id = rows["results"][0]["id"].as_str().unwrap();
        for action in ["detail", "update", "copy", "remove", "export", "import"] {
            let path = if ["export", "import"].contains(&action) {
                format!("/admin/{module}/{action}")
            } else {
                format!("/admin/{module}/{action}/{id}")
            };
            let response = app.route("GET", &path, json!({}), Some(&admin), None).await;
            assert_eq!(response.status().unwrap().code, 200, "{path}");
            let Body::Text(html) = response.body() else {
                panic!("missing generated {path} view")
            };
            if action == "export" {
                assert!(
                    response.mimetype().unwrap().starts_with("text/csv"),
                    "{path}"
                );
                assert!(html.contains("id"), "{path}");
            } else {
                assert!(html.contains("api-management-seed"), "{path}");
            }
            if module == "article" && action == "update" {
                assert!(
                    html.contains("contenteditable"),
                    "rich editor must be mounted"
                );
            }
        }
    }
    let manager = app.token("developer", vec!["MANAGER"]).await;
    let user = app.token("developer", vec!["USER"]).await;
    assert_eq!(
        app.route("GET", "/admin/auth/search", json!({}), Some(&manager), None)
            .await
            .status()
            .unwrap()
            .code,
        200
    );
    assert_eq!(
        app.route(
            "GET",
            "/admin/article/search",
            json!({}),
            Some(&manager),
            None
        )
        .await
        .status()
        .unwrap()
        .code,
        401
    );
    assert_eq!(
        app.route("GET", "/admin/article/search", json!({}), Some(&user), None)
            .await
            .status()
            .unwrap()
            .code,
        401
    );
    app.close().await;
}
