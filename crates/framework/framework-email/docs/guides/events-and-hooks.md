# Register email and supervise signup hooks

Select the event integration when application handlers should reuse email sending through the host. Signup's non-awaited extension hook has its own name and lifecycle; connect it explicitly when your application wants that composition.

## Register named handlers

Enable `server` as in the [event consumer manifest](../../consumers/event/Cargo.toml). Define the application delegation first, then register it alongside the ordinary constituent.

<!-- example: server -->
```rust
use serde_json::json;
use stackpress_framework_email::{events::EMAIL_SEND, plugin};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ConfigLayers, ServerBuilder};

pub async fn delegate(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    invocation.emit(EMAIL_SEND, request, response).await?;
    Ok(Flow::Continue)
}

pub fn configure(app: &mut ServerBuilder) -> Result<()> {
    let mut layers = ConfigLayers::new();
    layers.value("application", json!({"email": {"jsonTransport": true}}))?;
    app.config_layers(layers)?;
    app.plugin("email", plugin::register)?;
    app.on("email-email-send", delegate)?;
    Ok(())
}
```

Listen installs `email-send` for truthy config. Missing/null/false/zero/empty string are inactive; an empty object is active. This example deliberately uses a local JSON transport. Neither registration nor the event adds a request, CSRF, auth, method, sender or recipient guard.

## Choose response sharing deliberately

`invocation.emit` uses the supplied request and response. `server.resolve(EMAIL_SEND, owned_input)` owns fresh child state. The reusable `EmailSend::handle` is also available directly without registration.

The source event first skips a truthy body or a truthy non-200 status, then checks config and input. An auth-update results body causes email-send to skip in the preserved shared auth response. Fresh resolves still send; the framework does not clear prior output automatically. Normal skip/success returns Continue, so other matching handlers may still run according to priority, registration order and explicit abort.

## Own non-awaited work

Submit owned inputs under the source extension name `email-email-send`, distinct from `email-send`. The source signup submits that hook followed by `auth-phone-verify`; each event retains its own handler ordering, while independent jobs can complete in either order.

<!-- example: server -->
```rust
use serde_json::json;
use stackpress_framework_email::{EmailResult, tasks::TaskSupervisor};
use stackpress_server::Server;
use std::sync::Arc;
use tokio::runtime::Handle;

pub async fn submit_and_drain(runtime: &Handle, host: Arc<Server>) -> EmailResult<()> {
    let mut jobs = TaskSupervisor::new();
    jobs.resolve_on(
        runtime,
        host,
        "email-email-send",
        json!({
            "from": "sender@example.test", "to": "reader@example.test", "text": "Welcome"
        }),
    );
    while let Some(outcome) = jobs.join_next().await {
        outcome??;
    }
    jobs.shutdown().await;
    assert!(jobs.is_empty());
    Ok(())
}
```

Submission itself does not await delivery. This finite helper subsequently drains outcomes to show lifecycle ownership; a long-lived application retains its supervisor and chooses its later drain/shutdown point. Missing hooks resolve normally, child responses are discarded, and thrown errors remain in outcomes. Drop requests abort; awaited shutdown aborts remaining jobs and waits for resource destruction. See [task reference](../api/tasks.md) for nested error types.
