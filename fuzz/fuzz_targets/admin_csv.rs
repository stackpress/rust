//! Exercise raw-byte admission, all decoders, quote-aware sniffing and bounded CSV structure.
#![no_main]
use libfuzzer_sys::fuzz_target;
use stackpress_framework_admin::transfer::{Limits, parse, write};

fuzz_target!(|bytes: &[u8]| {
    let limits = Limits::default();
    if let Ok(records) = parse(bytes, limits) {
        // Export uses a fixed UTF-8 comma dialect; a successful write must read back exactly.
        if let Ok(csv) = write(&records, limits) {
            assert_eq!(parse(csv.as_bytes(), limits).ok().as_ref(), Some(&records));
        }
    }
    // Lowered caps exercise refusals without letting the fuzz input widen any limit.
    let _ = parse(
        bytes,
        Limits {
            bytes: 4096,
            rows: 4,
            columns: 4,
            cell: 64,
        },
    );
});
