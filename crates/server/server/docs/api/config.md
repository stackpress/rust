# ConfigLayers API

Ordered explicit JSON-object inputs, with recursive object merge and scalar/array/null replacement.
Source: [config.rs](../../src/config.rs). ConfigLayers is Default, with no implicit file/env reads.

```rust
use stackpress_server::ConfigLayers;
let mut layers = ConfigLayers::new();
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Layer operations](#1-layer-operations) · [2. Budgets and
decoding](#2-budgets-and-decoding)

## 1. Layer operations

All fallible appends stage their checks before publishing. Defaults are registered separately with
ServerBuilder::config_section and apply before these layers, regardless of registration timing.

### 1.1. new

`ConfigLayers::new() -> Self`

Start an empty external layer sequence.

```rust
let layers = ConfigLayers::new();
```

**Returns** An empty accumulator; Default is equivalent.

### 1.2. json

`json(&mut self, label: &str, text: &str) -> Result<&mut Self>`

Parse one labeled JSON object before appending it.

```rust
layers.json("file", r#"{"profiles":{"limit":2}}"#)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `label` | `&str` | 1–256 bytes without controls; diagnostic identity only. |
| `text` | `&str` | JSON object text; raw bytes also bounded. |

**Returns** The mutable accumulator, or label, JSON, object-root, depth/node/byte/layer-count error.
A rejected append preserves accepted layers and merged state. Errors do not dump the document.

### 1.3. value

`value(&mut self, label: &str, value: serde_json::Value) -> Result<&mut Self>`

Append an already constructed JSON object.

```rust
layers.value("code", serde_json::json!({"profiles": {"limit": 1}}))?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `label` | `&str` | Same diagnostic identity contract as json. |
| `value` | `serde_json::Value` | Consumed object-root data; null inside it remains replacement data. |

**Returns** The mutable accumulator or identity/root/budget error with no partial append. Repeated
labels are permitted. This checks data shape/budgets, not ownership of top-level sections.

### 1.4. entries

`entries(&self) -> impl Iterator<Item = (&str, &serde_json::Value)>`

Inspect retained input order and values.

```rust
layers.json("file", "{}")?;
assert_eq!(layers.entries().next().unwrap().0, "file");
```

**Returns** A borrowing iterator over original labels/objects, not a merged document. No cloning or
serialization is performed.

## 2. Budgets and decoding

Limits use Lib defaults: max_bytes 1 MiB, max_entries 10,000, max_depth 64. At most 1,024 external
layers are retained. Each object and the aggregate merged output must satisfy Nest limits and
encoded JSON length. Cumulative encoded input bytes are bounded too, even when later layers
overwrite the same keys; json also checks raw text bytes before parsing.

Build checks aggregate defaults, every merged layer, and unknown top-level sections before typed
decoding/validators. A section's serde policy controls unknown inner fields. Decoding/validation
errors retain section/type/plugin/site and last contributing layer metadata where available; no
config payload is automatically logged. See [Lib parsing limits](../../../../lib/docs/api/parsers.md).

Related: [typed config guide](../guides/typed-config.md), [builder](builder.md).
