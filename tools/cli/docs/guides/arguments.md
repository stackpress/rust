# Parsing arguments

Operator settings and operation data can use the same flag names. Parse the emit boundary first so
an operation's `--verbose false` cannot accidentally turn on developer diagnostics.

## Check the boundary

This supplied-argv example requires no shell or process reads. Its first verbose flag is operator
metadata; the second is a literal target key:

```rust
use stackpress_cli::Terminal;
use stackpress_lib::ParseLimits;

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let argv = ["-v", "emit", "search", "--verbose", "false", "--limit", "0"].map(String::from);
    let terminal = Terminal::parse(&argv, ParseLimits::default())?;
    assert!(terminal.options().verbose);
    assert_eq!(terminal.expect::<bool>(&["verbose", "v"])?, Some(false));
    assert_eq!(terminal.expect::<u32>(&["limit"])?, Some(0));
    assert_eq!(terminal.command(), "search");
    Ok(())
}
```

`expect` uses literal alias presence, not truthiness. A present wrong type returns an error before
trying later aliases. For a present JSON null, `expect::<Option<String>>` returns Some(None).
Original target tokens remain available through `args()`.

## Supply defaults and bounds

Use TerminalOptions with `parse_with_options` for an application brand/color/default setting.
Explicit operator flags override those defaults. `parse_os` rejects non-UTF-8 with an argument
index; all parse entry points exclude the executable name and bound total operator/event/target
bytes and entries before target parsing.

Before emit, supported flags are `-v/--verbose`, `-f/--force`, and `-b/--bootstrap`. Grouped `-vf`,
`-vfb file.json`, `-bfile.json`, and `--bootstrap=file.json` work. `--` may precede emit. After the
event, target parsing uses [Lib argv grammar](../../../../crates/lib/docs/api/parsers.md), including
its own `--` terminator, coercion, positional keys, and short-flag behavior.

You can now distinguish operator intent from operation data. See [Terminal
reference](../api/terminal.md) for exact parser errors and accessors.
