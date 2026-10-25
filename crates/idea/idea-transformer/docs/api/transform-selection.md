# Transform selection and input API

TransformSelection owns runtime contract metadata and auxiliary snapshots for one run. It
does not register executable functions. Both it and TransformInputs implement Debug/Default.
Source: [registry/context.rs](../../src/registry/context.rs).

```rust
use stackpress_idea_transformer::{TransformInputs, TransformSelection};
```

Each following snippet is independent. Neither type performs implicit filesystem or
environment reads.

## 1. TransformInputs

TransformInputs has private fields and owns all snapshot bytes. Logical names use the
[portable path rules](output-plan.md#4-portable-paths-and-budgets); input values are not reports.

### 1.1. TransformInputs::new

`TransformInputs::new() -> Self`

Start a snapshot collection before invoking callbacks.

```rust
let inputs = TransformInputs::new();
let selection = TransformSelection::new().with_inputs(inputs);
```

**Returns** An empty collection, infallibly, equivalent to Default.

### 1.2. insert

`insert(&mut self, name: &str, contents: impl Into<String>) -> Result<()>`

Declare a unique logical input.

```rust
let mut inputs = TransformInputs::new();
inputs.insert("templates/profile.txt", "Profile body\n")?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Portable relative logical name; never an absolute host path. |
| `contents` | `impl Into<String>` | Snapshot text to own. |

**Returns** Unit or an invalid-name, duplicate, count, or byte-budget error. At most 1,024
inputs and 1,048,576 combined content bytes are allowed; failed inserts leave the collection's
accounting unchanged. The caller constructs the supplied text before this validation.

## 2. TransformSelection

Selection fields are private. Complete the metadata before borrowing it for planning.

### 2.1. TransformSelection::new

`TransformSelection::new() -> Self`

Start with no runtime contract declarations or auxiliary inputs.

```rust
let selection = TransformSelection::new();
```

**Returns** An empty selection, infallibly, equivalent to Default.

### 2.2. contract

`contract(&mut self, id: &str, revision: &str) -> Result<()>`

Supply an exact selected runtime revision for generator preflight.

```rust
let mut selection = TransformSelection::new();
selection.contract("profile-runtime", "1")?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&str` | Unique valid portable logical path. |
| `revision` | `&str` | 1–128 ASCII letters/digits, `.`, `_`, or `-`. |

**Returns** Unit or invalid/duplicate/budget errors. Repeated IDs fail even for identical
revisions; at most 1,024 contracts are allowed. This is independent of the writer's report version.

### 2.3. with_inputs

`with_inputs(self, inputs: TransformInputs) -> Self`

Attach the entire auxiliary snapshot.

```rust
let mut inputs = TransformInputs::new();
inputs.insert("templates/profile.txt", "Profile body\n")?;
let selection = TransformSelection::new().with_inputs(inputs);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `inputs` | `TransformInputs` | Complete bounded collection moved into the selection. |

**Returns** The owned selection for chaining; replaces previous inputs. Callbacks read it
through [TransformContext::input](transform-context.md#5-input), which records only used names.

Related: [TransformSpec](transform-spec.md), [register transforms](../guides/registering-transforms.md).
