# ProjectLoader API

ProjectLoader loads native Idea/JSON files from an explicit resolution root. It is Debug and
Clone; clones copy the root and hold no source cache. Source:
[source/native.rs](../../src/source/native.rs).

```rust
use stackpress_idea_transformer::{ProjectLoader, SourceLoader};
let parent = std::env::temp_dir().canonicalize()?;
let temp = tempfile::tempdir_in(parent)?;
std::fs::write(
    temp.path().join("schema.idea"),
    "model Profile { id String }",
)?;
```

The setup creates real temporary input for each independent example. It requires `tempfile`.

## 1. ProjectLoader::new

`ProjectLoader::new(root: impl AsRef<Path>) -> std::result::Result<Self, LoadError>`

Bind entry and `@/` lookup to a canonical directory.

```rust
let loader = ProjectLoader::new(temp.path())?;
let source = loader.load(None, "schema", 1024)?;
assert_eq!(source.name, "schema.idea");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `root` | `impl AsRef<Path>` | Absolute existing directory, canonicalized once. |

**Returns** The loader, or LoadError for relative/non-directory roots and canonicalization IO
failures. The root is a resolution base, not a filesystem sandbox.

## 2. root

`root(&self) -> &Path`

Inspect the anchored resolution directory.

```rust
let loader = ProjectLoader::new(temp.path())?;
assert_eq!(loader.root(), temp.path());
```

**Returns** The canonical root without changing or rereading process cwd.

## 3. Native load behavior

The inherited [load](source-loader.md#1-sourceloaderload) tries exact file, `.idea`, `.json`,
`index.idea`, then `index.json`, omitting extension candidates for explicit extensions. Ordinary
imports resolve at the importing canonical file. Absolute and parent-relative imports work;
`@/` uses the root. A source symlink resolves to its canonical target, including target syntax.

Acquisition rejects nonregular files before opening, checks metadata size, reads at most one
detection byte beyond the bound, validates UTF-8, and compares size/mtime before and after.
Missing candidates fall through; permission and other IO errors do not. Directories without
an index retain a distinct cause. Source changes are detected where possible, without an atomic
snapshot guarantee against outside writers. Unsupported extensions fail rather than evaluating code.

Related: [compose sources](../guides/composing-sources.md), [MemoryLoader](memory-loader.md).
