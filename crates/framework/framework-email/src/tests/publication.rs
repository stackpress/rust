//! Source option and content edge cases without a listener or external provider.
use crate::content::{ContentRequest, ContentResolver, NativeContentResolver};
use crate::options::{TransportKind, TransportOptions, number, string};
use serde_json::{Value, json};

/// Source numeric coercion retains empty, array and radix cases used by pool rate options.
#[test]
fn numeric_options_keep_javascript_coercion_boundaries() {
    for (input, expected) in [
        (Value::Null, Some(0.0)),
        (json!(false), Some(0.0)),
        (json!(true), Some(1.0)),
        (json!([]), Some(0.0)),
        (json!(["12"]), Some(12.0)),
        (json!("0x10"), Some(16.0)),
        (json!("0b11"), Some(3.0)),
        (json!("0o17"), Some(15.0)),
        (json!("0b2"), None),
        (json!([1, 2]), None),
        (json!({}), None),
    ] {
        assert_eq!(number(&input), expected, "{input}");
    }
    // Array stringification differs from object JSON and preserves null's empty slot.
    assert_eq!(
        string(&json!([null, [1, 2], {"value":3}])),
        ",1,2,[object Object]"
    );
    assert_eq!(string(&Value::Null), "null");
    assert_eq!(string(&json!(false)), "false");
}

/// URL precedence, duplicate options, prototype properties and numeric failures match source.
#[test]
fn transport_url_options_keep_precedence_and_error_paths() {
    let config = serde_json::from_value(json!({"url":"direct://relay.test?tag=a&tag=b&tag=c&tls.rejectUnauthorized=false&tls.toString=ignored&toString=ignored&fraction=1.5&ignored.path=value","jsonTransport":true})).unwrap();
    let resolved = TransportOptions::resolve(&config).unwrap();
    assert_eq!(resolved.kind, TransportKind::Smtp);
    assert_eq!(resolved.fields["direct"], true);
    assert_eq!(resolved.fields["tag"], json!(["a", "b", "c"]));
    assert_eq!(resolved.fields["fraction"], 1.5);
    assert_eq!(resolved.fields["tls"], json!({"rejectUnauthorized":false}));
    assert!(!resolved.fields.contains_key("toString"));
    assert!(!resolved.fields.contains_key("ignored.path"));
    for (config, expected) in [
        (json!({"url":17}), "email url must be a string"),
        (
            json!({"SES":{"ses":{},"aws":{}}}),
            "Using legacy SES configuration",
        ),
        (
            json!("smtp://relay.test?rateDelta=Infinity"),
            "nonfinite URL option numbers",
        ),
        (
            json!("smtp://user:pass%@relay.test"),
            "Malformed email URL auth escape",
        ),
    ] {
        let result = TransportOptions::resolve(&serde_json::from_value(config.clone()).unwrap());
        assert!(result.is_err(), "expected source error for {config}");
        assert!(
            result.unwrap_err().to_string().contains(expected),
            "{config}"
        );
    }
}

/// Every documented newline alias selects Windows framing while other values keep Unix mode.
#[test]
fn newline_and_unknown_service_options_keep_source_defaults() {
    for (newline, expected) in [
        (json!("WIN"), true),
        (json!("windows"), true),
        (json!("dos"), true),
        (json!("\r\n"), true),
        (json!("unix"), false),
        (Value::Null, false),
    ] {
        let options = TransportOptions::resolve(
            &serde_json::from_value(json!({"newline":newline,"service":"unknown.example.test"}))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(options.windows_newlines(), expected);
        assert_eq!(options.smtp().host, "localhost");
    }
}

/// Native data content handles URL-safe bytes, odd base64 and malformed/non-data locations.
#[tokio::test]
async fn data_uri_and_native_file_failures_remain_observable() {
    for (uri, expected) in [
        ("data:application/octet-stream;base64,-_8", vec![251, 255]),
        ("data:;base64,a", vec![]),
        ("data:text/plain;utf8,%E2%9C%93", "✓".as_bytes().to_vec()),
        ("data:text/plain,bad%escape", b"bad%escape".to_vec()),
        ("data:missing-comma", vec![]),
    ] {
        let bytes = NativeContentResolver
            .read(ContentRequest {
                location: uri.into(),
                options: Default::default(),
                http_headers: false,
            })
            .await
            .unwrap();
        assert_eq!(bytes, expected, "{uri}");
    }
    assert!(
        crate::content::parse_data_uri("ordinary text")
            .unwrap()
            .is_none()
    );
    // A unique absent path exercises real file IO without reading a shared application file.
    let missing =
        std::env::temp_dir().join(format!("stackpress_email_missing_{}", std::process::id()));
    let error = NativeContentResolver
        .read(ContentRequest {
            location: missing.to_string_lossy().into(),
            options: Default::default(),
            http_headers: false,
        })
        .await
        .unwrap_err();
    assert!(
        error
            .downcast_ref::<std::io::Error>()
            .is_some_and(|error| error.kind() == std::io::ErrorKind::NotFound)
    );
}

/// An invalid HTTP authority fails during native URL parsing before any connection can be opened.
#[tokio::test]
async fn malformed_native_http_location_retains_parse_error() {
    let error = NativeContentResolver
        .read(ContentRequest {
            location: "http://[invalid".into(),
            options: Default::default(),
            http_headers: true,
        })
        .await
        .unwrap_err();
    assert!(error.downcast_ref::<url::ParseError>().is_some());
}

/// Owned readers are consumed once, while malformed source Buffer objects preserve their error.
#[tokio::test]
async fn reader_slots_and_invalid_buffer_data_follow_source_resolution() {
    let mut mail = crate::NativeSendMailOptions::default();
    mail.content.insert(
        "text.content".into(),
        crate::Content::Reader(Box::pin(tokio::io::empty())),
    );
    let resolver: std::sync::Arc<dyn ContentResolver> = std::sync::Arc::new(NativeContentResolver);
    let bytes = crate::content::resolve(&mut mail, "text", &json!("ignored"), &resolver, true)
        .await
        .unwrap();
    assert!(bytes.is_empty());
    assert!(mail.content.is_empty());
    let error = crate::content::resolve(
        &mut mail,
        "text",
        &json!({"type":"Buffer","data":"invalid"}),
        &resolver,
        true,
    )
    .await
    .unwrap_err();
    assert_eq!(error.to_string(), "invalid Buffer data");
}
