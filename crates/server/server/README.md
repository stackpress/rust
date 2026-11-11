# stackpress-server

Compile-time plugins configure a transport-neutral Stackpress host. Top-level handling runs the request, operation, response and error lifecycle; nested dispatch stays explicit.

```rust
use stackpress_server::{BuildResult, ServerBuilder};
# async fn example() -> BuildResult<()> {
let server = ServerBuilder::new().build().await?;
# let _ = server;
# Ok(())
# }
```

Building freezes registrations. The caller supplies the runtime and transport; construction starts no listener.

Licence: [Apache-2.0](LICENSE).
