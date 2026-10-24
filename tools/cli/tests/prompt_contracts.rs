//! Watchdog-backed prompt behavior without native stdin or global process-stream mutation.

use std::error::Error as _;
use std::future::Future;
use std::io::{self, IsTerminal, Write};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use regex::Regex;
use stackpress_cli::prompt::{
    BufferedInput, InvalidMode, Prefill, PromptAction, PromptCancellation, PromptError,
    PromptEvent, PromptInput, PromptOptions, PromptResult, PromptState, PromptStatus,
    RenderOptions, Validation, prompt, prompt_native,
};
use tokio::sync::{Notify, mpsc};

#[path = "../../../crates/lib/tests/support/watchdog.rs"]
mod watchdog;

/// Exercise a complete supplied-text prompt with readable non-TTY output.
async fn lines(options: &PromptOptions, text: &str) -> (PromptResult<String>, String) {
    // Drive a finite supplied transcript and capture human output without borrowing process
    // streams.
    let mut input = BufferedInput::lines(text);
    let mut output = Vec::new();
    let result = prompt(
        options,
        &mut input,
        &mut output,
        &PromptCancellation::default(),
        RenderOptions::default(),
    )
    .await;
    (result, String::from_utf8(output).unwrap())
}

/// Protect each independent async scenario with the specified inner deadline.
async fn bounded(future: impl Future<Output = ()>) {
    // An individual async deadline identifies the scenario before the outer process watchdog
    // fires.
    tokio::time::timeout(Duration::from_secs(2), future)
        .await
        .expect("prompt scenario deadline");
}

/// Observe actual validator destruction, including cancellation and panic propagation.
struct Dropped(Arc<AtomicUsize>);
impl Drop for Dropped {
    /// Count the exact owned callback resource release.
    fn drop(&mut self) {
        // A visible counter proves callback-owned state is actually released, not merely
        // abandoned.
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Pure transitions reject stale results and preserve state on failed checks.
#[test]
fn state_validation_generation_editing_and_bounds() {
    // Freeze a Unicode candidate, test stale IDs and failed checks, then cancel its next
    // generation.
    let mut options = PromptOptions::new("Name?");
    options.default = Some("é😀".into());
    let mut state = PromptState::new(&options).unwrap();
    assert_eq!(state.current(), "");
    assert_eq!(state.default_value(), "é😀");
    assert_eq!(
        state.apply(&options, PromptEvent::Tab).unwrap(),
        PromptAction::Render
    );
    state.apply(&options, PromptEvent::Backspace).unwrap();
    assert_eq!(state.current(), "é");
    let PromptAction::Validate { id, value } = state.apply(&options, PromptEvent::Enter).unwrap()
    else {
        panic!()
    };
    assert_eq!(value, "é");
    assert_eq!(
        state.apply(&options, PromptEvent::Enter).unwrap(),
        PromptAction::None
    );
    assert_eq!(
        state
            .apply(&options, PromptEvent::Text("ignored".into()))
            .unwrap(),
        PromptAction::None
    );
    assert_eq!(
        state
            .apply(
                &options,
                PromptEvent::Validated {
                    id: id + 1,
                    result: Validation::Valid
                }
            )
            .unwrap(),
        PromptAction::None
    );
    let oversized = "x".repeat(options.limits.error_bytes + 1);
    assert!(matches!(
        state.apply(
            &options,
            PromptEvent::Validated {
                id,
                result: Validation::Invalid(oversized)
            }
        ),
        Err(PromptError::Limit(_))
    ));
    assert_eq!(state.status(), PromptStatus::Validating(id));
    state
        .apply(
            &options,
            PromptEvent::Validated {
                id,
                result: Validation::Invalid("retry".into()),
            },
        )
        .unwrap();
    assert_eq!(state.current(), "é");
    assert_eq!(state.error_message(), "retry");
    assert!(
        state
            .apply(&options, PromptEvent::Text("\x1b".into()))
            .is_err()
    );
    assert_eq!(state.error_message(), "retry");
    let PromptAction::Validate { id: next, .. } =
        state.apply(&options, PromptEvent::Enter).unwrap()
    else {
        panic!()
    };
    assert!(next > id);
    assert_eq!(
        state.apply(&options, PromptEvent::Cancel).unwrap(),
        PromptAction::Aborted
    );
    assert_eq!(
        state
            .apply(
                &options,
                PromptEvent::Validated {
                    id: next,
                    result: Validation::Valid
                }
            )
            .unwrap(),
        PromptAction::None
    );
    assert_eq!(state.status(), PromptStatus::Aborted);

    options.limits.answer_bytes = 6;
    let mut state = PromptState::new(&options).unwrap();
    state
        .apply(&options, PromptEvent::Text("é😀".into()))
        .unwrap();
    assert!(
        state
            .apply(&options, PromptEvent::Text("!".into()))
            .is_err()
    );
    assert_eq!(state.current(), "é😀");
    state.apply(&options, PromptEvent::Backspace).unwrap();
    assert_eq!(state.current(), "é");
    state.apply(&options, PromptEvent::Backspace).unwrap();
    state.apply(&options, PromptEvent::Backspace).unwrap();
    assert!(state.default_value().is_empty());
    state.apply(&options, PromptEvent::Eof).unwrap();
    assert_eq!(state.status(), PromptStatus::Ended);
    assert_eq!(
        state.apply(&options, PromptEvent::Enter).unwrap(),
        PromptAction::None
    );
}

/// Default/required/pattern/Keep/Clear and Unicode editing match the accepted contract.
async fn editing_and_validation() {
    // Run the same edit/submit grammar through defaults, built-in rules and custom rejection
    // policies.
    let mut options = PromptOptions::new("Name?");
    let (result, output) = lines(&options, "Ada\r\n").await;
    assert_eq!(result.unwrap(), "Ada");
    assert_eq!(output, "? Name?\n? Name? Ada\n");
    options.default = Some("Chris".into());
    assert_eq!(lines(&options, "\n").await.0.unwrap(), "Chris");
    assert_eq!(lines(&options, "\t\x7fs\n").await.0.unwrap(), "Chris");
    options.prefill = Prefill::Editable;
    assert_eq!(lines(&options, "\t\x7fs\n").await.0.unwrap(), "Chris");
    options.default = None;
    options.required = true;
    let (result, output) = lines(&options, "\nAda\n").await;
    assert_eq!(result.unwrap(), "Ada");
    assert!(output.contains("You must provide a value\n"));
    options = options.with_pattern(Regex::new("^[a-z]+$").unwrap());
    options.invalid = InvalidMode::Clear;
    assert_eq!(lines(&options, "123\nada\n").await.0.unwrap(), "ada");
    options = options.without_pattern();
    options = options.validator(|value| {
        Ok(if value == "admin" {
            Validation::Valid
        } else {
            Validation::Invalid("Role is invalid".into())
        })
    });
    options.invalid = InvalidMode::Keep;
    let (result, output) = lines(&options, "user\n\x7f\x7f\x7f\x7fadmin\n").await;
    assert_eq!(result.unwrap(), "admin");
    assert!(output.contains("Role is invalid\n"));
    // Clearing a rejected default prevents a repeated empty submit from restoring it.
    options.invalid = InvalidMode::Clear;
    options.default = Some("user".into());
    assert_eq!(lines(&options, "\n\nadmin\n").await.0.unwrap(), "admin");
    options = options.without_validator();
    options.default = None;
    assert_eq!(lines(&options, "é😀\x7f!\n").await.0.unwrap(), "é!");
}

/// Earlier validation rules prevent later callbacks; malformed patterns use Rust's compiler.
async fn rule_order_render_transform_and_errors() {
    // Count custom calls to prove required/regex precedence before testing display and source
    // failures.
    let calls = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&calls);
    let mut options = PromptOptions::new("\x1b[2J Name?");
    options.required = true;
    options = options.with_pattern(Regex::new("^ok$").unwrap());
    options.invalid = InvalidMode::Clear;
    options = options.validator(move |_| {
        count.fetch_add(1, Ordering::SeqCst);
        Ok(Validation::Valid)
    });
    options = options.transform(|value, final_state| {
        Ok(if final_state {
            format!("FINAL:{value}\x1b[2J")
        } else {
            "***".into()
        })
    });
    let (result, output) = lines(&options, "\nwrong\nok\n").await;
    assert_eq!(result.unwrap(), "ok");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(output.contains("FINAL:ok"));
    assert!(!output.contains('\x1b'));
    let malformed = String::from("[");
    assert!(Regex::new(&malformed).is_err());

    options = options.without_transform();
    options =
        options.validator(|_| Err(PromptError::callback(io::Error::other("validator cause"))));
    let error = lines(&options, "ok\n").await.0.unwrap_err();
    assert_eq!(error.to_string(), "prompt callback failed");
    assert_eq!(error.source().unwrap().to_string(), "validator cause");
    options = options.without_validator();
    options = options.transform(|_, final_state| {
        if final_state {
            Err(PromptError::callback(io::Error::other("transform cause")))
        } else {
            Ok(String::new())
        }
    });
    assert_eq!(
        lines(&options, "ok\n")
            .await
            .0
            .unwrap_err()
            .source()
            .unwrap()
            .to_string(),
        "transform cause"
    );
    options =
        options.transform(|_, _| Err(PromptError::callback(io::Error::other("initial transform"))));
    assert!(matches!(
        lines(&options, "").await.0,
        Err(PromptError::Callback(_))
    ));
}

/// Deterministic channel source reports actual consumption, not merely a successful send.
struct ObservedInput {
    /// Real bounded input queue.
    input: mpsc::Receiver<PromptEvent>,
    /// Sufficiently bounded test-only receipt channel, never awaited after consuming input.
    consumed: mpsc::Sender<()>,
}
impl PromptInput for ObservedInput {
    /// Send an immediate receipt only after an event is removed from the input queue.
    async fn next(&mut self) -> io::Result<Option<PromptEvent>> {
        // A nonblocking receipt establishes real consumption without introducing another
        // cancellation await.
        let event = self.input.recv().await;
        if event.is_some() {
            self.consumed.try_send(()).unwrap();
        }
        Ok(event)
    }
}

/// Delayed validation buffers edits serially; rejection replays them without concurrent validation.
async fn queued_edits_and_async_eof() {
    // Hold validation with Notify so actual consumed receipts prove later edits were queued first.
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let signal = Arc::clone(&started);
    let wait = Arc::clone(&release);
    let mut options = PromptOptions::new("Role?");
    options.invalid = InvalidMode::Clear;
    options = options.async_validator(move |value| {
        let signal = Arc::clone(&signal);
        let wait = Arc::clone(&wait);
        async move {
            if value == "good" {
                return Ok(Validation::Valid);
            }
            signal.notify_one();
            wait.notified().await;
            Ok(Validation::Invalid("retry".into()))
        }
    });
    let (sender, receiver) = mpsc::channel(8);
    let (consumed, mut receipts) = mpsc::channel(8);
    let mut input = ObservedInput {
        input: receiver,
        consumed,
    };
    let mut output = Vec::new();
    let cancellation = PromptCancellation::new();
    let driver = prompt(
        &options,
        &mut input,
        &mut output,
        &cancellation,
        RenderOptions::default(),
    );
    let controller = async {
        sender.send(PromptEvent::Text("bad".into())).await.unwrap();
        sender.send(PromptEvent::Enter).await.unwrap();
        started.notified().await;
        sender.send(PromptEvent::Text("good".into())).await.unwrap();
        sender.send(PromptEvent::Enter).await.unwrap();
        // Four receipts prove both later edits were consumed while validation was still pending.
        for _ in 0..4 {
            receipts.recv().await.unwrap();
        }
        release.notify_one();
    };
    let (result, ()) = tokio::join!(driver, controller);
    assert_eq!(result.unwrap(), "good");

    let mut input = BufferedInput::lines("good\n");
    assert_eq!(
        prompt(
            &options,
            &mut input,
            &mut output,
            &cancellation,
            RenderOptions::default()
        )
        .await
        .unwrap(),
        "good"
    );
}

/// Retained text behind a queued Enter still counts against the next validation's queue budget.
async fn retained_queue_budget() {
    // Retain text behind a queued Enter, then attempt to exceed the next aggregate payload budget.
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let signal = Arc::clone(&started);
    let wait = Arc::clone(&release);
    let mut options = PromptOptions::new("Bound?");
    options.invalid = InvalidMode::Clear;
    options.limits.answer_bytes = 4;
    options = options.async_validator(move |_| {
        let signal = Arc::clone(&signal);
        let wait = Arc::clone(&wait);
        async move {
            signal.notify_one();
            wait.notified().await;
            Ok(Validation::Invalid("retry".into()))
        }
    });
    let (sender, receiver) = mpsc::channel(8);
    let (consumed, mut receipts) = mpsc::channel(8);
    let mut input = ObservedInput {
        input: receiver,
        consumed,
    };
    let mut output = Vec::new();
    let cancellation = PromptCancellation::new();
    let driver = prompt(
        &options,
        &mut input,
        &mut output,
        &cancellation,
        RenderOptions::default(),
    );
    let controller = async {
        sender.send(PromptEvent::Text("a".into())).await.unwrap();
        sender.send(PromptEvent::Enter).await.unwrap();
        started.notified().await;
        sender.send(PromptEvent::Enter).await.unwrap();
        sender.send(PromptEvent::Text("bb".into())).await.unwrap();
        for _ in 0..4 {
            receipts.recv().await.unwrap();
        }
        release.notify_one();
        started.notified().await;
        // bb remains behind the second Enter; ccc would bring retained text to five bytes.
        sender.send(PromptEvent::Text("ccc".into())).await.unwrap();
    };
    let (result, ()) = tokio::join!(driver, controller);
    assert!(matches!(result, Err(PromptError::Limit(_))));
}

/// Both explicit cancellation and Ctrl-C events interrupt a pending validation and drop its future.
async fn cancellation_and_drop() {
    // Observe validator start before signalling either shared cancellation or an explicit Cancel
    // event.
    for event_cancellation in [false, true] {
        let started = Arc::new(Notify::new());
        let dropped = Arc::new(AtomicUsize::new(0));
        let signal = Arc::clone(&started);
        let count = Arc::clone(&dropped);
        let mut options = PromptOptions::new("Pending?");
        options = options.async_validator(move |_| {
            let signal = Arc::clone(&signal);
            let count = Arc::clone(&count);
            async move {
                let _resource = Dropped(count);
                signal.notify_one();
                std::future::pending().await
            }
        });
        let (sender, mut input) = mpsc::channel(4);
        let cancellation = PromptCancellation::new();
        let mut output = Vec::new();
        let driver = prompt(
            &options,
            &mut input,
            &mut output,
            &cancellation,
            RenderOptions::default(),
        );
        let controller = async {
            sender.send(PromptEvent::Enter).await.unwrap();
            started.notified().await;
            if event_cancellation {
                sender.send(PromptEvent::Cancel).await.unwrap();
            } else {
                cancellation.cancel();
            }
        };
        let (result, ()) = tokio::join!(driver, controller);
        assert!(matches!(result, Err(PromptError::Aborted)));
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
    }
    let options = PromptOptions::new("never rendered");
    let cancellation = PromptCancellation::new();
    cancellation.cancel();
    let mut output = Vec::new();
    let mut input = BufferedInput::lines("anything\n");
    assert!(matches!(
        prompt(
            &options,
            &mut input,
            &mut output,
            &cancellation,
            RenderOptions::default()
        )
        .await,
        Err(PromptError::Aborted)
    ));
    assert!(output.is_empty());
    // External future drop is distinct from cooperative cancellation and releases callback state.
    let released = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&released);
    let mut options = PromptOptions::new("Drop?");
    options = options.async_validator(move |_| {
        let count = Arc::clone(&count);
        async move {
            let _owned = Dropped(count);
            std::future::pending().await
        }
    });
    let mut input = BufferedInput::lines("\n");
    let cancellation = PromptCancellation::new();
    let mut future = Box::pin(prompt(
        &options,
        &mut input,
        &mut output,
        &cancellation,
        RenderOptions::default(),
    ));
    std::future::poll_fn(|context| {
        assert!(future.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(future);
    assert_eq!(released.load(Ordering::SeqCst), 1);
}

/// Partial writes and flush failures retain exact OS causes and never retry a final render.
struct FailingOutput {
    /// Number of bytes accepted before returning BrokenPipe.
    remaining: usize,
    /// Select a flush failure independently of write failure.
    fail_flush: bool,
}
impl Write for FailingOutput {
    /// Accept a bounded prefix, then fail on the following write_all iteration.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // Accept a bounded prefix so write_all must observe a later error after partial output.
        if self.remaining == 0 {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        let accepted = bytes.len().min(self.remaining);
        self.remaining -= accepted;
        Ok(accepted)
    }
    /// Prove flush errors are not silently ignored.
    fn flush(&mut self) -> io::Result<()> {
        // A separate switch proves flushing is part of successful prompt delivery.
        if self.fail_flush {
            Err(io::ErrorKind::BrokenPipe.into())
        } else {
            Ok(())
        }
    }
}

/// Input/output/render/queue limits terminate; plain clear-on-done emits no escapes.
async fn failure_and_render_limits() {
    // Fail each output phase and render/input budget, keeping cancellation and EOF
    // distinguishable.
    let options = PromptOptions::new("Name?");
    assert!(matches!(lines(&options, "").await.0, Err(PromptError::Eof)));
    assert!(matches!(
        lines(&options, "\u{3}").await.0,
        Err(PromptError::Aborted)
    ));
    assert!(matches!(
        lines(&options, "\u{1b}").await.0,
        Err(PromptError::Configuration(_))
    ));
    for mut output in [
        FailingOutput {
            remaining: 2,
            fail_flush: false,
        },
        FailingOutput {
            remaining: 1000,
            fail_flush: true,
        },
        FailingOutput {
            remaining: "? Name?\n".len() + 2,
            fail_flush: false,
        },
    ] {
        let mut input = BufferedInput::lines("a\n");
        let error = prompt(
            &options,
            &mut input,
            &mut output,
            &PromptCancellation::new(),
            RenderOptions::default(),
        )
        .await
        .unwrap_err();
        assert!(matches!(error, PromptError::Io(_)));
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<io::Error>()
                .is_some()
        );
        assert_eq!(error.to_string(), "prompt I/O failed");
    }
    for terminal in [false, true] {
        let mut output = Vec::new();
        let mut input = BufferedInput::lines("a\n");
        prompt(
            &options,
            &mut input,
            &mut output,
            &PromptCancellation::new(),
            RenderOptions {
                is_terminal: terminal,
                clear_on_done: true,
                columns: Some(80),
            },
        )
        .await
        .unwrap();
        assert_eq!(output.contains(&0x1b), terminal);
        if terminal {
            assert!(output.ends_with(b"\r\x1b[2K"));
        }
    }
    let mut forged = BufferedInput::new(
        [PromptEvent::Validated {
            id: 1,
            result: Validation::Valid,
        }]
        .into_iter(),
    );
    assert!(matches!(
        prompt(
            &options,
            &mut forged,
            &mut Vec::new(),
            &PromptCancellation::new(),
            RenderOptions::default()
        )
        .await,
        Err(PromptError::Configuration(_))
    ));
    let mut options = PromptOptions::new("limits");
    options.limits.answer_bytes = 1;
    assert!(matches!(
        lines(&options, "é").await.0,
        Err(PromptError::Limit(_))
    ));
    options.default = Some("é".into());
    assert!(PromptState::new(&options).is_err());
    options.default = None;
    options.limits.render_bytes = 1;
    assert!(PromptState::new(&options).is_err());
    options.limits.render_bytes = 40;
    options.message = "\x1b".repeat(20);
    assert!(matches!(
        lines(&options, "").await.0,
        Err(PromptError::Limit(_))
    ));
    options.message = "x".into();
    options = options.transform(|_, _| Ok("x".repeat(41)));
    assert!(matches!(
        lines(&options, "").await.0,
        Err(PromptError::Limit(_))
    ));
    options = options.without_transform();
    options.limits.pending_events = 1;
    options = options.async_validator(|_| std::future::pending());
    // Two queued edits exceed the one-event bound while validation remains unresolved.
    assert!(matches!(
        lines(&options, "\na\n").await.0,
        Err(PromptError::Limit(_))
    ));
}

/// Native pipes never spawn a reader; supplied/default answers still run the real validation path.
async fn noninteractive_answers() {
    // On a real pipe, absence must fail without reading while explicit/default answers still
    // validate.
    if io::stdin().is_terminal() {
        return;
    }
    let mut options = PromptOptions::new("Native?");
    let mut output = Vec::new();
    let cancellation = PromptCancellation::new();
    let render = RenderOptions::default();
    assert!(matches!(
        prompt_native(&options, None, &mut output, &cancellation, render).await,
        Err(PromptError::NonInteractive)
    ));
    assert!(output.is_empty());
    options.default = Some("default".into());
    assert_eq!(
        prompt_native(&options, None, &mut output, &cancellation, render)
            .await
            .unwrap(),
        "default"
    );
    assert_eq!(
        prompt_native(
            &options,
            Some("explicit"),
            &mut output,
            &cancellation,
            render
        )
        .await
        .unwrap(),
        "explicit"
    );
    options = options.with_pattern(Regex::new("^good$").unwrap());
    let rejected = prompt_native(&options, Some("bad"), &mut output, &cancellation, render)
        .await
        .unwrap_err();
    assert!(matches!(&rejected, PromptError::InvalidAnswer(message) if message == "Invalid input"));
    assert_eq!(rejected.to_string(), "supplied prompt answer was rejected");
    options.limits.answer_bytes = 1;
    options.default = None;
    assert!(matches!(
        prompt_native(&options, Some("long"), &mut output, &cancellation, render).await,
        Err(PromptError::Limit(_))
    ));
    cancellation.cancel();
    assert!(matches!(
        prompt_native(&options, None, &mut output, &cancellation, render).await,
        Err(PromptError::Aborted)
    ));
}

/// A panic is propagated, then independent prompt state/validation can run normally.
async fn panic_isolation() {
    // Let a validator unwind in its task, then prove a fresh prompt has no poisoned framework
    // state.
    let panicked = tokio::spawn(async {
        let mut options = PromptOptions::new("Panic?");
        options = options.validator(|_| panic!("intentional validator panic"));
        let _ = lines(&options, "\n").await;
    })
    .await
    .unwrap_err();
    assert!(panicked.is_panic());
    assert_eq!(
        lines(&PromptOptions::new("Fresh?"), "ok\n")
            .await
            .0
            .unwrap(),
        "ok"
    );
}

/// A ready completion cannot revive cancellation raised while its own future was being polled.
async fn cancellation_at_completion() {
    // The callback signals cancellation during the same poll that produces its ready result.
    let cancellation = PromptCancellation::new();
    let signal = cancellation.clone();
    let options = PromptOptions::new("Cancel?").async_validator(move |_| {
        signal.cancel();
        async { Ok(Validation::Valid) }
    });
    let mut input = BufferedInput::lines("accepted?\n");
    let mut output = Vec::new();
    let result = prompt(
        &options,
        &mut input,
        &mut output,
        &cancellation,
        RenderOptions::default(),
    )
    .await;
    // No final accepted answer may be published after the cancellation signal has been raised.
    assert!(matches!(result, Err(PromptError::Aborted)));
    assert!(!String::from_utf8(output).unwrap().contains("accepted?"));
}

/// Explicit/default native answers are candidates, not edits appended to an Editable prefill.
async fn supplied_prefill_is_not_duplicated() {
    // Reuse one default with Editable policy; neither fallback nor an override may duplicate it.
    let mut options = PromptOptions::new("Name?");
    options.default = Some("Ada".into());
    options.prefill = Prefill::Editable;
    let cancellation = PromptCancellation::new();
    let mut output = Vec::new();
    let render = RenderOptions::default();
    assert_eq!(
        prompt_native(&options, Some("Grace"), &mut output, &cancellation, render)
            .await
            .unwrap(),
        "Grace"
    );
    assert_eq!(
        prompt_native(&options, Some(""), &mut output, &cancellation, render)
            .await
            .unwrap(),
        ""
    );
    if !io::stdin().is_terminal() {
        assert_eq!(
            prompt_native(&options, None, &mut output, &cancellation, render)
                .await
                .unwrap(),
            "Ada"
        );
    }
}

/// Exact child entry cannot recursively launch the external watchdog parent.
#[tokio::test(flavor = "current_thread")]
#[ignore = "run by external watchdog"]
async fn prompt_scenarios_child() {
    // Each scenario gets its own two-second deadline inside the exact watchdog-owned child entry.
    bounded(editing_and_validation()).await;
    bounded(rule_order_render_transform_and_errors()).await;
    bounded(queued_edits_and_async_eof()).await;
    bounded(retained_queue_budget()).await;
    bounded(cancellation_and_drop()).await;
    bounded(failure_and_render_limits()).await;
    bounded(noninteractive_answers()).await;
    bounded(panic_isolation()).await;
    bounded(cancellation_at_completion()).await;
    bounded(supplied_prefill_is_not_duplicated()).await;
}

/// Real deadlock-class behavior has an external ten-second kill/reap deadline.
#[test]
fn prompt_scenarios_are_watchdog_bounded() {
    // Launch only the exact helper to prevent recursion and verify it exits and is reaped.
    let result = watchdog::run(
        Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "prompt_scenarios_child",
            "--nocapture",
        ]),
        "prompt-contracts",
        Duration::from_secs(10),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}

/// Intentionally never return, proving the watchdog is independent of executor progress.
#[test]
#[ignore = "deliberately blocked watchdog self-test"]
fn blocked_watchdog_child() {
    // Park forever intentionally; only the external watchdog should terminate this helper.
    loop {
        std::thread::park();
    }
}

/// A blocked child is killed and reaped at the short self-test deadline.
#[test]
fn watchdog_kills_and_reaps_a_blocked_child() {
    // Require a timeout termination so the watchdog itself cannot silently provide false green
    // evidence.
    let result = watchdog::run(
        Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "blocked_watchdog_child",
        ]),
        "prompt-watchdog-self-test",
        Duration::from_millis(250),
    )
    .unwrap();
    assert!(result.timed_out);
    assert!(!result.status.success());
}
