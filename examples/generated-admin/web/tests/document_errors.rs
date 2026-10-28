//! Public document/API representation boundaries over real generated host registration.
use generated_admin_web::{database, host};
use stackpress_lib::{Error, Flow, handler};
use stackpress_server::{
    ServerBuilder,
    http::{HttpAdapter, HttpOutput},
};
use std::sync::Arc;

/// Exercise status, privacy, mount and diagnostic behaviour without a listener.
#[test]
fn document_failures_keep_html_and_api_failures_keep_json() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(prove_document_errors());
}

/// Keep all real SQLite connections inside the owned finite runtime.
async fn prove_document_errors() {
    stackpress_framework_ui::render::initialize_ssr().unwrap();
    for mount in ["/", "/demo/admin/"] {
        let directory = tempfile::tempdir().unwrap();
        let engine = database::open(&directory.path().join("fixture.sqlite"), false)
            .await
            .unwrap();
        database::initialize(&engine).await.unwrap();
        let mut builder = ServerBuilder::new();
        host::configure(
            &mut builder,
            engine.clone(),
            host::Config {
                mount: mount.into(),
                csrf: "fixture".into(),
                csr: false,
                transfer_limits: Default::default(),
            },
            Arc::new(host::FixturePolicy),
            "https://admin.invalid".into(),
        )
        .unwrap();
        // An app-owned request failure proves representation without swallowing diagnostics.
        builder
            .on_priority(
                "request",
                1000,
                handler(|req, _, _| {
                    Box::pin(async move {
                        if req.headers.get("x-test-failure").is_some() {
                            return Err(Error::new("PRIVATE_DOCUMENT_FAILURE")
                                .with_code(500)
                                .with_public_message("<script>public but not page copy</script>"));
                        }
                        Ok(Flow::Continue)
                    })
                }),
            )
            .unwrap();
        let app = builder.build().await.unwrap();
        let adapter = HttpAdapter::new(Default::default()).unwrap();
        for (method, path, actor, failure, status, html) in [
            ("GET", "profile/search", "denied", false, 403, true),
            (
                "GET",
                "profile/search?state=%7Bbad",
                "denied",
                false,
                403,
                true,
            ),
            (
                "GET",
                "profile/search?state=%7Bbad",
                "editor",
                false,
                400,
                true,
            ),
            (
                "GET",
                "profile/detail?key=%7B%22id%22%3A%22missing%22%7D",
                "editor",
                false,
                404,
                true,
            ),
            ("GET", "profile/search", "editor", true, 500, true),
            ("GET", "profile/search", "editor", false, 200, true),
            ("POST", "api/profile/search", "denied", false, 403, false),
            ("POST", "api/profile/search", "editor", true, 500, false),
            ("GET", "profile/export", "denied", false, 403, false),
            ("GET", "profile/export", "editor", true, 500, false),
            ("GET", "unknown/search", "editor", false, 404, false),
            ("GET", "project/remove", "editor", false, 404, false),
        ] {
            let mut request = http::Request::builder()
                .method(method)
                .uri(format!("{mount}{path}"))
                .header("x-admin-actor", actor)
                // Representation follows compiled route authority, never Accept guessing.
                .header(
                    "accept",
                    if html {
                        "application/json"
                    } else {
                        "text/html"
                    },
                );
            if failure {
                request = request.header("x-test-failure", "1");
            }
            let reply = adapter
                .handle(&app, request.body(Vec::new()).unwrap())
                .await
                .unwrap();
            if failure {
                assert!(
                    reply
                        .report
                        .failures
                        .iter()
                        .any(|failure| failure.error.message() == "PRIVATE_DOCUMENT_FAILURE")
                );
            }
            let HttpOutput::Response(response) = reply.output else {
                panic!("buffered output required")
            };
            assert_eq!(response.status().as_u16(), status, "{method} {mount}{path}");
            let body = String::from_utf8(response.body().clone()).unwrap();
            assert_eq!(
                response.headers()["content-type"]
                    .to_str()
                    .unwrap()
                    .starts_with("text/html"),
                html
            );
            if html && status >= 400 {
                assert!(body.starts_with("<!doctype html>"));
                assert!(body.contains(&format!("href=\"{mount}profile/search\"")));
                assert_eq!(response.headers()["cache-control"], "no-store");
                for forbidden in [
                    "PRIVATE_DOCUMENT_FAILURE",
                    "<script",
                    "admin-seed",
                    "%ZZ",
                    "public but not page copy",
                ] {
                    assert!(!body.contains(forbidden), "{forbidden} leaked into {body}");
                }
                assert!(body.contains(if status == 403 {
                    "Ask an administrator"
                } else {
                    "list"
                }));
            } else if !html {
                let envelope: serde_json::Value = serde_json::from_str(&body).unwrap();
                assert_eq!(envelope["code"], status);
            }
        }
        drop(app);
        engine.close().await.unwrap();
    }
}

/// Static failures escape link attributes without emitting protected data or scripts.
#[test]
fn error_markup_does_not_allow_attribute_breakout() {
    let body = generated_admin_web::render::failure_document(500, "/x\" onclick=\"alert(1)");
    assert!(!body.contains("href=\"/x\" onclick"));
    assert!(body.contains("&quot;"));
    assert!(!body.contains("<script"));
}
