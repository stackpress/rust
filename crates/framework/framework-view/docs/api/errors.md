# ViewError

Safe public failures with separate native diagnostic causes. Source: [lib.rs](../../src/lib.rs).

```rust
use stackpress_framework_view::ViewError;
use std::error::Error;
fn main() {
    let error = ViewError::from_source(std::io::Error::other("private diagnostic"));
    assert_eq!(error.to_string(), "Could not render page");
    assert!(error.source().is_some());
}
```

## Variants and public messages

Display is safe text. Debug and explicit source traversal are developer diagnostics.

| Variant | Meaning / Display |
| --- | --- |
| `Failure(Arc<dyn Error + Send + Sync>)` | Retained cause; `Could not render page`. |
| `InvalidPolicy` | `Invalid page configuration`. |
| `ExecutorUnavailable` | `Native SSR requires initialize_ssr() inside an active Tokio runtime`. |
| `InvalidSnapshot` | `Invalid public page data`. |
| `ContractMismatch` | `Page version changed. Refresh to continue.` |
| `SnapshotLimit` | `Page data is too large`. |
| `HtmlLimit` | `Rendered page is too large`. |

`from_source(cause: impl std::error::Error + Send + Sync + 'static)` retains a cause.
**Returns** `ViewError::Failure`, without exposing the cause through Display.
`std::error::Error::source` returns Some only for Failure. Native page registration wraps this
error into the existing foundation error/lifecycle path.

The enum is Clone, Debug, PartialEq/Eq and implements Display/Error. Policy variants compare
by kind; Failure values compare by shared Arc identity, not equal cause text. It is not a
serialized public error envelope and does not trigger automatic retries or reloads.

Related: [recovery](../guides/contract-recovery.md), [SSR startup](../../../../leptos/docs/api/initialize-ssr.md).
