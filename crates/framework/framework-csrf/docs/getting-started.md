# Complete a CSRF round trip

A form needs an outgoing token before a later submission can be checked. This walkthrough stages that token and constructs a separate incoming request, so you can see which state validation reads.

Use the checked-in [direct consumer manifest](../../framework-email/consumers/direct/Cargo.toml) as a dependency template: `stackpress-framework-csrf`, `stackpress-lib` and `serde_json` are enough for this synchronous example. Its relative paths assume its checked-in location; point them at your checkout when creating another application. No `server` feature is required.

## Generate and submit

Start with a response, then copy the generated token into the next request's cookie and aggregate form data. The copying below models a browser's next request; it does not make outgoing cookies visible to validation automatically.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_csrf::Service;
use stackpress_lib::{Request, Response, Result};

pub fn round_trip() -> Result<()> {
    let csrf = Service::default();
    let mut response = Response::new();
    let token = csrf.generate(&mut response)?;
    assert_eq!(token.len(), 64);
    let mut request = Request::builder().data(json!({"csrf": token})).build()?;
    request.session.set("csrf", token, None)?;
    assert!(csrf.valid(&request, &mut response)?);
    csrf.clear(&mut request, &mut response)?;
    assert!(!response.data().has("csrf"));
    Ok(())
}
```

`generate` stages an outgoing cookie and a `{name, token}` form projection under `response.data()["csrf"]`. `valid` reads only the incoming request's session cookie and aggregate data. The assertions show a successful comparison and removal of the outgoing form projection; the incoming cookie remains.

## What just happened

The service holds configuration rather than per-user token state. Generation uses 32 OS-random bytes encoded as 64 lowercase hex characters; validation accepts arbitrary equal, nonempty UTF-8 strings with equal byte lengths. Generated-token format is not an acceptance rule.

You now have the direct mechanism working without a server. Continue with [incoming and outgoing state](concepts.md), or [register and configure](guides/register-and-configure.md) to retrieve this service from an event invocation.
