//! Numeric prefix conversion follows source parseInt/parseFloat rather than Number coercion.
use super::sql_number;
use stackpress_framework_schema::source::Value;

/// Integer SQL values retain signs, hexadecimal prefixes and the first nonnumeric boundary.
#[test]
fn integer_prefixes_match_source_number_conversion() {
    for (text, expected) in [
        ("  +0x1f-tail", 31.0),
        ("-0Xf", -15.0),
        ("012tail", 12.0),
        ("-12.75", -12.0),
        ("+", 0.0),
        ("0x", 0.0),
        ("words", 0.0),
    ] {
        assert_eq!(sql_number(&Value::from(text), true), expected, "{text}");
    }
}

/// Floating SQL values retain decimal/exponent prefixes and source Infinity spellings only.
#[test]
fn floating_prefixes_and_infinity_match_source_number_conversion() {
    for (text, expected) in [
        ("  -1.25e2tail", -125.0),
        ("+.5x", 0.5),
        ("1e", 1.0),
        ("0x10", 0.0),
        ("inf", 0.0),
        ("NaN", 0.0),
        ("", 0.0),
        ("Infinitytail", f64::INFINITY),
        ("+Infinity", f64::INFINITY),
        ("-Infinity", f64::NEG_INFINITY),
    ] {
        assert_eq!(sql_number(&Value::from(text), false), expected, "{text}");
    }
}
