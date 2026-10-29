//! Checked values and optional helpers require no application contract.
use leptos_independent::Initial;
use stackpress_leptos::components::ImageSource;

/// Explicit inputs and strict images cannot silently acquire a broader navigation policy.
#[test]
fn admission_and_images() {
    assert!(Initial::new("system", "en", String::new()).is_err());
    assert!(Initial::new("light", "unknown", String::new()).is_err());
    assert!(Initial::new("light", "en", "x".repeat(1025)).is_err());
    assert!(ImageSource::new("/%E9%9B%AA.png", 1024, 32, 32).is_ok());
    for path in [
        "https://image.invalid/a.png",
        "/a.svg",
        "/%2e%2e/a.png",
        "/a.png?q=x",
        "/a%.png",
    ] {
        assert!(ImageSource::new(path, 1, 1, 1).is_err(), "{path}");
    }
    assert!(ImageSource::new("/a.png", 1024 * 1024 + 1, 1, 1).is_err());
    assert!(ImageSource::new("/a.png", 1, 4097, 1).is_err());
}

/// Sanitization is closed and idempotent; executable terminators remain inert replay text.
#[cfg(all(feature = "rich", feature = "async"))]
#[test]
fn hostile_helpers() {
    use stackpress_leptos::{replay::ResourceReplay, rich::SafeHtml};
    for text in [
        "<script>bad()</script><p>safe</p>",
        "<a href=\"javascript:bad()\">bad</a>",
        "<svg onload=\"bad()\"></svg>",
    ] {
        let clean = SafeHtml::html(text).unwrap();
        assert_eq!(SafeHtml::html(clean.as_str()).unwrap(), clean);
        assert!(!clean.as_str().contains("<script") && !clean.as_str().contains("javascript:"));
    }
    let value =
        ResourceReplay::new(vec![(0, leptos_independent::RESOURCE_TEXT.into())], 1024).unwrap();
    let encoded = value.script_json().unwrap();
    assert!(!encoded.contains("</script>"));
    assert_eq!(ResourceReplay::decode(&encoded, 1024).unwrap(), value);
    assert!(ResourceReplay::decode("[[0,\"a\"],[0,\"b\"]]", 1024).is_err());
}

/// Dirty locale changes, exact rounding and composition preserve draft/value separation.
#[cfg(feature = "format")]
#[test]
fn explicit_edit_policy() {
    use stackpress_leptos::format::{
        DisplayLocale,
        edit::{DecimalPolicy, EditSelection, EditState},
    };
    use std::ops::Bound::Unbounded;
    let policy = DecimalPolicy::new(2, 24, Unbounded, Unbounded).unwrap();
    let mut edit = EditState::new(policy, None, DisplayLocale::French).unwrap();
    edit.input("1234,135", EditSelection::end("1234,135"))
        .unwrap();
    edit.set_locale(DisplayLocale::English);
    assert_eq!(edit.draft(), "1234,135");
    assert!(edit.commit(true).unwrap());
    assert_eq!(edit.canonical().as_deref(), Some("1234.14"));
    edit.composition_start();
    edit.input("-", EditSelection::end("-")).unwrap();
    edit.composition_cancel();
    assert_eq!(edit.canonical().as_deref(), Some("1234.14"));
}

/// Completed stale/foreign work cannot publish, and root disposal cancels pending tickets.
#[cfg(feature = "async")]
#[test]
fn async_completion_ownership() {
    use leptos::prelude::*;
    use stackpress_leptos::components::{ComboboxState, SelectOptions};
    let owner = Owner::new();
    let pending = owner.with(|| {
        let state = ComboboxState::new(SelectOptions::new(Vec::new()).unwrap(), None).unwrap();
        let old = futures::executor::block_on(
            state
                .request("old".into(), async { SelectOptions::new(Vec::new()) })
                .unwrap(),
        )
        .unwrap();
        let fresh = futures::executor::block_on(
            state
                .request("fresh".into(), async { SelectOptions::new(Vec::new()) })
                .unwrap(),
        )
        .unwrap();
        assert!(!state.publish(old));
        assert!(state.publish(fresh));
        let foreign = ComboboxState::new(SelectOptions::new(Vec::new()).unwrap(), None).unwrap();
        let completion = futures::executor::block_on(
            foreign
                .request("fresh".into(), async { SelectOptions::new(Vec::new()) })
                .unwrap(),
        )
        .unwrap();
        assert!(!state.publish(completion));
        state
            .request("pending".into(), std::future::pending())
            .unwrap()
    });
    owner.cleanup();
    assert!(futures::executor::block_on(pending).is_none());
}
