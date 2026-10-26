//! Typed public metadata, finite policies and route encoding at both wire boundaries.

use stackpress_framework_view::*;

/// Metadata is checked identically when decoded, not just when built natively.
#[test]
fn metadata_and_assets_are_checked() {
    for value in [
        "https://host/x",
        "//host/x",
        "/a\\b",
        "/a#b",
        "/%",
        "/%0x",
        "/a/../b",
        "/%2e%2e/x",
        "/<x>",
        "/a b",
    ] {
        assert!(PublicLocation::new(value).is_err(), "{value}");
    }
    assert!(PublicLocation::new("/ui/a%2Fb?id=%E9%9B%AA&zero=0").is_ok());
    for value in ["a", "/a/", "/a//b", "/a/../b", "/a?x", "/%2f"] {
        assert!(BasePath::new(value).is_err(), "{value}");
    }
    let base = BasePath::new("/demo/v1").unwrap();
    assert_eq!(base.strip("/demo/v1/ui"), Some("/ui"));
    assert_eq!(base.strip("/demo/v10/ui"), None);
    assert_eq!(base.strip("/demo/v1//evil"), None);
    assert_eq!(BasePath::new("/").unwrap(), BasePath::default());
    let contract = PageContract::new(1, "build-1").unwrap();
    let mut assets = PageAssets {
        build: "build-1".into(),
        base,
        module: PublicLocation::new("/assets/start.js").unwrap(),
        stylesheet: Some(PublicLocation::new("/assets/style.css").unwrap()),
    };
    let json = assets.script_json().unwrap();
    assert_eq!(PageAssets::decode(&json, "build-1").unwrap(), assets);
    assert_eq!(
        PageAssets::decode(&json, "old").unwrap_err(),
        ViewError::ContractMismatch
    );
    assert!(PageAssets::decode("{}", "build-1").is_err());
    assert!(PageAssets::decode(&" ".repeat(16385), "build-1").is_err());
    assets.module = PublicLocation::new("/assets/start.js?x=1").unwrap();
    assert!(assets.validate(&contract).is_err());
    for (revision, build) in [(0, "ok"), (1, ""), (1, "bad\"id")] {
        assert!(PageContract::new(revision, build).is_err());
    }
    let mut contract = contract;
    contract.locale = "fr-CA".into();
    contract.theme = "dark".into();
    assert!(contract.validate().is_ok());
    contract.locale = "fr--CA".into();
    assert!(contract.validate().is_err());
    contract.locale = "fr".into();
    contract.theme = "auto".into();
    assert!(contract.validate().is_err());
}

/// Final statuses preserve zero/empty data without disclosing native error causes.
#[test]
fn status_and_lower_limits_roundtrip() {
    let status = PublicStatus::new(404, Some("Missing <profile>".into())).unwrap();
    assert_eq!(status.code(), 404);
    assert_eq!(status.error(), Some("Missing <profile>"));
    let decoded: PublicStatus =
        serde_json::from_str(&serde_json::to_string(&status).unwrap()).unwrap();
    assert_eq!(decoded, status);
    for (code, error) in [
        (199, None),
        (600, None),
        (200, Some("wrong".into())),
        (500, Some("x".repeat(8193))),
    ] {
        assert!(PublicStatus::new(code, error).is_err());
    }
    assert!(serde_json::from_str::<PublicStatus>(r#"{"code":200,"error":"bad"}"#).is_err());
    let seed = PageSnapshot {
        revision: 1,
        build: "test".into(),
        page: PageId::try_from("profiles".to_owned()).unwrap(),
        location: "/ui/profiles?id=0".into(),
        locale: "en".into(),
        theme: "light".into(),
        status,
        data: serde_json::json!({"total":0,"empty":"","no":false}),
    };
    let bytes = serde_json::to_vec(&seed).unwrap().len();
    let limits = PageLimits {
        snapshot_bytes: bytes,
        html_bytes: 4,
    };
    let json = seed.script_json_with(limits).unwrap();
    let decoded = PageSnapshot::<serde_json::Value>::decode_with(&json, 1, "test", limits).unwrap();
    assert_eq!(decoded.data["total"], 0);
    assert_eq!(decoded.data["empty"], "");
    assert_eq!(decoded.data["no"], false);
    assert_eq!(
        seed.script_json_with(PageLimits {
            snapshot_bytes: bytes - 1,
            ..limits
        })
        .unwrap_err(),
        ViewError::SnapshotLimit
    );
    assert!(RenderedPage::with_limits("abcd".into(), limits).is_ok());
    assert_eq!(
        RenderedPage::with_limits("abcde".into(), limits).unwrap_err(),
        ViewError::HtmlLimit
    );
    assert!(
        PageLimits {
            snapshot_bytes: MAX_SNAPSHOT_BYTES + 1,
            ..limits
        }
        .validate()
        .is_err()
    );
    assert!(
        PageLimits {
            html_bytes: MAX_HTML_BYTES + 1,
            ..limits
        }
        .validate()
        .is_err()
    );
    assert!(
        PageSnapshot::<serde_json::Value>::decode_with(
            &"x".repeat(bytes * 6 + 1),
            1,
            "test",
            limits
        )
        .is_err()
    );
}

/// Compiled definitions produce one encoded path and reject ambiguous parameter sets.
#[test]
fn catalog_and_exact_parameters() {
    let route = PageRoute::new(
        "detail",
        &[RouteSegment::Static("ui"), RouteSegment::Param("id")],
    )
    .unwrap();
    let base = BasePath::new("/demo").unwrap();
    assert_eq!(route.pattern(), "/ui/:id");
    assert_eq!(
        route
            .location(&base, &[("id", "雪/ &?0")])
            .unwrap()
            .as_str(),
        "/demo/ui/%E9%9B%AA%2F%20%26%3F0"
    );
    for params in [
        vec![],
        vec![("other", "x")],
        vec![("id", "")],
        vec![("id", "..")],
        vec![("id", "x"), ("id", "y")],
    ] {
        assert!(route.location(&base, &params).is_err());
    }
    assert!(PageRoute::new("bad", &[]).is_err());
    assert!(PageRoute::new("bad", &[RouteSegment::Static("..")]).is_err());
    assert!(
        PageRoute::new(
            "bad",
            &[RouteSegment::Param("id"), RouteSegment::Param("id")]
        )
        .is_err()
    );
    assert!(RouteCatalog::new(base.clone(), vec![]).is_err());
    assert!(RouteCatalog::new(base.clone(), vec![route.clone(), route.clone()]).is_err());
    let catalog = RouteCatalog::new(base.clone(), vec![route]).unwrap();
    assert_eq!(catalog.base(), &base);
    assert_eq!(catalog.routes().count(), 1);
    assert_eq!(catalog.pattern("detail").unwrap().as_str(), "/demo/ui/:id");
    assert!(catalog.pattern("unknown").is_err());
    assert_eq!(encode_uri_component("0 false"), "0%20false");
}

/// Serialization errors keep their developer cause while Display stays safe.
#[test]
fn serialization_preserves_cause_without_disclosure() {
    use std::error::Error;
    struct Broken;
    impl serde::Serialize for Broken {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("private serializer sentinel"))
        }
    }
    let seed = PageSnapshot {
        revision: 1,
        build: "test".into(),
        page: PageId::try_from("test".to_owned()).unwrap(),
        location: "/".into(),
        locale: "en".into(),
        theme: "light".into(),
        status: PublicStatus::ok(),
        data: Broken,
    };
    let error = seed.script_json().unwrap_err();
    assert!(!error.to_string().contains("sentinel"));
    assert!(
        error
            .source()
            .unwrap()
            .to_string()
            .contains("private serializer sentinel")
    );
    assert_eq!(error.clone(), error);
}
/// Compiled captures preserve encoded boundaries and never select outside their mount.
#[test]
fn compiled_capture_boundaries() {
    use stackpress_framework_view::{BasePath, PageRoute, PublicLocation, RouteSegment};
    let route = PageRoute::new(
        "detail",
        &[RouteSegment::Static("items"), RouteSegment::Param("key")],
    )
    .unwrap();
    let base = BasePath::new("/app").unwrap();
    let location = PublicLocation::new("/app/items/%E9%9B%AA%2F%2B?return=a").unwrap();
    assert_eq!(
        route.captures(&base, &location).unwrap(),
        vec![("key", "%E9%9B%AA%2F%2B")]
    );
    for path in [
        "/application/items/a",
        "/app/items/",
        "/app/items/a/b",
        "/app/other/a",
        "/app/items",
    ] {
        assert!(
            route
                .captures(&base, &PublicLocation::new(path).unwrap())
                .is_none(),
            "{path}"
        );
    }
}
