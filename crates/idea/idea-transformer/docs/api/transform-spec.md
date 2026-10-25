# TransformSpec API

TransformSpec is Debug and has private metadata fields. Its builder methods construct data;
[Registry::add](registry.md#12-add) validates it atomically. Source:
[registry.rs](../../src/registry.rs).

```rust
use stackpress_idea_transformer::TransformSpec;
let spec = TransformSpec::new("profile");
```

Use that setup for each independent snippet. Prerequisites and contracts are checked against
the full selection at planning time.

## 1. TransformSpec::new

`TransformSpec::new(id: impl Into<String>) -> Self`

Describe one compiled identity.

```rust
let spec = TransformSpec::new("profile");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `impl Into<String>` | Unique portable path spelling, 1–128 bytes; `aggregate` is reserved. |

**Returns** An unvalidated spec with empty ownership, prerequisite, and contract lists. No IO runs.

## 2. owns

`owns(self, partitions: &[&str]) -> Self`

Choose exclusive first-component output partitions.

```rust
let spec = spec.owns(&["contracts", "server"]);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `partitions` | `&[&str]` | At most 1,024 valid single-component paths; case aliases conflict. |

**Returns** The owned spec with its partition list replaced. Empty lists permit contribution-only
work such as inspecting prior declarations without file claims; module contributions themselves
still require owned partitions/files. Different transforms cannot share a partition.

## 3. requires

`requires(self, ids: &[&str]) -> Self`

Require compiled predecessors without reordering them.

```rust
let spec = spec.requires(&["base"]);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `ids` | `&[&str]` | At most 1,024 distinct nonempty IDs of at most 128 bytes. |

**Returns** The spec with required IDs replaced. Planning fails for missing/later/self IDs,
including cycles. IDs are compared exactly; requirements do not select code automatically.

## 4. after_if_present

`after_if_present(self, ids: &[&str]) -> Self`

Add ordering constraints for optional selected generators.

```rust
let spec = spec.after_if_present(&["metrics"]);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `ids` | `&[&str]` | Same distinct/count/length rules as requires. |

**Returns** The spec with optional IDs replaced. Missing IDs are allowed; selected IDs must
occur earlier. Required and optional lists are each checked for duplicates within that list.

## 5. requires_contract

`requires_contract(self, id: &str, revision: &str) -> Self`

Require an exact app-owned runtime revision.

```rust
let spec = spec.requires_contract("profile-runtime", "1");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&str` | Valid portable logical path; unique among this spec's required contracts. |
| `revision` | `&str` | 1–128 ASCII letters/digits, `.`, `_`, or `-`. |

**Returns** The owned spec with a requirement appended. At most 1,024 contracts are allowed;
duplicate IDs are invalid even if revisions agree. Planning requires the exact supplied
[selection contract](transform-selection.md#22-contract). The writer's contract_version is a
separate ownership/report revision.

Related: [register transforms](../guides/registering-transforms.md), [registry](registry.md).
