# Source loader API

`SourceLoader` supplies bounded immutable text snapshots. Native and memory adapters implement
the same trait; custom adapters own canonical identity and acquisition policy.
Source: [source/mod.rs](../../src/source/mod.rs).

```rust
use stackpress_idea_transformer::{MemoryLoader, SourceId, SourceKind, SourceLoader};
let mut loader = MemoryLoader::new("/project")?;
loader.insert("schema.idea", "model Profile { id String }");
```

Use this setup for the independent snippets below. No process directory or environment is
consulted by this fixture.

## 1. SourceLoader::load

`load(&self, importer: Option<&SourceId>, specifier: &str, max_bytes: usize) -> std::result::Result<LoadedSource, LoadError>`

Resolve and acquire one snapshot. Implementations must bound reads/clones before acquisition
and return one identity for every alias of a source. `&T` implements SourceLoader when `T` does,
including unsized loaders; a composer can borrow an adapter.

```rust
let source = loader.load(None, "schema", 1024)?;
assert_eq!(source.name, "schema.idea");
assert_eq!(source.kind, SourceKind::Idea);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `importer` | `Option<&SourceId>` | Canonical importing file, or `None` for root-relative entry lookup. |
| `specifier` | `&str` | Import spelling; adapters determine resolution policy. |
| `max_bytes` | `usize` | Remaining per-call acquisition budget, in UTF-8 bytes. |

**Returns** An owned LoadedSource, or LoadError for resolution, IO, UTF-8, format, or budget
failure. The composer defensively checks returned size and repeated-identity consistency too.

## 2. SourceId

SourceId is an opaque PathBuf identity with Debug, Clone, Eq, Ord, and Hash. It has no required
host existence check; a custom loader chooses its canonical namespace.

### 2.1. SourceId::new

`SourceId::new(path: impl Into<PathBuf>) -> Self`

Construct a canonical identity for an adapter.

```rust
let id = SourceId::new("/project/schema.idea");
assert_eq!(id.as_path(), std::path::Path::new("/project/schema.idea"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl Into<PathBuf>` | Adapter-canonical path; no normalization is performed here. |

**Returns** The identity, infallibly. Native paths may contain non-Unicode components.

### 2.2. as_path

`as_path(&self) -> &Path`

Borrow the identity without converting native bytes lossily.

```rust
let source = loader.load(None, "schema", 1024)?;
assert!(source.id.as_path().is_absolute());
```

**Returns** The stored path; performs no filesystem IO.

## 3. Snapshot data

`LoadedSource` has Debug, Clone, PartialEq, and Eq. All fields are public. Cloning copies its
owned text and identity independently.

| Field | Type | Meaning |
| --- | --- | --- |
| `id` | `SourceId` | Canonical identity for caching, cycles, and graph edges. |
| `name` | `String` | Logical display name, project-relative where possible. |
| `kind` | `SourceKind` | `Idea` or `Json`; Debug, Clone, Copy, PartialEq, Eq. |
| `text` | `String` | Bounded UTF-8 snapshot; can contain secrets. |

`SourceKind::Json` means normalized schema JSON, optionally with checked typed-reference
metadata. It never means executable JavaScript. Only explicit developer diagnostics should
render snapshot contents.

Related: [ProjectLoader](project-loader.md), [MemoryLoader](memory-loader.md),
[load errors](errors.md#2-loaderror), [composition](composition.md).
