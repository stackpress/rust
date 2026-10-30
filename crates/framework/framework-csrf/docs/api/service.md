# Service and configuration

`stackpress_framework_csrf::Service` is a cloneable configuration holder operating on `stackpress_lib::Request` and `Response`. The [round-trip example](../getting-started.md#generate-and-submit) shows generate, valid and clear together.

## Construct and inspect

`Service::default()` selects `csrf` and `DEFAULT_ERROR`. `Service::new(config: CsrfConfig) -> Service` retains supplied overrides; `service.name() -> &str` borrows the effective field name.

<!-- example: direct -->
```rust
use stackpress_framework_csrf::{CsrfConfig, DEFAULT_ERROR, Service};

pub fn configured_service() -> Service {
    let service = Service::new(CsrfConfig {
        name: Some("form_token".into()),
        error: Some(DEFAULT_ERROR.into()),
    });
    assert_eq!(service.name(), "form_token");
    service
}
```

The returned service stores no token or request. Both config fields are `Option<String>`; their serde representation is an object, and missing fields use `None`. `CsrfConfig` also implements Default, Clone, Debug, Eq and serde serialization/deserialization.

| Config field | Omitted value | Meaning |
| --- | --- | --- |
| `name` | `csrf` | Incoming cookie/data and outgoing cookie/form name |
| `error` | `DEFAULT_ERROR` | Public 419 explanation |

`DEFAULT_ERROR` is exactly: “This page may have been requested from an external source. We corrected the issue. Please try again.”

## Operations

All three operations return the existing framework `Result`. Their parameters are the request/response carriers owned by the calling process.

| Method | Parameters | Mutation and errors |
| --- | --- | --- |
| `generate` | `res: &mut Response` | OS entropy; stage cookie then `{name, token}` data. Propagates entropy/cookie/carrier/response errors, without rollback. |
| `valid` | `req: &Request`, `res: &mut Response` | Reads incoming scalar strings. A mismatch sets source 419; response/public-error validation errors propagate. |
| `clear` | `req: &mut Request`, `res: &mut Response` | Remove aggregate request field, outgoing cookie, outgoing data, in that order. Propagates existing mutation errors. |

**Returns:** `generate` returns the generated `String`; `valid` returns `bool`, false after staging 419 / Page Expired on mismatch; `clear` returns `()`. Successful validation performs no clear. The [handler guide](../guides/protect-a-process.md) shows the application's explicit abort and clear decisions.

The direct [implementation](../../src/lib.rs) and [source branch tests](../../tests/main.rs) define byte comparison and carrier behavior. See [source behavior](../source-behavior.md) for intentionally accepted arbitrary nonempty strings and incoming-cookie retention.
