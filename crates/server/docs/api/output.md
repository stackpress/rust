# OutputGuard API

Adapter-owned one-attempt authority, kept outside handler-visible Response replacement. Source:
[output.rs](../../src/output.rs). OutputGuard is Default, non-Clone, and starts Open.

```rust
use stackpress_lib::{DeliveryState, Response};
use stackpress_server::OutputGuard;
let mut guard = OutputGuard::default();
let mut response = Response::new();
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Native authority](#1-native-authority)

## 1. Native authority

A handler may replace the whole Response; an adapter must retain this guard separately. The guard
controls only its writer. Arbitrary application writers and external effects stay outside its scope.

### 1.1. state

`state(&self) -> DeliveryState`

Inspect adapter attempt history.

```rust
assert_eq!(guard.state(), DeliveryState::Open);
```

**Returns** A copy of adapter state, independent of current Response delivery state.

### 1.2. dispatch

`async fn dispatch<F, Fut>(&mut self, response: &mut Response, writer: F) -> Result<()>`

Consume one native output opportunity, or honor explicit suppression. F is FnOnce(ResponseView) ->
Fut + Send; Fut: Future<Output = Result<()>> + Send.

```rust
response.results(&"hello")?;
guard
    .dispatch(&mut response, |view| async move {
        assert!(!matches!(view.body, stackpress_lib::Body::Empty));
        Ok(())
    })
    .await?;
assert_eq!(guard.state(), DeliveryState::Sent);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `response` | `&mut Response` | Open permits attempt; Stopped suppresses without calling writer. |
| `writer` | `F` | One borrowed-scope async writer; it receives an owned ResponseView. |

**Returns** Unit or consumed-authority/foreign-finalization/response-validation/writer Error. Both
owners become Attempted before awaiting writer. Success becomes Sent, error Failed; a dropped future
retains Attempted and cannot retry. Stopped consumes guard as Stopped. Foreign Sent/Attempted/Failed
output fails this guard instead of repeating delivery. Synchronous writers may block and partial
writes cannot be rolled back.

Related: [reports](reports.md), [HTTP](http.md), [Lib delivery](../../../lib/docs/api/response.md).
