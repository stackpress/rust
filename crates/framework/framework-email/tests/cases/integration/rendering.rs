//! Check either the finite public form or the real session view selected by feature unification.

use super::{Fixture, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageId, PageRenderer, PageSnapshot, PublicLocation,
    PublicStatus, ViewError,
    leptos::render::{LeptosRenderer, initialize_ssr},
};
use stackpress_leptos::leptos::{self, prelude::*};
use stackpress_lib::{Body, Flow, Invocation, Request, Response};
use stackpress_server::{Server, ServerInvocationExt};

/// Only source-intended CSRF/form/error data crosses into the SSR/hydration seed.
#[derive(Clone, Serialize, Deserialize)]
struct Form {
    name: String,
    token: String,
    email: String,
    error: String,
}
/// Initialize through the existing renderer adapter; no web endpoint is involved.
pub(super) fn initialize() {
    initialize_ssr().unwrap();
}
/// Dependency features are unified by Cargo, so inspect the source view contribution at runtime.
pub(super) fn native_view(server: &Server) -> bool {
    server
        .listeners()
        .any(|listener| listener.pattern == "GET /auth/signin/email" && listener.priority == -100)
}
/// Trusted component structure uses ordinary escaped Leptos text/attribute binding.
fn component(page: &PageSnapshot<Form>) -> std::result::Result<AnyView, ViewError> {
    let data = page.data.clone();
    Ok(leptos::html::main()
        .child(leptos::html::h1().child("Sign in"))
        .child(leptos::html::p().attr("role", "alert").child(data.error))
        .child(
            leptos::html::form()
                .attr("method", "post")
                .child(leptos::html::label().attr("for", "email").child("Email"))
                .child(
                    leptos::html::input()
                        .attr("id", "email")
                        .attr("name", "email")
                        .attr("value", data.email),
                )
                .child(
                    leptos::html::input()
                        .attr("type", "hidden")
                        .attr("name", data.name)
                        .attr("value", data.token),
                )
                .child(
                    leptos::html::button()
                        .attr("type", "submit")
                        .child("Send sign-in email"),
                ),
        )
        .into_any())
}
/// Render only this finite page's response, preserving redirects/prior output and original status.
pub(super) async fn response(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if !res.data().has("finite_form") || res.redirected() || !matches!(res.body(), Body::Empty) {
        return Ok(Flow::Continue);
    }
    let state = inv.service::<Fixture>()?;
    state.record("render");
    let data = Form {
        name: state.csrf.name().into(),
        token: res
            .data()
            .path_as::<String>("csrf.token")?
            .unwrap_or_default(),
        email: req
            .data
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .into(),
        error: res.error().unwrap_or_default().into(),
    };
    let code = res.status().map_or(200, |status| status.code);
    let snapshot = PageSnapshot {
        revision: 1,
        build: "slice9b-m4".into(),
        page: PageId::try_from("signin-email".to_owned()).unwrap(),
        location: "/auth/signin/email".into(),
        locale: "en".into(),
        theme: Some("light".into()),
        status: PublicStatus::new(code, res.error().map(str::to_owned)).unwrap(),
        data,
    };
    let mut contract = PageContract::new(1, "slice9b-m4").unwrap();
    contract.theme = Some("light".into());
    let renderer = LeptosRenderer::new(
        contract,
        PageAssets {
            theme_bootstrap: None,
            build: "slice9b-m4".into(),
            base: BasePath::default(),
            module: PublicLocation::new("/assets/fixture.js").unwrap(),
            stylesheet: None,
        },
        component,
    )
    .unwrap();
    let html = renderer
        .render(snapshot)
        .await
        .map_err(stackpress_lib::Error::from_source)?;
    res.html(html.into_html())?;
    res.status_code(code, None)?;
    Ok(Flow::Continue)
}
/// Both renderer contracts keep injected markup inert and native fixture resources private.
fn document(res: &Response) -> (&str, &str) {
    let Body::Text(html) = res.body() else {
        panic!("rendered HTML")
    };
    assert!(!html.contains("<script>untrusted()"));
    assert!(!html.contains("SMTP_PRIVATE_SENTINEL"));
    assert!(!html.contains("provider fixture failure"));
    assert!(!html.contains("database-seed"));
    let seed = html
        .split("id=\"stackpress-page\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    assert!(!seed.contains('<'));
    (html, seed)
}
/// Correlate HTML/seed CSRF and status for the selected renderer, including the source sent page.
pub(super) fn assert_page(res: &Response, token: &str, case: &Json, native: bool) {
    let (html, seed) = document(res);
    let page: PageSnapshot<Json> = serde_json::from_str(seed).unwrap();
    assert_eq!(page.status.code(), res.status().unwrap().code);
    if native {
        // This public tuple exists only when session/leptos-ssr contributes its real view.
        assert_eq!(page.page.as_str(), "session-email");
        let (kind, props): (String, Json) = serde_json::from_value(page.data).unwrap();
        assert_eq!(kind, "email");
        assert_eq!(props["config"]["csrf_token"], token);
        assert_eq!(props["config"]["csrf_name"], "csrf");
        assert_eq!(props["code"], page.status.code());
        assert_eq!(props["error"], res.error().unwrap_or_default());
        // Upstream email.tsx renders a confirmation, without a form, for this exact branch.
        let sent = page.status.code() == 200 && case["method"] == "POST" && case["mode"] == "magic";
        assert_eq!(html.contains("<form"), !sent);
        assert_eq!(
            html.contains("A magic link has been sent to your email address."),
            sent
        );
        if sent {
            assert!(!html.contains("name=\"csrf\""));
            return;
        }
        assert!(html.contains("for=\"session-email\""));
    } else {
        // Retain the original fixture projection assertions when no native view is selected.
        assert_eq!(page.page.as_str(), "signin-email");
        let form: Form = serde_json::from_value(page.data).unwrap();
        assert_eq!(form.token, token);
        assert_eq!(form.name, "csrf");
        assert_eq!(form.error, res.error().unwrap_or_default());
        assert!(html.contains("for=\"email\""));
    }
    assert!(html.contains("name=\"csrf\""));
    assert!(html.contains(&format!("value=\"{token}\"")));
    assert!(html.contains("role=\"alert\""));
}
/// Untrusted input stays escaped in bound fixture HTML or inert native request data.
#[tokio::test]
async fn public_form_projection_escapes_input_and_excludes_native_provider_resources() {
    let case = super::vectors()["cases"][0].clone();
    let (server,state)=super::host(case.clone(),Some(serde_json::json!({"host":"relay.example.test","auth":{"pass":"SMTP_PRIVATE_SENTINEL"}}))).await;
    let mut req = super::request(&case);
    let input = "\"><script>untrusted()</script>&";
    req.data.set("email", input).unwrap();
    let mut res = Response::new();
    server
        .handle("GET /auth/signin/email", &mut req, &mut res)
        .await
        .unwrap();
    let native = native_view(&server);
    let token = res.data().path_as::<String>("csrf.token").unwrap().unwrap();
    assert_page(&res, &token, &case, native);
    let (html, seed) = document(&res);
    let decoded: PageSnapshot<Json> = serde_json::from_str(seed).unwrap();
    if native {
        // Source email.tsx:57–62 never binds request email into the input's value.
        assert_eq!(decoded.data[1]["input"]["email"], input);
        let email = html
            .split("<input")
            .filter_map(|fragment| fragment.split_once('>'))
            .find_map(|(attributes, _)| attributes.contains("name=\"email\"").then_some(attributes))
            .unwrap();
        assert!(email.contains("value=\"\""));
    } else {
        // The finite fixture deliberately binds email; keep its original escaping proof.
        assert!(html.contains("&lt;script&gt;"));
        let form: Form = serde_json::from_value(decoded.data).unwrap();
        assert_eq!(form.email, input);
    }
    state.engine.close().await.unwrap();
}
