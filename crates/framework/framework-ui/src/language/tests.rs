//! Phrase policy, ownership and real Leptos node composition proofs.

use super::*;

/// Make a checked explicit fixture, with no hidden default locale discovery.
fn language(locale: &str, phrases: &[(&str, &str)], token: &str) -> Language {
    Language::new(
        locale.into(),
        phrases
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect(),
        token.into(),
    )
    .unwrap()
}

/// The text-only API preserves Unicode, whitespace, falsy text and slot ordering.
#[test]
fn phrase_and_text_policy() {
    let fixture = language("en_US", &[("hello %s", "  สวัสดี %s  "), ("empty", "")], "%s");
    assert_eq!(fixture.locale(), "en_US");
    assert_eq!(fixture.phrase("missing"), "missing");
    assert_eq!(fixture.text("empty", &[]), "");
    assert_eq!(fixture.text("hello %s", &["โลก".into()]), "  สวัสดี โลก  ");
    assert_eq!(
        fixture.text("%s/%s/%s/%s", &["0".into(), "false".into(), "".into()]),
        "0/false//"
    );
    assert_eq!(
        fixture.text(
            "%s-end-%s",
            &["start".into(), "finish".into(), "ignored".into()]
        ),
        "start-end-finish"
    );
    assert_eq!(fixture.text("plain", &["ignored".into()]), "plain");
    assert_eq!(
        language("fr-FR", &[], "{}").text("{} + {}", &["un".into(), "deux".into()]),
        "un + deux"
    );
    assert_eq!(
        fixture.text("%s", &["<script>alert(1)</script>".into()]),
        "<script>alert(1)</script>"
    );
}

/// Invalid maps cannot replace a mounted locale; boundaries count UTF-8 bytes.
#[test]
fn checked_language_bounds() {
    assert!(Language::new("".into(), BTreeMap::new(), "%s".into()).is_err());
    assert!(Language::new("en US".into(), BTreeMap::new(), "%s".into()).is_err());
    assert!(Language::new("e".repeat(65), BTreeMap::new(), "%s".into()).is_err());
    assert!(Language::new("en".into(), BTreeMap::new(), String::new()).is_err());
    assert!(Language::new("en".into(), BTreeMap::new(), "é".repeat(17)).is_err());
    assert!(
        Language::new(
            "en".into(),
            [("x".into(), "é".repeat(2049))].into(),
            "%s".into()
        )
        .is_err()
    );
    assert!(
        Language::new(
            "en".into(),
            (0..257)
                .map(|index| (index.to_string(), "v".into()))
                .collect(),
            "%s".into()
        )
        .is_err()
    );
    assert!(
        Language::new(
            "en".into(),
            (0..17)
                .map(|index| (index.to_string(), "x".repeat(4096)))
                .collect(),
            "%s".into()
        )
        .is_err()
    );
    assert!(
        Language::new(
            "en".into(),
            [("x".repeat(4096), "y".repeat(4096))].into(),
            "{}".into()
        )
        .is_ok()
    );
}

/// Scoped replacement clears missing dictionaries and invalidates all consumers.
#[test]
fn scoped_replacement_and_disposal() {
    let owner = Owner::new();
    let other = Owner::new();
    let context = owner.with(|| {
        assert!(use_language().is_err());
        let context = LanguageContext::new(language("en", &[("greet", "Hello")], "%s"));
        provide_context(context);
        assert_eq!(use_language().unwrap().text("greet", &[]), "Hello");
        context
    });
    let unrelated =
        other.with(|| LanguageContext::new(language("en", &[("greet", "Hello")], "%s")));
    let consumer = owner.with(|| Memo::new(move |_| context.text("greet", &[])));
    assert_eq!(consumer.get(), "Hello");
    context.change(language("fr", &[("greet", "Bonjour")], "%s"));
    assert_eq!(consumer.get(), "Bonjour");
    assert_eq!(unrelated.text("greet", &[]), "Hello");
    context.change(language("fr", &[], "%s"));
    assert_eq!(consumer.get(), "greet");
    owner.cleanup();
    assert!(context.0.try_get_untracked().is_none());
    other.cleanup();
}

/// Real nodes remain nodes; hostile strings remain escaped literal text.
#[cfg(feature = "ssr")]
#[test]
fn translated_nodes_render_as_safe_views() {
    let owner = Owner::new();
    owner.with(|| {
        let context = LanguageContext::new(language("en", &[], "%s"));
        let slots = vec![
            ViewFn::from(|| view! { <strong>"0"</strong> }),
            ViewFn::from(|| "<script>false</script>"),
        ];
        let html = view! { <LanguageProvider language=context>
            <Translate language=context phrase=" %s / %s / %s " slots/>
        </LanguageProvider> }
        .to_html();
        assert!(html.contains("<strong>0</strong>"));
        assert!(html.contains("&lt;script&gt;false&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(html.contains(" / "));
    });
    owner.cleanup();
}
