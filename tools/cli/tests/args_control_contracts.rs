//! Supplied argv and exact human-output contracts; no global process-stream mutation.

use std::error::Error as _;
use std::io::{self, Write};

use stackpress_cli::{
    ArgumentError, ColorMode, Control, Level, OutputEnvironment, Terminal, TerminalOptions,
};
use stackpress_lib::ParseLimits;

/// Build the same supplied argv an application wrapper would pass, without its executable name.
fn parse(args: &[&str]) -> Result<Terminal, ArgumentError> {
    // Supply argv directly so the fixture cannot accidentally exercise shell splitting.
    Terminal::parse(
        &args.iter().map(|arg| (*arg).into()).collect::<Vec<_>>(),
        ParseLimits::default(),
    )
}

/// Operator options stop at emit/event; target grammar is exactly the existing lib parser.
#[test]
fn operator_boundary_preserves_target_data_and_original_tokens() {
    // Mix operators with identically named target keys, literal spaces and a target terminator.
    let terminal = parse(&[
        "-vf",
        "-b",
        "config.json",
        "emit",
        "profile-search",
        "--verbose",
        "false",
        "--force",
        "0",
        "--bootstrap",
        "target data & =",
        "--json",
        "{\"enabled\":false}",
        "-xy",
        "--",
        "--positional",
        "two words",
    ])
    .unwrap();
    assert_eq!(terminal.command(), "profile-search");
    assert!(terminal.options().verbose && terminal.options().force);
    assert_eq!(
        terminal.options().bootstrap.as_deref(),
        Some(std::path::Path::new("config.json"))
    );
    assert_eq!(terminal.args()[5], "target data & =");
    assert_eq!(terminal.expect::<bool>(&["verbose"]).unwrap(), Some(false));
    assert_eq!(terminal.expect::<i64>(&["force"]).unwrap(), Some(0));
    assert_eq!(
        terminal
            .expect::<String>(&["bootstrap"])
            .unwrap()
            .as_deref(),
        Some("target data & =")
    );
    assert_eq!(
        terminal.data().path_as::<bool>("json.enabled").unwrap(),
        Some(false)
    );
    assert_eq!(terminal.expect::<bool>(&["x"]).unwrap(), Some(true));
    assert_eq!(
        terminal.expect::<String>(&["0"]).unwrap().as_deref(),
        Some("--positional")
    );
    assert_eq!(
        terminal.expect::<String>(&["1"]).unwrap().as_deref(),
        Some("two words")
    );
    // Target short bootstrap is a true flag under lib grammar, never an operator config path.
    assert!(
        parse(&["emit", "event", "-b", "file"])
            .unwrap()
            .options()
            .bootstrap
            .is_none()
    );
}

/// Presence wins over truthiness, including a null decoded as Option<T>.
#[test]
fn alias_presence_and_type_errors_are_explicit() {
    // Arrange false, zero, empty and null before a later truthy alias to expose fallback mistakes.
    let terminal = parse(&[
        "emit", "event", "--f", "false", "--n", "0", "--e=", "--nil", "null", "--later", "true",
    ])
    .unwrap();
    assert_eq!(
        terminal.expect::<bool>(&["missing", "f", "later"]).unwrap(),
        Some(false)
    );
    assert_eq!(terminal.expect::<i64>(&["n", "later"]).unwrap(), Some(0));
    assert_eq!(
        terminal.expect::<String>(&["e", "later"]).unwrap(),
        Some(String::new())
    );
    assert_eq!(
        terminal
            .expect::<Option<String>>(&["nil", "later"])
            .unwrap(),
        Some(None)
    );
    assert_eq!(terminal.expect::<bool>(&[]).unwrap(), None);
    assert_eq!(terminal.expect::<bool>(&["absent"]).unwrap(), None);
    let error = terminal.expect::<String>(&["f", "later"]).unwrap_err();
    assert!(error.source().is_some());
    assert!(!error.to_string().contains("false"));
}

/// Grouped short switches and explicit path spellings remain deterministic.
#[test]
fn supported_operator_forms_and_presentation_defaults() {
    // Exercise each documented short/long operator spelling against the same parsed settings.
    for args in [
        vec!["-vfb", "file.json"],
        vec!["-vfbfile.json"],
        vec!["--verbose", "--force", "--bootstrap=file.json"],
    ] {
        let mut all = args;
        all.extend(["emit", "event"]);
        let terminal = parse(&all).unwrap();
        assert!(terminal.options().verbose && terminal.options().force);
        assert_eq!(
            terminal.options().bootstrap.as_deref(),
            Some(std::path::Path::new("file.json"))
        );
    }
    assert_eq!(parse(&["--", "emit", "event"]).unwrap().command(), "event");
    let terminal = Terminal::parse_with_options(
        &["emit".into(), "event".into()],
        ParseLimits::default(),
        TerminalOptions {
            brand: "[test]".into(),
            color: ColorMode::Never,
            ..TerminalOptions::default()
        },
    )
    .unwrap();
    assert_eq!(terminal.control().brand(), "[test]");
}

/// Malformed options, event selectors and dangling target long flags fail without partial state.
#[test]
fn usage_and_budget_failures() {
    // Cover invalid operator shapes first, then the exact byte/node acceptance boundary.
    for args in [
        vec![],
        vec!["emit"],
        vec!["serve"],
        vec!["--unknown", "emit", "e"],
        vec!["-"],
        vec!["-vx", "emit", "e"],
        vec!["-b"],
        vec!["-b", "--force"],
        vec!["--bootstrap="],
        vec!["--bootstrap", ""],
        vec!["emit", ""],
        vec!["emit", "\rprivate"],
        vec!["emit", "   "],
        vec!["--", "-v", "emit", "e"],
    ] {
        let error = parse(&args).unwrap_err();
        assert!(
            matches!(error, ArgumentError::Usage(_)),
            "{args:?}: {error}"
        );
        assert!(error.source().is_none());
    }
    assert!(matches!(
        parse(&["emit", "event", "--dangling"]),
        Err(ArgumentError::Data(_))
    ));
    let args = ["emit".into(), "e".into()];
    assert!(
        Terminal::parse(
            &args,
            ParseLimits {
                max_bytes: 5,
                max_entries: 2,
                ..ParseLimits::default()
            }
        )
        .is_ok()
    );
    for limits in [
        ParseLimits {
            max_bytes: 4,
            ..ParseLimits::default()
        },
        ParseLimits {
            max_entries: 1,
            ..ParseLimits::default()
        },
    ] {
        assert!(Terminal::parse(&args, limits).is_err());
    }
}

/// Native invalid encodings are never silently replaced with a different event/value.
#[cfg(unix)]
#[test]
fn non_utf8_is_a_typed_indexed_error() {
    // Construct native invalid bytes without lossy conversion and require their original index.
    use std::os::unix::ffi::OsStringExt;
    let args = [
        "emit".into(),
        "event".into(),
        std::ffi::OsString::from_vec(vec![0xff]),
    ];
    let error = Terminal::parse_os(&args, ParseLimits::default()).unwrap_err();
    assert!(matches!(error, ArgumentError::NonUtf8(2)));
    assert_eq!(error.to_string(), "argument 2 is not UTF-8");
    assert!(error.source().is_none());
    assert!(Terminal::parse_os(&args[..2], ParseLimits::default()).is_ok());
    assert!(
        Terminal::parse_os(
            &args,
            ParseLimits {
                max_bytes: 0,
                ..ParseLimits::default()
            }
        )
        .is_err()
    );
}

/// Every level matches upstream ANSI colors, with explicit auto/no-color overrides.
#[test]
fn levels_color_policy_substitution_and_injection() {
    // Use injected output snapshots to make TTY, NO_COLOR and ANSI behavior deterministic.
    let tty = OutputEnvironment {
        is_terminal: true,
        no_color: false,
    };
    for (level, code) in [
        (Level::Error, 31),
        (Level::Info, 34),
        (Level::Success, 32),
        (Level::System, 35),
        (Level::Warning, 33),
    ] {
        let control = Control::new("[test]", ColorMode::Auto);
        assert_eq!(
            control.format(level, "Hello %s", &["World"], tty),
            format!("\x1b[{code}m[test] Hello World\x1b[0m\n")
        );
        assert_eq!(
            control.format(level, "plain", &[], OutputEnvironment::default()),
            "[test] plain\n"
        );
        assert_eq!(
            control.format(
                level,
                "plain",
                &[],
                OutputEnvironment {
                    no_color: true,
                    ..tty
                }
            ),
            "[test] plain\n"
        );
    }
    assert_eq!(
        Control::new("", ColorMode::Always).format(
            Level::Error,
            "x",
            &[],
            OutputEnvironment::default()
        ),
        "\x1b[31mx\x1b[0m\n"
    );
    let plain = Control::new("  ", ColorMode::Never);
    assert_eq!(
        plain.format(Level::Info, "%s -> %s", &["alpha", "beta", "ignored"], tty),
        "alpha -> beta\n"
    );
    assert_eq!(
        plain.format(Level::Output, "%s -> %s", &["%s", "beta"], tty),
        "%s -> beta\n"
    );
    assert_eq!(
        plain.format(Level::Output, "%s -> %s", &["alpha"], tty),
        "alpha -> %s\n"
    );
    let injected = Control::new("\x1b[2J", ColorMode::Never).format(
        Level::Error,
        "\r%s\u{009b}%s",
        &["\x1b]0;title\x07", "\u{202e}"],
        tty,
    );
    assert!(!injected[..injected.len() - 1].chars().any(char::is_control));
    assert!(!injected.contains('\u{202e}'));
    // Detection reads only the native environment; injected snapshots prove the policy branches.
    let _ = OutputEnvironment::stderr();
}

/// A controlled partial writer failure is surfaced without starting a second diagnostic.
struct BrokenWriter;
impl Write for BrokenWriter {
    /// Fail before accepting bytes.
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        // A permanently broken sink must propagate its OS kind instead of accepting diagnostic
        // bytes.
        Err(io::ErrorKind::BrokenPipe.into())
    }
    /// Flush does not hide write failure.
    fn flush(&mut self) -> io::Result<()> {
        // This fixture fails writes only, keeping the source of failure unambiguous.
        Ok(())
    }
}

/// Convenience methods route to the chosen sink and prompt wiring creates no input owner.
#[test]
fn writer_helpers_and_prompt_forwarding() {
    // Send every level to one buffer, then test a broken sink and default-presence prompt wiring.
    let control = Control::new("[test]", ColorMode::Never);
    let environment = OutputEnvironment::default();
    let mut output = Vec::new();
    control.output(&mut output, environment, "o", &[]).unwrap();
    control.error(&mut output, environment, "e", &[]).unwrap();
    control.info(&mut output, environment, "i", &[]).unwrap();
    control.success(&mut output, environment, "s", &[]).unwrap();
    control.system(&mut output, environment, "y", &[]).unwrap();
    control.warning(&mut output, environment, "w", &[]).unwrap();
    assert_eq!(
        String::from_utf8(output).unwrap(),
        "[test] o\n[test] e\n[test] i\n[test] s\n[test] y\n[test] w\n"
    );
    assert_eq!(
        control
            .output(&mut BrokenWriter, environment, "x", &[])
            .unwrap_err()
            .kind(),
        io::ErrorKind::BrokenPipe
    );
    let required = control.input("Name?", None);
    assert!(required.required);
    assert!(required.default.is_none());
    let default = control.input("Name?", Some(""));
    assert!(!default.required);
    assert_eq!(default.default.as_deref(), Some(""));
}
