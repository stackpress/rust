# Handle your first aggregate event

An application can register the framework with its generated application model family before it has a database or command provider. This example builds a finite event host and resolves one application event. It opens no listener.

## Build a small host

Use the dependencies and locked graph of the [full consumer](../consumers/full/Cargo.toml): the nine aggregate leaves, lib, server, session, SQL and Tokio. Put the following code in your application module and call `first_aggregate().await` from its async entry point.

<!-- example: full -->
```rust
// Generate this application module from its Idea imports before compiling.
#[allow(dead_code, missing_docs, unexpected_cfgs)]
#[path = "models/mod.rs"]
mod models;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;

async fn health(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"ready")?;
    Ok(Flow::Continue)
}

static CLIENT: stackpress_framework::CompiledClient = stackpress_framework::CompiledClient {
    bindings: stackpress_framework::FrameworkBindings {
        session: Some(
            stackpress_framework_session::composite::register_with::<models::stores::family::Models>,
        ),
        api: Some(stackpress_framework_api::register_with::<models::stores::family::Models>),
    },
    build: "first-application",
    describe: Vec::new,
    models: &[],
};

fn application(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework::register_with(app, &CLIENT)
}

pub async fn first_aggregate() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let mut app = ServerBuilder::new();
    app.plugin("framework", application)?;
    app.on("health", health)?;
    let server = app.build().await?;
    let reply = server
        .resolve("health", stackpress_lib::Nest::new())
        .await?;
    assert_eq!(reply.response.results.unwrap().as_str(), Some("ready"));
    assert!(server.host().service::<stackpress_sql::Engine>().is_err());
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()?
        .shutdown()
        .await;
    Ok(())
}
```

The assertion is your first success: the application event returns `ready` even though no Engine exists. `plugin` accepts the ordinary registrar, while the named async function handles the event. The last call closes the session hook lifecycle owned by this host; no authentication work was submitted.

## Where config goes

Use the G47 convention in the application crate:

| Original Stackpress | Rust application location |
| --- | --- |
| `config/common.ts` | `src/config.rs`: shared values, templates, computed or environment-derived values and typed code bindings. |
| `config/develop.ts` | `config/develop.json`: development data, passed with CLI `-b`. |
| `config/build.ts` | `config/build.json`: build-environment data, passed with CLI `-b`. |

JSON files hold data and cannot hold code values; those belong in `src/config.rs`. Precedence,
lowest first: **plugin section defaults → shared `src/config.rs` layer → `-b` environment file →
explicit app code overrides**. In the app build function, install the shared ConfigLayers before
the CLI-supplied ConfigLayers, then append explicit overrides. The existing
`ServerBuilder::config_layers` append order is sufficient; aggregate registration does not read
these files or discover config code.

The [converted sql-profile app](../../../../examples/sql-profile/README.md#where-configuration-goes)
uses this order with separate SQLite paths and a shared result-limit fallback. Its
[real-builder test](../../../../examples/sql-profile/tests/cases/configuration.rs) proves file
loading, precedence and rejection. Follow the [typed-config guide](../../../server/server/docs/guides/typed-config.md#own-external-input-explicitly)
for layer ownership or the [CLI argument guide](../../../../tools/cli/docs/guides/arguments.md#choose-an-environment-file)
for the `-b` boundary. Paths are relative to the process working directory, not the config file.

## What the registration did

`register_with` selected the compiled stages and queued their startup callbacks with the application-generated bindings. Each selected session/API stage requires its binding; a missing binding returns a descriptive construction error. Registration does not execute a command. Missing session/access/auth configuration does not add a global guard or auth routes.

You now have a host that can resolve application events. Add [compiled model callbacks](guides/registration.md) when generated operations are available, or bind a [finite command provider](guides/commands.md) before emitting command events. The [concepts page](concepts.md) explains the ordering those additions retain.
