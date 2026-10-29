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

/// Counted text and node templates share locale categories, including French zero.
#[test]
fn integer_plural_categories_and_text() {
    for locale in ["en", "en-US", "fr", "fr-FR", "FR_CA"] {
        let fixture = language(
            locale,
            &[("%s item", "%s article"), ("%s items", "%s articles")],
            "%s",
        );
        for count in [0, 1, 2] {
            let one = count == 1 || (locale.to_ascii_lowercase().starts_with("fr") && count == 0);
            assert_eq!(
                fixture.plural_category(count),
                if one {
                    PluralCategory::One
                } else {
                    PluralCategory::Other
                }
            );
            assert_eq!(
                fixture.plural_text(count, "%s item", "%s items", &[count.to_string()]),
                format!("{count} {}", if one { "article" } else { "articles" })
            );
        }
    }
    assert_eq!(
        language("unmapped", &[], "%s").plural_text(2, "%s item", "%s items", &["2".into()]),
        "2 items"
    );
}

/// Real view slots stay nodes while grammar follows count and locale.
#[cfg(feature = "ssr")]
#[test]
fn counted_node_translation() {
    for locale in ["en", "fr"] {
        let owner = Owner::new();
        owner.with(|| {
            let context = LanguageContext::new(language(locale, &[], "%s"));
            for count in [0_u64, 1, 2] {
                let html = view! { <Translate language=context one="%s item" phrase="%s items" count=Signal::derive(move || count)
                    slots=vec![ViewFn::from(move || view! { <strong>{count}</strong> })]/> }.to_html();
                assert!(html.contains(&format!("<strong>{count}</strong>")));
                assert_eq!(html.contains("items"), !(count == 1 || (locale == "fr" && count == 0)));
            }
            let count = RwSignal::new(0_u64);
            let text = Memo::new(move |_| context.plural_text(count.get(), "%s item", "%s items", &[count.get().to_string()]));
            assert_eq!(text.get(), if locale == "fr" { "0 item" } else { "0 items" });
            count.set(1); assert_eq!(text.get(), "1 item");
            count.set(2); assert_eq!(text.get(), "2 items");
            context.change(language(if locale == "fr" { "en" } else { "fr" }, &[], "%s"));
            count.set(0); assert_eq!(text.get(), if locale == "fr" { "0 items" } else { "0 item" });
        });
        owner.cleanup();
    }
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
