# Handling errors and recovery

A caller needs safe public output while you retain the cause of a failure. The host keeps the
original error in its report and gives error listeners fresh public error state to inspect or
rebuild.

## Rebuild successful public output

This example intentionally recovers from a temporary operation failure. It verifies public success
and diagnostic failure independently:

```rust
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{LifecycleOutcome, LifecycleStage, ServerBuilder, ServerInvocationExt};

async fn unavailable(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    Err(Error::new("private backend detail")
        .with_code(503)
        .with_public_message("Try later"))
}

async fn recover(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    assert_eq!(inv.stage()?, LifecycleStage::Error);
    assert_eq!(inv.failure()?.unwrap().stage, LifecycleStage::Operation);
    // Discard the public error field as well as its status and partial output.
    *res = Response::with_limits(res.limits());
    res.results(&"cached result")?;
    Ok(Flow::Continue)
}

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = ServerBuilder::new();
    app.on("search", unavailable)?;
    app.on("error", recover)?;
    let reply = app
        .build()
        .await?
        .call("search", serde_json::json!({}))
        .await?;
    assert!(reply.report.is_success(&reply.response));
    assert_eq!(
        reply.report.outcome,
        LifecycleOutcome::Failed {
            stage: LifecycleStage::Operation
        }
    );
    assert_eq!(reply.report.failures.len(), 1);
    assert!(
        !reply
            .response
            .try_to_json()?
            .to_string()
            .contains("private backend detail")
    );
    Ok(())
}
```

Replacing the response removes its public error; setting only results/status would retain it. The
report still owns the original error and its frames. Log `report` for metadata, or explicitly select
`report.developer()` when developer messages and sources are appropriate.

## Keep the failure path finite

A failed Request guard never permits Operation, even after recovery. An Error Abort skips later
hooks while allowing Open output. An Error listener failure retains both errors, stops hooks, and
uses sanitized 500 output while Open. It never recursively dispatches Error. A Response failure may
get another offer, but Response itself runs at most once.

If conversion failed before a real Request existed, no callback runs: inspect the returned Input
failure and public 400. Normalized load/parse failures do get the error offer. After finalized
output, errors remain diagnostic and cannot authorize a replacement write. Output failures never
call hooks.

You can now recover public output without erasing diagnostic history. See the full [lifecycle
rules](../concepts.md#one-lifecycle-per-top-level-call), [report API](../api/reports.md), and [Lib
error disclosure](../../../../lib/docs/guides/errors-and-diagnostics.md).
