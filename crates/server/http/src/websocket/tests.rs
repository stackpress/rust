//! Public policy boundaries and Origin normalization, independent of network activity.
use super::*;

#[test]
fn origin_ports_never_fall_back_after_invalid_explicit_input() {
    assert_eq!(
        origin("https://EXAMPLE.test").unwrap(),
        origin("https://example.test:443").unwrap()
    );
    assert_eq!(
        origin("http://[::1]").unwrap(),
        origin("http://[::1]:80").unwrap()
    );
    assert_ne!(
        origin("http://example.test").unwrap(),
        origin("https://example.test").unwrap()
    );
    for value in [
        "https://example.test:65536",
        "https://example.test:abc",
        "https://example.test:",
        "https://user@example.test",
        "https://example.test/path",
        "https://example.test?q=1",
        "https://example.test#f",
        "file:///tmp/a",
        "null",
        "//example.test",
    ] {
        assert!(origin(value).is_err(), "accepted {value}");
    }
    assert!(origin(&"x".repeat(2049)).is_err());
    let mut request = Request::new();
    assert!(!origin_allowed(&request, &[], false, false).unwrap());
    assert!(origin_allowed(&request, &[], true, false).unwrap());
    request.headers.set("origin", "null").unwrap();
    assert!(origin_allowed(&request, &[], false, true).unwrap());
    request
        .headers
        .set("origin", "https://example.test")
        .unwrap();
    assert!(origin_allowed(&request, &["https://EXAMPLE.test:443"], false, false).unwrap());
    assert!(!origin_allowed(&request, &["http://example.test"], false, false).unwrap());
}

#[test]
fn private_policy_prevents_dependency_panics_and_unbounded_storage() {
    use std::time::Duration;
    let limits = WebSocketHostLimits::default();
    assert_eq!(limits.max_sessions(), 64);
    for cap in [0, usize::MAX] {
        assert!(limits.clone().with_max_sessions(cap).is_err());
    }
    for age in [Duration::ZERO, Duration::from_secs(3601)] {
        assert!(limits.clone().with_max_age(age).is_err());
    }
    for (message, frame) in [(0, 1), (1, 0), (65537, 1), (1, 65537)] {
        assert!(limits.clone().with_message_limits(message, frame).is_err());
    }
    for (count, bytes) in [(0, 1), (17, 1), (1, 0), (1, 1048577)] {
        assert!(limits.clone().with_queue_limits(count, bytes).is_err());
    }
    for (read, write, max) in [
        (0, 0, 262144),
        (16385, 0, 262144),
        (1, 16385, 262144),
        (1, 1, 262145),
        (1, 1, 1),
        (1, 1, 65536),
    ] {
        assert!(limits.clone().with_buffers(read, write, max).is_err());
    }
    assert!(limits.clone().with_buffers(1, 0, 65550).is_ok());
    assert!(
        limits
            .with_timing(
                Duration::ZERO,
                Duration::from_secs(1),
                Duration::from_secs(1),
                Duration::from_secs(1),
                Duration::from_secs(1)
            )
            .is_err()
    );
    for code in [999, 1004, 1005, 1006, 1015, 2000, 5000] {
        assert!(WsClose::new(code, "").is_err());
    }
    assert!(WsClose::new(1000, "é".repeat(62)).is_err());
    let close = WsClose::new(3000, "done").unwrap();
    assert_eq!((close.code(), close.reason()), (3000, "done"));
    assert!(
        WebSocketOptions::default()
            .with_subprotocol("bad protocol")
            .is_err()
    );
    assert!(
        WebSocketOptions::default()
            .with_snapshot(&["bad header"], &[])
            .is_err()
    );
    assert!(
        WebSocketOptions::default()
            .with_snapshot(&[], &[""])
            .is_err()
    );
}
