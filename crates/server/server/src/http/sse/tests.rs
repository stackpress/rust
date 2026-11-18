//! Wire encoding and hostile metadata proofs.
use super::*;
#[test]
fn encoder_preserves_lines_and_rejects_injection() {
    let l = Default::default();
    for (input, expected) in [
        ("", "data: \n\n"),
        ("a\r\nb\rc\n", "data: a\ndata: b\ndata: c\ndata: \n\n"),
        ("é", "data: é\n\n"),
    ] {
        assert_eq!(
            SseEvent::data(input).unwrap().encode(&l).unwrap(),
            expected.as_bytes()
        );
    }
    assert_eq!(
        SseEvent::data("x")
            .unwrap()
            .id("")
            .unwrap()
            .event("ready")
            .unwrap()
            .retry(0)
            .unwrap()
            .encode(&l)
            .unwrap(),
        b"id: \nevent: ready\nretry: 0\ndata: x\n\n"
    );
    assert_eq!(
        SseEvent::comment("a\rb").unwrap().encode(&l).unwrap(),
        b": a\n: b\n\n"
    );
    for field in ["x\r", "x\n", "x\0"] {
        assert!(SseEvent::data("x").unwrap().id(field).is_err());
    }
    assert!(SseEvent::data("x").unwrap().event("").is_err());
    assert!(SseEvent::comment("x").unwrap().id("x").is_err());
    assert!(SseEvent::comment("x").unwrap().event("x").is_err());
    assert!(SseEvent::comment("x").unwrap().retry(0).is_err());
}
#[test]
fn encoder_charges_wire_bytes_before_growth() {
    let l = Default::default();
    assert_eq!(
        SseEvent::data("x".repeat(65528))
            .unwrap()
            .encode(&l)
            .unwrap()
            .len(),
        65536
    );
    assert!(SseEvent::data("x".repeat(65529)).is_err());
    assert!(SseEvent::data("x".repeat(65528)).unwrap().retry(0).is_err());
    assert!(SseEvent::data("x").unwrap().id("x".repeat(1025)).is_err());
    assert!(SseEvent::data("x").unwrap().event("x".repeat(257)).is_err());
    assert!(SseEncodeLimits::new(0, 1, 1).is_err());
    let small = SseEncodeLimits::new(8, 1, 1).unwrap();
    assert!(SseEvent::data("xx").unwrap().encode(&small).is_err());
    assert!(
        SseOptions::new(
            Duration::ZERO,
            Duration::from_secs(1),
            Duration::from_secs(2),
            100
        )
        .is_err()
    );
}
#[test]
fn cursor_missing_empty_duplicate_and_bounded() {
    let mut request = Request::new();
    assert_eq!(last_event_id(&request, 1024).unwrap(), None);
    request.headers.set("Last-Event-ID", "").unwrap();
    assert_eq!(last_event_id(&request, 1024).unwrap(), Some(""));
    request.headers.set("Last-Event-ID", "ab").unwrap();
    assert!(last_event_id(&request, 1).is_err());
    request.headers.append("Last-Event-ID", "c").unwrap();
    assert!(last_event_id(&request, 1024).is_err());
}
