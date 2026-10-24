# Native TTY API

Available only with feature `tty` (enabled by default). This is native stdin ownership, not a pipe
reader. Prefer prompt_native unless writing a driver. Source: [tty.rs](../../src/tty.rs).

```rust
use stackpress_cli::prompt::{PromptCancellation, PromptInput};
use stackpress_cli::tty::{TtyInput, TtySession};
use std::time::Duration;
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. TtySession](#1-ttysession) · [2. TtyInput](#2-ttyinput)

## 1. TtySession

Non-Clone synchronous raw-mode guard and one process-wide framework reader lease. Explicit close
reports errors; Drop restores best effort without panic. Existing raw mode is preserved. Native
examples require terminal stdin and are compile-checked only.

### 1.1. acquire

`TtySession::acquire() -> io::Result<Self>`

Acquire exclusive framework stdin ownership.

```rust
fn owned_session() -> std::io::Result<()> {
    let mut session = TtySession::acquire()?;
    session.close()
}
```

**Returns** An owned raw-mode session or Unsupported for non-TTY, AlreadyExists for another
framework lease, or OS query/setup error. Failed setup releases provisional ownership via the guard;
failed restoration retains the lease rather than advertising availability.

### 1.2. poll

`poll(&mut self, timeout: Duration) -> io::Result<Option<crossterm::event::Event>>`

Poll synchronously under the exclusive reader lease.

```rust
fn poll_worker() -> std::io::Result<()> {
    let mut session = TtySession::acquire()?;
    let _event = session.poll(Duration::from_millis(20))?;
    session.close()
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `timeout` | `Duration` | Timeout for native crossterm poll, not arbitrary competing calls. |

**Returns** Some native event, None on no readiness, or native/NotConnected error for closed use.
Run this on an owned input worker, not an async executor thread. Competing direct crossterm
internals and OS stalls are outside the timeout guarantee.

### 1.3. close

`close(&mut self) -> io::Result<()>`

Restore this guard's raw transition and release its lease.

```rust
fn close_session() -> std::io::Result<()> {
    let mut session = TtySession::acquire()?;
    session.close()?;
    session.close()
}
```

**Returns** Unit; repeated successful close is idempotent. OS restoration failure is returned and
keeps restoration flags for Drop's final attempt. Drop cannot guarantee OS success.

## 2. TtyInput

Non-Clone async PromptInput owning one raw guard, one named worker, and a 32-event native queue. The
worker only polls/maps events; it never runs application validation or writers. Drop requests stop
and synchronously joins, with a 20-ms poll/backpressure interval under exclusive-reader assumptions;
OS stalls/direct competing crossterm calls are outside that bound.

Keys map Enter/Tab/Backspace, Ctrl-C or Esc to Cancel, Ctrl-D to Eof, and printable scalars to Text.
Release notifications, other control keys, mouse/focus/resize, and unsupported keys are ignored.
Ctrl-C also signals PromptCancellation independent of queue backpressure.

### 2.1. acquire

`TtyInput::acquire(cancellation: PromptCancellation) -> io::Result<Self>`

Acquire stdin and start one controlled polling worker.

```rust
async fn owned_input() -> std::io::Result<()> {
    let mut input = TtyInput::acquire(PromptCancellation::new())?;
    input.close().await
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `cancellation` | `PromptCancellation` | Shared signal notified by native Ctrl-C; prompt driver also observes it. |

**Returns** TtyInput or session acquisition/OS/thread-spawn error. Failed spawn drops the
provisional guard; no detached pipe reader is created. Programmatic cancellation should be paired
with prompt/owner cleanup; explicit close stops this worker.

### 2.2. buffered_events

`buffered_events(&self) -> usize`

Inspect current async native queue length without reading stdin.

```rust
async fn inspect_queue() -> std::io::Result<()> {
    let mut input = TtyInput::acquire(PromptCancellation::new())?;
    assert!(input.buffered_events() <= 32);
    input.close().await
}
```

**Returns** A diagnostic snapshot of queued events, not a readiness promise; the worker can enqueue
immediately afterward.

### 2.3. close

`async fn close(&mut self) -> io::Result<()>`

Signal stop, close backpressure, await completion, then join this exact worker.

```rust
async fn close_input() -> std::io::Result<()> {
    let mut input = TtyInput::acquire(PromptCancellation::new())?;
    input.close().await?;
    input.close().await
}
```

**Returns** Unit or worker panic/raw-mode restoration error from the worker result. Native polling
errors are forwarded through next. Repeated close is idempotent. Dropping a pending close still
stops/joins via Drop. Drop cannot report cleanup errors; use explicit close when evidence is
required.

### 2.4. PromptInput::next

`async fn next(&mut self) -> io::Result<Option<PromptEvent>>`

Receive one queued native event without blocking the executor.

```rust
async fn read_input() -> std::io::Result<()> {
    let mut input = TtyInput::acquire(PromptCancellation::new())?;
    let _event = input.next().await?;
    input.close().await
}
```

**Returns** Some event, None after channel exhaustion, or original forwarded input I/O error.
Receive is cancellation-safe; borrowing it does not transfer input-owner cleanup.

Related: [native prompts](../guides/native-prompts.md), [driver ownership](prompt-driver.md).
