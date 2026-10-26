//! Actual native wire decoding, including incompatible and hostile public data.

use super::*;
use stackpress_framework_view::MAX_SNAPSHOT_BYTES;

/// Real bootstrap metadata shape, matching lane A's build-controlled root fixture.
fn assets() -> String {
    serde_json::json!({"build": BUILD, "base": "", "module": "/assets/start.js", "stylesheet": null}).to_string()
}

/// Decoder admits exact public text/settings and rejects drift before mounting.
#[test]
fn public_seed_admission() {
    let initial = packaged().unwrap();
    assert!(initial.page.reply.rows.is_empty());
    assert_eq!(initial.language.locale(), "en");
    assert_eq!(initial.theme, Theme::Light);
    let mut snapshot = stackpress_example_profile_ui_contracts::snapshot(initial.page).unwrap();
    let decoded = decode(&serde_json::to_string(&snapshot).unwrap(), &assets()).unwrap();
    assert_eq!(decoded.page.input.id, "abc");
    assert_eq!(decoded.page_id, "profiles");
    assert_eq!(decoded.status.code(), 200);
    assert_eq!(decoded.language.text("Profiles", &[]), "Profiles");
    assert_eq!(decoded.theme, Theme::Light);
    snapshot.locale = "fr".into();
    snapshot.theme = "dark".into();
    let translated = decode(&snapshot.script_json().unwrap(), &assets()).unwrap();
    assert_eq!(translated.language.text("Profiles", &[]), "Profils");
    assert_eq!(translated.theme, Theme::Dark);
    snapshot.theme = "unknown".into();
    assert!(decode(&serde_json::to_string(&snapshot).unwrap(), &assets()).is_err());
    snapshot.theme = "light".into();
    snapshot.locale = "../fr".into();
    assert!(decode(&serde_json::to_string(&snapshot).unwrap(), &assets()).is_err());
    snapshot.locale = "en".into();
    snapshot.revision += 1;
    assert!(decode(&serde_json::to_string(&snapshot).unwrap(), &assets()).is_err());
    snapshot.revision = REVISION;
    snapshot.build = "old-assets".into();
    assert!(decode(&serde_json::to_string(&snapshot).unwrap(), &assets()).is_err());
    snapshot.build = BUILD.into();
    snapshot.data.input.id.clear();
    assert!(decode(&serde_json::to_string(&snapshot).unwrap(), &assets()).is_err());
    snapshot.data.input.id = "abc".into();
    snapshot.data.reply.total = 1;
    assert!(decode(&serde_json::to_string(&snapshot).unwrap(), &assets()).is_err());
    assert!(decode("{}", &assets()).is_err());
}

/// Reply admission protects bounded/typed shape while preserving malicious text as data.
#[test]
fn typed_reply_admission() {
    let hostile = r#"{"revision":1,"rows":[{"id":"a","name":"<script>false</script>"}],"total":1}"#;
    assert_eq!(
        reply(hostile, None).unwrap().rows[0].name,
        "<script>false</script>"
    );
    assert_eq!(
        reply(r#"{"revision":1,"rows":[],"total":0}"#, None)
            .unwrap()
            .total,
        0
    );
    for invalid in [
        "null",
        "{}",
        r#"{"revision":999,"rows":[],"total":0}"#,
        r#"{"revision":1,"rows":[],"total":1}"#,
        r#"{"revision":1,"rows":[],"total":0,"secret":true}"#,
    ] {
        assert!(reply(invalid, None).is_err());
    }
    assert!(reply(&"x".repeat(MAX_SNAPSHOT_BYTES + 1), None).is_err());
}

/// Real asset-shaped metadata controls the mount; no missing/malformed fallback exists.
#[test]
fn bounded_asset_admission_and_mount_agreement() {
    let mut snapshot =
        stackpress_example_profile_ui_contracts::snapshot(packaged().unwrap().page).unwrap();
    let mounted = serde_json::json!({"build": BUILD, "base": "/office", "module": "/assets/start.js", "stylesheet": null}).to_string();
    assert!(decode(&serde_json::to_string(&snapshot).unwrap(), &mounted).is_err());
    snapshot.location = "/office/ui/profiles?id=abc".into();
    let decoded = decode(&serde_json::to_string(&snapshot).unwrap(), &mounted).unwrap();
    assert_eq!(
        decoded
            .base
            .join("/ui-api/v1/profile-search")
            .unwrap()
            .as_str(),
        "/office/ui-api/v1/profile-search"
    );
    for invalid in [
        "{}",
        "null",
        r#"{"build":"old","base":"","module":"/assets/start.js","stylesheet":null}"#,
        r#"{"build":"profile-ui-m0-1","base":"//evil","module":"/assets/start.js","stylesheet":null}"#,
        r#"{"build":"profile-ui-m0-1","base":"","module":"javascript:x","stylesheet":null}"#,
        r#"{"build":"profile-ui-m0-1","base":"","module":"/assets/start.js?q","stylesheet":null}"#,
        r#"{"build":"profile-ui-m0-1","base":"","module":"/assets/start.js","stylesheet":true}"#,
    ] {
        assert!(stackpress_framework_view::PageAssets::decode(invalid, BUILD).is_err());
    }
    let stylesheet = serde_json::json!({"build": BUILD, "base": "", "module": "/assets/start.js", "stylesheet": "/assets/app.css"}).to_string();
    assert!(stackpress_framework_view::PageAssets::decode(&stylesheet, BUILD).is_ok());
    assert!(
        stackpress_framework_view::PageAssets::decode(&"x".repeat(16 * 1024 + 1), BUILD).is_err()
    );
}

/// Public dictionary snapshots roundtrip as data and cannot bypass provider checks.
#[test]
fn checked_language_wire_roundtrip() {
    let language = profile_language("fr").unwrap();
    let json = serde_json::to_string(&language).unwrap();
    let decoded: Language = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, language);
    for invalid in [
        r#"{"locale":"../en","phrases":{},"placeholder":"%s"}"#,
        r#"{"locale":"en","phrases":{},"placeholder":""}"#,
        r#"{"locale":"en","phrases":{},"placeholder":"%s","extra":1}"#,
    ] {
        assert!(serde_json::from_str::<Language>(invalid).is_err());
    }
}

/// HTTP and IPC preserve valid host rejection identity through the common client boundary.
#[test]
fn host_rejection_is_not_transport_failure() {
    let text = r#"{"code":422,"status":"Unprocessable Entity","error":"Choose another prefix.","errors":{},"total":0}"#;
    for status in [None, Some(422)] {
        let failure = reply(text, status).unwrap_err();
        assert_eq!(
            failure.kind(),
            stackpress_example_profile_ui_contracts::SearchFailureKind::Host
        );
        assert!(failure.to_string().contains("Choose another prefix."));
    }
    assert!(matches!(
        reply("private non-JSON failure", Some(503)),
        Err(SearchFailure::Protocol(_))
    ));
}
