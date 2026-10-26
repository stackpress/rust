//! Real HTTP/lifecycle/render integration, including early Abort and failed load.

mod support;

use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{
    ServerBuilder,
    http::{HttpAdapter, HttpOutput},
};

/// Obtain complete native HTML without opening a socket.
async fn html(app: &stackpress_server::Server, path: &str) -> (u16, String) {
    let input = http::Request::builder().uri(path).body(Vec::new()).unwrap();
    let reply = HttpAdapter::new(Default::default())
        .unwrap()
        .handle(app, input)
        .await
        .unwrap();
    match reply.output {
        HttpOutput::Response(res) => (
            res.status().as_u16(),
            String::from_utf8(res.into_body()).unwrap(),
        ),
        _ => panic!("expected buffered output"),
    }
}

/// Real generated operation output is already present before client execution.
#[tokio::test(flavor = "multi_thread")]
async fn generated_html_and_safe_seed() {
    let (_directory, app) = support::application(stackpress_framework_view::BasePath::default())
        .await
        .unwrap();
    let (a, b) = tokio::join!(
        html(&app, "/ui/profiles?id=abc123"),
        html(&app, "/ui/profiles?id=abc456")
    );
    assert_eq!(a.0, 200);
    assert!(a.1.contains("Ada Lovelace"));
    assert!(!a.1.contains("Grace Hopper"));
    assert!(b.1.contains("Grace Hopper"));
    assert!(a.1.contains("<title>Profiles</title>"));
    assert!(a.1.contains("Search public demo profiles"));
    assert!(a.1.contains("id=\"stackpress-page\""));
    let invalid = html(&app, "/ui/profiles?id=").await;
    assert!(!invalid.1.contains("<main>"));
}

/// Child Abort intentionally stops page processing before a render is allocated.
async fn abort(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.html("operation aborted")?;
    Ok(Flow::Abort)
}

/// Delegation preserves the child response and never appends a page shell.
#[tokio::test]
async fn child_abort_never_renders() {
    stackpress_framework_ui::render::initialize_ssr().unwrap();
    let mut builder = ServerBuilder::new();
    builder.on("profile-search", abort).unwrap();
    builder
        .get("/ui/profiles", stackpress_example_profile_ui_web::page)
        .unwrap();
    let app = builder.build().await.unwrap();
    let (_, body) = html(&app, "/ui/profiles?id=abc").await;
    assert_eq!(body, "operation aborted");
}
