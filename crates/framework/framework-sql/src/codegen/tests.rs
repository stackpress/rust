//! SQL adapter failure causes and encoded-data headroom, independent of engine-unit coverage.

use std::error::Error;

use super::*;
use stackpress_idea_transformer::TransformErrorKind;
use stackpress_lib::template::{TemplateError, TemplateErrorKind};

/// Missing selected context keeps a concrete typed cause and never discloses private data.
#[test]
fn template_errors_retain_kind_identity_and_private_cause() {
    let templates = Templates::new().unwrap();
    let error = templates
        .render(
            "SQL selected template failed",
            "[%missing%]",
            &json!({"private": "PRIVATE_SENTINEL"}),
        )
        .unwrap_err();
    assert_eq!(error.kind(), TransformErrorKind::Callback);
    assert_eq!(error.to_string(), "SQL selected template failed");
    assert!(!format!("{error}").contains("PRIVATE_SENTINEL"));
    let cause = error
        .source()
        .unwrap()
        .downcast_ref::<TemplateError>()
        .unwrap();
    assert_eq!(cause.kind(), TemplateErrorKind::MissingValue);
    // Unselected strict branches do not need payload values or invoke a fallback.
    assert_eq!(
        templates
            .render(
                "SQL optional template failed",
                "[%#?:enabled%][%missing%][%/?:enabled%]",
                &json!({"enabled": false})
            )
            .unwrap(),
        ""
    );
}

/// JSON escaping may exceed one MiB even when real emitted Rust remains below its cap.
#[test]
fn encoded_context_headroom_does_not_widen_output() {
    let templates = Templates::new().unwrap();
    let atom = literal(&"\\\"".repeat(160_000));
    let context = json!({"value": atom});
    assert!(serde_json::to_vec(&context).unwrap().len() > 1024 * 1024);
    let output = templates
        .render(
            "SQL literal template failed",
            "pub const VALUE: &str = [%value%];",
            &context,
        )
        .unwrap();
    assert!(output.len() < 1024 * 1024);
    assert!(output.contains(context["value"].as_str().unwrap()));
    // The independent engine output ceiling still refuses an actual oversized result.
    let error = templates
        .render(
            "SQL oversized template failed",
            "[%value%]",
            &json!({"value": "x".repeat(1024 * 1024 + 1)}),
        )
        .unwrap_err();
    assert_eq!(error.kind(), TransformErrorKind::Limit);
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<TemplateError>()
            .unwrap()
            .kind(),
        TemplateErrorKind::Limit
    );
}
