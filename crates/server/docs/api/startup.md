# Startup API

Config, Listen, and Route are consuming builder-time tasks. They are separate from runtime events.
Source: [startup.rs](../../src/startup.rs); shared listener methods:
[registration](registration.md).

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, StartupContext, StartupSpec, StartupStage};
use std::sync::Arc;

async fn configure(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new("ready".to_owned()))?;
    Ok(Flow::Continue)
}
async fn ready(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&true)?;
    Ok(Flow::Continue)
}
// Listen and Route contribute only the registrations permitted in their phase.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("ready", ready)?;
    Ok(Flow::Continue)
}
async fn route(start: &mut StartupContext) -> Result<Flow> {
    start.get("/ready", ready)?;
    Ok(Flow::Continue)
}
let mut app = ServerBuilder::new();
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Stages and errors](#1-stages-and-errors) · [2. Scheduling](#2-scheduling) · [3.
Context access](#3-context-access)

## 1. Stages and errors

StartupStage is Clone/Copy/Debug/Eq/PartialEq: Registration, Config, Listen, Route. Registration is
the selected-function stage, not a queued task phase. A task is a Lib TaskHandler<StartupContext>
with a Send borrowing future; named async functions register directly.

BuildResult<T> aliases `std::result::Result<T, BuildError>`. BuildError implements Debug, Display,
and std::error::Error, with public fields `stage: StartupStage`, `plugin: Option<String>`, `label:
String`, `registered: SourceLocation`, and `error: stackpress_lib::Error`. Display includes the
underlying developer error message; source() preserves that Error. It is not a public wire view.
`From<Error>` uses Registration/application metadata and the conversion caller site.

## 2. Scheduling

Higher priorities run first within a phase; ties preserve task insertion order. Scheduling occurs
during direct/selected registration, before queues are consumed. Invalid labels fail before
insertion.

### 2.1. StartupSpec::new

`StartupSpec::new(stage: StartupStage) -> Self`

Describe a zero-priority step.

```rust
let spec = StartupSpec::new(StartupStage::Config);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `stage` | `StartupStage` | Config, Listen, or Route; Registration fails at startup insertion. |

**Returns** A spec with label equal to the stage's Debug spelling. Its fields are private.

### 2.2. priority

`priority(self, priority: i64) -> Self`

Choose priority inside this phase.

```rust
let spec = StartupSpec::new(StartupStage::Config).priority(10);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `priority` | `i64` | Signed priority; higher first. |

**Returns** The owned spec for chaining, without changing phase order.

### 2.3. label

`label(self, label: impl Into<String>) -> Self`

Name the step for build diagnostics.

```rust
let spec = StartupSpec::new(StartupStage::Config).label("open-resource");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `label` | `impl Into<String>` | Validated at insertion: 1–256 bytes, no controls. |

**Returns** The owned spec; setting a label alone is infallible.

### 2.4. ServerBuilder::startup

`startup(&mut self, spec: StartupSpec, step: impl TaskHandler<StartupContext>) -> Result<()>`

Queue an explicitly described step.

```rust
app.startup(
    StartupSpec::new(StartupStage::Config).label("greetings"),
    configure,
)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `spec` | `StartupSpec` | Phase, priority, and diagnostic label. |
| `step` | `impl TaskHandler<StartupContext>` | Owned Send + Sync callback; future borrows exclusive context. |

**Returns** Unit or invalid label/Registration-phase error. Plugin identity and actual caller site
are captured automatically.

### 2.5. ServerBuilder::on_config

`on_config(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()>`

Queue one zero-priority Config step.

```rust
app.on_config(configure)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `step` | `impl TaskHandler<StartupContext>` | Named async callback or Lib task helper; queued, not run now. |

**Returns** Unit or scheduling error. The default label is Config.

### 2.6. ServerBuilder::on_listen

`on_listen(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()>`

Queue one zero-priority Listen step.

```rust
app.on_listen(listen)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `step` | `impl TaskHandler<StartupContext>` | Named async callback or Lib task helper; queued, not run now. |

**Returns** Unit or scheduling error. The default label is Listen.

### 2.7. ServerBuilder::on_route

`on_route(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()>`

Queue one zero-priority Route step.

```rust
app.on_route(route)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `step` | `impl TaskHandler<StartupContext>` | Named async callback or Lib task helper; queued, not run now. |

**Returns** Unit or scheduling error. The default label is Route.

## 3. Context access

Services mutate only in Config; reads work during startup. Named services are separate from typed
singletons. The following snippets define independent Config callbacks to show the required context.

### 3.1. stage

`stage(&self) -> StartupStage`

Inspect current mutation permissions.

```rust
async fn inspect(start: &mut StartupContext) -> Result<Flow> {
    assert_eq!(start.stage(), StartupStage::Config);
    Ok(Flow::Continue)
}
```

**Returns** The active startup stage by value.

### 3.2. config

`config<T: Any + Send + Sync>(&self) -> Result<&T>`

Borrow a registered typed config section.

```rust
async fn inspect(start: &mut StartupContext) -> Result<Flow> {
    let _limit = start.config::<u64>()?;
    Ok(Flow::Continue)
}
```

**Returns** Borrowed T or missing-type Error. Any implies a static concrete type; lookup does not
read request/process data.

### 3.3. register

`register<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()>`

Insert a typed singleton during Config.

```rust
async fn inspect(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new("ready".to_owned()))?;
    Ok(Flow::Continue)
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `Arc<T>` | Explicit shared ownership of a Send + Sync + static resource. |

**Returns** Unit or duplicate-type/outside-Config error; an existing service is not replaced.

### 3.4. register_named

`register_named<T: Any + Send + Sync>(&mut self, name: &str, value: Arc<T>) -> Result<()>`

Insert a named resource during Config.

```rust
async fn inspect(start: &mut StartupContext) -> Result<Flow> {
    start.register_named("primary", Arc::new("ready".to_owned()))?;
    Ok(Flow::Continue)
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | 1–256 bytes, no controls; unique across all named types. |
| `value` | `Arc<T>` | Named shared resource. |

**Returns** Unit or phase/name/duplicate-name error. Typed singleton slots remain independent.

### 3.5. replace_service

`replace_service<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()>`

Customize an existing typed singleton during Config.

```rust
async fn inspect(start: &mut StartupContext) -> Result<Flow> {
    start.replace_service(Arc::new("custom".to_owned()))?;
    Ok(Flow::Continue)
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `Arc<T>` | Replacement for existing concrete type T. |

**Returns** Unit or outside-Config/missing-type error. Records original and latest replacement
sites; old application-retained Arcs remain valid.

### 3.6. service

`service<T: Any + Send + Sync>(&self) -> Result<&T>`

Read an earlier constructed singleton.

```rust
async fn inspect(start: &mut StartupContext) -> Result<Flow> {
    let _service = start.service::<String>()?;
    Ok(Flow::Continue)
}
```

**Returns** Borrowed T or missing/wrong-type Error. No registry lock or payload Clone.

### 3.7. service_named

`service_named<T: Any + Send + Sync>(&self, name: &str) -> Result<&T>`

Read a named instance with a checked concrete type.

```rust
async fn inspect(start: &mut StartupContext) -> Result<Flow> {
    let _service = start.service_named::<String>("primary")?;
    Ok(Flow::Continue)
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Exact named slot. |

**Returns** Borrowed T or missing/wrong-type Error; it does not fall back to the singleton.

Related: [services guide](../guides/services.md), [host context](context.md), [builder](builder.md).
