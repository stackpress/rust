//! Real SSR and direct reusable form/head/helper contracts, independent of native handles.
use leptos::prelude::*;
use serde_json::{Value, json};
use stackpress_framework_session::{
    presentation::{self, Presentation},
    views::{self, Page, PageProps, PasswordRules},
};

/// Pure source password checklist vectors include source UI/server special-character divergence.
#[test]
fn source_weighted_password_strength() {
    let cases: Value = serde_json::from_str(include_str!("../password-strength.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let rules: PasswordRules = serde_json::from_value(case["rule"].clone()).unwrap();
        let checks = views::password_checklist(&rules);
        let secret = case["secret"].as_str().unwrap();
        assert_eq!(checks.len(), case["checks"].as_array().unwrap().len());
        for (check, expected) in checks.iter().zip(case["checks"].as_array().unwrap()) {
            assert_eq!(check.id, expected["id"]);
            assert_eq!(check.label, expected["label"]);
            assert_eq!(check.weight, expected["weight"].as_f64().unwrap());
            assert_eq!(check.pass(secret), expected["pass"].as_bool().unwrap());
            assert_eq!(check.score(secret), expected["score"].as_f64().unwrap());
        }
        let actual = views::evaluate_password(secret, &checks);
        if case["result"]["score"].is_null() {
            assert!(actual.score.is_nan())
        } else {
            assert_eq!(actual.score, case["result"]["score"].as_f64().unwrap());
        }
        let failed = actual
            .failed
            .into_iter()
            .map(|(id, label)| json!({"id":id,"label":label}))
            .collect::<Vec<_>>();
        assert_eq!(json!(failed), case["result"]["failed"]);
    }
}
/// Every source page/head renders, escapes hostile values, and round-trips its intentional DTO.
#[tokio::test]
async fn every_source_page_head_and_intentional_public_payload() {
    stackpress_framework_view::leptos::render::initialize_ssr().unwrap();
    for kind in [
        Page::Signup,
        Page::Signin,
        Page::Username,
        Page::Email,
        Page::Phone,
        Page::Otp,
        Page::Link,
        Page::TwoFactor,
        Page::Account,
        Page::Update,
        Page::Security,
        Page::Password,
        Page::Export,
        Page::Remove,
        Page::Enroll,
        Page::RemoveFactor,
    ] {
        for theme in [None, Some("light".into()), Some("dark".into())] {
            let mut props = PageProps {
                theme: theme.clone(),
                path: "/auth/account".into(),
                input: json!({"name":"</script><img src=x onerror=alert(1)>","secret":"source-intended-secret","redirect_uri":"/after?x=<tag>"}),
                results: json!({"name":"Ada <雪>","image":"/avatar/ada.png","auth":{"username":{"token":"ada-user"},"email":{"token":"ada@example.test"},"phone":{"token":"+639171234567"}},"url":"data:image/png;base64,fixture","secret":"SOURCE-INTENDED-ENROLLMENT-SECRET","authId":"factor"}),
                ..Default::default()
            };
            props.config.menu=serde_json::from_value(json!([{"name":"Email <&>","path":"/auth/signin/email","icon":"envelope","target":null,"type":null},{"name":"Join","path":"/auth/signup","icon":null,"target":"_self","type":"footer"}])).unwrap();
            props.config.password = Some(PasswordRules {
                min: Some(8),
                upper: true,
                special: true,
                ..Default::default()
            });
            let host = Presentation::default();
            let html = presentation::render(kind, props, &host).await.unwrap();
            assert!(html.contains("<title>"), "{kind:?}");
            assert!(!html.contains("<img src=x onerror="));
            assert!(html.contains(r#"aria-hidden="true""#));
            assert!(!html.contains("SIGNING-SEED"));
            let start = html
                .split(r#"id="stackpress-page">"#)
                .nth(1)
                .unwrap()
                .split("</script>")
                .next()
                .unwrap();
            let seed: stackpress_framework_view::PageSnapshot<(Page, PageProps)> =
                stackpress_framework_view::PageSnapshot::decode(start, 1, "session-v1").unwrap();
            assert_eq!(seed.data.0, kind);
            assert_eq!(
                seed.data.1.results["secret"],
                "SOURCE-INTENDED-ENROLLMENT-SECRET"
            );
            if theme.is_none() {
                assert!(html.contains("data-theme-unset"));
                assert!(html.contains("data-initial-theme-pending"));
            }
        }
    }
}
/// Direct named Body/Head/Page APIs and forms reuse the same public contracts without plugins.
#[tokio::test]
async fn direct_forms_helpers_and_magic_success_branch() {
    stackpress_framework_view::leptos::render::initialize_ssr().unwrap();
    let owner = Owner::new();
    owner.with(|| {
        let props = PageProps::default();
        // Custom pages can reuse the layout with ordinary Leptos children, without plugins.
        let custom =
            views::account_layout(&props, view! {<p>"Custom account content"</p>}.into_any())
                .to_html();
        assert!(custom.contains("Custom account content"));
        assert!(custom.contains("/auth/account/security"));
        assert_eq!(views::use_account_base(&props), "/auth");
        for form in [
            views::signup_form(&props),
            views::username_form(&props),
            views::phone_form(&props),
            views::email_form(&props),
        ] {
            assert!(form.to_html().contains("<form"));
        }
        let strengths = views::password_strength(
            RwSignal::new("Ab1!".into()),
            PasswordRules {
                min: Some(8),
                ..Default::default()
            },
        )
        .to_html();
        assert!(strengths.contains("At least 8 characters"));
        assert!(
            views::account_back_header("Title", Some("/back".into()), Some("user"))
                .to_html()
                .contains(r#"aria-label="Back""#)
        );
        assert!(
            views::signup::body(props.clone())
                .to_html()
                .contains("Sign Up")
        );
        let mut sent = props.clone();
        sent.method = "POST".into();
        sent.code = 200;
        sent.input = json!({"auth":"magic"});
        assert!(
            views::body(Page::Email, sent)
                .to_html()
                .contains("A magic link has been sent")
        );
        assert!(
            views::body(Page::Enroll, props)
                .to_html()
                .contains("Regenerate Secret")
        );
    });
    owner.cleanup();
}

/// Root-local language lookup translates source labels and keeps the dictionary out of native state.
#[tokio::test]
async fn checked_language_projection_and_empty_base_navigation() {
    stackpress_framework_view::leptos::render::initialize_ssr().unwrap();
    let mut props = PageProps::default();
    props.config.base = String::new();
    props.language = Some(
        stackpress_leptos::language::Language::new(
            "fr".into(),
            [
                (
                    "Personal Information".into(),
                    "Informations personnelles".into(),
                ),
                ("Name".into(), "Nom".into()),
                ("Back".into(), "Retour".into()),
                ("Profile Picture".into(), "Portrait".into()),
            ]
            .into_iter()
            .collect(),
            "%s".into(),
        )
        .unwrap(),
    );
    let html = presentation::render(Page::Account, props.clone(), &Presentation::default())
        .await
        .unwrap();
    assert!(html.contains("Informations personnelles"));
    assert!(html.contains("Nom"));
    assert!(html.contains(r#"alt="Portrait""#));
    let password = presentation::render(Page::Password, props, &Presentation::default())
        .await
        .unwrap();
    assert!(password.contains(r#"aria-label="Retour""#));
    // AccountLayout alone uses source truthy fallback; useAccountBase remains literal.
    assert!(html.contains(r#"href="/auth/account""#));
}

#[path = "account_ui/registration.rs"]
mod registration;
