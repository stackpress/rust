# Service, configuration and factories

Import these types from `stackpress_framework_email`. The [JSON quick start](../getting-started.md#send-through-json) demonstrates construction and `send` without delivery; [native content](../guides/native-content.md) demonstrates `send_native`.

## Configuration

`EmailConfig` is a cloneable serde union: `Options(Map<String, Value>)` or `ConnectionString(String)`. It retains source/extension fields and adds no defaults itself. Native resources and callbacks live in factory bindings rather than JSON.

`EmailConfig::from_section(value: Option<&Value>) -> Result<Option<EmailConfig>, serde_json::Error>` handles source activation separately from direct construction.

| Parameter/mode | Meaning |
| --- | --- |
| `value: None` or null/false/zero/empty string | Returns None |
| Empty object or other object | Active Options, fields retained |
| Nonempty string | Active ConnectionString, syntax resolved later |
| Other truthy JSON type | Serde union decoding error |

**Returns:** the decoded optional config; this function neither constructs a transport nor performs IO. Direct users can deserialize an `EmailConfig` without this activation helper.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::EmailConfig;

pub fn activation_modes() -> Result<(), serde_json::Error> {
    assert!(EmailConfig::from_section(None)?.is_none());
    assert!(EmailConfig::from_section(Some(&json!(false)))?.is_none());
    assert!(EmailConfig::from_section(Some(&json!({})))?.is_some());
    Ok(())
}
```

An empty object is truthy in the original source. Keeping that distinction prevents a documentation example from silently changing plugin activation.

## Direct operations

`Service` is Clone and holds config plus `Arc<dyn TransportFactory>`. Its constructor binds them without delivery.

| Method | Parameters | Result/errors |
| --- | --- | --- |
| `Service::new` | `config: EmailConfig`, `factory: Arc<dyn TransportFactory>` | Returns Service; no constructor IO |
| `send` | `&self`, `options: SendMailOptions` | Awaits fresh factory/transport; propagates factory/send errors |
| `send_native` | `&self`, `options: NativeSendMailOptions` | Same ownership, preserving Buffer/Readable content; propagates factory/send errors |

**Returns:** `new` returns Service. Both send methods return `EmailResult<SentMessageInfo>`. Results retain provider fields/native outputs. Dropping a send future drops its per-call transport/readers; remote effects already performed are not retried or rolled back.

`EmailError` is `Box<dyn std::error::Error + Send + Sync + 'static>` and `EmailResult<T>` is its Result alias. `MailFuture<'a>` is a pinned boxed Send future producing that result; it can borrow the one mutable transport for `'a`.

## Implement a per-call factory

`TransportFactory: Send + Sync` implements `create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>>`. The returned object owns its state and cannot borrow the factory/config. `Transport: Send` implements `send(&mut self, SendMailOptions) -> MailFuture<'_>` and may override `send_native` with `NativeSendMailOptions`.

This complete-provider fake inspects pool config and native slots without delivery. It demonstrates the binding boundary rather than an SMTP pool implementation.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::{
    EmailConfig, EmailResult, MailFuture, NativeFactory, NativeSendMailOptions, SendMailOptions,
    Service, Transport, TransportFactory,
};
use std::sync::Arc;

struct InspectFactory;
struct InspectTransport(EmailConfig);
impl TransportFactory for InspectFactory {
    fn create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>> {
        Ok(Box::new(InspectTransport(config.clone())))
    }
}
impl Transport for InspectTransport {
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
        self.send_native(options.into())
    }
    fn send_native(&mut self, options: NativeSendMailOptions) -> MailFuture<'_> {
        Box::pin(async move {
            Ok(json!({"originalConfig": self.0, "options": options.fields,
                "nativeSlotCount": options.content.len()})
            .into())
        })
    }
}

pub async fn inspect_pool_binding() -> EmailResult<()> {
    let config: EmailConfig = serde_json::from_value(json!({"pool": true, "maxMessages": 25}))?;
    let factory = NativeFactory::default().with_provider(Arc::new(InspectFactory));
    let service = Service::new(config, Arc::new(factory));
    let info = service
        .send(serde_json::from_value(json!({"text": "example"}))?)
        .await?;
    assert_eq!(info["originalConfig"]["maxMessages"], 25);
    Ok(())
}
```

The factory receives the original object, not a filtered/resolved substitute. Replace the inspecting fake with your application's native implementation when pool behavior or another complete-provider capability is required.

A transport does not need Sync or Clone. The default `Transport::send_native` delegates ordinary fields to `send` when native content is empty, and errors when native content is present. Custom transports must opt into reader support explicitly. The [ownership doctests](../../src/service.rs) reject non-Send transports/futures; [task contracts](tasks.md) reject parent borrows escaping into background work.
