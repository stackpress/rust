# Getting started

A small operation should be callable before you choose an HTTP server or command interface. Here you
will register a greeting handler, give it plain input, and print `Hello, Ada` from its response.

## Quick start

Use Rust 1.96 or newer and this local checkout. Create a disposable consumer outside the repository:

```sh
cargo new --bin /tmp/stackpress-lib-hello --edition 2024
```

This creates a separate application with its own manifest. In
`/tmp/stackpress-lib-hello/Cargo.toml`, add these entries under `[dependencies]`, replacing the path
with your checkout's absolute path:

```toml
stackpress-lib = { path = "/path/to/stackpress/rust/crates/lib" }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Tokio runs this example's async `main`; the library itself does not choose an executor. Replace the
consumer's `src/main.rs` with:

```rust
use stackpress_lib::{
    DispatchOutcome, EmitterBuilder, Flow, Invocation, Request, Response, Result,
};

async fn greet(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Read typed operation input; absence has an explicit fallback.
    let name = req
        .data
        .path_as::<String>("name")?
        .unwrap_or_else(|| "world".into());
    res.results(&format!("Hello, {name}"))?;
    Ok(Flow::Continue)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Finish registration before consuming the builder into an immutable table.
    let mut builder = EmitterBuilder::new();
    builder.on("greet", greet)?;
    let emitter = builder.build();

    // Resolve owns a fresh request and response for this capability call.
    let call = emitter
        .resolve("greet", serde_json::json!({"name": "Ada"}))
        .await?;
    assert_eq!(call.outcome, DispatchOutcome::Completed);
    let result = call.response.results.expect("greet sets results");
    println!("{}", result.as_str().expect("greet returns text"));
    Ok(())
}
```

`greet` receives mutable input/output and immutable invocation metadata. `results` stores the
greeting, and `Flow::Continue` permits any later matching listeners to run. The assertions make the
expected behavior visible if the example changes.

Run the consumer:

```sh
cargo run --manifest-path /tmp/stackpress-lib-hello/Cargo.toml
```

Its output includes `Hello, Ada`. A compiler failure at registration usually means the handler
signature or its returned future does not satisfy the
[handler contract](api/event.md#1-handler-contract).

## What just happened

`on("greet", greet)` registered an exact event at priority zero. `build()` moved that registration
into a table that supports concurrent shared access. No listener can be added to the finished table.

`resolve` converted the JSON object into a `Request`, created a `Response`, and awaited every
matching listener in order. It returned a `Resolution`: `outcome` describes execution, while
`response` contains the status and results. `results` selected status 200 and total 1; completed
dispatch alone would not set a status.

The `?` operators propagate ordinary failures. A wrong type in `name`, a failed response conversion,
or a handler error is different from an event with no listeners. See
[execution outcomes](concepts.md#outcomes-and-phases) before using these results as a transport
response.

## Next steps

You have made a named async handler callable with plain data and observed its result. Add a
[route that calls this operation](guides/events-and-routing.md), or learn how to
[retain the exact request and response with `emit`](concepts.md#emit-resolve-and-nesting).

For lookup, use [events](api/event.md), [requests](api/request.md), and
[responses](api/response.md).
