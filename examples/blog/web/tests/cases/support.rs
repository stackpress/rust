//! Owned SQLite fixtures shared by the finite app integration proofs.
use serde_json::{Value, json};
use stackpress_lib::{Request, Response};
use stackpress_server::{ConfigLayers, Server};
use std::sync::Arc;

/// Fixture lifetime owns every mutable file, the host and its shutdown barrier.
pub struct App {
    pub root: tempfile::TempDir,
    pub server: Arc<Server>,
}
impl App {
    /// Explicitly install/populate a fresh database, then build the complete host.
    pub async fn new() -> Self {
        Self::configured(json!({})).await
    }
    /// Override real application layers, retaining this fixture's file/network ownership.
    pub async fn configured(extra: Value) -> Self {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let path = root.path().join("blog.sqlite");
        blog_web::plugins::store::setup(&path).await.unwrap();
        let mut layers = ConfigLayers::new();
        // Webhook delivery itself is a separate framework proof; never make network requests.
        layers
            .value(
                "owned app fixture",
                json!({"sqlite":{"path":path},
            "client":{"revisions":root.path().join("revisions")},
            "database":{"migrations":root.path().join("migrations")},"api":{"webhooks":[]}}),
            )
            .unwrap();
        layers
            .value("explicit app fixture overrides", extra)
            .unwrap();
        let server = blog_web::client::build(layers, "abc123", Default::default())
            .await
            .unwrap();
        let app = Self { root, server };
        assert_eq!(app.emit("populate", json!({})).await["code"], 200);
        app
    }
    /// Preserve native Date results through the source auth bridge at the test wire boundary.
    pub async fn emit(&self, event: &str, data: Value) -> Value {
        let mut request = Request::builder().data(data).build().unwrap();
        let mut response = Response::new();
        self.server
            .emit(event, &mut request, &mut response)
            .await
            .unwrap();
        json!({"code":response.status().map(|status|status.code),
            "results":stackpress_framework_session::auth::events::json_result(&response).unwrap(),
            "error":response.error(),"total":response.total()})
    }
    /// Invoke actual registered routes with source cookies and optional API authorization.
    pub async fn route(
        &self,
        method: &str,
        path: &str,
        data: Value,
        token: Option<&str>,
        authorization: Option<&str>,
    ) -> Response {
        let mut request = Request::builder()
            .method(method)
            .url(path)
            .data(data)
            .build()
            .unwrap();
        request.session.set("csrf", "fixture-csrf", None).unwrap();
        if let Some(token) = token {
            request.session.set("session", token, None).unwrap();
        }
        if let Some(value) = authorization {
            request.headers.set("authorization", value).unwrap();
        }
        let mut response = Response::new();
        let report = self
            .server
            .handle_route(&mut request, &mut response)
            .await
            .unwrap();
        // Expected 404 recovery is itself a lifecycle failure; retain only unexpected 500s.
        assert!(
            response.status().is_none_or(|status| status.code != 500),
            "route {path}: {:?}",
            report
                .failures
                .iter()
                .map(|failure| &failure.error)
                .collect::<Vec<_>>()
        );
        response
    }
    /// Source identity tokens carry explicit roles for real authorization tests.
    pub async fn token(&self, id: &str, roles: Vec<&str>) -> String {
        self.server
            .host()
            .service::<stackpress_framework_session::Service>()
            .unwrap()
            .create(
                json!({"id":id,"name":"Fixture Reader","roles":roles})
                    .as_object()
                    .unwrap()
                    .clone(),
            )
            .await
            .unwrap()
    }
    /// Reap only this host's hooks and SQLite resources before deleting the fixture directory.
    pub async fn close(self) {
        blog_web::client::shutdown(&self.server).await.unwrap();
        drop(self.server);
        self.root.close().unwrap();
    }
}
