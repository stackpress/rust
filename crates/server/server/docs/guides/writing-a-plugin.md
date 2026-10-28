# Writing a plugin

A feature often needs several operations with shared startup rules. Put those contributions behind
one registration function, then let the application select it and its prerequisites explicitly.

## Register a compiled function

A plugin needs no trait implementation. This complete in-process example selects storage before a
dependent feature and confirms the selected order:

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{PluginSpec, ServerBuilder};

async fn ready(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&true)?;
    Ok(Flow::Continue)
}

fn storage(_: &mut ServerBuilder) -> Result<()> {
    Ok(())
}

fn profiles(app: &mut ServerBuilder) -> Result<()> {
    app.on("profiles-ready", ready)?;
    Ok(())
}

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = ServerBuilder::new();
    app.plugin("storage", storage)?;
    app.plugin_with(PluginSpec::new("profiles").requires(&["storage"]), profiles)?;
    let server = app.build().await?;
    assert_eq!(
        server
            .plugins()
            .map(|plugin| plugin.id.as_str())
            .collect::<Vec<_>>(),
        ["storage", "profiles"]
    );
    assert!(
        server
            .call("profiles-ready", serde_json::json!({}))
            .await?
            .response
            .result()
            .is_some()
    );
    Ok(())
}
```

The empty storage callback stands in for another compiled plugin. `requires` checks presence and
order before either callback runs; it does not initialize storage or load a package. Moving storage
after profiles makes build fail rather than silently sorting the list.

## Grow the feature

Queue resource initialization with `on_config`, operations with `on_listen`, and host routes with
`on_route`. Direct registration also accepts handlers and routes. Plain helper calls keep their
contributing plugin identity; separately identified plugins belong in the application's selected
list. A plugin callback cannot add another selected plugin after preflight seals selection.

For optional prerequisites use `after_if_present`. For actual compilation exclusion put both the
module and selection behind Cargo `cfg`, as in the canonical [experimental
plugin](../../../../../examples/capability-host/src/lib.rs). Config data alone cannot exclude or enable
compiled code.

You have packaged a feature with an explicit dependency. Continue with [typed
config](typed-config.md) or [services](services.md); use [PluginSpec](../api/plugin.md) and [builder
composition](../api/builder.md) for exact constraints.
