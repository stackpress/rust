# Error and diagnostics API

Owned failures with separate public and developer views. Source: [error.rs](../../src/error.rs).
`Result<T>` aliases `std::result::Result<T, Error>`; Error implements Display, Debug, and
std::error::Error, with no Clone or implicit serialization.

```rust
use stackpress_lib::{Error, ErrorKind, Result, SourceLocation};

let error = Error::new("private operation failure");
```

Run each snippet independently inside a function returning `Result<()>`, using this fresh `error`
unless it declares its own. Developer messages/causes may be sensitive; public_view explicitly
selects output fields.

Contents: [Public error and diagnostic values](#1-public-error-and-diagnostic-values) ·
[Error construction](#2-error-construction) · [Error inspection](#3-error-inspection) ·
[Error enrichment and public output](#4-error-enrichment-and-public-output)

## 1. Public error and diagnostic values

ErrorKind is Clone/Copy/Debug/Eq/PartialEq/Serialize. The kinds are categories, not a complete
product error taxonomy; status and public text can be selected independently.

| ErrorKind | Meaning |
| --- | --- |
| `InvalidInput` | Malformed/unacceptable input. |
| `Limit` | Checked resource budget or counter exhaustion. |
| `Registration` | Invalid registration/matcher operation. |
| `State` | Invalid load/delivery/finalization transition. |
| `External` | Owned source failure. |
| `Domain` | Application handler failure; new defaults here. |

| Public type | Fields / behavior |
| --- | --- |
| `PublicError` | `code: u16`, `status: String`, `error: String`, `errors: serde_json::Value`. Clone/Debug/Serialize; no private diagnostics. |
| `SourceLocation` | `file: String`, `line: u32`, `column: u32`. Clone/Debug/Eq/PartialEq/Serialize. |
| `SourceSpan` | `start: usize` inclusive and `end: usize` exclusive byte offsets. Clone/Copy/Debug/Eq/PartialEq/Serialize. |
| `InvocationFrame` | `event`, `pattern`, `listener`, `label`, `phase`: String; `plugin: Option<String>`; `priority: i64`; `registered: SourceLocation`; `replaced` and `phase_registered`: Option<SourceLocation>. Clone/Debug/Eq/PartialEq/Serialize. |

Frames are root-first and identify a listener phase without copying capture values. `registered` is
the original contribution site; `replaced` is the replacement site, and `phase_registered` is a
before/after hook site when applicable. These captured sites do not claim an exact failing
expression inside an arbitrary future. SourceSpan fields are public; with_span checks ordering but
cannot validate offsets against an unavailable source document.

### 1.1. SourceLocation::caller

`SourceLocation::caller() -> Self`

Capture the tracked synchronous caller location.

```rust
let location = SourceLocation::caller();
assert!(location.line > 0);
```

**Returns** An owned file/line/column record. A wrapper must itself forward track_caller if it wants
the external caller rather than its own call site.

## 2. Error construction

New errors capture an origin and optional native backtrace once. Backtrace availability follows the
runtime environment. Logical frames/origins are useful even when native capture is disabled.

### 2.1. Error::new

`Error::new(message: impl Into<String>) -> Self`

Create a private Domain failure.

```rust
let error = Error::new("database detail");
assert_eq!(error.public_view().error, "Internal Server Error");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `message` | `impl Into<String>` | Owned message; no constructor length check. |

**Returns** An Error with kind Domain, code 500, empty object field errors, no
path/span/cause/frames, and no chosen public message.

### 2.2. Error::invalid

`Error::invalid(message: impl Into<String>) -> Self`

Create an input error whose supplied message is deliberately public.

```rust
let error = Error::invalid("Name is required");
assert_eq!(error.public_view().error, "Name is required");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `message` | `impl Into<String>` | Owned message; no constructor length check. |

**Returns** An Error with InvalidInput, code 400, and the same developer/public message.

### 2.3. Error::limit

`Error::limit(message: impl Into<String>) -> Self`

Create a budget error using the public-input-message policy.

```rust
let error = Error::limit("input byte limit exceeded");
assert_eq!(error.kind(), ErrorKind::Limit);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `message` | `impl Into<String>` | Owned message; no constructor length check. |

**Returns** An Error with Limit, code 400, and the supplied message exposed publicly.

### 2.4. Error::from_source

`Error::from_source(cause: impl std::error::Error + Send + Sync + 'static) -> Self`

Own an external error, retaining its concrete source for chain inspection/downcasting. Display is
`external operation failed`; the source alone carries the underlying explanation. This is an
intentional change from copying the cause's Display into both layers.

```rust
let cause = "invalid".parse::<u32>().expect_err("fixture");
let error = Error::from_source(cause);
assert_eq!(error.kind(), ErrorKind::External);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `cause` | `impl std::error::Error + Send + Sync + 'static` | Owned concrete source; its Display remains available through source traversal. |

**Returns** An External Error with code 500 and default private-public separation. No blanket
From<E> exists; map arbitrary sources with map_err(Error::from_source).

### 2.5. Error::with_cause

`with_cause(self, cause: impl std::error::Error + Send + Sync + 'static) -> Self`

Attach or replace a cause without changing current classification/message/status.
Choose a wrapper message describing distinct operation context. Developer formatters print the
wrapper and each source once; callers must keep public disclosure explicit through `public_view`.

```rust
let error = error.with_cause(std::io::Error::other("reader detail"));
assert!(std::error::Error::source(&error).is_some());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `cause` | `impl std::error::Error + Send + Sync + 'static` | Owned source replacing any prior cause. |

**Returns** The moved enriched error; no cause chain is cloned or formatted into public output.

## 3. Error inspection

Inspection borrows retained metadata or copies small values. Display includes developer message and
optional ` at path`; Debug also exposes origin/frames/cause. Neither is the public wire
representation.

### 3.1. Error::kind

`kind(&self) -> ErrorKind`

Read stable classification.

```rust
assert_eq!(error.kind(), ErrorKind::Domain);
```

**Returns** The copied kind.

### 3.2. Error::code

`code(&self) -> u16`

Read selected application code.

```rust
assert_eq!(error.code(), 500);
```

**Returns** The numeric code, without HTTP legality validation.

### 3.3. Error::message

`message(&self) -> &str`

Read the original developer message.

```rust
assert_eq!(error.message(), "private operation failure");
```

**Returns** Borrowed developer text, without the optional Display path suffix.

### 3.4. Error::path

`path(&self) -> Option<&str>`

Inspect optional field/event context.

```rust
assert!(error.path().is_none());
```

**Returns** Borrowed context, or None; arbitrary path text is not reparsed.

### 3.5. Error::origin

`origin(&self) -> &SourceLocation`

Inspect the original captured construction site.

```rust
assert!(error.origin().line > 0);
```

**Returns** Borrowed origin; later propagation does not replace it.

### 3.6. Error::frames

`frames(&self) -> &[InvocationFrame]`

Inspect the deepest dispatcher-annotated call path.

```rust
assert!(error.frames().is_empty());
```

**Returns** Borrowed root-first frames; empty before dispatch annotation. Outer propagation retains
an existing deepest path.

### 3.7. Error::span

`span(&self) -> Option<SourceSpan>`

Inspect optional source byte offsets.

```rust
assert!(error.span().is_none());
```

**Returns** A copied span or None.

### 3.8. Error::backtrace

`backtrace(&self) -> &std::backtrace::Backtrace`

Inspect original native capture state.

```rust
let _capture_state = error.backtrace().status();
```

**Returns** Borrowed captured/disabled/unsupported backtrace; no recapture occurs.

### 3.9. std::error::Error::source

`source(&self) -> Option<&(dyn std::error::Error + 'static)>`

Use the standard error trait to inspect the immediate owned cause.

```rust
let error = Error::from_source(std::io::Error::other("reader failed"));
assert!(
    std::error::Error::source(&error)
        .unwrap()
        .is::<std::io::Error>()
);
```

**Returns** A borrowed cause or None. This is std::error::Error::source, not an inherent method.

## 4. Error enrichment and public output

Enrichment consumes and returns the Error so original source ownership stays explicit. No implicit
public serialization or automatic error event is performed.

### 4.1. Error::with_kind

`with_kind(self, kind: ErrorKind) -> Self`

Override classification without changing code/message or losing cause/frames.

```rust
let error = error.with_kind(ErrorKind::State);
assert_eq!(error.kind(), ErrorKind::State);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `kind` | `ErrorKind` | Selected category. |

**Returns** The moved error with the new kind.

### 4.2. Error::with_code

`with_code(self, code: u16) -> Self`

Override application status independently of dispatch flow.

```rust
let error = error.with_code(419);
assert_eq!(error.public_view().status, "Page Expired");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `code` | `u16` | Any application code; catalog determines the public status label. |

**Returns** The moved error; unknown public labels become Unknown.

### 4.3. Error::with_public_message

`with_public_message(self, message: impl Into<String>) -> Self`

Select an intentional public message independently of developer text.

```rust
let error = error.with_public_message("Could not complete operation");
assert_eq!(error.public_view().error, "Could not complete operation");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `message` | `impl Into<String>` | Explicit disclosure text, without a length check here. |

**Returns** The moved error; response application may later enforce message size.

### 4.4. Error::with_errors

`with_errors(self, errors: serde_json::Value) -> Self`

Attach intentional public validation data.

```rust
let error = error.with_errors(serde_json::json!({"name": ["required"]}));
assert!(error.public_view().errors.is_object());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `errors` | `serde_json::Value` | Any JSON here; Response application later requires an object root and budgets. |

**Returns** The moved error; no root-shape check or redaction is performed on these explicit fields.

### 4.5. Error::at_path

`at_path(self, path: impl Into<String>) -> Self`

Attach/replace field context without altering original cause or origin.

```rust
let error = error.at_path("profile.name");
assert_eq!(error.path(), Some("profile.name"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl Into<String>` | Diagnostic context string, not a checked DataPath. |

**Returns** The moved error with the selected path.

### 4.6. Error::with_span

`with_span(self, start: usize, end: usize) -> Result<Self>`

Attach ordered inclusive-start/exclusive-end byte offsets.

```rust
let error = error.with_span(2, 5)?;
assert_eq!(error.span().unwrap().end, 5);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `start` | `usize` | Inclusive byte start. |
| `end` | `usize` | Exclusive byte end, at least start. |

**Returns** The moved original error on success. Reversed offsets return a new InvalidInput error;
equality is allowed. Source length/UTF-8 boundaries are not checked.

### 4.7. Error::public_view

`public_view(&self) -> PublicError`

Select code, catalog/Unknown status, chosen message, and explicit field errors.

```rust
let public = error.public_view();
let json = serde_json::to_value(public)?;
assert!(json.get("origin").is_none());
```

**Returns** A cloned public data object; absent public message becomes Internal Server Error even
when code differs from 500. Origin/path/span/frames/cause/backtrace are omitted. Explicit field
errors are retained unchanged.

### 4.8. From conversions

`Error::from(cause) for serde_json::Error, std::io::Error, std::num::ParseIntError`

Use the three concrete conversions with `?`; they preserve the owned concrete source and External
classification. Their Display messages are respectively `JSON operation failed`, `I/O operation
failed`, and `integer parsing failed`. Source traversal supplies the detailed explanation.

```rust
let parsed: Result<u32> = "invalid".parse::<u32>().map_err(Error::from);
assert_eq!(parsed.unwrap_err().kind(), ErrorKind::External);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `cause` | `One of the three listed error types` | Owned cause passed directly or via the ? operator. |

**Returns** An External Error, infallibly, with the same defaults as from_source. Other source types
require explicit mapping; no arbitrary thrown-value conversion exists.

## Related

[Errors and diagnostics guide](../guides/errors-and-diagnostics.md) · [Status](status.md) ·
[Event](event.md) · [Response](response.md) · [API index](README.md)
