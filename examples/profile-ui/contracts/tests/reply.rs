//! Shared HTTP/IPC error decoding, status agreement and public-only projection.

use stackpress_example_profile_ui_contracts::*;

/// One pinned actual framework envelope, including explicit zero and safe field data.
fn host_error() -> serde_json::Value {
    serde_json::json!({"code":400,"status":"Bad Request","error":"Choose another prefix.","errors":{"id":"Unknown prefix"},"total":0})
}

/// Both adapters decode the same rejection, while HTTP additionally checks status agreement.
#[test]
fn success_and_host_error_are_distinct() {
    let text = host_error().to_string();
    for status in [None, Some(400)] {
        let SearchOutcome::Rejected(error) = decode_search_reply(&text, status).unwrap() else {
            panic!("error became success");
        };
        assert_eq!(error.total, 0);
        assert_eq!(error.errors["id"], "Unknown prefix");
        assert_eq!(error.message(), "Choose another prefix.");
        let failure = SearchFailure::Host(error);
        assert_eq!(failure.kind(), SearchFailureKind::Host);
        assert_eq!(failure.kind().as_str(), "host");
        assert_eq!(
            failure.to_string(),
            "Search rejected: Choose another prefix."
        );
    }
    let success = r#"{"revision":1,"rows":[],"total":0}"#;
    assert!(
        matches!(decode_search_reply(success, Some(200)).unwrap(), SearchOutcome::Success(reply) if reply.total==0)
    );
    assert!(matches!(
        decode_search_reply(success, None).unwrap(),
        SearchOutcome::Success(_)
    ));
    for status in [200, 401, 500] {
        assert_eq!(
            decode_search_reply(&text, Some(status)).unwrap_err(),
            ReplyDecodeError::StatusMismatch
        );
    }
    assert_eq!(
        decode_search_reply(success, Some(400)).unwrap_err(),
        ReplyDecodeError::StatusMismatch
    );
}

/// Malformed/unknown/duplicate/private fields and impossible success/error combinations fail.
#[test]
fn strict_bounded_reply_shapes() {
    for invalid in [
        "null",
        "{}",
        r#"{"revision":1,"rows":[],"total":1}"#,
        r#"{"code":400,"status":"Bad Request","error":"safe","errors":{},"total":0,"stack":"PRIVATE"}"#,
        r#"{"code":400,"status":"Bad Request","error":"safe","errors":{},"total":0,"total":0}"#,
        r#"{"code":400,"status":"Bad Request","error":"safe","errors":{},"total":0,"rows":[]}"#,
        r#"{"code":200,"status":"OK","error":"safe","errors":{},"total":0}"#,
        r#"{"code":400,"status":"Bad Request","error":"safe","errors":[],"total":0}"#,
        r#"{"code":400,"status":"Bad Request","error":"safe","errors":{},"total":1}"#,
    ] {
        assert_eq!(
            decode_search_reply(invalid, None).unwrap_err(),
            ReplyDecodeError::Invalid
        );
    }
    let mut value = host_error();
    value["error"] = "x".repeat(8193).into();
    assert_eq!(
        decode_search_reply(&value.to_string(), None).unwrap_err(),
        ReplyDecodeError::Invalid
    );
    value = host_error();
    value["status"] = "x".repeat(8193).into();
    assert_eq!(
        decode_search_reply(&value.to_string(), None).unwrap_err(),
        ReplyDecodeError::Invalid
    );
    assert_eq!(
        decode_search_reply(&"x".repeat(1024 * 1024 + 1), None).unwrap_err(),
        ReplyDecodeError::Limit
    );
    assert_eq!(
        decode_search_reply(r#"{"revision":999,"rows":[],"total":0}"#, None).unwrap_err(),
        ReplyDecodeError::ContractMismatch
    );
}

/// Native projection is allowlisted; empty safe text has a deliberate visible fallback.
#[test]
fn native_projection_and_empty_message_policy() {
    let mut value = host_error();
    value["stack"] = "PRIVATE_STACK".into();
    value["results"] = "PRIVATE_RESULT".into();
    value["native"] = "PRIVATE_SERVICE".into();
    let public = HostError::from_envelope(&value).unwrap();
    let serialized = serde_json::to_string(&public).unwrap();
    assert!(!serialized.contains("PRIVATE"));
    assert!(matches!(
        decode_search_reply(&serialized, None).unwrap(),
        SearchOutcome::Rejected(_)
    ));
    assert!(HostError::from_envelope(&serde_json::json!({})).is_err());
    value["error"] = "".into();
    assert_eq!(
        HostError::from_envelope(&value).unwrap().message(),
        "Bad Request"
    );
    value["status"] = "".into();
    assert_eq!(
        HostError::from_envelope(&value).unwrap().message(),
        "Search rejected"
    );
    value["code"] = "400".into();
    assert!(HostError::from_envelope(&value).is_err());
}

/// Transport and protocol messages are fixed strings, independent of failed-response contents.
#[test]
fn non_host_failure_identity() {
    for transport in [
        TransportFailure::Start,
        TransportFailure::Unavailable,
        TransportFailure::CancelledOrDenied,
    ] {
        let failure = SearchFailure::Transport(transport);
        assert_eq!(failure.kind().as_str(), "transport");
        assert!(!failure.to_string().is_empty());
    }
    for protocol in [
        ReplyDecodeError::Invalid,
        ReplyDecodeError::ContractMismatch,
        ReplyDecodeError::Limit,
        ReplyDecodeError::StatusMismatch,
    ] {
        let failure = SearchFailure::Protocol(protocol);
        assert_eq!(failure.kind().as_str(), "protocol");
        assert!(!failure.to_string().is_empty());
    }
}
