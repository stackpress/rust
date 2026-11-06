//! Raw error guard order and G42's cooperative multiple-handler contribution.
use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result, SourceLocation};
use stackpress_server::{ServerBuilder, StartupContext};

/// A second handler observes Continue; terminal Abort must skip it.
async fn later(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.data_mut()?.set("later", true)?;
    Ok(Flow::Continue)
}
/// Register after the app contribution, retaining FIFO semantics.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("error", later)?;
    Ok(Flow::Continue)
}

#[tokio::test(flavor = "current_thread")]
async fn error_guards_preserve_terminal_abort_non_get_and_truthy_bodies() {
    let mut builder = ServerBuilder::new();
    blog_web::plugins::app::register(&mut builder).unwrap();
    builder.on_listen(listen).unwrap();
    let app = builder.build().await.unwrap();
    for (method, mime, body, terminal) in [
        ("GET", "terminal/arguments", Body::Empty, true),
        ("POST", "text/plain", Body::Empty, false),
        ("GET", "text/plain", Body::Text("existing".into()), false),
        ("GET", "text/plain", Body::Bytes(vec![]), false),
        (
            "GET",
            "text/plain",
            Body::Value(serde_json::json!({}).try_into().unwrap()),
            false,
        ),
    ] {
        let mut request = Request::builder()
            .method(method)
            .mimetype(mime)
            .build()
            .unwrap();
        let mut response = Response::new();
        response.set("text/plain", body.clone()).unwrap();
        app.emit("error", &mut request, &mut response)
            .await
            .unwrap();
        match (&body, response.body()) {
            (Body::Empty, Body::Empty) => {}
            (Body::Text(expected), Body::Text(actual)) => assert_eq!(actual, expected),
            (Body::Bytes(expected), Body::Bytes(actual)) => assert_eq!(actual, expected),
            (Body::Value(expected), Body::Value(actual)) => {
                assert_eq!(actual.to_json().unwrap(), expected.to_json().unwrap())
            }
            _ => panic!("body representation changed"),
        }
        assert_eq!(response.data().has("later"), !terminal);
        assert!(!response.data().has("stack"));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn json_error_captures_only_owned_absolute_snippets_without_rendering() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let path = root.path().join("source.rs");
    std::fs::write(&path, "before\nmain\nafter\n").unwrap();
    let site = SourceLocation {
        file: path.to_str().unwrap().into(),
        line: 2,
        column: 3,
    };
    let mut builder = ServerBuilder::new();
    blog_web::plugins::app::register(&mut builder).unwrap();
    let app = builder.build().await.unwrap();
    let mut request = Request::builder()
        .data(serde_json::json!({"json":true}))
        .build()
        .unwrap();
    let mut response = Response::new();
    response.status_code(500, None).unwrap();
    response
        .set_stack(vec![
            site.clone(),
            site,
            SourceLocation {
                file: "relative.rs".into(),
                line: 2,
                column: 1,
            },
        ])
        .unwrap();
    app.emit("error", &mut request, &mut response)
        .await
        .unwrap();
    let stack = response.data().get("stack").unwrap().to_json().unwrap();
    assert!(stack[0].get("snippet").is_none());
    assert_eq!(stack[1]["snippet"]["before"], "before");
    assert_eq!(stack[1]["snippet"]["main"], "main");
    assert_eq!(stack[1]["snippet"]["location"], "  ^");
    assert_eq!(stack[1]["snippet"]["after"], "after");
    assert!(stack[2].get("snippet").is_none());
    assert!(matches!(response.body(), Body::Empty));
    assert_eq!(response.status().unwrap().code, 500);
}
