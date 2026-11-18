#![no_main]
use libfuzzer_sys::fuzz_target;
use stackpress_server::http::sse::{SseEncodeLimits, SseEvent};
fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        for result in [SseEvent::data(text), SseEvent::comment(text)] {
            if let Ok(event) = result {
                let encoded = event.encode(&SseEncodeLimits::default()).unwrap();
                assert!(encoded.len() <= 65536);
                assert!(encoded.ends_with(b"\n\n"));
                assert!(!encoded.contains(&b'\r'));
                let _ = event.clone().id(text);
                let _ = event.event(text);
            }
        }
    }
});
