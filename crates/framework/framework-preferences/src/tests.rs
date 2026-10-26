//! Critical policy/state/cookie boundaries run natively without a browser.
use super::*;
use stackpress_framework_language::{LocaleRegistry, PreferenceError};
use stackpress_framework_view::BasePath;
use std::{cell::Cell, collections::BTreeMap};

/// Use a mounted app so cookie and return-path boundaries cannot pass accidentally.
fn scope() -> CookieScope {
    CookieScope::new(
        "https://preferences.invalid".into(),
        BasePath::new("/demo").unwrap(),
    )
    .unwrap()
}
/// Compile two exact canonical locales and one explicit alias.
fn action() -> SettingsAction {
    SettingsAction::new(
        LocaleRegistry::new(
            vec!["en".into(), "fr".into()],
            "en".into(),
            BTreeMap::from([("fr_FR".into(), "fr".into())]),
        )
        .unwrap(),
        scope(),
        ThemePreference::System,
    )
}
/// Serialize an intended dirty-page update with encoded query context.
fn update() -> SettingsUpdate {
    SettingsUpdate {
        locale: "fr_FR".into(),
        theme: ThemePreference::Dark,
        location: "/demo/en/preferences?q=a%2Fb&return=%252Fhome".into(),
    }
}
/// Prefix and saved settings are deterministic, while corrupt/ambiguous values fall back.
#[test]
fn select_and_admit_settings() {
    let action = action();
    for (cookie, locale, theme) in [
        ("", "en", ThemePreference::System),
        (
            "stackpress-locale=fr; stackpress-theme=dark",
            "fr",
            ThemePreference::Dark,
        ),
        ("stackpress-theme=light", "en", ThemePreference::Light),
        ("stackpress-theme=system", "en", ThemePreference::System),
        ("stackpress-theme=unknown", "en", ThemePreference::System),
        (
            "stackpress-theme=dark; stackpress-theme=light",
            "en",
            ThemePreference::System,
        ),
        (
            "stackpress-theme=dark; stackpress-theme",
            "en",
            ThemePreference::System,
        ),
    ] {
        assert_eq!(
            action.select("/demo/preferences", cookie).unwrap(),
            Preferences {
                locale: locale.into(),
                theme
            }
        );
    }
    assert_eq!(
        action
            .select("/demo/en/preferences", "stackpress-locale=fr")
            .unwrap()
            .locale,
        "en"
    );
    assert!(action.select("/outside/preferences", "").is_err());
    assert_eq!(
        action.select("/demo/preferences", &"x".repeat(4097)),
        Err(PreferenceError::Limit)
    );
    let body = serde_json::to_string(&update()).unwrap();
    let reply = action
        .apply("POST", Some("https://preferences.invalid"), &body)
        .unwrap();
    assert_eq!(
        reply.location,
        "/demo/fr/preferences?q=a%2Fb&return=%252Fhome"
    );
    assert_eq!(reply.preferences.theme, ThemePreference::Dark);
    assert_eq!(reply.cookies.len(), 2);
    assert!(reply.cookies.iter().all(|value| {
        value.contains("Path=/demo; Max-Age=31536000; HttpOnly; SameSite=Lax; Secure")
    }));
    let text = serde_json::to_string(&reply).unwrap();
    assert!(!text.contains("cookie"));
    assert_eq!(
        action.acknowledge(&text, &update()).unwrap().location,
        reply.location
    );
    for (method, origin, input, error) in [
        (
            "GET",
            Some("https://preferences.invalid"),
            body.as_str(),
            PreferenceError::Request,
        ),
        ("POST", None, body.as_str(), PreferenceError::Origin),
        (
            "POST",
            Some("https://evil.invalid"),
            body.as_str(),
            PreferenceError::Origin,
        ),
        (
            "POST",
            Some("https://preferences.invalid"),
            "{}",
            PreferenceError::Request,
        ),
        (
            "POST",
            Some("https://preferences.invalid"),
            "{\"locale\":\"en\",\"theme\":\"evil\",\"location\":\"/demo/preferences\"}",
            PreferenceError::Request,
        ),
    ] {
        assert_eq!(action.apply(method, origin, input).unwrap_err(), error);
    }
    assert_eq!(
        action
            .apply(
                "POST",
                Some("https://preferences.invalid"),
                &"x".repeat(1025)
            )
            .unwrap_err(),
        PreferenceError::Limit
    );
    for field in ["locale", "theme", "location", "extra"] {
        let mut wire = serde_json::to_value(&reply).unwrap();
        match field {
            "locale" => wire["preferences"]["locale"] = "en".into(),
            "theme" => wire["preferences"]["theme"] = "light".into(),
            "location" => wire["location"] = "/demo/fr/elsewhere".into(),
            _ => wire["extra"] = true.into(),
        }
        assert!(action.acknowledge(&wire.to_string(), &update()).is_err());
    }
    assert!(action.acknowledge(&"x".repeat(1025), &update()).is_err());
    assert!(action.acknowledge("{}", &update()).is_err());
}
/// Failed persistence changes no acknowledged setting/location; one root permits one writer.
#[test]
fn single_writer_and_failure_preserve_content() {
    let initial = Preferences {
        locale: "en".into(),
        theme: ThemePreference::System,
    };
    let mut state = PreferenceState::new(initial.clone(), "/demo/en/preferences".into());
    assert_eq!(state.status(), SaveState::Ready);
    assert!(state.validate(&action(), "{}").is_err());
    let desired = Preferences {
        locale: "fr".into(),
        theme: ThemePreference::Dark,
    };
    let submitted = state.begin(desired.clone()).unwrap();
    assert_eq!(state.status(), SaveState::Pending);
    assert!(state.begin(initial.clone()).is_none());
    assert!(state.validate(&action(), "{}").is_err());
    state.fail();
    assert_eq!(state.status(), SaveState::Failed);
    assert_eq!(state.current(), &initial);
    assert_eq!(state.location(), "/demo/en/preferences");
    let _ = state.begin(desired.clone()).unwrap();
    let reply = action()
        .apply(
            "POST",
            Some("https://preferences.invalid"),
            &serde_json::to_string(&submitted).unwrap(),
        )
        .unwrap();
    let text = serde_json::to_string(&reply).unwrap();
    state.validate(&action(), &text).unwrap();
    state.complete(&action(), &text).unwrap();
    assert!(state.complete(&action(), &text).is_err());
    assert_eq!(state.current(), &desired);
    assert_eq!(state.location(), "/demo/fr/preferences");
    assert_eq!(state.status(), SaveState::Ready);
}
/// Delete malformed/duplicate flash with the issuing scope, keep hostile text inert data.
#[test]
fn flash_consumption_bounds_and_root_dedupe() {
    let flash = FlashCookie::new(scope());
    assert!(flash.consume("").unwrap().clear_cookie.is_none());
    let hostile = FlashNotice::new(
        "notice-1".into(),
        "</script><script>pwned=1</script> & 雪".into(),
    )
    .unwrap();
    let issued = flash.issue(&hostile).unwrap();
    let cookie = issued.split(';').next().unwrap();
    let consumed = flash.consume(cookie).unwrap();
    assert_eq!(consumed.notice, Some(hostile.clone()));
    assert_eq!(
        consumed.clear_cookie.unwrap(),
        "stackpress-flash=; Path=/demo; Max-Age=0; HttpOnly; SameSite=Lax; Secure"
    );
    for header in [
        "stackpress-flash=x".into(),
        "stackpress-flash=zz".into(),
        format!("{cookie};{cookie}"),
        format!("stackpress-flash={}", "a".repeat(3602)),
    ] {
        let consumed = flash.consume(&header).unwrap();
        assert!(consumed.notice.is_none());
        assert!(consumed.clear_cookie.is_some());
    }
    assert!(flash.consume(&"x".repeat(4097)).is_err());
    for (id, message) in [("", "text"), ("bad/id", "text"), ("id", "")] {
        assert!(FlashNotice::new(id.into(), message.into()).is_err());
    }
    assert!(FlashNotice::new("id".into(), "x".repeat(513)).is_err());
    assert!(FlashNotice::new("x".repeat(65), "text".into()).is_err());
    assert!(
        serde_json::from_str::<FlashNotice>(r#"{"id":"x","message":"","extra":true}"#).is_err()
    );
    let mut root = FlashRoot::default();
    assert_eq!(root.admit(hostile.clone()), Some(hostile.clone()));
    assert!(root.admit(hostile.clone()).is_none());
    // Another root/tab may legitimately observe the same cookie before clear arrives.
    assert!(FlashRoot::default().admit(hostile).is_some());
    for id in 1..64 {
        assert!(
            root.admit(FlashNotice::new(format!("n{id}"), "text".into()).unwrap())
                .is_some()
        );
    }
    assert!(
        root.admit(FlashNotice::new("full".into(), "text".into()).unwrap())
            .is_none()
    );
}
/// No implicit HTTPS downgrade, domain scope or origin repair is allowed.
#[test]
fn scope_and_theme_tokens() {
    for origin in [
        "file:///tmp/x",
        "https://user@preferences.invalid",
        "https://preferences.invalid/",
        "https://preferences.invalid/path",
        "https://preferences.invalid?x=1",
        "https://preferences.invalid#x",
    ] {
        assert!(CookieScope::new(origin.into(), BasePath::default()).is_err());
    }
    let dev = CookieScope::new("http://preferences.invalid".into(), BasePath::default()).unwrap();
    let flash = FlashCookie::new(dev)
        .issue(&FlashNotice::new("id".into(), "public".into()).unwrap())
        .unwrap();
    assert!(flash.contains("Path=/;"));
    assert!(!flash.contains("Secure"));
    for theme in [
        ThemePreference::Light,
        ThemePreference::Dark,
        ThemePreference::System,
    ] {
        assert_eq!(ThemePreference::parse(theme.as_str()), Some(theme));
    }
    assert_eq!(ThemePreference::parse("DARK"), None);
}
/// Host projection never substitutes for fresh authorization, including forged projections.
#[test]
fn explicit_host_policy_or_fail_closed() {
    struct Policy;
    impl HostPolicy<bool> for Policy {
        fn permits(&self, context: &bool, _: &ActionId) -> bool {
            *context
        }
    }
    let action = ActionId::new("private.mutate").unwrap();
    let calls = Cell::new(0);
    for (policy, allowed) in [
        (None, true),
        (Some(&Policy as &dyn HostPolicy<bool>), false),
    ] {
        let access = HostAccess::new(policy, &allowed);
        assert!(
            !access
                .project(vec![action.clone()])
                .unwrap()
                .contains(&action)
        );
        let forged: AllowedActions = serde_json::from_str(r#"["private.mutate"]"#).unwrap();
        assert!(forged.contains(&action));
        assert!(access.run(&action, || calls.set(calls.get() + 1)).is_err());
    }
    assert_eq!(calls.get(), 0);
    let access = HostAccess::new(Some(&Policy), &true);
    assert!(
        access
            .project(vec![action.clone()])
            .unwrap()
            .contains(&action)
    );
    access.run(&action, || calls.set(calls.get() + 1)).unwrap();
    assert_eq!(calls.get(), 1);
    assert!(access.project(vec![action.clone(); 65]).is_err());
    assert!(access.project(vec![action.clone(), action]).is_err());
    for key in ["", "bad/action", "bad token"] {
        assert!(ActionId::new(key).is_err());
    }
    assert!(ActionId::new("x".repeat(65)).is_err());
    assert_eq!(String::from(ActionId::new("a").unwrap()), "a");
    assert_eq!(AccessDenied.to_string(), "Action unavailable");
}

/// A POST success is insufficient when cookies were blocked, partial or ambiguous.
#[test]
fn confirm_actual_cookie_readback() {
    let action = action();
    let body = serde_json::to_string(&update()).unwrap();
    let reply = action
        .confirm(
            "POST",
            Some("https://preferences.invalid"),
            &body,
            "stackpress-locale=fr; stackpress-theme=dark",
        )
        .unwrap();
    assert!(reply.cookies.is_empty());
    for cookie in [
        "",
        "stackpress-locale=fr",
        "stackpress-theme=dark",
        "stackpress-locale=en; stackpress-theme=dark",
        "stackpress-locale=fr; stackpress-theme=dark; stackpress-theme=dark",
    ] {
        assert!(
            action
                .confirm("POST", Some("https://preferences.invalid"), &body, cookie)
                .is_err()
        );
    }
}

/// Escaped wire expansion cannot create a cookie that this consumer cannot read.
#[test]
fn flash_wire_expansion_is_bounded() {
    let flash = FlashCookie::new(scope());
    let controls = FlashNotice::new("id".into(), "\u{0000}".repeat(512)).unwrap();
    assert_eq!(flash.issue(&controls).unwrap_err(), PreferenceError::Limit);
    let safe = FlashNotice::new("id".into(), "x".repeat(512)).unwrap();
    let issued = flash.issue(&safe).unwrap();
    assert_eq!(
        flash
            .consume(issued.split(';').next().unwrap())
            .unwrap()
            .notice,
        Some(safe)
    );
}
