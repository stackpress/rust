//! Exercise the actual bounded parse/compile entry, including malformed valid UTF-8 source.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    // The parser's public source is UTF-8; invalid bytes have no parser representation.
    if let Ok(source) = std::str::from_utf8(bytes) {
        let _ = stackpress_idea_parser::parse("fuzz.idea", source, &Default::default());
    }
});
