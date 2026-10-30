# Own signup hooks and shutdown

Signup returns without awaiting its extension jobs. Bind the native Hooks service to the finished server/runtime, then let the lifecycle owner drain outcomes or stop and reap the group. This keeps the source nonawaited behavior with explicit Rust ownership.

## Build, bind and finish

The auth registrar installs Hooks even without auth route config. Bind the service after `Arc<Server>` exists; retaining a weak server reference avoids a service/server ownership cycle.

<!-- example: server -->
```rust
use stackpress_framework_session::auth::{self, Hooks};
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;
use std::sync::Arc;

pub async fn owned_hooks() -> Result<()> {
    let mut app = ServerBuilder::new();
    app.plugin("auth", auth::register)?;
    let server = Arc::new(app.build().await.map_err(|failure| failure.error)?);
    let hooks = server.host().service::<Hooks>()?;
    hooks.bind(&server, tokio::runtime::Handle::current());
    assert_eq!(hooks.pending(), 0);
    let outcomes = hooks.drain().await;
    assert!(outcomes.is_empty());
    hooks.shutdown().await;
    Ok(())
}
```

This finite host runs no hooks or listener. With signup, submissions before binding stay queued; binding schedules them in submission order. Completion order belongs to the runtime. [Hooks source](../../src/auth/hooks.rs) and [externally bounded supervision proof](../../tests/main/supervision.rs) cover ownership, cancellation and nested work.

## Keep source hook names and payload timing

Signup submits `email-email-send` and then `auth-phone-verify`, each with its own owned request/response. `email-email-send` is the original extension hook name and is distinct from the transport's `email-send`. Payloads are captured before known email/phone/username secrets are removed from the parent result; no unknown extension auth record is silently scrubbed. [Signup event source](../../src/auth/events.rs) and [Lane B hook tests](../../../framework-email/tests/cases/event_supervision.rs) pin this order.

## Observe or stop jobs

`Hooks::drain() -> Vec<TaskOutcome>` joins the currently taken group and retains job failures. New submissions belong to the next group. Canceling the wait aborts that taken group; the explicit `shutdown().await` barrier aborts/joins current jobs and discards unscheduled work. Stop calls before shutdown, and invoke these lifecycle waits from the owner rather than a hook joining itself. See [Hooks API](../api/auth.md#hooks) and [email task ownership](../../../framework-email/docs/api/tasks.md).
