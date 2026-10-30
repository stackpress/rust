# Native adapters and IO bindings

`NativeFactory::default()` selects compiled native adapters without IO at construction. The [finite driver example](../guides/transports.md#replace-delivery-drivers-for-a-finite-example) binds all three delivery seams; the [complete-provider example](service.md#implement-a-per-call-factory) covers library gaps.

## Factory bindings

The factory implements TransportFactory and Clone. Each builder method consumes self and returns the configured NativeFactory, enabling chaining.

| Method | Parameter | Responsibility |
| --- | --- | --- |
| `with_smtp` | `Arc<dyn SmtpBackend>` | Replace SMTP IO while keeping built-in mapping |
| `with_process` | `Arc<dyn ProcessBackend>` | Replace sendmail execution/exit status |
| `with_ses` | `Arc<dyn SesClient>` | Supply source native SES client/command behavior |
| `with_content` | `Arc<dyn ContentResolver>` | Replace file/URL reads |
| `with_provider` | `Arc<dyn TransportFactory>` | Complete source capability fallback before content is consumed |

**Returns:** the updated factory. `create(&self, &EmailConfig)` returns an owned Transport or a config/capability error. Unsupported source options are routed to the configured provider or return a named error; source-ignored fields stay unused. No binding silently strips unsupported capabilities.

## Backend request and result contracts

Import these types from `backend`. All backend traits are Send + Sync; `ProviderFuture<'a, T>` is a pinned boxed Send future returning EmailResult<T> and may borrow its provider for `'a`.

| Trait operation  |  Owned request |
| ---  |  --- |
| `SmtpBackend::send(&self, SmtpRequest)`  |  `options: SmtpOptions`, `envelope: Value`, `message: Vec<u8>` |
| `ProcessBackend::run(&self, SendmailRequest)`  |  `program: String`, `args: Vec<String>`, `message: Vec<u8>` |
| `SesClient::region(&self)`  |  No request |
| `SesClient::send(&self, SesRequest)`  |  `input: Value`, `raw: Vec<u8>` |

**Returns:** SMTP yields `ProviderFuture<SentMessageInfo>`; process yields `ProviderFuture<Option<i32>>` (None for signal termination); SES region yields `ProviderFuture<Option<String>>` with None by default; SES send yields `ProviderFuture<Value>`. Native errors are retained in each result. Native SES uses failed/missing/empty region as us-east-1, then formats source Message-ID and raw output. Sendmail keeps source leading-dash rejection and None/0 success, named 127 and other exit errors. The [source transport table](../source-behavior.md) details projections and native framing differences.

`LettreBackend` and `NativeProcessBackend` are the default concrete drivers. SMTP connects only during send. Process IO uses argv without a shell, closes stdin and awaits child completion; pending cancellation requests kill-on-drop without claiming synchronous reaping. No live delivery is part of documentation verification.

## Resolve file or URL content

Import `ContentResolver`, `ContentRequest`, `ContentFuture` and `NativeContentResolver` from `content`. A resolver is Send + Sync and implements `read(&self, request: ContentRequest) -> ContentFuture<'_>`.

| ContentRequest field | Meaning |
| --- | --- |
| `location: String` | Selected file path, HTTP(S) URL or data URI |
| `options: Map<String, Value>` | Original node/extension fields |
| `http_headers: bool` | Whether this source MIME read honors httpHeaders |

**Returns:** ContentFuture resolves owned Vec<u8> or a native error. The default resolver performs native IO only when read. The fake below shows an application replacement that still passes through native MIME mapping.

<!-- example: direct -->
```rust
use stackpress_framework_email::content::{ContentFuture, ContentRequest, ContentResolver};
use stackpress_framework_email::{EmailResult, NativeFactory};
use std::sync::Arc;

struct ExampleContent;
impl ContentResolver for ExampleContent {
    fn read(&self, request: ContentRequest) -> ContentFuture<'_> {
        Box::pin(async move {
            assert_eq!(request.location, "/example/report.txt");
            Ok(b"example report".to_vec())
        })
    }
}

pub fn factory_with_content() -> EmailResult<NativeFactory> {
    Ok(NativeFactory::default().with_content(Arc::new(ExampleContent)))
}
```

Only a later content read invokes this fake. Source MIME access flags, JSON bypass and URL behavior are documented in [configuration](../guides/configuration.md#file-and-url-resolution).

## SMTP error fields

`SmtpError` is re-exported at the crate root. Its public `fields: Value` retains source code/command/responseCode/response and applicable rejected/rejectedErrors details; Display retains the source-shaped explanation and Error::source retains the lettre cause. Downcast the native EmailError to inspect these fields.

The built-in SDK does not expose source EHLO response lines or every protocol/compiler capability. Those limits, TLS identity mechanics and complete-provider choices are recorded in [source behavior](../source-behavior.md); a fake or compiled example establishes no remote negotiation/timing guarantee.
