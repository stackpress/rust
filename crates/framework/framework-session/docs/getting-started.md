# Create and read your first session

A handler often needs to know whether a cookie contains a verified identity before it performs its own work. Start with the direct JWT service: it creates a token and loads one owned, memoized Session without a server, SQL or UI.

Use the [direct consumer manifest](../consumers/direct/Cargo.toml) as the dependency template: session with `native`, `serde_json`, and Tokio macros/runtime. Its relative paths assume the checked-in location; adjust them for another application. Call the named async function below from your runtime.

## Create a token

Give the service ordinary object claims, then load the returned token. The default constructor retains the source key `session`, seed `abc123` and no added expiry; these are source defaults rather than a protection policy.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_session::{Service, SessionData, SessionResult};

pub async fn first_session() -> SessionResult<()> {
    let service = Service::default();
    let claims: SessionData = serde_json::from_value(json!({
        "id": "person-1", "roles": ["USER"]
    }))
    .unwrap();
    let token = service.create(claims).await?;
    let caller = service.load(token);
    assert!(!caller.guest().await);
    assert_eq!(caller.data().await.unwrap()["id"], "person-1");
    assert!(caller.can(&[]).await?);
    Ok(())
}
```

The assertions verify an identity round trip and the source rule that an empty permission request succeeds. No access map was supplied, so this example grants no named event permission. [Native tests](../tests/main/native.rs) pin defaults, expiry, invalid tokens and `can` semantics.

## What just happened

`Service` owns shared configuration and a clock. Each `load` creates a separate Session whose first `data()` verifies the token and caches either its claims or null; later calls on that same instance reuse the result. A new instance verifies again. The Session's raw token is read-only and the Session itself is not implicitly cloneable.

You now have the direct identity used by the server adapter. Read [concepts](concepts.md) to distinguish it from stored Auth/API credentials, [register identity](guides/registration.md) to read the configured request cookie, or [identity reference](api/identity.md) to add explicit expiry and permission checks.
