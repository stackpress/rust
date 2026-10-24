# Prompt options and errors API

Configuration and callback contracts are independent of input ownership. Source:
[prompt/options.rs](../../src/prompt/options.rs). Import from `stackpress_cli::prompt`.

```rust
use stackpress_cli::prompt::{PromptError, PromptOptions, PromptResult, Validation, Validator};

async fn check(value: String) -> PromptResult<Validation> {
    Ok(if value == "Ada" {
        Validation::Valid
    } else {
        Validation::Invalid("Use Ada".into())
    })
}
let options = PromptOptions::new("Name?");
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Fields and limits](#1-fields-and-limits) · [2. Option builders](#2-option-builders) ·
[3. Callback adapters](#3-callback-adapters) · [4. Errors](#4-errors)

## 1. Fields and limits

PromptOptions owns private regex/validator/transformer storage and these public fields. No field
opens stdin, installs signals, or starts tasks.

| Field | Type | Default |
| --- | --- | --- |
| message | String | Supplied question. |
| default | Option<String> | None. |
| required | bool | false. |
| prefill | Prefill | Tab. |
| invalid | InvalidMode | Keep. |
| pattern_error | String | Invalid input. |
| limits | PromptLimits | Finite defaults below. |

PromptLimits is Clone/Copy/Debug/Default with public usize fields: answer_bytes 4,096; error_bytes
1,024; render_bytes 8,192; pending_events 64. Answer bytes bound each default/candidate and
aggregate queued text; pending_events bounds queued edit/Enter events during validation. Render
bytes bound sanitized prompt/error/framing; callbacks own their allocation/blocking policy. Static
checks reject render_bytes <32, pending_events ==0, or oversized message/default/pattern_error
before native acquisition. answer_bytes ==0 permits empty candidates. Runtime rendering still needs
room for actual text and framing. Required-empty rejects before pattern, then exactly one user
validator. Rust regex uses search semantics; use anchors for whole-answer validation.

## 2. Option builders

Builders consume/return PromptOptions. They replace the selected callback/pattern; other options
remain. Transformers affect display only and receive is_final; accepted answers remain
untransformed.

### 2.1. new

`PromptOptions::new(message: impl Into<String>) -> Self`

Create optional Tab/Keep input with default limits.

```rust
let options = PromptOptions::new("Name?");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `message` | `impl Into<String>` | Question; sanitized at render time. |

**Returns** Owned options with defaults above; size validation is deferred to state/driver
construction.

### 2.2. pattern

`pattern(self, source: &str) -> Result<Self, regex::Error>`

Compile a Rust regex before input acquisition.

```rust
let options = options.pattern("^[A-Z][a-z]+$")?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `source` | `&str` | Rust regex syntax/search semantics, not JavaScript regex. |

**Returns** Options or regex syntax/size error; no terminal state changes.

### 2.3. with_pattern

`with_pattern(self, pattern: regex::Regex) -> Self`

Use a precompiled immutable regex.

```rust
let options = options.with_pattern(regex::Regex::new("^Ada$")?);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `pattern` | `regex::Regex` | Consumed compiled matcher. |

**Returns** Options with replaced pattern, without recompiling.

### 2.4. without_pattern

`without_pattern(self) -> Self`

Remove regex validation.

```rust
let options = options.without_pattern();
```

**Returns** Options retaining required and custom checks.

### 2.5. validator

`validator(self, callback: impl Fn(&str) -> PromptResult<Validation> + Send + Sync + 'static) -> Self`

Install a synchronous borrowed-answer validator.

```rust
let options = options.validator(|value| {
    Ok(if value == "Ada" {
        Validation::Valid
    } else {
        Validation::Invalid("Use Ada".into())
    })
});
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `callback` | `impl Fn(&str) -> PromptResult<Validation> + Send + Sync + 'static` | Runs after required/regex; synchronous code may block. |

**Returns** Options with replaced validator. Validation::Invalid permits correction; callback Err
terminates and retains its cause.

### 2.6. async_validator

`async_validator<F, Fut>(self, callback: F) -> Self`

Install a named async validator without caller boxing. F: Fn(String) -> Fut + Send + Sync + 'static;
Fut: Future<Output = PromptResult<Validation>> + Send + 'static.

```rust
let options = options.async_validator(check);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `callback` | `F` | Owns a bounded submitted candidate; future polled locally. |

**Returns** Options with replaced validator. The driver spawns no validation task; cancellation/drop
abandons the future, while blocking callback work remains its own responsibility.

### 2.7. without_validator

`without_validator(self) -> Self`

Remove the custom validator.

```rust
let options = options.without_validator();
```

**Returns** Options retaining required/regex checks; captured owners in the old callback are
dropped.

### 2.8. transform

`transform(self, callback: impl Fn(&str, bool) -> PromptResult<String> + Send + Sync + 'static) -> Self`

Transform live/final display without changing the answer.

```rust
let options = options.transform(|value, is_final| {
    Ok(if is_final {
        "[saved]".into()
    } else {
        "*".repeat(value.chars().count())
    })
});
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `callback` | `impl Fn(&str, bool) -> PromptResult<String> + Send + Sync + 'static` | Display value and final flag; output bounded/sanitized by renderer. |

**Returns** Options with replaced transformer. Callback errors terminate; display masking does not
create secure secret storage.

### 2.9. without_transform

`without_transform(self) -> Self`

Restore direct value/default display.

```rust
let options = options.without_transform();
```

**Returns** Options without transformer; validation/default policy is unchanged.

## 3. Callback adapters

Validation is Clone/Debug/Eq/PartialEq with Valid and Invalid(String). Validator is Sync or Async,
containing boxed Send + Sync functions. Transformer is the public alias `Box<dyn Fn(&str, bool) ->
PromptResult<String> + Send + Sync>`. Ordinary option builders handle boxing; direct Validator
adapters are also public.

### 3.1. Validator::sync

`Validator::sync(callback: impl Fn(&str) -> PromptResult<Validation> + Send + Sync + 'static) -> Self`

Adapt a borrowed synchronous check.

```rust
let validator = Validator::sync(|_| Ok(Validation::Valid));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `callback` | `impl Fn(&str) -> PromptResult<Validation> + Send + Sync + 'static` | One candidate check. |

**Returns** Validator::Sync; callback not run during construction.

### 3.2. Validator::asynchronous

`Validator::asynchronous<F, Fut>(callback: F) -> Self`

Adapt a check owning String; same F/Fut bounds as async_validator.

```rust
let validator = Validator::asynchronous(check);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `callback` | `F` | Fn(String) returning owned Send static validation future. |

**Returns** Validator::Async with internally boxed future; no task is started.

## 4. Errors

PromptResult<T> aliases Result<T, PromptError>. PromptError implements Debug/Display/
std::error::Error; Display omits answer text and arbitrary I/O/callback messages.

| Variant | Meaning |
| --- | --- |
| Aborted | Explicit cancellation, including native Ctrl-C/Esc. |
| Eof | Input ended without acceptance. |
| InvalidAnswer(String) | One explicit/default answer rejected, with bounded validation message. |
| NonInteractive | Native input unavailable without explicit/default answer. |
| Limit(&'static str) | Answer/error/render/queue budget exceeded. |
| Configuration(&'static str) | Bad options, control text, or forged completion event. |
| Io(io::Error) | Native input/output/acquisition/cleanup error; From<io::Error> preserves it. |
| Callback(Box<dyn Error + Send + Sync>) | User validation/transformer failure with retained cause. |

Only Io and Callback expose source(). Validation rejection normally returns editable Invalid, not
PromptError; the native single-answer path converts that rejection to InvalidAnswer.

### 4.1. PromptError::callback

`PromptError::callback(error: impl std::error::Error + Send + Sync + 'static) -> Self`

Preserve a concrete validation/transformer cause.

```rust
let error = PromptError::callback(std::io::Error::other("validation service failed"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `error` | `impl std::error::Error + Send + Sync + 'static` | Owned original source. |

**Returns** A typed Callback error, distinct from Invalid or cancellation.

Related: [prompt guide](../guides/prompts.md), [state](prompt-state.md),
[drivers](prompt-driver.md).
