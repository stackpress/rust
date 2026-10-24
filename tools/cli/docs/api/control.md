# Control and color API

Explicit human formatting and supplied writers; machine envelopes never use Control. Source:
[control.rs](../../src/control.rs). Control is Debug and keeps its fields private.

```rust
use stackpress_cli::{ColorMode, Control, Level, OutputEnvironment};
let control = Control::new("[app]", ColorMode::Never);
let environment = OutputEnvironment::default();
let mut output = Vec::<u8>::new();
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Presentation types](#1-presentation-types) · [2. Formatting](#2-formatting)

## 1. Presentation types

ColorMode is Clone/Copy/Debug/Default/Eq/PartialEq: Auto (default), Always, Never. Auto requires a
terminal output and no nonempty NO_COLOR; Always explicitly overrides both. Level is
Clone/Copy/Debug/Eq/PartialEq: Output (uncolored), Error (red), Info (blue), Success (green), System
(magenta), Warning (yellow). There is no implicit severity filtering or logging sink.

OutputEnvironment is Clone/Copy/Debug/Default/Eq/PartialEq with public `is_terminal: bool` and
`no_color: bool`, both false by default. It describes the chosen output independently of stdin.

### 1.1. OutputEnvironment::stderr

`OutputEnvironment::stderr() -> Self`

Snapshot native stderr and NO_COLOR.

```rust
let native = OutputEnvironment::stderr();
```

**Returns** The current stderr.is_terminal and whether NO_COLOR is present/nonempty; it never reads
stdin or writes output.

## 2. Formatting

All caller text has controls and bidi overrides replaced with visible replacement characters. Brand
and final text are trimmed. Generated ANSI surrounds only sanitized text; one terminating newline is
renderer-owned. Control does not flush writers.

### 2.1. new

`Control::new(brand: &str, color: ColorMode) -> Self`

Create immutable brand/color policy.

```rust
let control = Control::new(" [app] ", ColorMode::Auto);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `brand` | `&str` | Sanitized and trimmed human prefix. |
| `color` | `ColorMode` | Auto/Always/Never policy. |

**Returns** A Control without streams, input ownership, or I/O.

### 2.2. brand

`brand(&self) -> &str`

Read the normalized label.

```rust
assert_eq!(control.brand(), "[app]");
```

**Returns** Borrowed printable brand.

### 2.3. format

`format(&self, level: Level, message: &str, variables: &[&str], environment: OutputEnvironment) -> String`

Substitute original-template placeholders without I/O.

```rust
assert_eq!(
    control.format(Level::Info, "%s -> %s", &["a", "b"], environment),
    "[app] a -> b\n"
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `level` | `Level` | Fixed generated color when policy allows. |
| `message` | `&str` | Original template with sequential %s placeholders. |
| `variables` | `&[&str]` | Literal replacements; extra values ignored, unused placeholders retained. |
| `environment` | `OutputEnvironment` | Chosen output identity/NO_COLOR snapshot. |

**Returns** Owned sanitized text with optional generated ANSI and one final newline. Inserted %s
text is not consumed again.

### 2.4. write

`write(&self, writer: &mut (dyn Write + Send), environment: OutputEnvironment, level: Level, message: &str, variables: &[&str]) -> io::Result<()>`

Format and write one human diagnostic through an explicit sink.

```rust
control.write(
    &mut output,
    environment,
    Level::Warning,
    "Check %s",
    &["input"],
)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `writer` | `&mut (dyn Write + Send)` | Exclusive synchronous human sink. |
| `environment` | `OutputEnvironment` | Output properties, not input properties. |
| `level` | `Level` | Severity/color selection. |
| `message` | `&str` | Template. |
| `variables` | `&[&str]` | Literal substitutions. |

**Returns** Unit or original writer I/O error. write_all may accept a prefix before failing; no new
message attempt/flush is added and arbitrary writers may block.

### 2.5. output

`output(&self, writer: &mut (dyn Write + Send), environment: OutputEnvironment, message: &str, variables: &[&str]) -> io::Result<()>`

Write a human message using Level::Output.

```rust
control.output(&mut output, environment, "Loaded %s", &["profiles"])?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `writer` | `&mut (dyn Write + Send)` | Chosen human sink. |
| `environment` | `OutputEnvironment` | Chosen output properties. |
| `message` | `&str` | Template. |
| `variables` | `&[&str]` | Literal substitutions. |

**Returns** Unit or writer I/O error, as write. Always uncolored.

### 2.6. error

`error(&self, writer: &mut (dyn Write + Send), environment: OutputEnvironment, message: &str, variables: &[&str]) -> io::Result<()>`

Write a human message using Level::Error.

```rust
control.error(&mut output, environment, "Loaded %s", &["profiles"])?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `writer` | `&mut (dyn Write + Send)` | Chosen human sink. |
| `environment` | `OutputEnvironment` | Chosen output properties. |
| `message` | `&str` | Template. |
| `variables` | `&[&str]` | Literal substitutions. |

**Returns** Unit or writer I/O error, as write. Red when enabled.

### 2.7. info

`info(&self, writer: &mut (dyn Write + Send), environment: OutputEnvironment, message: &str, variables: &[&str]) -> io::Result<()>`

Write a human message using Level::Info.

```rust
control.info(&mut output, environment, "Loaded %s", &["profiles"])?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `writer` | `&mut (dyn Write + Send)` | Chosen human sink. |
| `environment` | `OutputEnvironment` | Chosen output properties. |
| `message` | `&str` | Template. |
| `variables` | `&[&str]` | Literal substitutions. |

**Returns** Unit or writer I/O error, as write. Blue when enabled.

### 2.8. success

`success(&self, writer: &mut (dyn Write + Send), environment: OutputEnvironment, message: &str, variables: &[&str]) -> io::Result<()>`

Write a human message using Level::Success.

```rust
control.success(&mut output, environment, "Loaded %s", &["profiles"])?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `writer` | `&mut (dyn Write + Send)` | Chosen human sink. |
| `environment` | `OutputEnvironment` | Chosen output properties. |
| `message` | `&str` | Template. |
| `variables` | `&[&str]` | Literal substitutions. |

**Returns** Unit or writer I/O error, as write. Green when enabled.

### 2.9. system

`system(&self, writer: &mut (dyn Write + Send), environment: OutputEnvironment, message: &str, variables: &[&str]) -> io::Result<()>`

Write a human message using Level::System.

```rust
control.system(&mut output, environment, "Loaded %s", &["profiles"])?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `writer` | `&mut (dyn Write + Send)` | Chosen human sink. |
| `environment` | `OutputEnvironment` | Chosen output properties. |
| `message` | `&str` | Template. |
| `variables` | `&[&str]` | Literal substitutions. |

**Returns** Unit or writer I/O error, as write. Magenta when enabled.

### 2.10. warning

`warning(&self, writer: &mut (dyn Write + Send), environment: OutputEnvironment, message: &str, variables: &[&str]) -> io::Result<()>`

Write a human message using Level::Warning.

```rust
control.warning(&mut output, environment, "Loaded %s", &["profiles"])?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `writer` | `&mut (dyn Write + Send)` | Chosen human sink. |
| `environment` | `OutputEnvironment` | Chosen output properties. |
| `message` | `&str` | Template. |
| `variables` | `&[&str]` | Literal substitutions. |

**Returns** Unit or writer I/O error, as write. Yellow when enabled.

### 2.11. input

`input(&self, question: &str, answer: Option<&str>) -> PromptOptions`

Create prompt options without acquiring input or rendering.

```rust
let options = control.input("Name?", Some("Ada"));
assert!(!options.required);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `question` | `&str` | Question text for later rendering. |
| `answer` | `Option<&str>` | Default, including Some empty string; None makes this convenience required. |

**Returns** PromptOptions with default set and required = answer.is_none(). It is an options
builder, not a native prompt call; it does not apply brand/color to prompt rendering.

Related: [human output guide](../guides/human-output.md), [prompt options](prompt-options.md).
