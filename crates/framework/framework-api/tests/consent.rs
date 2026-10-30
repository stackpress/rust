//! Source consent/head/layout contracts and direct public library rendering.
use serde_json::json;
use stackpress_framework_api::views::{Consent, host};
use stackpress_framework_view::preferences::ThemePreference;
use stackpress_leptos::language::Language;

fn page() -> Consent {
    Consent::new(
        json!({"api":{"scopes":{"read":{"icon":"globe","name":"Read <profile>","description":"Permission details"},"write":{"icon":"key","name":"Write","description":""}}},"brand":{"name":"Example","favicon":"/brand.svg"}}),
        json!({"scope":"read unknown write read","redirect_uri":"https://client.test/return?state=a%26b&error=old&error=older?discard"}),
        json!({"name":"App </script>","secret":"plaintext-app-key"}),
    )
}
#[test]
fn original_known_scope_order_duplicates_unchecked_native_post_and_denial() {
    let page = page();
    assert_eq!(
        page.items()
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        ["read", "write", "read"]
    );
    assert_eq!(
        page.revert(),
        "https://client.test/return?state=a%26b&error=denied"
    );
    let html = host::document(page).unwrap();
    assert!(html.contains("method=\"post\""));
    assert_eq!(html.matches("name=\"scopes[]\"").count(), 3);
    assert!(!html.contains(" checked"));
    assert!(html.contains("role=\"switch\""));
    assert!(html.contains("Read &lt;profile&gt;"));
    assert!(html.contains("Grant Access"));
    assert!(html.contains("Sign Up"));
    assert!(html.contains(">Grant<"));
    assert!(html.contains(">Deny<"));
    assert!(html.contains("plaintext-app-key"));
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(!html.contains("name=\"csrf\""));
}
#[test]
fn unknown_scope_falls_back_to_original_personal_information_sentence() {
    let mut page = page();
    page.request = json!({"scope":"unknown"});
    page.application = json!({});
    page.config = json!({});
    let html = host::document(page).unwrap();
    assert!(html.contains("Unknown is asking permissions to access your personal information."));
    assert!(!html.contains("role=\"switch\""));
    assert!(html.contains("Stackpress"));
    assert!(html.contains("href=\"/?error=denied\""));
}
#[test]
fn original_head_mime_styles_translation_and_explicit_themes() {
    for (favicon, mime) in [
        ("/brand.png", "image/png"),
        ("/brand.svg", "image/svg+xml"),
        ("/brand.ico", "image/x-icon"),
        ("/brand.SVG", "image/x-icon"),
    ] {
        let mut page = page();
        page.config["brand"]["favicon"] = json!(favicon);
        page.styles = vec!["/one.css".into(), "/two.css".into()];
        page.language = Language::new(
            "fr".into(),
            std::collections::BTreeMap::from([
                ("Grant Access".into(), "Autoriser".into()),
                ("Sign Up".into(), "Inscription".into()),
                ("Grant".into(), "Accorder".into()),
                ("Deny".into(), "Refuser".into()),
            ]),
            "%s".into(),
        )
        .unwrap();
        page.theme = Some(ThemePreference::Dark);
        let html = host::document(page).unwrap();
        assert!(html.contains(mime));
        assert!(html.contains("Autoriser"));
        assert!(html.contains("Inscription"));
        assert!(html.contains("Accorder"));
        assert!(html.contains("Refuser"));
        assert!(html.find("global.css").unwrap() < html.find("one.css").unwrap());
        assert!(html.find("one.css").unwrap() < html.find("two.css").unwrap());
        assert!(html.contains("data-theme=\"dark\""));
        assert!(!html.contains("data-theme-unset"));
        assert!(!html.contains("initial-theme-"));
    }
    let html = host::document(page()).unwrap();
    assert!(html.contains("data-theme-unset"));
    assert!(html.contains("initial-theme-"));
    assert!(html.contains("data-theme=\"light\""));
}
#[test]
fn seed_is_inert_while_source_destination_is_retained_as_data() {
    let mut page = page();
    page.request["redirect_uri"] = json!("javascript:caller?x=1");
    assert_eq!(page.revert(), "javascript:caller?x=1&error=denied");
    let html = host::document(page).unwrap();
    assert!(!html.contains("App </script>"));
    assert!(html.contains("\\u003c/script\\u003e"));
    assert!(html.contains("href=\"javascript:caller?x=1&amp;error=denied\""));
}
#[test]
fn query_controls_duplicates_and_original_second_question_mark() {
    let mut page = page();
    page.request = json!({"redirect_uri":"/return?x=+&error=x&z=%09&error=y?ignored"});
    assert_eq!(page.revert(), "/return?x=+&error=denied&z=%09");
    page.request = json!({"redirect_uri":""});
    assert_eq!(page.revert(), "?error=denied");
}

#[tokio::test]
async fn prior_developer_output_and_outer_abort_survive_the_selected_view_chain() {
    use stackpress_lib::{Flow, MethodPattern, Request, Response};
    use stackpress_server::{RouteSpec, ServerBuilder};
    for abort in [false, true] {
        let mut builder = ServerBuilder::new();
        builder
            .route_with(
                RouteSpec::new(MethodPattern::Any, "/auth/oauth").priority(1),
                stackpress_lib::handler(move |_, res, _| {
                    Box::pin(async move {
                        res.html("developer output")?;
                        Ok(if abort { Flow::Abort } else { Flow::Continue })
                    })
                }),
            )
            .unwrap();
        host::register(&mut builder).unwrap();
        builder
            .route_with(
                RouteSpec::new(MethodPattern::Any, "/auth/oauth").priority(-101),
                stackpress_lib::handler(|_, res, _| {
                    Box::pin(async move {
                        res.headers_mut()?.set("x-tail", "ran")?;
                        Ok(Flow::Continue)
                    })
                }),
            )
            .unwrap();
        let server = builder.build().await.unwrap();
        let mut req = Request::builder().url("/auth/oauth").build().unwrap();
        let mut res = Response::new();
        server.handle_route(&mut req, &mut res).await.unwrap();
        assert!(matches!(res.body(),stackpress_lib::Body::Text(text) if text=="developer output"));
        assert_eq!(
            res.headers().get("x-tail"),
            if abort { None } else { Some("ran") }
        );
    }
}
