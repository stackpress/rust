//! The additive callback error seam preserves explicit classification and concrete typed sources.

use stackpress_idea_transformer::{TransformError, TransformErrorKind};
use std::error::Error;

/// Checked context is independent of cause Display, without changing parser/IO conversions.
#[test]
fn contextual_source_is_checked_and_downcastable() {
    let error = TransformError::with_source(
        TransformErrorKind::Callback,
        "framework schema failed",
        std::io::Error::other("secret sentinel"),
    )
    .unwrap();
    assert_eq!(error.kind(), TransformErrorKind::Callback);
    assert_eq!(error.to_string(), "framework schema failed");
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .is_some()
    );
    for context in [
        String::new(),
        " \t".into(),
        "bad\ncontext".into(),
        "x".repeat(1025),
    ] {
        let error = TransformError::with_source(
            TransformErrorKind::Parse,
            context,
            std::io::Error::other("private"),
        )
        .unwrap_err();
        assert_eq!(error.kind(), TransformErrorKind::Other);
        assert_eq!(error.to_string(), "invalid error context");
    }
}
