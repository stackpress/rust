//! Real Leptos SSR consumes an explicit public form projection, never the native fixture state.

use super::{Fixture, Result};
use serde::{Deserialize, Serialize};
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageId, PageRenderer, PageSnapshot, PublicLocation,
    PublicStatus, ViewError,
    leptos::render::{LeptosRenderer, initialize_ssr},
};
use stackpress_leptos::leptos::{self, prelude::*};
use stackpress_lib::{Body, Flow, Invocation, Request, Response};
use stackpress_server::ServerInvocationExt;

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
/// Read the actual inert hydration seed and correlate its CSRF value to HTML and outgoing cookie.
pub(super) fn assert_form(res: &Response, token: &str) {
    let Body::Text(html) = res.body() else {
        panic!("rendered HTML")
    };
    assert!(html.contains("name=\"csrf\""));
    assert!(html.contains(&format!("value=\"{token}\"")));
    assert!(html.contains("for=\"email\""));
    assert!(html.contains("role=\"alert\""));
    assert!(!html.contains("SMTP_PRIVATE_SENTINEL"));
    assert!(!html.contains("provider fixture failure"));
    let seed = html
        .split("id=\"stackpress-page\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let page: PageSnapshot<Form> = serde_json::from_str(seed).unwrap();
    assert_eq!(page.data.token, token);
    assert_eq!(page.data.name, "csrf");
    assert_eq!(page.status.code(), res.status().unwrap().code);
    assert_eq!(page.data.error, res.error().unwrap_or_default());
}
/// Untrusted form text remains intentionally visible as data, escaped in HTML and inert JSON.
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
    let Body::Text(html) = res.body() else {
        panic!("HTML")
    };
    assert!(!html.contains("<script>untrusted()"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("SMTP_PRIVATE_SENTINEL"));
    let seed = html
        .split("id=\"stackpress-page\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let decoded: PageSnapshot<Form> = serde_json::from_str(seed).unwrap();
    assert_eq!(decoded.data.email, input);
    assert!(!seed.contains('<'));
    state.engine.close().await.unwrap();
}
