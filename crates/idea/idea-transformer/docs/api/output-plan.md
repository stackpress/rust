# Output plan API

OutputPlan owns complete fresh file claims, independent of prior disk contents. It is Debug
and Default; its fields are private. Sources: [plan/mod.rs](../../src/plan/mod.rs),
[path rules](../../src/plan/path.rs), and [GeneratedFile](../../src/registry.rs).

```rust
use stackpress_idea_transformer::{
    GeneratedFile, OutputPlan, Registry, Result, TransformContext, TransformSpec,
};
fn emit(_: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    plan.write("contracts/profile.rs", "pub struct Profile;\n")?;
    plan.contribute_module("contracts", "profile")
}
let mut registry = Registry::new();
registry.add(TransformSpec::new("profile").owns(&["contracts"]), emit)?;
let plan = registry.plan(&Default::default(), vec![])?;
```

Each following block uses this setup independently. Contents:
[1. Files](#1-generatedfile) · [2. Inspection](#2-inspection) ·
[3. Callback claims](#3-callback-claims) · [4. Paths](#4-portable-paths-and-budgets)

## 1. GeneratedFile

GeneratedFile is Debug with public `path: String` and `contents: String` fields. Constructing
it proposes output; registration/planning validates the claim.

### 1.1. GeneratedFile::new

`GeneratedFile::new(path: impl Into<String>, contents: impl Into<String>) -> Self`

Propose one complete UTF-8 file for a legacy callback or app aggregate.

```rust
let file = GeneratedFile::new("index.txt", "Profile\n");
assert_eq!(file.path, "index.txt");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl Into<String>` | Relative slash-separated path. |
| `contents` | `impl Into<String>` | Complete new bytes; no patching prior files. |

**Returns** An unvalidated file value, infallibly. App aggregates may claim only root-level
files that do not collide with partition names. The writer reserves its own hidden manifest.

## 2. Inspection

Inspection methods borrow the completed plan; they grant no write authority. File text here
is pre-formatting text; OutputWorkspace formats a bounded copy for comparison/installation.

### 2.1. executed

`executed(&self) -> &[String]`

Read callback execution order.

```rust
assert_eq!(plan.executed(), &["profile"]);
```

**Returns** Selected IDs in registration order, including functions that wrote no files.

### 2.2. inputs

`inputs(&self) -> &[String]`

Inspect the consumed auxiliary-input names.

```rust
assert!(plan.inputs().is_empty());
```

**Returns** Names sorted and deduplicated, without values. Root/import source acquisition is
separate composition evidence and does not appear in this list.

### 2.3. files

`files(&self) -> impl Iterator<Item = (&str, &str, &str)>`

Inspect proposed relative paths, owners, and contents.

```rust
assert_eq!(plan.files().count(), 3);
let (path, owner, contents) = plan.files().next().unwrap();
assert_eq!(
    (path, owner, contents),
    ("contracts/mod.rs", "aggregate", "pub mod profile;\n")
);
```

**Returns** Borrowed triples in stable lexical path order, independent of callback order.

### 2.4. contributions

`contributions(&self) -> &[ModuleContribution]`

Inspect ordered declarations, also available to later callbacks.

```rust
let contribution = &plan.contributions()[0];
assert_eq!(contribution.owner, "profile");
assert_eq!(contribution.partition, "contracts");
assert_eq!(contribution.module, "profile");
```

**Returns** Contributions in callback/declaration order. ModuleContribution is Debug, Clone,
PartialEq, and Eq, with public `owner: String`, `partition: String`, and `module: String` fields.
The module name excludes `.rs`.

## 3. Callback claims

Only Registry grants active callback authority. A default plan alone cannot accept writes;
callbacks may inspect previous contributions but cannot mutate other owners' files.

### 3.1. write

`write(&mut self, path: impl Into<String>, contents: impl Into<String>) -> Result<()>`

Claim one fresh file under an active transform's exact partition spelling.

```rust
fn write_text(_: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    plan.write("data/message.txt", "Hello\n")
}
let mut text = Registry::new();
text.add(TransformSpec::new("text").owns(&["data"]), write_text)?;
assert_eq!(text.plan(&Default::default(), vec![])?.files().count(), 1);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl Into<String>` | Valid `partition/...` path; root files are aggregate-owned. |
| `contents` | `impl Into<String>` | Complete proposed UTF-8 file. |

**Returns** Unit, or inactive-scope, path, ownership, collision, count, or byte-budget errors.
Duplicate claims fail even for identical bytes. Failed claims leave ownership/accounting unchanged.

### 3.2. contribute_module

`contribute_module(&mut self, partition: &str, module: &str) -> Result<()>`

Declare a public module for the single automatic aggregate writer.

```rust
assert_eq!(plan.contributions()[0].module, "profile");
assert!(
    plan.files()
        .any(|(path, owner, _)| path == "mod.rs" && owner == "aggregate")
);
```

The setup calls `plan.contribute_module("contracts", "profile")` after writing its module.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `partition` | `&str` | Owned partition and conservative ASCII Rust identifier. |
| `module` | `&str` | Conservative ASCII Rust identifier; keywords, `_`, and `mod` are rejected. |

**Returns** Unit or invalid-identifier, duplicate, ownership, or contribution-budget errors.
Planning then requires an owned `partition/module.rs` file and writes partition/root `mod.rs`
under owner `aggregate`. Duplicates are case-insensitive; at most 1,024 contributions are
allowed. Supplied aggregates or plugin files colliding with these indexes fail planning.
If no contributions exist, no automatic indexes are invented.

## 4. Portable paths and budgets

Paths are 1–240 ASCII bytes, at most 16 slash-separated components, with only letters, digits,
`_`, `-`, `.`, and `/`. A path cannot start with `.`. Components cannot be empty, `.` or `..`,
end with `.`, or have a Windows device stem (`CON`, `PRN`, `AUX`, `NUL`, `COM0`–`COM9`,
`LPT0`–`LPT9`, case-insensitive). Absolute paths, backslashes, controls, spaces, and Unicode
are outside this vocabulary. The root destination's final name uses the same validator.

File/ancestor and case aliases collide, including differently cased shared directories with
different leaves: `data/Models/a.rs` versus `data/models/b.rs`. Exact same-spelling directory
siblings such as `data/models/a.rs` and `data/models/b.rs` are valid. Partition comparisons
also reject ASCII case aliases across registrations.

Plans allow at most 1,024 file claims and 1,048,576 combined content bytes, including aggregates.
Formatted output is checked against the same byte ceiling. Claims perform no disk IO; trusted
callbacks remain responsible for any direct IO they perform themselves.

Related: [module guide](../guides/generating-modules.md), [workspace](output-workspace.md).
