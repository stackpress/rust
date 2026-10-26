//! Configuration failures retain private typed diagnostics without publishing their payloads.
use serde::{Deserialize, Serialize, Serializer};
use stackpress_server::{ConfigLayers, ServerBuilder};
use std::error::Error as _;

/// Deliberately failing serialization supplies a recognizable private diagnostic.
#[derive(Deserialize)]
struct BadDefaults;
impl Serialize for BadDefaults {
    /// Fail before emitting defaults, with a diagnostic that must remain private.
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("PRIVATE serializer diagnostic"))
    }
}

/// JSON coordinates remain available privately after the rejected layer leaves state untouched.
#[test]
fn configuration_retains_private_parse_cause() {
    let mut layers = ConfigLayers::new();
    let error = layers.json("review", "{\n\"PRIVATE\": }").err().unwrap();
    let cause = error
        .source()
        .unwrap()
        .downcast_ref::<serde_json::Error>()
        .unwrap();
    assert_eq!(cause.line(), 2);
    assert!(cause.column() > 0);
    assert!(!error.to_string().contains("PRIVATE"));
    assert!(
        !serde_json::to_string(&error.public_view())
            .unwrap()
            .contains("PRIVATE")
    );
    assert_eq!(layers.entries().count(), 0);
}

/// Failed default serialization retains its typed cause without publishing sensitive text.
#[test]
fn configuration_retains_private_serialization_cause() {
    let mut builder = ServerBuilder::new();
    let error = builder
        .config_section("bad", BadDefaults, |_| Ok(()))
        .err()
        .unwrap();
    assert!(
        error
            .source()
            .unwrap()
            .to_string()
            .contains("PRIVATE serializer")
    );
    assert!(!error.to_string().contains("PRIVATE"));
    assert!(
        !serde_json::to_string(&error.public_view())
            .unwrap()
            .contains("PRIVATE")
    );
}
