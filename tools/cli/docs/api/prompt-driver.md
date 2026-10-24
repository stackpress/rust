# Prompt driver API

Explicit async input, cancellation, and output orchestration; no implicit host dispatch or global
pipe reader. Source: [prompt/driver.rs](../../src/prompt/driver.rs).

```rust
use stackpress_cli::prompt::{
    BufferedInput, PromptCancellation, PromptEvent, PromptInput, PromptOptions, RenderOptions,
    prompt, prompt_native,
};
let options = PromptOptions::new("Name?");
let cancellation = PromptCancellation::new();
let mut output = Vec::<u8>::new();
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Cancellation](#1-cancellation) · [2. Input sources](#2-input-sources) · [3. Prompt
functions](#3-prompt-functions)

## 1. Cancellation

PromptCancellation is Clone/Debug/Default. Clone shares the named watch signal; new/default creates
an independent signal with no process-wide signal handler. Cancellation is permanent.

### 1.1. new

`PromptCancellation::new() -> Self`

Start uncancelled explicit work.

```rust
let cancellation = PromptCancellation::new();
```

**Returns** An independent retained signal, even without subscribers.

### 1.2. cancel

`cancel(&self)`

Cancel and wake all current waiters.

```rust
cancellation.cancel();
assert!(cancellation.is_cancelled());
```

**Returns** Unit; cancellation stays observable by late subscribers.

### 1.3. is_cancelled

`is_cancelled(&self) -> bool`

Inspect cancellation before acquiring input/calling user code.

```rust
assert!(!cancellation.is_cancelled());
```

**Returns** True once cancel has been called.

### 1.4. cancelled

`async fn cancelled(&self)`

Await cancellation without a check/subscribe race.

```rust
cancellation.cancel();
cancellation.cancelled().await;
```

**Returns** Unit after cancellation; an uncancelled signal waits. Dropping this future cancels only
the wait.

## 2. Input sources

PromptInput: Send is the substitution boundary. A bounded Tokio mpsc::Receiver<PromptEvent> already
implements it. next must be cancellation-safe: dropping its future must not lose an already consumed
event, block the executor, or leave an unjoinable input worker.

### 2.1. PromptInput::next

`next(&mut self) -> impl Future<Output = io::Result<Option<PromptEvent>>> + Send`

Read one event from your explicitly owned source.

```rust
let mut input = BufferedInput::new([PromptEvent::Enter].into_iter());
assert_eq!(input.next().await?, Some(PromptEvent::Enter));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `self` | `&mut Self` | Exclusive event-source access. |

**Returns** A Send future yielding Some event, None for EOF, or input I/O error. The driver may drop
next during cancellation or validation selection.

### 2.2. BufferedInput::new

`BufferedInput::new(events: I) -> Self where I: Iterator<Item = PromptEvent> + Send`

Wrap finite supplied events lazily.

```rust
let input =
    BufferedInput::new([PromptEvent::Text("Ada".into()), PromptEvent::Enter].into_iter());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `events` | `I` | Owned Send iterator; caller owns workload policy. |

**Returns** BufferedInput<I> without I/O, thread, or whole-event-list copy; exhaustion yields None.

### 2.3. BufferedInput::lines

`BufferedInput::lines(text: &str) -> BufferedInput<impl Iterator<Item = PromptEvent> + Send + '_>`

Map already supplied UTF-8 pipe-style text to events.

```rust
let mut input = BufferedInput::lines("Ada\n");
assert_eq!(input.next().await?, Some(PromptEvent::Text("A".into())));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `&str` | Borrowed caller-supplied/bounded text; never reads stdin. |

**Returns** A lazy borrowed source: CR/LF submit, CRLF coalesces to one Enter, Ctrl-C cancels,
Tab/backspace map to named events. Other controls become Text and are rejected by state. EOF does
not implicitly submit.

## 3. Prompt functions

Built-in required/regex checks precede one custom validation. During pending validation, edits/Enter
queue under byte/event limits and replay only after rejection. Cancellation wins and drops
validation; EOF after submission waits for that validation, then ends if rejected. Inputs cannot
forge Validated completions. Writers and synchronous callbacks can block; arbitrary code is not made
cancellable.

### 3.1. prompt

`async fn prompt<I: PromptInput>(options: &PromptOptions, input: &mut I, writer: &mut (dyn Write + Send), cancellation: &PromptCancellation, render: RenderOptions) -> PromptResult<String>`

Drive supplied events with explicit human output.

```rust
let mut input = BufferedInput::lines("Ada\n");
let answer = prompt(
    &options,
    &mut input,
    &mut output,
    &cancellation,
    RenderOptions::default(),
)
.await?;
assert_eq!(answer, "Ada");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `options` | `&PromptOptions` | Question/default/validation/limits. |
| `input` | `&mut I` | Cancellation-safe event source. |
| `writer` | `&mut (dyn Write + Send)` | Chosen human sink, normally stderr. |
| `cancellation` | `&PromptCancellation` | Permanent signal; pre-cancel performs no callbacks/writes. |
| `render` | `RenderOptions` | Chosen output properties/geometry. |

**Returns** The untransformed accepted String only after successful final rendering, or
Aborted/Eof/Io/Callback/Limit/Configuration. Invalid input stays editable. Output failure terminates
without retry/new prompt. Drop releases borrows/local validator; if caller owns native input, it
must also close/drop that owner.

### 3.2. prompt_native

`async fn prompt_native(options: &PromptOptions, answer: Option<&str>, writer: &mut (dyn Write + Send), cancellation: &PromptCancellation, render: RenderOptions) -> PromptResult<String>`

Select explicit answer, noninteractive default, or owned native TTY driver.

```rust
let answer = prompt_native(
    &options,
    Some("Ada"),
    &mut output,
    &cancellation,
    RenderOptions::default(),
)
.await?;
assert_eq!(answer, "Ada");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `options` | `&PromptOptions` | Same validation/render policy. |
| `answer` | `Option<&str>` | Explicit complete candidate; Some empty string overrides all prefill. |
| `writer` | `&mut (dyn Write + Send)` | Human output; no implicit global output. |
| `cancellation` | `&PromptCancellation` | Explicit cancellation signal. |
| `render` | `RenderOptions` | Chosen output, not stdin identity. |

**Returns** Accepted original String or prompt/native acquisition/cleanup error. Explicit answers
bypass stdin but still validate/render; rejection returns InvalidAnswer(message). Without TTY input
or with tty disabled, default is submitted or NonInteractive returns immediately. Native input owns
one TtyInput; successful return requires close success. A primary prompt error wins if cleanup also
fails, with best-effort restoration on Drop. No blocked pipe worker is started.

Related: [prompt guide](../guides/prompts.md), [native guide](../guides/native-prompts.md),
[TTY](tty.md).
