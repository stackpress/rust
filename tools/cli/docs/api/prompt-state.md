# Prompt state and rendering API

Pure serialized editing transitions, with separately callable display rendering. Sources:
[prompt/state.rs](../../src/prompt/state.rs), [prompt/render.rs](../../src/prompt/render.rs).

```rust
use stackpress_cli::prompt::{
    PromptAction, PromptEvent, PromptOptions, PromptState, PromptStatus, Validation,
};
let options = PromptOptions::new("Name?");
let mut state = PromptState::new(&options)?;
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. State vocabulary](#1-state-vocabulary) · [2. State operations](#2-state-operations) ·
[3. Driver render options](#3-driver-render-options)

## 1. State vocabulary

Prefill is Tab (default preview copied by Tab/Enter) or Editable (default begins as editing text).
InvalidMode is Keep (retain rejected candidate) or Clear (clear candidate and default). Both enums
are Clone/Copy/Debug/Eq/PartialEq. PromptStatus has Editing, Validating(u64), Complete, Aborted,
Ended and is Clone/Copy/Debug/Eq/PartialEq. PromptState is Debug, non-Clone, with private fields.

PromptEvent is Debug/Eq/PartialEq: Text(String), Backspace, Tab, Enter, Cancel, Eof, and Validated {
id: u64, result: Validation }. Text must contain no control characters. Pure-state completion events
are for driver authors; public prompt rejects them from external input.

PromptAction is Debug/Eq/PartialEq: None, Render, Validate { id: u64, value: String },
Complete(String), Aborted, Eof. It tells a driver what work to perform; pure apply never invokes
async validation or I/O. Use the same options for every transition/render.

## 2. State operations

Edits/repeated Enter are ignored while validating by the pure machine; prompt's driver separately
queues bounded edits. Cancel/Eof terminate, stale completions are ignored, and terminal states
cannot be revived. Backspace removes one Unicode scalar, not necessarily one grapheme.

### 2.1. new

`PromptState::new(options: &PromptOptions) -> PromptResult<Self>`

Check options and construct initial editing state.

```rust
let state = PromptState::new(&options)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `options` | `&PromptOptions` | Fixed policy/default/budgets. |

**Returns** Editing state or invalid-options/text-budget error; no callbacks or I/O. Tab starts
current empty; Editable starts current with default.

### 2.2. current

`current(&self) -> &str`

Borrow editable/final text excluding Tab preview.

```rust
assert_eq!(state.current(), "");
```

**Returns** Borrowed current text.

### 2.3. default_value

`default_value(&self) -> &str`

Read the remaining live preview.

```rust
assert_eq!(state.default_value(), "");
```

**Returns** Borrowed preview; it may differ from configured default after edits/rejection.

### 2.4. error_message

`error_message(&self) -> &str`

Read the most recent rejection.

```rust
assert_eq!(state.error_message(), "");
```

**Returns** Borrowed checked error text, cleared on successful edit or acceptance.

### 2.5. status

`status(&self) -> PromptStatus`

Inspect editing/validation/termination.

```rust
assert_eq!(state.status(), PromptStatus::Editing);
```

**Returns** A copied status/token without exposing mutable text.

### 2.6. apply

`apply(&mut self, options: &PromptOptions, event: PromptEvent) -> PromptResult<PromptAction>`

Apply one bounded edit or completion atomically.

```rust
state.apply(&options, PromptEvent::Text("Ada😀".into()))?;
state.apply(&options, PromptEvent::Backspace)?;
let PromptAction::Validate { id, value } = state.apply(&options, PromptEvent::Enter)? else {
    panic!("Enter must submit");
};
assert_eq!(value, "Ada");
assert_eq!(
    state.apply(
        &options,
        PromptEvent::Validated {
            id,
            result: Validation::Valid
        }
    )?,
    PromptAction::Complete("Ada".into())
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `options` | `&PromptOptions` | Same policy used at creation. |
| `event` | `PromptEvent` | Typed edit/cancel/EOF or trusted matching completion. |

**Returns** A requested action or control-text/answer/error-budget/token-exhaustion error without
partial failed-check mutation. Enter copies remaining default when current is empty and reserves a
checked increasing token. Matching Valid completes; Invalid checks its message budget, applies
Keep/Clear, and reopens editing. Cancel/Eof invalidate pending completion.

### 2.7. render

`render(&self, options: &PromptOptions) -> PromptResult<PromptRender>`

Prepare sanitized display text without writing.

```rust
let render = state.render(&options)?;
assert_eq!(render.prompt, "? Name?");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `options` | `&PromptOptions` | Same policy including optional display transformation. |

**Returns** PromptRender or callback/render-limit error before output. PromptRender is
Debug/Eq/PartialEq with public prompt: String and error: String, containing no newline/generated
ANSI. Accepted empty answers do not restore a default preview; transformation/sanitization affect
display only.

## 3. Driver render options

RenderOptions is Clone/Copy/Debug/Default with `is_terminal: bool` false, `clear_on_done: bool`
false, `columns: Option<usize>` None. This selects output properties, independently of input. Plain
output emits initial/final prompt and validation errors without cursor escapes; clear_on_done does
not erase that transcript.

Terminal output uses injected nonzero columns or, with tty, captured native dimensions. Without tty,
terminal rendering needs an injected width. Native frames taller than captured height fail before
writing. Width uses Unicode non-CJK measurement at scalar boundaries, so a grapheme can split. A
scalar wider than the whole row, zero width/height, excessive framing, callback expansion, or
sanitization expansion returns an explicit configuration/limit error. Resize or external writes
during a prompt require cancellation and a new output session. Driver-owned render framing is
private; use [prompt](prompt-driver.md) to perform it.

Related: [options](prompt-options.md), [prompt guide](../guides/prompts.md).
