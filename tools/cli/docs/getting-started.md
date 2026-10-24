# Getting started

A command should run your application's selected plugins and return data that another program can
read. Here you will create an app-owned `stackpress` executable and print a JSON greeting.

## Create the application

Use Rust 1.96 or newer and this checkout. Create a consumer outside the repository:

```sh
cargo new --bin /tmp/stackpress-cli-hello --name stackpress --edition 2024
```

The package name makes its executable `stackpress`. Add these local dependencies under
`[dependencies]`, replacing each checkout path:

```toml
stackpress-lib = { path = "/path/to/stackpress/rust/crates/lib" }
stackpress-server = { path = "/path/to/stackpress/rust/crates/server" }
stackpress-cli = { path = "/path/to/stackpress/rust/tools/cli" }
tokio = { version = "1", features = ["macros", "rt"] }
```

Tokio supplies the application's async runtime. CLI defaults include the native TTY driver, though
this command does not ask for terminal input.

## Select the plugin

Replace `src/main.rs` with the following program. The named operation reads `name`; the app builder
installs the supplied config layers and explicitly selects the compiled plugin:

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};
use std::process::ExitCode;

async fn greet(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
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

async fn build_application(layers: ConfigLayers) -> BuildResult<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.plugin("greetings", greetings)?;
    app.build().await
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    stackpress_cli::main_with(build_application).await
}
```

`main_with` obtains process argv and sinks, then calls your builder once. The builder receives
already parsed data layers; installing them is the application's responsibility. No plugin is loaded
from a file or current directory.

Run the command:

```sh
cargo run --quiet --manifest-path /tmp/stackpress-cli-hello/Cargo.toml -- emit greet --name Ada
```

Stdout contains one JSON envelope line with `"results":"Hello, Ada"` and code 200; successful
non-verbose execution leaves stderr empty and exits 0. Cargo diagnostics, if any, are separate from
the executable's stdout. Use `-v emit greet --name Ada` to request developer diagnostics on stderr.

## What just happened

The literal `emit` separates operator options from the selected event. All tokens after `greet` are
target data; `--name Ada` becomes a string value. The shared host performs one lifecycle, and the
CLI serializes its final public response before attempting stdout once.

Usage/config/startup failures return exit 2 with no stdout. Operation or output failure returns 1;
unrecovered typed prompt cancellation returns 130 at the executable boundary. A deliberate public
error recovery may return 0 while its report retains diagnostic failure.

## Next steps

You now own a compiled command and have seen one capability's machine output. Learn [argument
boundaries](guides/arguments.md), add [human diagnostics](guides/human-output.md), or [test with
injected sinks](guides/testing.md). The canonical [profiles
application](../../../examples/capability-host/src/main.rs) uses this entry pattern for `stackpress
emit profile-search --id abc123`.
