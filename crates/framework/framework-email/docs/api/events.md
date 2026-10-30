# Server events and registrar

Enable `server` for these surfaces. The [named-handler guide](../guides/events-and-hooks.md#register-named-handlers) shows ordinary registration and explicit signup delegation.

## Reusable email-send process

`events::EMAIL_SEND` is exactly `email-send`. `EmailSend::new(config: Option<Value>, factory: Arc<dyn TransportFactory>)` stores the source section without validating unused transport options or performing IO.

| Method | Parameters | Errors |
| --- | --- | --- |
| `EmailSend::handle` | `&self`, `request: &mut Request`, `response: &mut Response` | Config/input conversion, factory/send and response-state errors; native causes retained |

**Returns:** `stackpress_lib::Result<Flow>`. Normal skip and success both return Continue. It skips truthy existing body, then truthy non-200 status, then missing/falsy config before converting input or constructing a transport. A status code zero is falsy. It adds no route/auth/CSRF/method/sender/recipient gate.

<!-- example: server -->
```rust
use serde_json::json;
use stackpress_framework_email::{NativeFactory, events::EmailSend};
use stackpress_lib::{Flow, Request, Response, Result};
use std::sync::Arc;

pub async fn reusable_event() -> Result<()> {
    let event = EmailSend::new(
        Some(json!({"jsonTransport": true})),
        Arc::new(NativeFactory::default()),
    );
    let mut request = Request::builder()
        .data(json!({"text": "event example"}))
        .build()?;
    let mut response = Response::new();
    assert_eq!(
        event.handle(&mut request, &mut response).await?,
        Flow::Continue
    );
    assert!(response.result().is_some());
    Ok(())
}
```

The fresh response allows the local JSON send. Using a response that already holds auth-update results would take the preserved body skip instead. Exceptions are thrown into the caller's existing lifecycle without this process manufacturing a status/error body.

## Plugin selection

Import `plugin::register` or `register_with_factory`. The raw `EmailSection(pub Value)` is a transparent serde wrapper with null as the selected missing-section default.

| Registrar | Parameters | Behavior/errors |
| --- | --- | --- |
| `register` | `app: &mut ServerBuilder` | Default NativeFactory; propagates existing contribution errors |
| `register_with_factory` | `app`, `factory: Arc<dyn TransportFactory>` | Same source listener with an application-owned factory |

**Returns:** framework Result<()>; Listen contributes email-send only for truthy config. Missing/null/false/zero/empty string are inactive and an empty object is active. Typed section/listener registration errors follow the server contract; multiple event handlers remain supported with priority/FIFO/abort semantics.

## Rich native readers

`NativeReader::new(reader: MailReader) -> NativeReader` wraps an owned Send reader. `take(&self) -> EmailResult<MailReader>` transfers once, with explicit poisoned/already-consumed errors. It never holds its short lock across IO/await.

Put `SharedNative::new(NativeReader::new(reader))` at a source content field as in [native content](../guides/native-content.md#cross-a-server-event-boundary). Result streams appear at original `results.message`; buffer outputs retain Node Buffer JSON. Snapshots share the transfer slot, and JSON conversion rejects rather than drains a live reader. Event conversion honors the request/response owner's existing parse limits.

The distinct signup hook is `email-email-send`. It is not renamed or automatically delegated by the email registrar; [owned tasks](tasks.md) describes explicit resolution.
