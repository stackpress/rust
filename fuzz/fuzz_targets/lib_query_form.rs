//! Exercise both public query and URL-encoded body entries, including invalid UTF-8 bodies.
#![no_main]
use libfuzzer_sys::fuzz_target;
use stackpress_lib::data::{ParseLimits, parse_body, parse_query};

fuzz_target!(|bytes: &[u8]| {
    let limits = ParseLimits::default();
    let body = parse_body("application/x-www-form-urlencoded", bytes, &limits);
    if let Ok(query) = std::str::from_utf8(bytes) {
        // These entry points share URL-form semantics and must agree on input acceptance.
        let query = parse_query(query, &limits);
        assert_eq!(body.is_ok(), query.is_ok());
    }
});
