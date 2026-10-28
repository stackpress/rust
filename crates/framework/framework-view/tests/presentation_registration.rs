//! Opt-in presentation composition through the existing startup and lifecycle machinery.
#![cfg(all(feature = "web-preferences", feature = "web-flash"))]
use serde_json::{Value, json};
use stackpress_framework_view::{BasePath, PublicLocation, flash, preferences, theme};
use stackpress_lib::{Body, Flow, Headers, Invocation, Request, Response, Result};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, ServerInvocationExt, StartupContext};
use std::sync::Arc;

/// Checked data does not select executable integrations by itself.
fn config(base: &str) -> Value {
    json!({"view_preferences": preferences::Config {
        origin: "https://presentation.invalid".into(), base: BasePath::new(base).unwrap(),
        locales: vec!["en".into(), "fr".into()], default_locale: "en".into(),
        aliases: [("fr-CA".into(), "fr".into())].into(),
        settings_path: PublicLocation::new("/settings").unwrap(),
        confirmation_path: PublicLocation::new("/confirm").unwrap(), serve_theme_bootstrap: true,
    }, "view_flash": flash::Config {
        origin: "https://presentation.invalid".into(), base: BasePath::new(base).unwrap(),
        pages: vec![PublicLocation::new("/page").unwrap()],
    }})
}
/// Admission uses actual current headers, never public allowed-action hints.
async fn admission(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new(preferences::Admission::new(|request| {
        request.headers.get("x-current-csrf") == Some("verified")
    })))?;
    Ok(Flow::Continue)
}
/// An application composite calls functions, without selecting plugins during sealed startup.
fn presentation(app: &mut ServerBuilder) -> Result<()> {
    app.on_config(admission)?;
    preferences::register(app)?;
    flash::register(app)
}
/// Trusted layers and executable selection remain separate operations.
fn builder(value: Value, selected: bool) -> ServerBuilder {
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers.value("test", value).unwrap();
    app.config_layers(layers).unwrap();
    if selected {
        app.plugin("presentation", presentation).unwrap();
    }
    app
}
/// Project only validated cookie content and finish HTML before the response hook.
async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let service = inv.service::<flash::Service>()?;
    let notice = service.notice(req)?;
    if let Some(notice) = notice {
        res.headers_mut()?.set("x-notice-id", notice.id())?;
        res.headers_mut()?
            .set("x-issued-cookie", &service.issue(&notice)?)?;
    }
    res.html("Readable fixed settings")?;
    if req.headers.get("x-fail") == Some("true") {
        res.status_code(500, None)?;
    }
    Ok(Flow::Continue)
}
/// Exercise one complete top-level route exchange, without an HTTP socket.
async fn exchange(
    app: &Server,
    method: &str,
    path: &str,
    body: Body,
    headers: &[(&str, &str)],
) -> Response {
    let mut values = Headers::new();
    for (key, value) in headers {
        values.set(key, value).unwrap();
    }
    let mut req = Request::builder()
        .url(format!("https://presentation.invalid{path}"))
        .method(method)
        .headers(values)
        .body(body)
        .build()
        .unwrap();
    let mut res = Response::new();
    let report = app.handle_route(&mut req, &mut res).await.unwrap();
    assert!(
        report.failures.is_empty() || res.status().is_some_and(|status| status.code == 404),
        "{report:?}"
    );
    res
}
/// One selection, no automatic writes, stable settings/confirmation envelopes and asset policy.
#[tokio::test]
async fn selected_settings_and_assets() {
    for base in ["", "/demo/v1"] {
        let app = builder(config(base), true).build().await.unwrap();
        assert_eq!(app.plugins().count(), 1);
        let body =
            json!({"locale":"fr","theme":"dark","location":format!("{base}/page")}).to_string();
        // CSRF admission is mandatory even with an allowed-looking caller payload.
        for (extra, status) in [
            (vec![], 403),
            (
                vec![
                    ("origin", "https://wrong.invalid"),
                    ("x-current-csrf", "verified"),
                ],
                403,
            ),
            (
                vec![
                    ("origin", "https://presentation.invalid"),
                    ("x-current-csrf", "verified"),
                ],
                200,
            ),
        ] {
            let res = exchange(
                &app,
                "POST",
                &format!("{base}/settings"),
                Body::Text(body.clone()),
                &extra,
            )
            .await;
            assert_eq!(res.status().unwrap().code, status);
            if status != 200 {
                assert!(res.headers().get("set-cookie").is_none());
            } else {
                let cookies = res.headers().get_all("set-cookie").unwrap();
                assert_eq!(cookies.len(), 2);
                for cookie in cookies {
                    assert!(
                        cookie.contains(&format!("Path={}/", base.trim_end_matches('/')))
                            || cookie.contains(&format!("Path={base};"))
                    );
                }
                assert!(cookies[0].starts_with("stackpress-locale=fr;"));
                assert!(cookies[1].starts_with("stackpress-theme=dark;"));
            }
        }
        // Read-only confirmation never stages a persistence header.
        let headers = [
            ("origin", "https://presentation.invalid"),
            ("x-current-csrf", "verified"),
            ("cookie", "stackpress-locale=fr; stackpress-theme=dark"),
        ];
        let res = exchange(
            &app,
            "POST",
            &format!("{base}/confirm"),
            Body::Text(body.clone()),
            &headers,
        )
        .await;
        assert_eq!(res.status().unwrap().code, 200);
        assert!(res.headers().get("set-cookie").is_none());
        for (body, status) in [
            (Body::Bytes(vec![0xff]), 400),
            (
                Body::Text("x".repeat(preferences::MAX_PREFERENCE_BYTES + 1)),
                413,
            ),
            (Body::Text("{}".into()), 400),
        ] {
            let res = exchange(&app, "POST", &format!("{base}/settings"), body, &headers).await;
            assert_eq!(res.status().unwrap().code, status);
            assert!(res.headers().get("set-cookie").is_none());
        }
        let asset = exchange(
            &app,
            "GET",
            &format!("{base}/assets/{}", theme::BOOTSTRAP_FILE),
            Body::Empty,
            &[],
        )
        .await;
        assert!(matches!(asset.body(), Body::Text(value) if value == theme::BOOTSTRAP_SOURCE));
        assert_eq!(
            asset.headers().get("cache-control"),
            Some("public, max-age=31536000, immutable")
        );
        assert_eq!(
            asset.headers().get("x-content-type-options"),
            Some("nosniff")
        );
        assert!(asset.headers().get("set-cookie").is_none());
    }
}
/// Feature/config presence alone cannot install routes, services or response hooks.
#[tokio::test]
async fn unselected_and_independent_flash() {
    assert!(builder(config(""), false).build().await.is_err()); // unclaimed data never selects executable code
    let app = builder(json!({}), false).build().await.unwrap();
    assert_eq!(app.plugins().count(), 0);
    assert_eq!(app.listeners().count(), 0);
    assert!(app.host().service::<preferences::Service>().is_err());
    assert!(app.host().service::<flash::Service>().is_err());
    let mut fixed = ServerBuilder::new();
    fixed
        .get(
            "/fixed",
            stackpress_lib::handler(|_, res, _| {
                Box::pin(async move {
                    res.html("Readable fixed Light settings")?;
                    Ok(Flow::Continue)
                })
            }),
        )
        .unwrap();
    let fixed = fixed.build().await.unwrap();
    let response = exchange(
        &fixed,
        "GET",
        "/fixed",
        Body::Empty,
        &[("cookie", "stackpress-theme=system; stackpress-flash=old")],
    )
    .await;
    assert_eq!(response.status().unwrap().code, 200);
    assert!(response.headers().get("set-cookie").is_none());
    for path in ["/settings", "/confirm", "/page"] {
        let res = exchange(&app, "POST", path, Body::Text("{}".into()), &[]).await;
        assert!(res.headers().get("set-cookie").is_none());
        assert_ne!(res.status().map(|s| s.code), Some(200));
    }
    for base in ["", "/demo/v1"] {
        let mut value = config(base);
        value.as_object_mut().unwrap().remove("view_preferences");
        let mut app = builder(value, false);
        app.plugin("flash", flash::register).unwrap();
        app.get(&format!("{base}/page"), page).unwrap();
        app.get(&format!("{base}/page-other"), page).unwrap();
        let app = app.build().await.unwrap();
        let notice = flash::FlashNotice::new("saved".into(), "Public <text>".into()).unwrap();
        let cookie = flash::FlashCookie::new(
            preferences::CookieScope::new(
                "https://presentation.invalid".into(),
                BasePath::new(base).unwrap(),
            )
            .unwrap(),
        )
        .issue(&notice)
        .unwrap();
        let cookie = cookie.split(';').next().unwrap();
        let res = exchange(
            &app,
            "GET",
            &format!("{base}/page"),
            Body::Empty,
            &[("cookie", cookie)],
        )
        .await;
        assert_eq!(res.headers().get("x-notice-id"), Some("saved"));
        let clear = res.headers().get("set-cookie").unwrap();
        assert!(clear.starts_with("stackpress-flash=;"));
        assert!(clear.contains("Max-Age=0"));
        for (method, path, headers) in [
            ("POST", format!("{base}/page"), vec![("cookie", cookie)]),
            (
                "GET",
                format!("{base}/page-other"),
                vec![("cookie", cookie)],
            ),
            (
                "GET",
                format!("{base}/page"),
                vec![("cookie", cookie), ("x-fail", "true")],
            ),
        ] {
            let res = exchange(&app, method, &path, Body::Empty, &headers).await;
            assert!(res.headers().get("set-cookie").is_none());
        }
    }
}
/// Startup rejects missing prerequisites, collisions and invalid typed inputs before publication.
#[tokio::test]
async fn rejected_startup() {
    let mut missing = ServerBuilder::new();
    missing
        .plugin("preferences", preferences::register)
        .unwrap();
    assert!(missing.build().await.is_err());
    let mut data = config("");
    data.as_object_mut().unwrap().remove("view_flash");
    let mut missing = builder(data, false);
    missing
        .plugin("preferences", preferences::register)
        .unwrap();
    let error = missing.build().await.err().unwrap();
    assert!(error.to_string().contains("Admission")); // no Admission service
    let mut missing = ServerBuilder::new();
    missing.plugin("flash", flash::register).unwrap();
    assert!(missing.build().await.is_err());
    let mut duplicate = builder(config(""), true);
    duplicate.on_config(admission).unwrap();
    let error = duplicate.build().await.err().unwrap();
    assert!(error.to_string().contains("Admission")); // conflicting typed service
    let mut duplicate = ServerBuilder::new();
    preferences::register(&mut duplicate).unwrap();
    assert!(preferences::register(&mut duplicate).is_err());
    let mut duplicate = ServerBuilder::new();
    flash::register(&mut duplicate).unwrap();
    assert!(flash::register(&mut duplicate).is_err());
    for (section, key, value) in [
        (
            "view_preferences",
            "origin",
            json!("https://presentation.invalid/path"),
        ),
        ("view_preferences", "locales", json!(["en", "en"])),
        ("view_preferences", "default_locale", json!("xx")),
        ("view_preferences", "settings_path", json!("/settings?x=1")),
        ("view_preferences", "settings_path", json!("/confirm")),
        ("view_preferences", "extra", json!(true)),
        ("view_flash", "pages", json!([])),
        ("view_flash", "pages", json!(["/page", "/page"])),
        ("view_flash", "pages", json!(["/page/:id"])),
    ] {
        let mut value_all = config("");
        value_all[section][key] = value;
        assert!(
            builder(value_all, true).build().await.is_err(),
            "{section}.{key}"
        );
    }
}

/// Test-only current-host authority; edited public JSON is deliberately ignored.
struct DomainPolicy;
impl stackpress_framework_view::access::HostPolicy<Request> for DomainPolicy {
    fn permits(&self, req: &Request, _: &stackpress_framework_view::access::ActionId) -> bool {
        req.headers.get("x-current-caller") == Some("verified")
            && (req.method.as_str() == "GET"
                || req.headers.get("x-current-csrf") == Some("verified"))
    }
}
/// Protected admission is unchanged with integrations selected or absent, including forged hints.
#[tokio::test]
async fn independent_protected_admission_and_lifecycle() {
    use stackpress_framework_view::access::{ActionId, HostAccess, HostPolicy};
    use std::sync::atomic::{AtomicUsize, Ordering};
    for base in ["", "/demo/v1"] {
        for selected in [false, true] {
            for configured in [false, true] {
                let mut app = builder(if selected { config(base) } else { json!({}) }, selected);
                let calls = Arc::new(AtomicUsize::new(0));
                let requests = Arc::new(AtomicUsize::new(0));
                let responses = Arc::new(AtomicUsize::new(0));
                for (event, counter) in [
                    ("request", requests.clone()),
                    ("response", responses.clone()),
                ] {
                    app.on(
                        event,
                        stackpress_lib::handler(move |_, _, _| {
                            counter.fetch_add(1, Ordering::SeqCst);
                            Box::pin(async { Ok(Flow::Continue) })
                        }),
                    )
                    .unwrap();
                }
                for method in [
                    stackpress_lib::MethodPattern::GET,
                    stackpress_lib::MethodPattern::POST,
                ] {
                    let calls = calls.clone();
                    app.route(
                        method,
                        &format!("{base}/private"),
                        stackpress_lib::handler(move |req, res, _| {
                            let policy: Option<&dyn HostPolicy<Request>> =
                                configured.then_some(&DomainPolicy);
                            let result = HostAccess::new(policy, req).run(
                                &ActionId::new("private.run").unwrap(),
                                || {
                                    calls.fetch_add(1, Ordering::SeqCst);
                                },
                            );
                            Box::pin(async move {
                                res.json(&json!({"allowed":result.is_ok()}))?;
                                if result.is_err() {
                                    res.status_code(403, None)?;
                                }
                                Ok(Flow::Continue)
                            })
                        }),
                    )
                    .unwrap();
                }
                let app = app.build().await.unwrap();
                for method in ["GET", "POST"] {
                    for headers in [
                        vec![],
                        vec![("x-current-caller", "verified")],
                        vec![
                            ("x-current-caller", "verified"),
                            ("x-current-csrf", "verified"),
                        ],
                    ] {
                        let before = calls.load(Ordering::SeqCst);
                        let allowed = configured
                            && !headers.is_empty()
                            && (method == "GET" || headers.len() == 2);
                        let res=exchange(&app,method,&format!("{base}/private?allowed=private.run"),Body::Text(r#"{"allowed":["private.run"],"caller":"verified","csrf":"verified"}"#.into()),&headers).await;
                        assert_eq!(res.status().unwrap().code, if allowed { 200 } else { 403 });
                        assert_eq!(calls.load(Ordering::SeqCst) - before, usize::from(allowed));
                        assert!(res.headers().get("set-cookie").is_none());
                    }
                }
                assert_eq!(requests.load(Ordering::SeqCst), 6);
                assert_eq!(responses.load(Ordering::SeqCst), 6);
            }
        }
    }
}
/// Optional assets and independent settings do not imply flash or domain authorization.
#[tokio::test]
async fn preferences_only_and_optional_asset() {
    let mut value = config("");
    value.as_object_mut().unwrap().remove("view_flash");
    value["view_preferences"]["serve_theme_bootstrap"] = json!(false);
    let mut app = builder(value, false);
    app.on_config(admission).unwrap();
    app.plugin("preferences", preferences::register).unwrap();
    let app = app.build().await.unwrap();
    let asset = exchange(
        &app,
        "GET",
        &format!("/assets/{}", theme::BOOTSTRAP_FILE),
        Body::Empty,
        &[],
    )
    .await;
    assert_eq!(asset.status().unwrap().code, 404);
    assert!(asset.headers().get("set-cookie").is_none());
}

/// Stored cookies cannot substitute for fresh admission or confirm ambiguous readback.
#[tokio::test]
async fn confirmation_denials_never_write_or_disclose_input() {
    for base in ["", "/demo/v1"] {
        let app = builder(config(base), true).build().await.unwrap();
        let body =
            json!({"locale":"fr","theme":"dark","location":format!("{base}/page")}).to_string();
        let saved = "stackpress-locale=fr; stackpress-theme=dark";
        let oversized = "x".repeat(preferences::MAX_COOKIE_BYTES + 1);
        // Readback is a separate admitted request; saved values do not carry authority.
        for (origin, csrf, cookie, status) in [
            ("https://presentation.invalid", "", saved, 403),
            ("https://wrong.invalid", "verified", saved, 403),
            ("https://presentation.invalid", "verified", "", 400),
            (
                "https://presentation.invalid",
                "verified",
                "stackpress-locale=fr; stackpress-locale; stackpress-theme=dark",
                400,
            ),
            (
                "https://presentation.invalid",
                "verified",
                "stackpress-locale=fr; stackpress-theme=dark; stackpress-theme",
                400,
            ),
            (
                "https://presentation.invalid",
                "verified",
                oversized.as_str(),
                413,
            ),
            ("https://presentation.invalid", "verified", saved, 200),
        ] {
            let res = exchange(
                &app,
                "POST",
                &format!("{base}/confirm"),
                Body::Text(body.clone()),
                &[
                    ("origin", origin),
                    ("x-current-csrf", csrf),
                    ("cookie", cookie),
                ],
            )
            .await;
            assert_eq!(res.status().unwrap().code, status);
            // Both accepted and rejected confirmations remain persistence-free.
            assert!(res.headers().get("set-cookie").is_none());
            if status != 200 {
                // Uniform errors must not reflect submitted cookies, paths or private causes.
                assert!(
                    matches!(res.body(), Body::Text(text) if serde_json::from_str::<Value>(text).unwrap() == json!({"error":"Action unavailable"}))
                );
            }
        }
        // Legacy normalization is read-only and cannot reset the independent French locale.
        let body =
            json!({"locale":"fr","theme":null,"location":format!("{base}/page")}).to_string();
        let res = exchange(
            &app,
            "POST",
            &format!("{base}/confirm"),
            Body::Text(body),
            &[
                ("origin", "https://presentation.invalid"),
                ("x-current-csrf", "verified"),
                ("cookie", "stackpress-locale=fr; stackpress-theme=system"),
            ],
        )
        .await;
        assert_eq!(res.status().unwrap().code, 200);
        assert!(res.headers().get("set-cookie").is_none());
        assert!(
            matches!(res.body(), Body::Text(text) if serde_json::from_str::<Value>(text).unwrap()["preferences"] == json!({"locale":"fr","theme":null}))
        );
    }
}
