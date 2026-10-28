# Testing in process

Most operation tests need no network. Build the real plugin, call its capability, and assert the
public result and lifecycle report separately.

## Compare raw and handled calls

The following named async function can be called from your test runtime. Its no-op listener shows
why raw completion and a successful top-level response are different assertions:

```rust
use stackpress_lib::{DispatchOutcome, Flow, Invocation, Request, Response, Result};
use stackpress_server::{LifecycleStage, ServerBuilder};

async fn noop(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    Ok(Flow::Continue)
}

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = ServerBuilder::new();
    app.on("noop", noop)?;
    let server = app.build().await?;
    let raw = server.resolve("noop", serde_json::json!({})).await?;
    assert_eq!(raw.outcome, DispatchOutcome::Completed);
    assert!(raw.response.status.is_none());
    let reply = server.call("noop", serde_json::json!({})).await?;
    assert!(!reply.report.is_success(&reply.response));
    assert_eq!(reply.response.status().unwrap().code, 404);
    assert_eq!(reply.report.failures[0].stage, LifecycleStage::Operation);
    Ok(())
}
```

Raw resolve preserves Lib behavior and produces a Resolution envelope. `call` supplies lifecycle
policy: ordinary completion with no body/status becomes a retained 404. Neither sends native output.
Use `handle` when assertions need the exact request mutations or response limits you supplied.

## Assert the boundary you own

Check config rejection before resource startup, dependency order, explicit Abort, failed Request
guards, and deliberate recovery where your plugin uses them. For routes, assert capture collisions
and unchanged query/post stores. For adapters, also inspect Output and native availability; public
success alone cannot prove delivery.

The canonical [application tests](../../../../../examples/capability-host/tests/) compare direct calls,
route delegation, CLI JSON, and buffered HTTP. The server's [contract tests](../../tests/) cover
shared runtime behavior, cancellation, ownership, and error decision tables.

You now have a test boundary that separates dispatch from host policy. Use the [Server
reference](../api/server.md) for entry choices and the [CLI testing
guide](../../../../../tools/cli/docs/guides/testing.md) for injected process sinks.
