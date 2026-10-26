# Executable entry API

The app owns the binary and compiled builder. These functions return ExitCode without calling
process::exit. Source: [entry.rs](../../src/entry.rs).

```rust
use stackpress_cli::{CliSinks, ConfigLayers, main_with, main_with_args};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{BuildResult, Server, ServerBuilder};

async fn hello(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"hello")?;
    Ok(Flow::Continue)
}
async fn application(layers: ConfigLayers) -> BuildResult<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.on("hello", hello)?;
    app.build().await
}
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Entry functions](#1-entry-functions) · [2. Exit and disclosure
policy](#2-exit-and-disclosure-policy)

## 1. Entry functions

Both accept `F: FnOnce(ConfigLayers) -> Fut + Send` and `Fut: Future<Output = BuildResult<Server>> +
Send`. The app must install the received layers. Invalid argv/config never invokes F; valid
argv/config consumes F exactly once.

### 1.1. main_with

`async fn main_with<F, Fut>(build: F) -> std::process::ExitCode`

Acquire process argv/stdout/stderr once and run the application pipeline.

```rust
async fn entry() -> std::process::ExitCode {
    main_with(application).await
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `build` | `F` | Async application composition with selected compiled plugins. |

**Returns** ExitCode from the policy below. No prompt or plugin discovery runs implicitly.
Bootstrap acquisition runs on the Tokio blocking pool; synchronous sink IO and application
callbacks may still block. Cancelling the async entry drops build/run work, but cannot abort
an in-flight native file operation or let invalid data invoke the builder.

### 1.2. main_with_args

`async fn main_with_args<F, Fut>(args: &[OsString], sinks: &mut CliSinks<'_>, build: F) -> std::process::ExitCode`

Use the identical pipeline with supplied arguments and sinks.

```rust
let argv = ["emit".into(), "hello".into()];
let (mut stdout, mut stderr) = (Vec::<u8>::new(), Vec::<u8>::new());
let exit = main_with_args(
    &argv,
    &mut CliSinks::new(&mut stdout, &mut stderr),
    application,
)
.await;
assert_eq!(exit, std::process::ExitCode::SUCCESS);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `args` | `&[OsString]` | OS tokens excluding executable name; non-UTF-8 is rejected. |
| `sinks` | `&mut CliSinks<'_>` | Exclusive borrowed machine/human writers. |
| `build` | `F` | FnOnce async builder, after parsing/config. |

**Returns** ExitCode using the same policy as main_with. The explicit bootstrap path is opened
exactly as supplied, read up to default max_bytes plus one detection byte, checked as UTF-8 and JSON
object under label cli-bootstrap. No path means no external layers. File contents are data, never
Rust/plugin imports. Only regular files are accepted, following input symlinks, with metadata
checks before open and on the acquired handle. Unsupported devices/FIFOs reject before waiting
for a writer. The cooperative-path policy is not race-free against a concurrent replacement;
a byte bound and blocking worker do not make stalled OS reads cancellable.

## 2. Exit and disclosure policy

The pipeline parses once, loads at most one bootstrap file, builds once, and calls run once. Its
stdout contract is one eligible JSON envelope/newline; usage/config/startup errors leave stdout
untouched. All automatic human diagnostics go to stderr through sanitized Control output.

| Code | Result |
| --- | --- |
| 0 | Final public success, including deliberate recovery and empty successful Abort. |
| 1 | Unrecovered execution/outer-run error, JSON conversion, stdout write/flush, foreign finalization, or stderr write/flush failure. |
| 2 | Argument usage/data error, bootstrap read/encoding/JSON error, ordinary startup/build failure. |
| 130 | Typed PromptError::Aborted in a startup or unrecovered execution cause chain. |

Output failure wins over cancellation; successful public recovery wins over a retained cancellation.
Detection checks concrete PromptError::Aborted through at most 33 error nodes, never message/status
heuristics. EOF/InvalidAnswer are not cancellation. run returns 0/1; this executable seam adds 130.
Non-verbose pre-run errors are concise labels; only parsed verbose selects developer causes. A bad
argv cannot request verbose via a second parse. Neither entry catches arbitrary panics. Auto color
remains plain for generic sinks.

Related: [application guide](../guides/application-binary.md), [execution](execution.md).
