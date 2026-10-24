# Register plugins through ordered startup work

Runtime registrations are immutable, but startup can still collect contributions in stages. Put the
builder in caller-owned startup state, run ordered tasks against it, and build the runtime table
only after successful startup.

## Define one plugin contribution

A plugin can be an ordinary registration function. Add a named operation and its registration
function:

```rust
use stackpress_lib::queue::{QueueOutcome, TaskQueue};
use stackpress_lib::{
    EmitterBuilder, EventEmitter, Flow, Invocation, Request, Response, Result,
};

async fn health(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"ready")?;
    Ok(Flow::Continue)
}

fn health_plugin(builder: &mut EmitterBuilder) -> Result<()> {
    builder.on("health", health)?;
    Ok(())
}
```

The registration function owns no runtime listener and requires no plugin trait. Calling it adds a
contribution to a builder that the host still controls.

## Sequence startup, then build

Define the mutable startup state and named tasks in the same consumer:

```rust
struct Startup {
    builder: EmitterBuilder,
    configured: bool,
}

async fn configure(state: &mut Startup) -> Result<Flow> {
    // Finish configuration before the lower-priority registration task runs.
    state.configured = true;
    Ok(Flow::Continue)
}

async fn register(state: &mut Startup) -> Result<Flow> {
    assert!(state.configured);
    health_plugin(&mut state.builder)?;
    Ok(Flow::Continue)
}

async fn bootstrap() -> Result<EventEmitter> {
    let mut state = Startup {
        builder: EmitterBuilder::new(),
        configured: false,
    };
    let mut tasks = TaskQueue::new();
    tasks.add(configure, 20)?.add(register, 10)?;
    let outcome = tasks.run(&mut state).await?;
    if outcome != QueueOutcome::Completed {
        // This host chooses not to expose a partially registered runtime table.
        return Err(stackpress_lib::Error::new("startup did not complete"));
    }
    Ok(state.builder.build())
}
```

Higher priority makes configuration finish first. The host's explicit outcome check prevents a
cooperative abort from silently becoming a usable partial runtime. This is an application
composition example, not an automatic `config` → `listen` → `route` lifecycle supplied by the crate.

Verify the assembled table in an async function returning `Result<()>`:

```rust
let emitter = bootstrap().await?;
let call = emitter.resolve("health", serde_json::json!({})).await?;
assert_eq!(call.response.results.unwrap().as_str(), Some("ready"));
```

The returned value proves registration happened before the table was built. Dispatch itself does not
rerun startup tasks.

## Understand consumed work

`TaskQueue` consumes the selected task before running before/action/after phases. An abort, error,
or dropped pending run keeps unstarted tasks queued, but the selected task is never replayed
automatically. After is a success-path hook; it is not cleanup after failure.

You can now separate mutable bootstrap work from immutable runtime dispatch. Use
[queue API](../api/queue.md) for ordering, hooks, and retained priority watermarks, or
[event API](../api/event.md) for builder composition and handles.
