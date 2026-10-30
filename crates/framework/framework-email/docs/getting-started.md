# Inspect your first email result

You can check a message's options and normalized output before choosing a delivery provider. Start with the local JSON transport: it returns inspectable data and sends nothing.

Use the [direct consumer manifest](../consumers/direct/Cargo.toml) as a dependency template with `stackpress-framework-email`, `serde_json` and Tokio's macros/runtime. Its paths assume its checked-in location; adjust path dependencies for another application. The example below is an async function to call from your runtime, with no `server` feature.

## Send through JSON

Decode a source-shaped configuration, bind the compiled factory and send one ordinary message. Choose `skipEncoding` to inspect an object rather than an encoded JSON string.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::{EmailConfig, EmailResult, NativeFactory, Service};
use std::sync::Arc;

pub async fn first_email() -> EmailResult<()> {
    let config: EmailConfig = serde_json::from_value(json!({
        "jsonTransport": true, "skipEncoding": true
    }))?;
    let mail = Service::new(config, Arc::new(NativeFactory::default()));
    let info = mail
        .send(serde_json::from_value(json!({
            "from": "sender@example.test",
            "to": "reader@example.test",
            "text": "Hello from Stackpress"
        }))?)
        .await?;
    assert_eq!(info["message"]["text"], "Hello from Stackpress");
    Ok(())
}
```

`Service` asks its factory for a fresh mutable transport on each call. The assertion checks the normalized text in the returned `SentMessageInfo`; no socket or external process is opened by JSON transport. A generated Message-ID also appears in the result.

## What just happened

`EmailConfig` keeps the source object or connection string, including extension fields. `send` accepts a map of message fields and returns the provider's info without inventing a success DTO. `skipEncoding: false` or omission makes JSON transport's `message` a JSON string instead of an object.

You have now exercised the same per-call service used by real adapters. Read [concepts](concepts.md) for ownership and error behavior, [choose a transport](guides/transports.md) for every variant, or [register and supervise](guides/events-and-hooks.md) for events.
