//! Real diagnostic boundaries preserve typed causes, distinct layers and bounded traversal.
use super::*;
use stackpress_lib::ErrorKind;

use crate::watchdog;

/// An explicit sink lets the real writer/flush boundaries fail with a concrete native cause.
struct BrokenSink {
    /// True selects flush failure after a successful write; false fails the write itself.
    fail_flush: bool,
}
impl Write for BrokenSink {
    /// Return a fixed native cause without retaining or displaying caller payload bytes.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.fail_flush {
            Ok(bytes.len())
        } else {
            Err(std::io::Error::other("fixture disk unavailable"))
        }
    }
    /// Exercise the independent delivery-completion failure path.
    fn flush(&mut self) -> std::io::Result<()> {
        if self.fail_flush {
            Err(std::io::Error::other("fixture flush unavailable"))
        } else {
            Ok(())
        }
    }
}

/// The CLI formatter renders each operation/source once and preserves concrete downcasts.
#[test]
#[ignore = "run by diagnostic watchdog"]
fn diagnostic_layers_child() {
    for (fail_flush, context, leaf) in [
        (
            false,
            "CLI diagnostic write failed",
            "fixture disk unavailable",
        ),
        (
            true,
            "CLI diagnostic flush failed",
            "fixture flush unavailable",
        ),
    ] {
        let error = write(
            &Control::new("", Default::default()),
            &mut BrokenSink { fail_flush },
            Level::Error,
            "PRIVATE payload",
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::External);
        assert_eq!(error.code(), 500);
        assert!(error.source().unwrap().is::<std::io::Error>());
        let outer = Error::new("command diagnostics failed").with_cause(error);
        let text = developer(&outer);
        assert_eq!(
            text,
            format!("command diagnostics failed; caused by: {context}; caused by: {leaf}")
        );
        for layer in ["command diagnostics failed", context, leaf] {
            assert_eq!(text.matches(layer).count(), 1);
        }
        assert_eq!(outer.public_view().error, "Internal Server Error");
        assert!(
            !serde_json::to_string(&outer.public_view())
                .unwrap()
                .contains("fixture")
        );
    }
    // Cancellation is a concrete source variant even behind multiple distinct framework contexts.
    let aborted = Error::new("prompt input acquisition failed").with_cause(PromptError::Aborted);
    let error = Error::new("command startup failed").with_cause(aborted);
    assert!(prompt_aborted(&error));
    assert!(
        error
            .source()
            .unwrap()
            .source()
            .unwrap()
            .is::<PromptError>()
    );
    assert!(!prompt_aborted(
        &Error::new("prompt was aborted").with_code(130)
    ));
}

/// A malformed source chain must not turn developer formatting into an unbounded traversal.
#[test]
fn developer_traversal_remains_bounded() {
    #[derive(Debug)]
    struct Cycle;
    impl std::fmt::Display for Cycle {
        /// A repeated label exposes the exact traversal count without hidden recursion.
        fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            out.write_str("cycle")
        }
    }
    impl StdError for Cycle {
        /// Deliberately violate acyclic ownership to prove the formatter's finite boundary.
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            Some(self)
        }
    }
    assert_eq!(developer(&Cycle).matches("cycle").count(), 33);
    assert!(!prompt_aborted(&Cycle));
}

/// Native substitutes and regression failures remain under an independent kill/reap owner.
#[test]
fn diagnostic_layers_are_watchdog_bounded() {
    let result = watchdog::run(
        std::process::Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "diagnostics::tests::diagnostic_layers_child",
            "--nocapture",
        ]),
        "diagnostic-layers",
        std::time::Duration::from_secs(3),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
