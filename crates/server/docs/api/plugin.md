# Plugin API

PluginSpec describes compile-time selection, not dynamic discovery. PluginDescriptor exposes
selected provenance after build. Both are Clone/Debug. Source: [plugin.rs](../../src/plugin.rs).

```rust
use stackpress_server::PluginSpec;
let spec = PluginSpec::new("profiles");
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Selection metadata](#1-selection-metadata) · [2. Descriptors](#2-descriptors)

## 1. Selection metadata

PluginSpec has no public fields. Specs are validated when selected. Each plugin has at most 1,024
required plus optional declarations; repeated declarations are accepted and graph edges
deduplicated.

### 1.1. PluginSpec::new

`PluginSpec::new(id: &str) -> Self`

Describe an identity before inserting it into a builder.

```rust
let spec = PluginSpec::new("profiles");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&str` | Later validated as 1–256 bytes with no controls. |

**Returns** A data-only spec with no prerequisites; construction itself is infallible.

### 1.2. requires

`requires(self, ids: &[&str]) -> Self`

Append required predecessors.

```rust
let spec = spec.requires(&["storage"]);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `ids` | `&[&str]` | Must be selected earlier than this plugin; self-dependency is invalid. |

**Returns** The owned spec for chaining; repeated calls append. Selection validates
names/count/self-reference; build checks presence, cycles, and order.

### 1.3. after_if_present

`after_if_present(self, ids: &[&str]) -> Self`

Append optional-present predecessors without loading them.

```rust
let spec = spec.after_if_present(&["metrics"]);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `ids` | `&[&str]` | Only selected names add ordering/cycle constraints. |

**Returns** The owned spec for chaining. An absent optional plugin is allowed; a selected optional
plugin must precede this one.

## 2. Descriptors

Server::plugins yields descriptors in selected order. Public fields are `id: String`, `required:
Vec<String>`, `after_if_present: Vec<String>`, and `registered: SourceLocation`. The site is the
application selection site. Descriptors contain no registration callback or runtime mutation
authority.

Related: [builder](builder.md), [writing a plugin](../guides/writing-a-plugin.md).
