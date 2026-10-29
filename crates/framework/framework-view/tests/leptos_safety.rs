//! G41 differential guard: one table pins independently owned safety policies.
#![cfg(feature = "leptos")]
use stackpress_framework_view::{PublicLocation, escape_script_json, leptos::safety::escape_html};
use stackpress_leptos::{navigation::LocalLocation, safety};

/// Pin both admission results and exact encoded bytes without normalizing either owner.
#[test]
fn strict_local_locations_do_not_drift() {
    let mut cases = vec![
        ("/", true),
        ("/a.png", true),
        ("/a?q=%2f&zero=0", true),
        ("/%E9%9B%AA.png", true),
        ("/a?dot=../%2e", true),
        ("", false),
        ("relative.png", false),
        ("//host/a.png", false),
        ("https://host/a.png", false),
        ("/a#fragment", false),
        ("/a\\b", false),
        ("/a b", false),
        ("/a\n", false),
        ("/雪.png", false),
        ("/a?unicode=雪", false),
        ("/%", false),
        ("/%0", false),
        ("/%xz", false),
        ("/a?x=%2", false),
        ("/./a", false),
        ("/../a", false),
        ("/%2e/a", false),
        ("/.%2E/a", false),
        ("/%2e%2E/a", false),
        ("/a/..", false),
        ("/<script>", false),
        ("/\"x", false),
        ("/`x", false),
        ("/a?x=%00", true),
        ("/a?x=%FF", true),
        ("/a?x='", true),
    ]
    .into_iter()
    .map(|(value, valid)| (value.to_owned(), valid))
    .collect::<Vec<_>>();
    cases.extend([
        (format!("/{}", "a".repeat(4095)), true),
        (format!("/{}", "a".repeat(4096)), false),
        (format!("/a?q={}", "x".repeat(4091)), true),
        (format!("/a?q={}", "x".repeat(4092)), false),
    ]);
    for (value, valid) in cases {
        let view = PublicLocation::new(value.clone());
        let ui = LocalLocation::new(value.clone());
        assert_eq!(view.is_ok(), valid, "view {value:?}");
        assert_eq!(ui.is_ok(), valid, "UI {value:?}");
        if valid {
            assert_eq!(view.unwrap().as_str(), value);
            assert_eq!(ui.unwrap().as_str(), value);
        }
    }
}

/// Script terminators and Unicode separators retain identical safe roundtrip bytes.
#[test]
fn script_and_html_escapers_do_not_drift() {
    for value in [
        "",
        "plain 雪",
        "</script><script>alert(1)</script>",
        "</template>&\"'<>\u{2028}\u{2029}",
        "&amp;\\u003c",
        "\0\n\r",
    ] {
        assert_eq!(escape_script_json(value), safety::escape_script_json(value));
        assert_eq!(escape_html(value), safety::escape_html(value));
    }
    assert_eq!(
        escape_script_json("</script>&\u{2028}\u{2029}"),
        "\\u003c/script\\u003e\\u0026\\u2028\\u2029"
    );
    assert_eq!(escape_html("<>&\"'雪"), "&lt;&gt;&amp;&quot;&#39;雪");
}

/// Only acknowledged explicit preferences map to UI choices; absence stays unset.
#[cfg(feature = "preferences")]
#[test]
fn acknowledged_theme_mapping_preserves_absence() {
    use stackpress_framework_view::{leptos::presentation, preferences::ThemePreference};
    use stackpress_leptos::state::Theme;
    assert_eq!(presentation::theme(None), None);
    assert_eq!(
        presentation::theme(Some(ThemePreference::Light)),
        Some(Theme::Light)
    );
    assert_eq!(
        presentation::theme(Some(ThemePreference::Dark)),
        Some(Theme::Dark)
    );
}
