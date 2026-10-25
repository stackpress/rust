# MemoryLoader API

MemoryLoader supplies deterministic in-memory source snapshots. Debug/Clone are available;
clones copy bytes independently. Source: [source/memory.rs](../../src/source/memory.rs).

```rust
use stackpress_idea_transformer::{MemoryLoader, SourceLoader};
```

Following examples are independent and use no source filesystem IO.

## 1. MemoryLoader::new

`MemoryLoader::new(root: impl AsRef<Path>) -> std::result::Result<Self, LoadError>`

Create a virtual absolute namespace.

```rust
let loader = MemoryLoader::new("/project")?;
assert!(loader.load(None, "missing", 1024).is_err());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `root` | `impl AsRef<Path>` | Absolute virtual root, lexically normalized. |

**Returns** An empty loader or LoadError for a relative root. No host directory must exist.

## 2. insert

`insert(&mut self, path: impl AsRef<Path>, text: impl Into<String>)`

Insert or replace a virtual snapshot.

```rust
let mut loader = MemoryLoader::new("/project")?;
loader.insert("schema.idea", "model Profile { id String }");
assert_eq!(loader.load(None, "schema", 1024)?.name, "schema.idea");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl AsRef<Path>` | Joined to the root and normalized; absolute paths retain absolute resolution. |
| `text` | `impl Into<String>` | Complete UTF-8 source to own. |

**Returns** Unit, infallibly; also replaces a prior alias at that path. Load-time byte budgets
are checked before cloning the stored text, not before the caller constructs it.

## 3. alias

`alias(&mut self, alias: impl AsRef<Path>, target: impl AsRef<Path>) -> std::result::Result<(), LoadError>`

Give a canonical source another fixture path for cycle/diamond tests.

```rust
let mut loader = MemoryLoader::new("/project")?;
loader.insert("schema.idea", "model Profile { id String }");
loader.alias("alias.idea", "schema.idea")?;
assert_eq!(
    loader.load(None, "alias", 1024)?.id,
    loader.load(None, "schema", 1024)?.id,
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `alias` | `impl AsRef<Path>` | Normalized alias path that must not hide a canonical file. |
| `target` | `impl AsRef<Path>` | Existing canonical virtual source; aliases are not chained. |

**Returns** Unit or LoadError for a missing target or a canonical-file collision. An existing
alias at the same unused source path can be replaced. Imported relative paths resolve from
the target's canonical parent. Candidate order matches [ProjectLoader](project-loader.md).

Related: [SourceLoader](source-loader.md), [composition](composition.md).
