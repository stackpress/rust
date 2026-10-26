//! Native precedence, action-admission and cookie/route boundary contracts.
use super::*;

/// Build the explicit English/French registry without guessing region aliases.
fn registry() -> LocaleRegistry {
    LocaleRegistry::new(
        vec!["en".into(), "fr".into()],
        "en".into(),
        BTreeMap::from([("fr-FR".into(), "fr".into())]),
    )
    .unwrap()
}

/// Prefix wins, legacy paths use canonical cookies, invalid/duplicate cookies use default.
#[test]
fn precedence_and_routes() {
    let registry = registry();
    let base = BasePath::new("/demo/v1").unwrap();
    for (path, cookie, locale, source, route) in [
        (
            "/demo/v1/en/gallery?q=a%2Fb",
            "stackpress-locale=fr",
            "en",
            LocaleSource::Prefix,
            "/gallery?q=a%2Fb",
        ),
        (
            "/demo/v1/fr-FR/gallery",
            "",
            "fr",
            LocaleSource::Prefix,
            "/gallery",
        ),
        (
            "/demo/v1/gallery",
            "x=1; stackpress-locale=fr",
            "fr",
            LocaleSource::Cookie,
            "/gallery",
        ),
        (
            "/demo/v1/gallery",
            "stackpress-locale=unknown",
            "en",
            LocaleSource::Default,
            "/gallery",
        ),
        (
            "/demo/v1/gallery",
            "stackpress-locale=en; stackpress-locale=fr",
            "en",
            LocaleSource::Default,
            "/gallery",
        ),
        (
            "/demo/v1/de/gallery",
            "",
            "en",
            LocaleSource::Default,
            "/de/gallery",
        ),
        ("/demo/v1/fr?x=1", "", "fr", LocaleSource::Prefix, "/?x=1"),
        ("/demo/v1/fr", "", "fr", LocaleSource::Prefix, "/"),
    ] {
        assert_eq!(
            registry.select(&base, path, cookie).unwrap(),
            Selection {
                locale: locale.into(),
                source,
                route: route.into()
            }
        );
    }
    assert_eq!(
        registry.select(&base, "/outside/gallery", ""),
        Err(PreferenceError::Request)
    );
    assert_eq!(
        registry.select(&base, "/demo/v1/gallery", &"x".repeat(4097)),
        Err(PreferenceError::Limit)
    );
}

/// Configuration rejects token injection, duplicates, unknown defaults/alias targets and chains.
#[test]
fn invalid_configuration() {
    for locales in [
        vec![],
        vec!["en".into(), "en".into()],
        vec!["en\r\n".into()],
    ] {
        assert!(LocaleRegistry::new(locales, "en".into(), BTreeMap::new()).is_err());
    }
    assert!(LocaleRegistry::new(vec!["en".into()], "fr".into(), BTreeMap::new()).is_err());
    for aliases in [
        BTreeMap::from([("en".into(), "en".into())]),
        BTreeMap::from([("fr".into(), "missing".into())]),
    ] {
        assert!(LocaleRegistry::new(vec!["en".into()], "en".into(), aliases).is_err());
    }
    for origin in [
        "https://gallery.invalid/path",
        "https://user:pass@gallery.invalid",
        "https://gallery.invalid?x=1",
        "file:///tmp/x",
        "https://gallery.invalid/",
    ] {
        assert!(PreferenceAction::new(registry(), origin.into(), BasePath::default()).is_err());
    }
}

/// The host acknowledges only a same-origin POST and scopes a public nonsecret cookie.
#[test]
fn action_admission_and_persistence() {
    let action = PreferenceAction::new(
        registry(),
        "https://gallery.invalid".into(),
        BasePath::new("/demo").unwrap(),
    )
    .unwrap();
    let update = || PreferenceUpdate {
        locale: "fr-FR".into(),
        location: "/demo/en/gallery?q=a%20b".into(),
    };
    let reply = action
        .apply("POST", Some("https://gallery.invalid"), update())
        .unwrap();
    assert_eq!(reply.locale, "fr");
    assert_eq!(reply.location, "/demo/fr/gallery?q=a%20b");
    assert_eq!(
        reply.cookie,
        "stackpress-locale=fr; Path=/demo; Max-Age=31536000; HttpOnly; SameSite=Lax; Secure"
    );
    for origin in [None, Some("https://evil.invalid"), Some("null")] {
        assert_eq!(
            action.apply("POST", origin, update()).unwrap_err(),
            PreferenceError::Origin
        );
    }
    assert_eq!(
        action
            .apply("GET", Some("https://gallery.invalid"), update())
            .unwrap_err(),
        PreferenceError::Request
    );
    assert_eq!(
        action
            .apply(
                "POST",
                Some("https://gallery.invalid"),
                PreferenceUpdate {
                    locale: "de".into(),
                    ..update()
                }
            )
            .unwrap_err(),
        PreferenceError::Locale
    );
    assert_eq!(
        action
            .apply(
                "POST",
                Some("https://gallery.invalid"),
                PreferenceUpdate {
                    location: "//evil.invalid".into(),
                    ..update()
                }
            )
            .unwrap_err(),
        PreferenceError::Request
    );
    assert_eq!(
        action
            .apply(
                "POST",
                Some("https://gallery.invalid"),
                PreferenceUpdate {
                    location: "/".repeat(1025),
                    ..update()
                }
            )
            .unwrap_err(),
        PreferenceError::Limit
    );
    let dev = PreferenceAction::new(
        registry(),
        "http://gallery.invalid".into(),
        BasePath::default(),
    )
    .unwrap();
    let reply = dev
        .apply(
            "POST",
            Some("http://gallery.invalid"),
            PreferenceUpdate {
                locale: "en".into(),
                location: "/gallery".into(),
            },
        )
        .unwrap();
    assert!(!reply.cookie.contains("Secure"));
    assert!(reply.cookie.contains("Path=/;"));
}

/// Route catalog and replacement preserve legacy/alias meaning and one-decode query bytes.
#[test]
fn compiled_prefixes_and_exact_return_context() {
    let registry = registry();
    assert_eq!(
        registry
            .select(
                &BasePath::default(),
                "/gallery",
                "stackpress-locale; stackpress-locale=fr"
            )
            .unwrap()
            .source,
        LocaleSource::Default
    );
    assert_eq!(registry.prefixes(), vec!["", "/en", "/fr", "/fr-FR"]);
    let base = BasePath::new("/demo/v1").unwrap();
    assert_eq!(
        registry
            .location(&base, "/demo/v1/en/gallery?q=a%2Fb&return=%252F", "fr-FR")
            .unwrap(),
        "/demo/v1/fr/gallery?q=a%2Fb&return=%252F"
    );
    assert_eq!(
        registry
            .select(&base, "/demo/v1/en/fr/gallery", "")
            .unwrap()
            .route,
        "/fr/gallery"
    );
    for path in [
        "/demo/v1/%2e%2e/gallery",
        "/demo/v1/gallery?q=%",
        "//evil.invalid",
        "/else/gallery",
    ] {
        assert!(registry.location(&base, path, "fr").is_err());
    }
    assert!(
        registry
            .location(&base, "/demo/v1/gallery", "missing")
            .is_err()
    );
}
