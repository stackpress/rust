//! Finite source-shaped auth boundary fakes join real CSRF/email/lifecycle/Leptos crates.
//! Lane A owns actual auth storage, challenge/template construction and application pages.

mod rendering;
mod support;
mod transports;

use serde_json::{Value as Json, json};
use stackpress_framework_csrf::Service as Csrf;
use stackpress_framework_email::{events::EMAIL_SEND, plugin};
use stackpress_lib::{
    Flow, Invocation, PublicError, Request, Response, Result, handler, queue::task,
};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, ServerInvocationExt, StartupContext};
use std::sync::{Arc, Mutex};
use support::Factory;

/// Original JS source output defines page boundary call order and message shape independently.
fn vectors() -> Json {
    serde_json::from_str(include_str!("../../fixtures/source-page-boundaries.json")).unwrap()
}
/// Shared finite fake state; every live mechanism remains in its owning production crate.
struct Fixture {
    case: Json,
    csrf: Csrf,
    trace: Mutex<Vec<String>>,
    factory: Arc<Factory>,
}
impl Fixture {
    /// Trace process boundaries, never scheduler timing or private provider data in public output.
    fn record(&self, event: &str) {
        self.trace.lock().unwrap().push(event.into());
    }
}
/// Fake only the not-yet-merged Lane A page boundary, preserving original CSRF/mail ordering.
async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let state = inv.service::<Fixture>()?;
    // The original page generates before checking the submitted incoming token.
    // The native top-level host requires a selected status before Response rendering.
    res.status_code(200, None)?;
    res.data_mut()?.set("finite_form", true)?;
    state.record("csrf.generate");
    state.csrf.generate(res)?;
    if req.method.as_str() != "POST" {
        if state.case["guest"] == false {
            state.record("csrf.clear");
            state.csrf.clear(req, res)?;
            state.record("redirect");
            res.redirect("/after")?;
        }
        return Ok(Flow::Continue);
    }
    state.record("csrf.valid");
    if !state.csrf.valid(req, res)? {
        return Ok(Flow::Continue);
    }
    if state.case["mode"] == "pass" {
        // Source password page supplies these overrides before the reusable event.
        req.data.set("password", true)?;
        req.data.set("type", "email")?;
        inv.emit("auth-signin", req, res).await?;
        if res.status().is_none_or(|status| status.code != 200) {
            return Ok(Flow::Continue);
        }
    } else {
        // Auth search/update/reload, challenge and template construction belong to Lane A.
        // This fake enters at their frozen mail boundary using the original-source message.
        if state.case["configured"] == false || state.case["sender"] == false {
            return Ok(Flow::Continue);
        }
        let sent = inv.resolve(EMAIL_SEND, state.case["mail"].clone()).await?;
        if sent.response.code != Some(200) {
            state.record("propagate");
            res.apply_envelope(sent.response)?;
            return Ok(Flow::Continue);
        }
        // Original magic issuance ends here; only OTP/password success clear and redirect.
        if state.case["mode"] == "magic" {
            return Ok(Flow::Continue);
        }
    }
    state.record("csrf.clear");
    state.csrf.clear(req, res)?;
    state.record("redirect");
    res.redirect(state.case["redirect"].as_str().unwrap_or("/after"))?;
    Ok(Flow::Continue)
}
/// Configure an entirely finite host; no session crate, auth page implementation or endpoint.
async fn host(case: Json, config: Option<Json>) -> (Server, Arc<Fixture>) {
    rendering::initialize();
    let state = Arc::new(Fixture {
        factory: Factory::new(case["throws"] == true),
        case,
        csrf: Csrf::default(),
        trace: Mutex::new(vec![]),
    });
    let mut app = ServerBuilder::new();
    let resource = state.clone();
    app.on_config(task(move |start: &mut StartupContext| {
        let resource = resource.clone();
        Box::pin(async move {
            start.register(resource)?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    // Prior handlers and email's own source skip cooperate with the same event response.
    app.on_priority(
        EMAIL_SEND,
        10,
        handler(|_: &mut Request, res: &mut Response, inv: Invocation<'_>| {
            Box::pin(async move {
                let state = inv.service::<Fixture>()?;
                state.record(EMAIL_SEND);
                if state.case["sendStatus"] == 419 {
                    res.set_error(PublicError {
                        code: 419,
                        status: "Page Expired".into(),
                        error: "provider public failure".into(),
                        errors: json!({}),
                    })?;
                }
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    plugin::register_with_factory(&mut app, state.factory.clone()).unwrap();
    let mut layers = ConfigLayers::new();
    if let Some(config) = config {
        layers.value("fixture", json!({"email":config})).unwrap();
    }
    app.config_layers(layers).unwrap();
    app.on(
        "auth-signin",
        handler(|_: &mut Request, res: &mut Response, inv: Invocation<'_>| {
            Box::pin(async move {
                let state = inv.service::<Fixture>()?;
                state.record("auth-signin");
                let code = state.case["signinStatus"].as_u64().unwrap() as u16;
                if code != 200 {
                    res.set_error(PublicError {
                        code,
                        status: "Unauthorized".into(),
                        error: "Invalid Password".into(),
                        errors: json!({}),
                    })?;
                } else {
                    res.status_code(200, None)?;
                }
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    app.on("finite-page", page).unwrap();
    app.on(
        "error",
        handler(|_: &mut Request, _: &mut Response, inv: Invocation<'_>| {
            Box::pin(async move {
                inv.service::<Fixture>()?.record("error");
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    app.on("response", rendering::response).unwrap();
    (app.build().await.unwrap(), state)
}
/// Fresh incoming form state stays independent from each generated outgoing token.
fn request(case: &Json) -> Request {
    let mut req=Request::builder().method(case["method"].as_str().unwrap()).url("/auth/signin/email")
        .data(json!({"email":"recipient@example.test","auth":case["mode"],"csrf":if case["valid"]==false {"wrong"}else{"incoming"},"redirect_uri":"/after"})).build().unwrap();
    req.session.set("csrf", "incoming", None).unwrap();
    req
}
/// Complete original trace matrix against real native mechanisms and SSR, with explicit fakes.
#[tokio::test]
async fn source_page_csrf_email_render_and_error_boundaries() {
    for case in vectors()["cases"].as_array().unwrap() {
        let (server, state) = host(
            case.clone(),
            (case["configured"] != false)
                .then(|| json!({"jsonTransport":true,"skipEncoding":true})),
        )
        .await;
        let mut req = request(case);
        let mut res = Response::new();
        let report = server
            .handle("finite-page", &mut req, &mut res)
            .await
            .unwrap();
        let actual: Vec<_> = state
            .trace
            .lock()
            .unwrap()
            .iter()
            .filter(|step| !matches!(step.as_str(), "render" | "error"))
            .cloned()
            .collect();
        assert_eq!(json!(actual), case["trace"], "{}", case["name"]);
        assert_eq!(req.session.get("csrf").unwrap().as_str(), Some("incoming"));
        // The oracle stops at the page boundary. A thrown Rust error then enters
        // the existing G12 host fresh-error replacement, which clears staged data/cookies.
        // Returned child statuses instead preserve that state for the form renderer.
        assert_eq!(
            res.data().has("csrf"),
            case["retainedCsrf"].as_bool().unwrap() && case["throws"] != true,
            "{} {}",
            case["name"],
            report.developer()
        );
        assert_eq!(res.headers().get("location"), case["redirect"].as_str());
        if case["throws"] == true {
            assert_eq!(res.status().unwrap().code, 500);
            assert_eq!(report.failures.len(), 1);
            assert_eq!(report.error_offers.len(), 1);
            assert_eq!(
                state
                    .factory
                    .created
                    .load(std::sync::atomic::Ordering::SeqCst),
                1
            );
            assert!(
                report
                    .developer()
                    .to_string()
                    .contains("provider fixture failure")
            );
        } else {
            assert_eq!(
                res.status().unwrap().code as u64,
                case["code"].as_u64().unwrap()
            );
        }
        if case["retainedCsrf"] == true && case["throws"] != true {
            let token = res.data().path_as::<String>("csrf.token").unwrap().unwrap();
            assert_eq!(token.len(), 64);
            assert_ne!(token, "incoming");
            assert_eq!(
                res.session().get("csrf").unwrap().as_str(),
                Some(token.as_str())
            );
            rendering::assert_form(&res, &token);
        } else {
            assert!(!state.trace.lock().unwrap().contains(&"render".into()));
        }
        if case["mode"] == "pass" && case["valid"] != false {
            assert_eq!(
                req.data.get("password").and_then(|v| v.as_bool()),
                Some(true)
            );
            assert_eq!(req.data.get("type").and_then(|v| v.as_str()), Some("email"));
        }
        let sent = state.factory.sent.lock().unwrap();
        if !sent.is_empty() {
            assert_eq!(json!(sent[0]), case["mail"]);
        }
    }
}
/// The same configured host leaves direct email usage to its calling process, without a global gate.
#[tokio::test]
async fn direct_email_reuse_has_no_automatic_csrf_or_auth_guard() {
    let case = vectors()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "otp-success")
        .unwrap()
        .clone();
    let (server, state) = host(case, Some(json!({"jsonTransport":true}))).await;
    let reply = server
        .resolve(EMAIL_SEND, json!({"text":"direct unprotected process"}))
        .await
        .unwrap();
    assert_eq!(reply.response.code, Some(200));
    assert_eq!(*state.trace.lock().unwrap(), vec!["email-send"]);
    assert_eq!(
        state
            .factory
            .created
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}
