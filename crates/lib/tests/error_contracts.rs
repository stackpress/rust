//! Contextual source chains retain concrete causes without repeating their explanations.

use stackpress_lib::{Error, ErrorKind, Result};
use std::error::Error as StdError;

/// Format each diagnostic layer exactly as a normal developer source traversal does.
fn developer_chain(error: &(dyn StdError + 'static)) -> String {
    let mut text = error.to_string();
    let mut current = error.source();
    while let Some(cause) = current {
        text.push_str("; caused by: ");
        text.push_str(&cause.to_string());
        current = cause.source();
    }
    text
}

/// Generic conversion supplies context while the source alone explains the failure.
#[test]
fn generic_source_context_preserves_private_cause_and_origin() -> Result<()> {
    let line = line!() + 1;
    let error = Error::from_source(std::io::Error::other("disk unavailable"))
        .at_path("bootstrap")
        .with_span(2, 5)?;
    assert_eq!(error.message(), "external operation failed");
    assert_eq!(error.kind(), ErrorKind::External);
    assert_eq!(error.origin().line, line);
    assert!(error.origin().file.ends_with("error_contracts.rs"));
    assert_eq!(error.path(), Some("bootstrap"));
    assert_eq!(error.span().unwrap().end, 5);
    assert!(error.source().unwrap().is::<std::io::Error>());
    assert_eq!(error.public_view().error, "Internal Server Error");
    assert_eq!(
        developer_chain(&error),
        "external operation failed at bootstrap; caused by: disk unavailable"
    );
    Ok(())
}

/// Parser and IO conversions remain typed through an additional contextual boundary.
#[test]
fn concrete_conversions_and_multiple_layers_do_not_repeat_causes() {
    let cause = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let explanation = cause.to_string();
    let inner = Error::from(cause);
    let error = Error::new("bootstrap configuration could not be decoded").with_cause(inner);
    let inner = error.source().unwrap().downcast_ref::<Error>().unwrap();
    assert!(inner.source().unwrap().is::<serde_json::Error>());
    assert_eq!(
        developer_chain(&error),
        format!(
            "bootstrap configuration could not be decoded; caused by: JSON operation failed; caused by: {explanation}"
        )
    );
    assert_eq!(
        Error::from(std::io::Error::other("reader")).message(),
        "I/O operation failed"
    );
    let error = Error::from("invalid".parse::<u32>().unwrap_err());
    assert_eq!(error.message(), "integer parsing failed");
    assert!(error.source().unwrap().is::<std::num::ParseIntError>());
}

/// Known lib boundaries retain operation context and the original concrete parser causes.
#[test]
fn lib_wrappers_keep_context_and_concrete_sources() {
    let error = stackpress_lib::data::parse_json_object("{", &Default::default()).unwrap_err();
    assert_eq!(error.message(), "JSON input could not be parsed");
    assert_eq!(error.code(), 400);
    assert!(error.source().unwrap().is::<serde_json::Error>());
    let error = stackpress_lib::Request::builder()
        .url("http://[")
        .build()
        .unwrap_err();
    assert_eq!(error.message(), "request URL could not be parsed");
    assert!(error.source().unwrap().is::<url::ParseError>());
    let error = stackpress_lib::EventPattern::regex("(", false)
        .err()
        .unwrap();
    assert_eq!(error.message(), "event pattern could not be compiled");
    assert_eq!(error.kind(), ErrorKind::Registration);
    assert!(error.source().unwrap().is::<regex::Error>());
}
