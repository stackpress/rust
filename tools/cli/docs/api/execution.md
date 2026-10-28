# Command execution API

One top-level invocation over parsed input and supplied sinks. Source:
[execution.rs](../../src/execution.rs). No process argument/config read happens here.

```rust
use stackpress_cli::{CliSinks, Terminal, run};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;

async fn hello(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"hello")?;
    Ok(Flow::Continue)
}
let mut builder = ServerBuilder::new();
builder.on("hello", hello)?;
let app = builder.build().await?;
let (mut stdout, mut stderr) = (Vec::<u8>::new(), Vec::<u8>::new());
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Data types](#1-data-types) · [2. Execution](#2-execution) · [3. Output
policy](#3-output-policy)

## 1. Data types

CliSinks<'a> has public `stdout: &'a mut (dyn Write + Send)` and `stderr: &'a mut (dyn Write +
Send)`. stdout is machine JSON; stderr is human output. Native identity cannot be inferred from
generic Write, so Auto adapter diagnostics stay plain.

CliContext (Debug) has public `verbose: bool`, `force: bool`, `config: Option<PathBuf>`, `brand:
String`. It is trusted adapter metadata available through `inv.adapter_context::<CliContext>()`,
inherited by nested calls, and never inserted into Request.data. force is intent, not implicit
authorization; config is the selected path, not proof that run loaded it. No I/O handles enter this
value.

CliRun (Debug) has public `exit_code: u8` (0/1) and `report: HandleReport`. [Executable
entry](entry.md) adds usage/config/startup policy and typed cancellation mapping to 130.

### 1.1. CliSinks::new

`CliSinks::new(stdout: &'a mut (dyn Write + Send), stderr: &'a mut (dyn Write + Send)) -> Self`

Borrow both output channels without global locks.

```rust
let sinks = CliSinks::new(&mut stdout, &mut stderr);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `stdout` | `&'a mut (dyn Write + Send)` | Machine-output sink. |
| `stderr` | `&'a mut (dyn Write + Send)` | Human-output sink. |

**Returns** A pair of exclusive writer borrows with no cloning, stdin ownership, or background I/O.

## 2. Execution

run moves Terminal data under its selected limits into one Request with URL / and MIME
terminal/arguments, then calls Server::handle_with_options once. It does not load bootstrap files or
build the host. Output follows lifecycle completion and cannot run hooks again.

### 2.1. run

`async fn run(app: &Server, terminal: Terminal, sinks: &mut CliSinks<'_>) -> Result<CliRun>`

Execute one parsed event and deliver only the public envelope to stdout.

```rust
let args = ["emit", "hello"].map(String::from);
let terminal = Terminal::parse(&args, Default::default())?;
let result = run(&app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr)).await?;
assert_eq!(result.exit_code, 0);
assert_eq!(
    serde_json::from_slice::<serde_json::Value>(&stdout)?["results"],
    "hello"
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `app` | `&Server` | Already built selected application. |
| `terminal` | `Terminal` | Consumed single parsed selector/data/operator settings. |
| `sinks` | `&mut CliSinks<'_>` | Exclusive caller-owned machine/human writers. |

**Returns** CliRun for normal lifecycle/conversion/write/diagnostic failures: exit_code is 0 only
for final public success with no terminal/delivery failure, otherwise 1. Outer Err means request
setup or host reporting could not start; no fabricated report/lifecycle retry. Panics propagate and
borrowed sinks are never detached.

## 3. Output policy

Open output is fully serialized by Response::try_to_json under final owning limits, then gets one
newline before a private guard consumes the native attempt. Text/Bytes also become public envelope
results here; this differs from HTTP raw representation. Stopped suppresses stdout. Foreign
Sent/Attempted/Failed states are Output failures with no retry. Conversion failure writes no
fallback JSON. write_all/flush may fail after partial stdout; neither stdout nor hooks repeat.

Stderr is attempted when verbose or public execution fails. Non-verbose failure gives concise status
text. Verbose explicitly formats retained causes and, for a directly applied response error without
stage failure, its diagnostic. A failed stderr write/flush records terminal Output failure even
after stdout succeeded. Control formatting sanitizes diagnostics. Caller-owned synchronous
writers/callbacks may block; arbitrary handler writes remain outside adapter authority.

Related: [testing guide](../guides/testing.md), [Server
reports](../../../../crates/server/server/docs/api/reports.md).
