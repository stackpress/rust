# Check a submitted process

Call validation inside the route or event that needs it. A named async handler keeps the application's abort decision next to its successful work.

## Validate before acting

Select the `server` feature and register the service as shown in [registration](register-and-configure.md). This handler can then retrieve it through the public invocation API. Import `ServerInvocationExt`, which supplies the typed `service` method.

<!-- example: server -->
```rust
use serde_json::json;
use stackpress_framework_csrf::Service;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;

pub async fn submit(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let csrf = invocation.service::<Service>()?;
    if !csrf.valid(request, response)? {
        return Ok(Flow::Abort);
    }
    response.results(&json!({"saved": true}))?;
    csrf.clear(request, response)?;
    Ok(Flow::Continue)
}
```

The false branch retains the service's source 419 response and returns the event runtime's explicit abort. On success, this example sets a local result and clears CSRF after its work. Replace that result with the process you own and choose its clear point deliberately.

## Keep handler composition explicit

Register `submit` with `app.on("form-submit", submit)` or your own selected route/event. Multiple matching handlers are allowed; priorities run first and equal priorities retain registration order. Any handler may abort. The framework adds no route ownership or conflict arbitration.

A form-display process calls `generate` separately and renders the projected fields. The service API does not choose a renderer or a form page; see [Service reference](../api/service.md) for mutation and error order.
