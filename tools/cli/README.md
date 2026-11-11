# stackpress-cli

Build an application-owned command-line executable around Stackpress operations. Argument parsing, formatting and prompts work independently; the crate supplies adapter primitives rather than a stock binary.

```rust
use stackpress_cli::Terminal;
use stackpress_lib::ParseLimits;
let args = ["emit", "health"].map(String::from);
let terminal = Terminal::parse(&args, ParseLimits::default())?;
let _ = terminal;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Applications register their compiled plugins and pass the host to `main_with`. Terminal IO is optional through the default `tty` feature.

Licence: [Apache-2.0](LICENSE).
