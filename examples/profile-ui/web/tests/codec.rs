//! Real buffered HTTP input, typed projection, mounted routes and sanitized errors.
use stackpress_example_profile_ui_contracts::{BUILD, REVISION, SearchPage, SearchReply};
use stackpress_framework_view::{BasePath, PageSnapshot};
use stackpress_server::{
    LifecycleStage,
    http::{HttpAdapter, HttpOutput},
};

/// Exercise the existing HTTP adapter without binding any socket.
async fn call(
    app: &stackpress_server::Server,
    method: &str,
    path: &str,
    mime: &str,
    bytes: Vec<u8>,
) -> (u16, String, stackpress_server::HandleReport) {
    let req = http::Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", mime)
        .header("x-private", "HEADER_SENTINEL")
        .header("cookie", "secret=COOKIE_SENTINEL")
        .body(bytes)
        .unwrap();
    let reply = HttpAdapter::new(Default::default())
        .unwrap()
        .handle(app, req)
        .await
        .unwrap();
    match reply.output {
        HttpOutput::Response(res) => (
            res.status().as_u16(),
            String::from_utf8(res.into_body()).unwrap(),
            reply.report,
        ),
        _ => panic!("expected native response"),
    }
}

/// Body fields are exact, input paths are allowlisted and the operation is fixed.
#[tokio::test]
async fn strict_fixed_codec() {
    let app = stackpress_example_profile_ui_web::application()
        .await
        .unwrap();
    for (body, mime, ok) in [
        (
            r#"{"revision":1,"id":"abc456"}"#.to_owned(),
            "application/json; charset=utf-8",
            true,
        ),
        (
            r#"{"revision":1,"id":"zzz"}"#.into(),
            "application/json",
            true,
        ),
        (
            r#"{"revision":1,"id":"abc","event":"private-event"}"#.into(),
            "application/json",
            false,
        ),
        (
            r#"{"revision":1,"id":"abc","id":"other"}"#.into(),
            "application/json",
            false,
        ),
        (
            r#"{"revision":999,"id":"abc"}"#.into(),
            "application/json",
            false,
        ),
        (
            r#"{"revision":1,"id":""}"#.into(),
            "application/json",
            false,
        ),
        (r#"{"id":"abc"}"#.into(), "application/json", false),
        (r#"{"revision":1,"id":"abc"}"#.into(), "text/plain", false),
        ("{".into(), "application/json", false),
        (
            format!(r#"{{"revision":1,"id":"{}"}}"#, "x".repeat(65536)),
            "application/json",
            false,
        ),
    ] {
        let (status, text, report) = call(
            &app,
            "POST",
            "/ui-api/v1/profile-search?id=abc123&event=private",
            mime,
            body.into_bytes(),
        )
        .await;
        assert!(!text.contains("HEADER_SENTINEL") && !text.contains("COOKIE_SENTINEL"));
        assert!(!text.contains("HandleReport") && !text.contains("private-event"));
        assert_eq!(
            report
                .stages
                .iter()
                .filter(|row| row.stage == LifecycleStage::Input)
                .count(),
            1
        );
        assert!(
            report
                .stages
                .iter()
                .filter(|row| row.stage == LifecycleStage::Operation)
                .count()
                <= 1
        );
        if ok {
            assert_eq!(status, 200, "{text}");
            let reply: SearchReply = serde_json::from_str(&text).unwrap();
            reply.validate().unwrap();
            assert!(
                !text.contains("Ada Lovelace"),
                "query must not replace typed body: {text}"
            );
        } else {
            assert!(status >= 400, "{text}");
        }
    }
}

/// SSR forms, canonical URLs, detail status and asset URLs share one trusted mount.
#[tokio::test]
async fn mounted_route_head_and_seed_agreement() {
    let app = stackpress_example_profile_ui_web::application_with_base(
        BasePath::new("/demo/v1").unwrap(),
    )
    .await
    .unwrap();
    let (status, html, report) = call(
        &app,
        "GET",
        "/demo/v1/ui/profiles?id=abc456&secret=QUERY_SENTINEL",
        "text/plain",
        vec![],
    )
    .await;
    assert_eq!(status, 200);
    assert!(report.failures.is_empty());
    assert!(html.contains("action=\"/demo/v1/ui/profiles\""));
    assert!(html.contains("src=\"/demo/v1/assets/start.js\""));
    assert!(!html.contains("SENTINEL"));
    let json = html
        .split("id=\"stackpress-page\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let snapshot = PageSnapshot::<SearchPage>::decode(json, REVISION, BUILD).unwrap();
    assert_eq!(snapshot.location, "/demo/v1/ui/profiles?id=abc456");
    assert_eq!(snapshot.status.code(), 200);
    assert_eq!(snapshot.data.reply.total, 1);
    let (status, html, _) = call(
        &app,
        "GET",
        "/demo/v1/ui/profiles/abc456",
        "text/plain",
        vec![],
    )
    .await;
    assert_eq!(status, 200);
    assert!(html.contains("Grace Hopper"));
    assert!(!html.contains("Ada Lovelace"));
    let (status, html, _) = call(
        &app,
        "GET",
        "/demo/v1/ui/profiles/zzz",
        "text/plain",
        vec![],
    )
    .await;
    assert_eq!(status, 404);
    assert!(html.contains("\"code\":404"));
    let (status, _, _) = call(&app, "GET", "/ui/profiles?id=abc", "text/plain", vec![]).await;
    assert_eq!(status, 404);
    let (status, text, _) = call(
        &app,
        "POST",
        "/demo/v1/ui-api/v1/profile-search",
        "application/json",
        br#"{"revision":1,"id":"abc123"}"#.to_vec(),
    )
    .await;
    assert_eq!(status, 200, "{text}");
    assert!(text.contains("Ada Lovelace"));
}

/// Foreign or incomplete output is observed once; a page never starts a second write.
#[tokio::test]
async fn foreign_delivery_and_conversion_failure() {
    use stackpress_lib::{DeliveryState, Error, Flow, Invocation, Request, Response, Result};
    use stackpress_server::ServerBuilder;
    async fn finalize(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        match req.data.path_as::<String>("mode")?.as_deref() {
            Some("stop") => res.stop()?,
            Some("failed") => {
                let _ = res
                    .dispatch_with(|_| async { Err(Error::new("private writer sentinel")) })
                    .await;
            }
            Some("attempted") => {
                // Poll the owned dispatch exactly once, then cancel it at its pending write.
                let mut pending =
                    Box::pin(res.dispatch_with(|_| std::future::pending::<Result<()>>()));
                std::future::poll_fn(|cx| {
                    assert!(pending.as_mut().poll(cx).is_pending());
                    std::task::Poll::Ready(())
                })
                .await;
                drop(pending);
            }
            _ => res.dispatch().await?,
        }
        Ok(Flow::Continue)
    }
    stackpress_framework_ui::render::initialize_ssr().unwrap();
    let mut builder = ServerBuilder::new();
    builder.get("/ui/profiles", finalize).unwrap();
    stackpress_example_profile_ui_web::register_pages(&mut builder, BasePath::default()).unwrap();
    let app = builder.build().await.unwrap();
    for (mode, state) in [
        ("sent", DeliveryState::Sent),
        ("failed", DeliveryState::Failed),
        ("attempted", DeliveryState::Attempted),
        ("stop", DeliveryState::Stopped),
    ] {
        let req = http::Request::builder()
            .uri(format!("/ui/profiles?mode={mode}&id=abc"))
            .body(vec![])
            .unwrap();
        let reply = HttpAdapter::new(Default::default())
            .unwrap()
            .handle(&app, req)
            .await
            .unwrap();
        if mode == "stop" {
            assert!(matches!(reply.output, HttpOutput::Suppressed));
        } else {
            assert!(
                matches!(reply.output,HttpOutput::Unavailable { state:observed } if observed==state)
            );
        }
        assert!(!reply.report.output_eligible);
    }
    // A conversion failure before Request normalization never dispatches page hooks.
    let req = http::Request::builder()
        .method("CONNECT")
        .uri("profiles.invalid:443")
        .body(vec![])
        .unwrap();
    let reply = HttpAdapter::new(Default::default())
        .unwrap()
        .handle(&app, req)
        .await
        .unwrap();
    assert!(matches!(reply.output,HttpOutput::Response(res) if res.status()==400));
    assert!(
        !reply
            .report
            .stages
            .iter()
            .any(|row| row.stage == LifecycleStage::Operation)
    );
}

/// The actual app definitions produce the same Leptos match and native public location.
#[tokio::test]
async fn encoded_route_contract() {
    use stackpress_example_profile_ui_web::routes;
    use stackpress_framework_ui::{leptos_router::PossibleRouteMatch, routing::CatalogPath};
    let base = BasePath::new("/demo").unwrap();
    let detail = routes::detail().unwrap();
    let path = detail
        .location(&BasePath::default(), &[("id", "雪/+&0")])
        .unwrap();
    let matched = CatalogPath::new(detail).test(path.as_str()).unwrap();
    let encoded = matched.params()[0].1.clone();
    assert_eq!(routes::decode_parameter(&encoded).unwrap(), "雪/+&0");
    for bad in ["%", "%0X", "%FF", "..", "%2e", ""] {
        assert!(routes::decode_parameter(bad).is_err());
    }
    let app = stackpress_example_profile_ui_web::application_with_base(base.clone())
        .await
        .unwrap();
    let request = base.join(path.as_str()).unwrap();
    let (status, html, _) = call(&app, "GET", request.as_str(), "text/plain", vec![]).await;
    assert_eq!(status, 404, "{html}");
    let text = html
        .split("id=\"stackpress-page\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let seed = PageSnapshot::<SearchPage>::decode(text, REVISION, BUILD).unwrap();
    assert_eq!(seed.data.input.id, "雪/+&0");
    assert_eq!(seed.location, format!("{}?return=abc", request.as_str()));
    assert_eq!(seed.status.code(), 404);
    assert!(html.contains("Profile not found"));
    assert_eq!(seed.data.detail.as_ref().unwrap().search_prefix, "abc");
    let (status, html, _) = call(
        &app,
        "GET",
        "/demo/ui/profiles/abc456?return=abc123",
        "text/plain",
        vec![],
    )
    .await;
    assert_eq!(status, 200);
    assert!(html.contains("Grace Hopper"));
    assert!(!html.contains("Ada Lovelace"));
}

/// Exact non-2xx host error contract, kept separate from successful SearchReply.
#[tokio::test]
async fn public_error_envelope_contract() {
    let app = stackpress_example_profile_ui_web::application()
        .await
        .unwrap();
    let (code, body, _) = call(
        &app,
        "POST",
        "/ui-api/v1/profile-search",
        "application/json",
        br#"{"revision":999,"id":"abc"}"#.to_vec(),
    )
    .await;
    assert_eq!(code, 400);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&body).unwrap(),
        serde_json::json!({
            "code":400,"status":"Bad Request","error":"Page version changed. Refresh to continue.","errors":{},"total":0
        })
    );
}

/// The same native page root consumes explicit French/dark settings and real detail seeds.
#[tokio::test]
async fn initial_settings_and_detail_projection() {
    let app = stackpress_example_profile_ui_web::application()
        .await
        .unwrap();
    for (id, code, heading) in [
        ("abc456", 200, "Grace Hopper"),
        ("missing", 404, "Profil introuvable"),
    ] {
        let path = format!("/ui/profiles/{id}?return=abc");
        let (status, html, _) = call(&app, "GET", &path, "text/plain", vec![]).await;
        assert_eq!(status, code);
        let text = html
            .split("id=\"stackpress-page\">")
            .nth(1)
            .unwrap()
            .split("</script>")
            .next()
            .unwrap();
        let mut seed = PageSnapshot::<SearchPage>::decode(text, REVISION, BUILD).unwrap();
        assert_eq!(seed.location, path);
        seed.locale = "fr".into();
        seed.theme = "dark".into();
        let html = stackpress_example_profile_ui_web::render_snapshot(seed)
            .await
            .unwrap();
        assert!(html.contains("lang=\"fr\""));
        assert!(html.contains("data-theme=\"dark\""));
        assert!(html.contains(heading));
        assert!(html.contains("Retour à la recherche"));
        assert!(html.contains("<title>Profil</title>"));
    }
}
