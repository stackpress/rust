# Choose a transport

Choose output first: inspect JSON, consume MIME, or deliver through an adapter. Configuration selects the transport before a message is read; the same direct service and reusable event use these variants.

## Construct every source variant

The following catalog constructs ordinary `EmailConfig` values without delivery. SMTP and connection strings select the SMTP path; stream appears in both output modes, and JSON in both encoding modes.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::{EmailConfig, EmailResult};

pub fn transport_configs() -> EmailResult<Vec<EmailConfig>> {
    let values = [
        json!({"host": "relay.example.test", "port": 587, "secure": false}),
        json!({"pool": true, "host": "relay.example.test", "maxConnections": 5,
            "maxMessages": 100, "rateDelta": 1000, "rateLimit": 0}),
        json!({"jsonTransport": true, "skipEncoding": true}),
        json!({"jsonTransport": true, "skipEncoding": false}),
        json!({"streamTransport": true, "buffer": true, "newline": "windows"}),
        json!({"streamTransport": true, "buffer": false}),
        json!({"sendmail": true, "path": "sendmail", "args": []}),
        json!({"SES": {}}),
        json!("smtps://user:password@relay.example.test:465/?tls.servername=mail.example.test"),
    ];
    values
        .into_iter()
        .map(|value| Ok(serde_json::from_value(value)?))
        .collect()
}
```

Constructing these values performs no IO. Pool requires a complete provider, SES requires a native client binding, and sending through the default SMTP/sendmail drivers performs external IO. The fake-driver example below keeps those boundaries inert.

| Choice | Behavior to expect |
| --- | --- |
| SMTP | One connection; supported TLS/auth options and partial recipient results; generated MIME omits Bcc |
| Pool | Complete original config goes to `with_provider`, including when maxMessages uses its source default |
| JSON | No delivery; `skipEncoding` selects object versus JSON string |
| Stream | No delivery; `buffer` selects bytes versus a reader with deferred compilation/errors |
| Sendmail | `-i`, explicit args or `-f from`, then recipients; explicit empty args suppress `-f` |
| SES | Native `SesClient` supplies SDK behavior; generated command fields are followed by arbitrary message.ses overrides |
| String | smtp/smtps/direct URL mapping, auth/query/TLS fields; no separate string transport type |

Pool `maxConnections || 5` and `maxMessages || 100` use source truthiness. Its complete-provider requirement is a Rust library-capability adaptation, not a new source option restriction. See [configuration](configuration.md) for defaults and precedence.

## Replace delivery drivers for a finite example

A driver binding preserves the built-in option/MIME/result mapping while replacing just its IO. These in-memory fakes demonstrate SMTP, sendmail and SES together; they acknowledge requests without delivering mail.

<!-- example: direct -->
```rust
use serde_json::{Value, json};
use stackpress_framework_email::backend::{
    ProcessBackend, ProviderFuture, SendmailRequest, SesClient, SesRequest, SmtpBackend,
    SmtpRequest,
};
use stackpress_framework_email::{
    EmailConfig, EmailResult, NativeFactory, SentMessageInfo, Service,
};
use std::sync::Arc;

struct InMemoryIo;
impl SmtpBackend for InMemoryIo {
    fn send(&self, request: SmtpRequest) -> ProviderFuture<'_, SentMessageInfo> {
        Box::pin(async move {
            assert!(!request.message.is_empty());
            Ok(json!({"accepted": ["reader@example.test"], "response": "250 example"}).into())
        })
    }
}
impl ProcessBackend for InMemoryIo {
    fn run(&self, request: SendmailRequest) -> ProviderFuture<'_, Option<i32>> {
        Box::pin(async move {
            assert_eq!(request.args[0], "-i");
            Ok(Some(0))
        })
    }
}
impl SesClient for InMemoryIo {
    fn send(&self, request: SesRequest) -> ProviderFuture<'_, Value> {
        Box::pin(async move {
            assert!(request.input["Content"]["Raw"].is_object());
            Ok(json!({"MessageId": "example-id"}))
        })
    }
}

pub async fn inspect_delivery_mapping() -> EmailResult<()> {
    let io = Arc::new(InMemoryIo);
    let factory = Arc::new(
        NativeFactory::default()
            .with_smtp(io.clone())
            .with_process(io.clone())
            .with_ses(io),
    );
    for value in [
        json!({"host": "relay.example.test"}),
        json!({"sendmail": true}),
        json!({"SES": {}}),
    ] {
        let config: EmailConfig = serde_json::from_value(value)?;
        let service = Service::new(config, factory.clone());
        let info = service
            .send(serde_json::from_value(json!({
                "from": "sender@example.test", "to": "reader@example.test", "text": "Example"
            }))?)
            .await?;
        assert!(info.fields.get("messageId").is_some());
    }
    Ok(())
}
```

`with_smtp`, `with_process` and `with_ses` select public seams. The SES default region method yields None, which the source adapter maps to us-east-1. Driver results still pass through each built-in result projection; fake acknowledgements establish mapping, not remote delivery.

## Bind capabilities outside the built-in adapter

Use `with_provider(Arc<dyn TransportFactory>)` for pool recycling, LMTP, callback/socket controls and other listed library gaps. A complete provider receives the original source configuration and message options/native content before those readers are consumed. Alternatively, pass your factory directly to `Service::new` to own the entire path.

The [provider example](../api/service.md#implement-a-per-call-factory) shows an inspecting fake at that complete boundary. An application binds its actual native provider there when delivery is required. No configured binding means a clear capability error; [source behavior](../source-behavior.md) lists supported mappings, compiler differences and live-provider limits.
