# Errors and bounded source reading

The crate's `Result<T>` aliases `std::result::Result<T, TransformError>`. Source acquisition
uses its distinct LoadError. Both preserve causes without automatically rendering source text.
Sources: [lib.rs](../../src/lib.rs) and [source/mod.rs](../../src/source/mod.rs).

```rust
use stackpress_idea_transformer::{LoadError, TransformError, TransformErrorKind, read_source};
use std::error::Error;
```

Each following snippet uses this setup independently. Exact contextual message strings are
not the classification contract; inspect `kind()` and then typed causes for diagnostics.

## 1. TransformError

TransformError is Debug, Display, and Error with private kind/message/cause fields. Display shows
the phase/ownership explanation; `Error::source()` exposes retained causes. `From<io::Error>`
uses `generation filesystem operation failed` and keeps the original IO error. `From<IdeaError>`
uses `Idea parsing failed` and retains the original diagnostic and logical coordinates privately. Contextual callback failures wrap the
original TransformError; legacy callbacks retain their existing Display/source chain with kind Callback. Panics are not converted.

### 1.1. TransformError::new

`TransformError::new(message: impl Into<String>) -> Self`

Describe a generator-specific validation failure.

```rust
let error = TransformError::new("Profile requires id and name");
assert!(error.source().is_none());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `message` | `impl Into<String>` | Caller-owned explanation; avoid secrets or source excerpts. |

**Returns** An error with kind Other and no underlying cause. Messages are not automatically sanitized when
the caller constructs them; the application adapter owns public disclosure policy.

### 1.2. kind and TransformErrorKind

`kind(&self) -> TransformErrorKind` returns a Copy, Eq, non-exhaustive enum. Existing
`new`/conversion signatures remain compatible; callers must retain a wildcard match arm.

| Kind | Meaning |
| --- | --- |
| Other | Custom error or an invariant without a narrower category. |
| Io | Native file, encoding or subprocess IO failure. |
| Parse | Idea/JSON parse, normalization or import topology failure. |
| LockExists | Cooperative writer lock is already present. |
| RecoveryRequired | Retained installation state requires explicit recovery. |
| Ownership | Path, partition, manifest or installed-byte ownership conflict. |
| Selection | Invalid selection, prerequisite, contract or auxiliary input declaration. |
| Limit | Acquisition, metadata, planning or subprocess resource limit. |
| Callback | A selected callback returned an error. |

Install phase wrappers retain the inner kind and source. Contextual callback errors have kind
Callback; inspect their source for an underlying planning/IO kind. Message wording is not stable.

## 2. LoadError

LoadError is Debug, Display, and Error. Its fields are private; Display shows the adapter's
message, and source exposes its optional native/encoding cause.

### 2.1. LoadError::new

`LoadError::new(message: impl Into<String>) -> Self`

Return a custom resolution/format/budget failure.

```rust
let error = LoadError::new("schema source not found");
assert!(error.source().is_none());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `message` | `impl Into<String>` | Safe adapter explanation without source contents. |

**Returns** A LoadError without a cause.

### 2.2. with_cause

`LoadError::with_cause(message: impl Into<String>, cause: impl Error + Send + Sync + 'static) -> Self`

Retain a native failure for deliberate developer diagnostics.

```rust
let error = LoadError::with_cause(
    "schema source not found",
    std::io::Error::from(std::io::ErrorKind::NotFound),
);
assert!(
    error
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .is_some()
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `message` | `impl Into<String>` | Display explanation. |
| `cause` | `impl Error + Send + Sync + 'static` | Owned original error. |

**Returns** A LoadError whose source chain preserves the cause rather than concatenating it
into public Display. Custom messages remain the adapter author's disclosure responsibility.

## 3. read_source

`read_source(path: &Path, max_bytes: usize) -> Result<String>`

Read bounded UTF-8 from a regular file, following input symlinks.

```rust
let parent = std::env::temp_dir().canonicalize()?;
let temp = tempfile::tempdir_in(parent)?;
let path = temp.path().join("input.txt");
std::fs::write(&path, "Profile")?;
assert_eq!(read_source(&path, 7)?, "Profile");
assert!(read_source(&path, 6).is_err());
```

The example needs `tempfile`; it proves the byte bound on an ordinary file.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&Path` | File to open. |
| `max_bytes` | `usize` | Maximum accepted bytes; one extra byte detects overflow. |

**Returns** UTF-8 text, retained IO/encoding causes, or a Limit error. Like
[ProjectLoader](project-loader.md), it rejects nonregular files before open and validates the
opened handle. Source symlinks are followed; [OutputWorkspace](output-workspace.md) separately
rejects output symlinks. Pre/open/post checks assume cooperative path writers and are not
race-free against a concurrent special-file replacement. The byte ceiling does not bound OS IO
latency; no wall-clock cancellation of regular-file reads is promised.

Related: [parser diagnostics](../../../idea-parser/docs/README.md),
[source contract](source-loader.md). Walk cause chains with a finite application-owned bound
when handling untrusted custom Error implementations.
