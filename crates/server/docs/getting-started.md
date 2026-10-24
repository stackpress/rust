# Getting started

You can make an operation callable before choosing a command or network server. Here you will
register a plugin, call its greeting capability, and print `Hello, Ada`.

## Create a consumer

Use Rust 1.96 or newer and this checkout. Create an application outside the repository:

```sh
cargo new --bin /tmp/stackpress-server-hello --edition 2024
```

In that consumer's `Cargo.toml`, add these entries under `[dependencies]`, replacing the path with
your checkout's absolute path:

```toml
stackpress-lib = { path = "/path/to/stackpress/rust/crates/lib" }
stackpress-server = { path = "/path/to/stackpress/rust/crates/server" }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt"] }
```

Tokio runs the application's async entry point. The server crate itself has no runtime dependency.

## Add the capability

Replace `src/main.rs` with this program. Start with a named handler, then select the plugin that
registers it:

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;

async fn greet(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Typed input rejects a present value of the wrong kind.
    let name = req
        .data
        .path_as::<String>("name")?
        .unwrap_or_else(|| "world".into());
    res.results(&format!("Hello, {name}"))?;
    Ok(Flow::Continue)
}

fn greetings(app: &mut ServerBuilder) -> Result<()> {
    app.on("greet", greet)?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Plugin selection is explicit compiled application code.
    let mut app = ServerBuilder::new();
    app.plugin("greetings", greetings)?;
    let server = app.build().await?;

    // call owns input conversion, one lifecycle, and a fresh response.
    let reply = server
        .call("greet", serde_json::json!({"name": "Ada"}))
        .await?;
    assert!(reply.report.is_success(&reply.response));
    println!(
        "{}",
        reply
            .response
            .result()
            .and_then(|value| value.as_str())
            .unwrap()
    );
    Ok(())
}
```

`greet` reads operation data and supplies a result. `greetings` is an ordinary registration
function; selecting it queues that function until `build`. The success assertion checks the final
public response as well as terminal failure state.

Run the consumer:

```sh
cargo run --manifest-path /tmp/stackpress-server-hello/Cargo.toml
```

The program prints `Hello, Ada`. A failed registration usually points to the [Lib handler
contract](../../lib/docs/api/event.md#1-handler-contract); a failed build returns
[BuildError](api/startup.md) with startup provenance.

## What just happened

`build` validated plugin dependencies, ran registration functions, decoded configuration, and
completed Config, Listen, and Route startup tasks. Only then did it publish an immutable Server.
This example needs no startup tasks because its handler uses no shared resources.

`call` allocated a host-local correlation ID and normalized the JSON object into a Request. It
loaded input, dispatched the optional `request` event, ran `greet`, and dispatched optional
`response` listeners. The result is still buffered: no native output happens until your application
prints or an adapter delivers it. Normal handler failures return in the report and public response,
so `await?` alone is insufficient to determine success.

## Next steps

You have selected a compiled plugin and observed its capability through the host lifecycle. Add
[typed config](guides/typed-config.md), initialize a [service](guides/services.md), or [expose the
operation as a route](guides/operation-routes.md). The canonical [profiles
plugin](../../../examples/capability-host/src/profiles.rs) combines those three tasks.
