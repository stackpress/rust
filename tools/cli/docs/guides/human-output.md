# Writing human output

Progress messages should be readable without corrupting a command's JSON stdout. Give Control an
explicit stderr or buffer writer and choose color from that output's properties.

## Format a diagnostic

Use a buffer first to verify exact text. The brand is optional, and `%s` variables are inserted as
literal values rather than interpreted again:

```rust
use stackpress_cli::{ColorMode, Control, OutputEnvironment};

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let control = Control::new("[profiles]", ColorMode::Never);
    let mut output = Vec::new();
    control.success(
        &mut output,
        OutputEnvironment::default(),
        "Loaded %s rows",
        &["3"],
    )?;
    assert_eq!(output, b"[profiles] Loaded 3 rows\n");
    Ok(())
}
```

Control owns the newline and sanitizes control characters/bidi overrides before adding any ANSI.
`success` selects green only when policy allows; Never makes this test deterministic.

## Select the real output

For native stderr, pass `OutputEnvironment::stderr()` and a mutable `std::io::stderr()` writer. Auto
uses color only for a terminal with NO_COLOR absent/empty. Always overrides NO_COLOR and terminal
detection; Never disables generated color. `output`/Level::Output is uncolored in every mode.

CliSinks accepts arbitrary Write objects, so automatic adapter diagnostics cannot infer native TTY
identity and remain plain under Auto. Your explicit human-output calls can supply a measured output
environment. Synchronous writers may block; a write error is returned without starting a new
message.

You now have predictable human output with a separate machine channel. Use [Control
reference](../api/control.md) for formatting, levels, and prompt-option convenience.
