# ServerBuilder API

Owns plugin selection, configuration contributions, composition, and consuming startup.
ServerBuilder is Default and non-Clone. Source: [builder.rs](../../src/builder.rs). For handler and
route methods shared with StartupContext, use [registration](registration.md).

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ConfigLayers, PluginSpec, ServerBuilder};

async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"hello")?;
    Ok(Flow::Continue)
}
fn greetings(app: &mut ServerBuilder) -> Result<()> {
    app.on("greet", greet)?;
    Ok(())
}
fn validate(_: &u64) -> Result<()> {
    Ok(())
}
let mut app = ServerBuilder::new();
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Construction](#1-construction)

## 1. Construction

Direct contributions precede all queued plugin callbacks. Configuration registration and selected
callbacks finish before async startup.

### 1.1. new

`ServerBuilder::new() -> Self`

Create an empty host accumulator. Default is equivalent.

```rust
let app = ServerBuilder::new();
```

**Returns** An empty builder; no executor, listener, native resource, or process input is acquired.

### 1.2. plugin

`plugin(&mut self, id: &str, register: fn(&mut ServerBuilder) -> Result<()>) -> Result<&mut Self>`

Select a compiled registration function at application-defined order.

```rust
app.plugin("greetings", greetings)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&str` | Unique identity, 1–256 UTF-8 bytes without control characters. |
| `register` | `fn(&mut ServerBuilder) -> Result<()>` | Plain function, queued until build. |

**Returns** The same mutable builder for chaining, or invalid identity, duplicate ID, sealed
selection, or plugin-budget error. At most 1,024 plugins may be selected. The callback has not run
on success.

### 1.3. plugin_with

`plugin_with(&mut self, spec: PluginSpec, register: fn(&mut ServerBuilder) -> Result<()>) -> Result<&mut Self>`

Select a plugin with explicit prerequisites.

```rust
app.plugin_with(
    PluginSpec::new("greetings").after_if_present(&["metrics"]),
    greetings,
)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `spec` | `PluginSpec` | Identity and required/optional-present predecessors. |
| `register` | `fn(&mut ServerBuilder) -> Result<()>` | Compiled contribution function. |

**Returns** The mutable builder or selection/spec error. Missing requirements, cycles, and wrong
supplied order are rejected by build before callbacks, not resolved by loading or reordering code.

### 1.4. config_layers

`config_layers(&mut self, layers: ConfigLayers) -> Result<&mut Self>`

Append external layers after previously supplied layers.

```rust
let mut layers = ConfigLayers::new();
layers.json("app", r#"{"limit":5}"#)?;
app.config_layers(layers)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `layers` | `ConfigLayers` | Consumed ordered object inputs; defaults still precede them. |

**Returns** The mutable builder, or aggregate layer/merge budget error without changing accepted
receiver layers. Unknown sections are checked at build.

### 1.5. config_section

`config_section<T>(&mut self, key: &str, defaults: T, validate: fn(&T) -> Result<()>) -> Result<()>`

Claim one typed section; T: Serialize + DeserializeOwned + Send + Sync + 'static.

```rust
app.config_section("limit", 3_u64, validate)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `key` | `&str` | Unique literal top-level identity, 1–256 bytes without controls. |
| `defaults` | `T` | Serializable lowest-precedence section; need not itself be an object. |
| `validate` | `fn(&T) -> Result<()>` | Semantic check after final decoding, before startup. |

**Returns** Unit or duplicate name/type, invalid identity, serialization, or default-budget error.
Decode/validator errors become BuildError during build. Each concrete type can be claimed once.

### 1.6. extend

`extend(&mut self, other: Self) -> Result<&mut Self>`

Move another unbuilt builder into this one while retaining contribution IDs/sites/owners.

```rust
let mut other = ServerBuilder::new();
other.on("greet", greet)?;
app.extend(other)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `other` | `ServerBuilder` | Consumed selection, config, listeners, routes, and phase tasks. |

**Returns** The mutable receiver, or sealed/count, duplicate plugin/section/type, config-budget, or
foundation composition error. Phase and listener insertion order is preserved; the donor is consumed
even on error. Its correlation prefix is not transferred.

### 1.7. correlation_prefix

`correlation_prefix(&mut self, prefix: &str) -> Result<&mut Self>`

Label this host's local correlation sequence.

```rust
app.correlation_prefix("profiles")?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `prefix` | `&str` | 1–256 bytes without controls; last accepted call wins. |

**Returns** The mutable builder or invalid-identity error. IDs become prefix:sequence, starting at
one; the prefix does not establish global uniqueness.

### 1.8. build

`async fn build(self) -> BuildResult<Server>`

Consume all registration and Config → Listen → Route tasks before publication.

```rust
app.plugin("greetings", greetings)?;
let server = app.build().await?;
assert_eq!(server.plugins().count(), 1);
```

**Returns** A fully constructed immutable Server, or BuildError with stage/plugin/label/site and
original Error. Missing/cyclic/wrong-order dependencies, decode/validation failure, task error, or
task Abort fail construction. Dropped futures drop the builder; no retryable/partial host escapes.
Callback panics propagate and completed external effects are not rolled back.

Related: [plugin guide](../guides/writing-a-plugin.md), [startup](startup.md),
[configuration](config.md).
