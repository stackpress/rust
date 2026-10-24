//! Physical-row wrapping and erasure through the public buffered prompt driver.

use std::process::Command;
use std::time::Duration;

use stackpress_cli::prompt::{
    BufferedInput, InvalidMode, PromptCancellation, PromptError, PromptEvent, PromptOptions,
    RenderOptions, Validation, prompt,
};

#[path = "../../../crates/lib/tests/support/watchdog.rs"]
mod watchdog;

/// Submit immediately and capture actual terminal framing at a chosen width.
async fn clear_at(message: &str, columns: usize) -> String {
    // Enter accepts optional empty input; clear-on-done must remove every preview row.
    let options = PromptOptions::new(message);
    let mut input = BufferedInput::new([PromptEvent::Enter].into_iter());
    let mut output = Vec::new();
    prompt(
        &options,
        &mut input,
        &mut output,
        &PromptCancellation::new(),
        RenderOptions {
            is_terminal: true,
            clear_on_done: true,
            columns: Some(columns),
        },
    )
    .await
    .unwrap();
    String::from_utf8(output).unwrap()
}

/// Check wide/emoji cells, exact right margins, errors, and shrinking from several rows.
async fn wrapped_scenarios() {
    // These expected row boundaries are hand-counted display columns, not UTF-8 byte lengths.
    for (width, painted, cleared) in [
        (
            2,
            "\r? \r\n界\r\n😀",
            "\r\x1b[2K\x1b[1A\r\x1b[2K\x1b[1A\r\x1b[2K",
        ),
        (
            3,
            "\r? \r\n界\r\n😀",
            "\r\x1b[2K\x1b[1A\r\x1b[2K\x1b[1A\r\x1b[2K",
        ),
        (4, "\r? 界\r\n😀", "\r\x1b[2K\x1b[1A\r\x1b[2K"),
        (6, "\r? 界😀", "\r\x1b[2K"),
        (8, "\r? 界😀", "\r\x1b[2K"),
    ] {
        assert_eq!(clear_at("界😀", width).await, format!("{painted}{cleared}"));
    }
    // Unicode-width understands common ZWJ/variation sequences and combining marks.
    assert_eq!(clear_at("👩‍💻", 4).await, "\r? 👩‍💻\r\x1b[2K");
    assert_eq!(clear_at("♥️", 4).await, "\r? ♥️\r\x1b[2K");
    assert_eq!(clear_at("e\u{301}", 3).await, "\r? e\u{301}\r\x1b[2K");
    assert_eq!(
        clear_at("a", 1).await,
        "\r?\r\n \r\na\r\x1b[2K\x1b[1A\r\x1b[2K\x1b[1A\r\x1b[2K"
    );

    // An invalid answer paints a four-row error; the correction must erase all four rows.
    let mut options = PromptOptions::new("x");
    options.invalid = InvalidMode::Clear;
    options = options.validator(|value| {
        Ok(if value == "ok" {
            Validation::Valid
        } else {
            Validation::Invalid("界😀 fail".into())
        })
    });
    let mut input = BufferedInput::new(
        [
            PromptEvent::Text("bad".into()),
            PromptEvent::Enter,
            PromptEvent::Text("ok".into()),
            PromptEvent::Enter,
        ]
        .into_iter(),
    );
    let mut output = Vec::new();
    prompt(
        &options,
        &mut input,
        &mut output,
        &PromptCancellation::new(),
        RenderOptions {
            is_terminal: true,
            clear_on_done: false,
            columns: Some(4),
        },
    )
    .await
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("? x \r\n[界\r\n😀 f\r\nail]"));
    assert!(output.contains("\r\x1b[2K\x1b[1A\r\x1b[2K\x1b[1A\r\x1b[2K\x1b[1A\r\x1b[2K? x "));
    assert!(output.ends_with("? x \r\nok\r\n"));

    // Zero/too-narrow widths fail before touching output; framing itself is budgeted too.
    for width in [0, 1] {
        let options = PromptOptions::new("界");
        let mut output = Vec::new();
        let mut input = BufferedInput::lines("\n");
        let error = prompt(
            &options,
            &mut input,
            &mut output,
            &PromptCancellation::new(),
            RenderOptions {
                is_terminal: true,
                columns: Some(width),
                ..RenderOptions::default()
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(error, PromptError::Configuration(_)));
        assert!(output.is_empty());
    }
    let mut options = PromptOptions::new("abcdefghijklmnopqrst");
    options.limits.render_bytes = 60;
    let mut input = BufferedInput::lines("\n");
    assert!(matches!(
        prompt(
            &options,
            &mut input,
            &mut Vec::new(),
            &PromptCancellation::new(),
            RenderOptions {
                is_terminal: true,
                columns: Some(1),
                ..RenderOptions::default()
            }
        )
        .await,
        Err(PromptError::Limit(_))
    ));
}

/// The internal timeout is backed by the parent process's independent deadline.
#[tokio::test(flavor = "current_thread")]
#[ignore = "run by render watchdog parent"]
async fn render_child() {
    // The outer parent remains able to kill this process if an executor thread blocks.
    tokio::time::timeout(Duration::from_secs(2), wrapped_scenarios())
        .await
        .unwrap();
}

/// Kill/reap a stalled renderer even if synchronous caller code blocks the executor.
#[test]
fn wrapped_rendering_is_bounded() {
    // Run the physical-row scenarios in a separate reaped process, not a potentially deadlocked
    // thread.
    let result = watchdog::run(
        Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "render_child",
            "--nocapture",
        ]),
        "render-contracts",
        Duration::from_secs(10),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
