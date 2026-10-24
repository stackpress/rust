# Building an application binary

A reusable CLI library cannot know your application's compiled plugins. Keep that choice in the
app's builder and pass the builder to main_with.

## Apply the incoming layers

The [getting-started program](../getting-started.md) is the minimal complete binary. Its key seam is
the builder below; replace the direct handler contribution with your selected plugin functions:

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};

async fn ready(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&true)?;
    Ok(Flow::Continue)
}

async fn build_application(layers: ConfigLayers) -> BuildResult<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.on("ready", ready)?;
    app.build().await
}

async fn entry() -> std::process::ExitCode {
    stackpress_cli::main_with(build_application).await
}
```

main_with reads one optional `-b` JSON file into layers; it cannot apply them to your private
builder. `app.config_layers(layers)?` makes that policy explicit. You can append further
code-derived overrides afterward. A bad argv/file returns before invoking the builder.

## Choose the entry seam

Use `main_with_args` for the same startup/file/exit pipeline with supplied argv and sinks. Use `run`
when your application already owns a built Server and Terminal. run does not load bootstrap data;
its config-path metadata is informational. No entry point discovers Rust plugins or prompts
implicitly.

You have isolated application composition from executable policy. Follow [testing](testing.md) to
validate the builder and exit codes without global streams, or consult [entry
reference](../api/entry.md) for callback bounds and failure precedence.
