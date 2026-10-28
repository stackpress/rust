//! Public serialization/security/size contracts.

use stackpress_framework_view::*;

/// Build a public string fixture independently of any native host.
fn seed(data: String) -> PageSnapshot<String> {
    PageSnapshot {
        revision: 1,
        build: "test".into(),
        page: PageId::try_from("profiles".to_owned()).unwrap(),
        location: "/ui/profiles?id=abc".into(),
        locale: "en".into(),
        theme: Some("light".into()),
        status: PublicStatus::ok(),
        data,
    }
}

/// Script terminators and Unicode separators remain inert and roundtrip exactly.
#[test]
fn hostile_seed_and_contract_drift() {
    let data = "</script><script>globalThis.pwned=1</script>&\"\u{2028}\u{2029}";
    let json = seed(data.into()).script_json().unwrap();
    assert!(!json.contains('<'));
    assert!(!json.contains('&'));
    assert_eq!(
        PageSnapshot::<String>::decode(&json, 1, "test")
            .unwrap()
            .data,
        data
    );
    assert_eq!(
        PageSnapshot::<String>::decode(&json, 2, "test").unwrap_err(),
        ViewError::ContractMismatch
    );
    assert_eq!(
        PageSnapshot::<String>::decode(&json, 1, "old").unwrap_err(),
        ViewError::ContractMismatch
    );
    assert_eq!(
        PageSnapshot::<String>::decode("{}", 1, "test").unwrap_err(),
        ViewError::InvalidSnapshot
    );
}

/// Exact final ceilings accept the boundary and reject one byte beyond it.
#[test]
fn bounds_and_checked_metadata() {
    let oversized = serde_json::to_string(&seed("x".repeat(MAX_SNAPSHOT_BYTES))).unwrap();
    assert_eq!(
        PageSnapshot::<String>::decode(&oversized, 1, "test").unwrap_err(),
        ViewError::SnapshotLimit
    );
    assert!(RenderedPage::new("x".repeat(MAX_HTML_BYTES)).is_ok());
    assert_eq!(
        RenderedPage::new("x".repeat(MAX_HTML_BYTES + 1)).unwrap_err(),
        ViewError::HtmlLimit
    );
    assert_eq!(
        seed("x".repeat(MAX_SNAPSHOT_BYTES))
            .script_json()
            .unwrap_err(),
        ViewError::SnapshotLimit
    );
    for name in ["", "../view", "profiles/list", "profiles.js"] {
        assert!(PageId::try_from(name.to_owned()).is_err());
    }
    let mut value = seed("ok".into());
    for path in ["https://user:pass@host/", "//host/ui", "relative", "/\n"] {
        value.location = path.into();
        assert_eq!(value.validate(1, "test"), Err(ViewError::InvalidSnapshot));
    }
}

/// Explicit choices and unset retain their distinct public wire shapes.
#[test]
fn saved_theme_or_unset() {
    for theme in [Some("light"), Some("dark"), None] {
        let mut contract = PageContract::new(1, "test").unwrap();
        contract.theme = theme.map(str::to_owned);
        contract.validate().unwrap();
        let mut snapshot = seed("public".into());
        snapshot.theme = theme.map(str::to_owned);
        let encoded = snapshot.script_json().unwrap();
        assert_eq!(
            PageSnapshot::<String>::decode(&encoded, 1, "test")
                .unwrap()
                .theme
                .as_deref(),
            theme
        );
    }
    for theme in [
        "system",
        "System",
        "auto",
        "system ",
        "",
        "</style><script>",
    ] {
        let mut contract = PageContract::new(1, "test").unwrap();
        contract.theme = Some(theme.to_owned());
        assert_eq!(contract.validate(), Err(ViewError::InvalidPolicy));
    }
}
