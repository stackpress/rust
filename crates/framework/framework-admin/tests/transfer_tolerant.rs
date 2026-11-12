//! G62 import encoding, delimiter and allocation boundaries, independent of code generation.
use stackpress_framework_admin::transfer::{self, CsvBytes, Limits, Upload};

/// Observe identical cells through every accepted decoder and delimiter path.
#[test]
fn encodings_preserve_unicode_and_bom_priority() {
    let text = "id,name\r\na,مرحبا 🦀";
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(text.as_bytes());
    let expected = vec![vec!["id", "name"], vec!["a", "مرحبا 🦀"]];
    for bytes in [text.as_bytes().to_vec(), bom] {
        assert_eq!(
            transfer::parse(&bytes, Limits::default()).unwrap(),
            expected
        );
    }
    for little in [true, false] {
        let mut bytes = if little {
            vec![0xff, 0xfe]
        } else {
            vec![0xfe, 0xff]
        };
        for unit in text.encode_utf16() {
            bytes.extend(if little {
                unit.to_le_bytes()
            } else {
                unit.to_be_bytes()
            });
        }
        assert_eq!(
            transfer::parse(&bytes, Limits::default()).unwrap(),
            expected
        );
    }
    let western =
        transfer::parse(b"id;name\na;caf\xe9 \x80 \x93quote\x94", Limits::default()).unwrap();
    assert_eq!(western[1], ["a", "café € “quote”"]);
    let arabic = transfer::parse(b"id\tname\na\t\xe3\xd1\xcd\xc8\xc7", Limits::default()).unwrap();
    assert_eq!(arabic[1], ["a", "مرحبا"]);
    // One ambiguous Arabic-mapped byte is not enough to override Western text.
    assert_eq!(
        transfer::parse(b"h\n\xdf", Limits::default()).unwrap()[1],
        ["ß"]
    );
    assert_eq!(
        transfer::parse(b"h\n\x81", Limits::default()).unwrap()[1],
        ["\u{81}"]
    );
    for bytes in [
        b"\xef\xbb\xbf\xff".as_slice(),
        b"\xff\xfe\x00",
        b"\xfe\xff\xd8\x00",
        b"\xff\xfe\x00\xdc",
    ] {
        assert!(transfer::parse(bytes, Limits::default()).is_err());
    }
}

/// Quoted delimiters/line breaks cannot skew the fixed-size nonblank record sample.
#[test]
fn sniffing_counts_only_delimiters_outside_quotes() {
    for delimiter in [",", ";", "\t"] {
        let csv = format!(
            "id{delimiter}name\r\na{delimiter}\"commas,,,;tabs\t and \"\"quotes\"\"\"\nb{delimiter}\"line\r\ninside\"\r\n"
        );
        let rows = transfer::parse(csv.as_bytes(), Limits::default()).unwrap();
        assert_eq!(rows[1], ["a", "commas,,,;tabs\t and \"quotes\""]);
        assert_eq!(rows[2], ["b", "line\r\ninside"]);
    }
    // Equal consistency prefers comma; a missing candidate never becomes the separator.
    assert_eq!(
        transfer::parse(b"a,b;c\nx,y;z", Limits::default()).unwrap()[0],
        ["a", "b;c"]
    );
    assert_eq!(
        transfer::parse(b"a;b\nsingle", Limits::default()).unwrap()[0],
        ["a;b"]
    );
    // Blank records are ignored only while sniffing, and remain data during parsing.
    assert_eq!(
        transfer::parse(b"\n\nid;name\na;Ada", Limits::default()).unwrap()[2],
        ["id", "name"]
    );
    // The more consistent candidate wins even when another occurs on every record.
    assert_eq!(
        transfer::parse(b"a,b;c\nx,y,z;w\nu,v;q", Limits::default()).unwrap()[0],
        ["a,b", "c"]
    );
    let csv = format!("h;v\n{}\nlast", vec!["a;b"; 16].join("\n"));
    assert_eq!(
        transfer::parse(csv.as_bytes(), Limits::default()).unwrap()[0],
        ["h", "v"]
    );
}

/// Raw input, decoded expansion and each structural append retain their independent ceilings.
#[test]
fn hostile_inputs_remain_bounded_after_decoding_and_sniffing() {
    let limits = Limits::default();
    for bytes in [vec![0xff; limits.bytes + 1], vec![b'"'; limits.bytes + 1]] {
        assert!(transfer::parse(&bytes, limits).is_err());
    }
    // Both legacy and UTF-16 can expand when represented as UTF-8.
    assert!(transfer::parse(b"h\n\x80\x80", Limits { bytes: 6, ..limits }).is_err());
    assert!(
        transfer::parse(
            &[
                0xff, 0xfe, b'h', 0, 10, 0, 0xac, 0x20, 0xac, 0x20, 0xac, 0x20, 0xac, 0x20, 0xac,
                0x20
            ],
            Limits {
                bytes: 16,
                ..limits
            }
        )
        .is_err()
    );
    for csv in [
        format!("h;v\n{};a", "x".repeat(limits.cell + 1)),
        format!("{}\n{}", vec!["h"; 101].join(";"), vec!["a"; 101].join(";")),
        format!("h;v\n{}", vec!["a;b"; 101].join("\n")),
        "h;v\n\"unterminated".into(),
    ] {
        assert!(transfer::parse(csv.as_bytes(), limits).is_err());
    }
    assert!(transfer::parse(b"h\na", Limits { rows: 0, ..limits }).is_err());
}

/// Old text callers and new raw file callers share a byte-preserving owned upload type.
#[test]
fn uploads_accept_legacy_json_and_raw_bytes_without_decoding() {
    let text = String::from("id;name\na;Ada");
    let csv = CsvBytes::from(text.clone());
    assert_eq!(csv.as_ref(), text.as_bytes());
    assert!(!csv.is_empty());
    assert_eq!(csv.len(), text.len());
    let mut cleared = csv.clone();
    cleared.clear();
    assert!(cleared.is_empty());
    assert!(CsvBytes::default().is_empty());
    let old: Upload = serde_json::from_str(r#"{"csv":"id;name\na;Ada"}"#).unwrap();
    assert_eq!(old.csv, csv);
    let bytes = vec![0xff, 0xfe, 0, 0];
    let upload = Upload {
        csv: bytes.clone().into(),
        decode_export_guards: false,
        confirm: false,
    };
    let wire = serde_json::to_string(&upload).unwrap();
    let decoded: Upload = serde_json::from_str(&wire).unwrap();
    assert_eq!(decoded.csv.as_bytes(), bytes);
}
