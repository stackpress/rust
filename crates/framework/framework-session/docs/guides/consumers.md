# Choose one of the five consumer patterns

Use a standalone consumer when you want to verify exactly what your application selects. Each checked-in consumer has its own workspace, lock and target; none relies on workspace development-feature unification.

## Compare import and composition paths

The [consumer runner](../../consumers/run.py) and [graph checker](../../consumers/graphs.py) are executable evidence for these patterns. Adjust relative path dependencies when copying a manifest to another location.

| Pattern | Manifest / executable proof | What it demonstrates |
| --- | --- | --- |
| Direct | [direct](../../consumers/direct/Cargo.toml), [source](../../consumers/direct/src/main.rs) | `stackpress_framework_session` with native JWT/schema use and no registrar |
| Facade | [facade](../../consumers/facade/Cargo.toml), [source](../../consumers/facade/src/main.rs) | `stackpress::framework_session` via opt-in forwarding features |
| Plugin | [plugin](../../consumers/plugin/Cargo.toml), [source](../../consumers/plugin/src/main.rs) | SQL-backed in-process composition, source aggregate order, API cookie-vs-credential reconciliation |
| Disabled | [disabled](../../consumers/disabled/Cargo.toml), [source](../../consumers/disabled/src/main.rs) | Default facade selects only stackpress-lib as a first-party dependency; session is absent |
| Browser | [browser](../../consumers/browser/Cargo.toml), [entry](../../consumers/browser/src/lib.rs) | One shared public page/snapshot with separate hydrate and CSR artifacts |

## Use the facade

Select root feature `framework-session-native` for the direct identity surface. The import path changes, while the underlying Service/Session contract stays the same.

<!-- example: facade -->
```rust
use stackpress::framework_session::{Service, SessionResult};

pub async fn facade_session() -> SessionResult<()> {
    let service = Service::default();
    let claims = serde_json::from_value(serde_json::json!({"id": "reader"})).unwrap();
    let token = service.create(claims).await?;
    assert!(!service.load(token).guest().await);
    Ok(())
}
```

`framework-session` exports the portable crate. Forwarders append `-native`, `-sql`, `-server`, `-auth-server`, `-leptos`, `-leptos-ssr`, `-browser`, `-leptos-hydrate` or `-leptos-csr`. All are opt-in in the [root manifest](../../../../../Cargo.toml); the [facade export](../../../../../src/lib.rs) is feature-gated. Do not combine renderer modes in an artifact.

## Verify the selection

Run `consumers/run.py` through the recorder with the absolute effort target. It checks graph, formatting, Clippy and finite executable assertions for the four native patterns, using --locked --offline. Run `consumers/graphs.py --check` for feature/facade/native/WASM boundaries; the disabled consumer is part of that proof. Browser `build.py hydrate` and `build.py csr` are separate finite builds; see [UI verification](shared-ui.md) and the [verification reference](../verification.md).

Consumer registry tuples must remain a subset of the root lock, with root bytes unchanged. Source native cryptography/SQL/services are excluded from browser graphs; native identity graphs exclude renderer/phone metadata. These are graph and finite-execution claims, not live-backend, listener or installed-platform acceptance.
