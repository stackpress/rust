# Composition API

Composer returns a non-final effective schema and its source/reference evidence. It owns no
state between calls. Sources: [compose/mod.rs](../../src/compose/mod.rs),
[merge.rs](../../src/compose/merge.rs), and [merge_type](../../src/lib.rs).

```rust
use stackpress_idea_parser::{CompileInputs, TypeConfig};
use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader, merge_type};
let mut loader = MemoryLoader::new("/project")?;
loader.insert(
    "schema.idea",
    "use \"./base\" model Profile { name String }",
);
loader.insert("base.idea", "model Profile { id String }");
```

Use this setup before each independent snippet. Contents:
[1. Composer](#1-composer) · [2. Limits](#2-compositionlimits) ·
[3. Result](#3-composedschema-and-importedge) · [4. merge_type](#4-merge_type)

## 1. Composer

`Composer<L>` accepts any SourceLoader, including a borrowed adapter. Construction is
infallible; loading and validation occur during `compose`.

### 1.1. Composer::new

`Composer::new(loader: L, limits: CompositionLimits) -> Self`

Choose a source adapter and acquisition/traversal budgets.

```rust
let composer = Composer::new(&loader, CompositionLimits::default());
assert_eq!(composer.compose("schema")?.sources.len(), 2);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `loader` | `L: SourceLoader` | Owned or borrowed source adapter. |
| `limits` | `CompositionLimits` | Per-run acquisition, traversal, expansion, and JSON budgets. |

**Returns** A composer with default CompileInputs. No load, output IO, or environment read runs.

### 1.2. with_inputs

`with_inputs(self, inputs: CompileInputs) -> Self`

Supply an explicit environment snapshot and parser/data limits.

```rust
let composer =
    Composer::new(loader, CompositionLimits::default()).with_inputs(CompileInputs::default());
assert!(composer.compose("schema")?.config.r#use.is_empty());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `inputs` | `CompileInputs` | Parser-owned input policy, stored for each fresh run. |

**Returns** The owned composer for chaining; replaces earlier inputs. See
[parser API](../../../idea-parser/docs/api/README.md) for environment and ParseLimits details.

### 1.3. compose

`compose(&self, entry: &str) -> Result<ComposedSchema>`

Load and merge one entry's import closure, with no output writes or cwd changes.

```rust
let result = Composer::new(loader, CompositionLimits::default()).compose("schema")?;
assert_eq!(result.config.model["Profile"].columns[0].name, "id");
assert_eq!(result.imports.len(), 1);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `entry` | `&str` | Loader entry specifier. |

**Returns** Owned effective config, source graph, provenance, and actual references. Errors
include parser/JSON/IO failures, active-stack cycles, changed canonical snapshots, and exhausted
budgets. Contextual errors retain typed causes. Counters/caches are private to this call, so a
failed run cannot poison a later call. See the [G19 table](../concepts.md#21-g19-merge-and-replacement)
for exact precedence; immutable accumulated declarations are replaced by children.

## 2. CompositionLimits

CompositionLimits is Debug, Clone, Copy, and Default. All fields are public `usize` values.
`HARD_DEPTH: usize = 64` caps active depth even when a caller raises `max_depth`.

| Field | Default | Charged work |
| --- | ---: | --- |
| `max_source_bytes` | 1,048,576 | Any single acquisition. |
| `max_total_bytes` | 16,777,216 | All acquisitions, including repeated alias/diamond reads. |
| `max_files` | 256 | Distinct canonical snapshots. |
| `max_depth` | 32 | Active import path including root; capped at 64. |
| `max_imports` | 4,096 | Encountered import edges, including repeated incoming edges. |
| `max_expanded_bytes` | 16,777,216 | Cumulative serialized config and actual reference bytes before retention/copy. |
| `max_json_nodes` | 100,000 | JSON values per normalized input, before node allocation. |

Collision losers still spend child-copy allowance. Parser limits independently bound text and
data nesting; JSON structural containers count toward depth. Limits return errors, never
truncated successful output.

## 3. ComposedSchema and ImportEdge

ComposedSchema is Debug/Clone. Its public fields keep separate acquisition, effective-owner,
and typed-reference facts.

| Field | Type | Meaning |
| --- | --- | --- |
| `config` | `SchemaConfig` | Ordered non-final schema; imports consumed, props retained, only root plugins. |
| `sources` | `Vec<LoadedSource>` | Unique snapshots in first-acquisition DFS order, root first. |
| `imports` | `Vec<ImportEdge>` | Encounter-order edges, including distinct incoming diamond edges. |
| `provenance` | `BTreeMap<Vec<String>, SourceId>` | Winning declaration, declaration-attribute, and whole-column origins. |
| `references` | `Vec<Reference>` | Real parser sites surviving adoption/replacement; spans and forward flags retained. |

Provenance paths use names: `model`, `Profile`, `columns`, `id`, or `model`, `Profile`,
`attributes`, `label`. Clones copy ordinary data deeply. Source text may contain secrets.
Transformer `Reference` re-exports the parser type unchanged; no placeholder-string discovery
occurs. Finalize explicitly with both config and sites as in
[compose sources](../guides/composing-sources.md).

ImportEdge is Debug/Clone/PartialEq/Eq with public `importer: SourceId`, `specifier: String`,
and `imported: SourceId`. A typed JSON reference may name an original Idea source, while this
edge and provenance identify the JSON artifact actually loaded.

## 4. merge_type

`merge_type(local: &mut TypeConfig, child: &TypeConfig)`

Apply the type/model primitive for one child. Use Composer when you also need provenance and
reference-site movement; this function only mutates config.

```rust
let composed = Composer::new(loader, CompositionLimits::default()).compose("schema")?;
let mut local: TypeConfig = composed.config.model["Profile"].clone();
let child = local.clone();
merge_type(&mut local, &child);
assert_eq!(local.columns.len(), 2);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `local` | `&mut TypeConfig` | Current accumulated declaration. |
| `child` | `&TypeConfig` | Immutable imported declaration. |

**Returns** Unit. An immutable local is replaced entirely. A mutable local keeps colliding
attributes and whole columns, adopts absent attributes, and prepends missing child columns in
child order. It never reverses or changes the child and performs no independent validation.

Related: [source loader](source-loader.md), [wrapper](../../../idea/docs/api/README.md).
