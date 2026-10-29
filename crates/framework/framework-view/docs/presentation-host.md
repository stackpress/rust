# Explicit native presentation contributions

Enable `presentation` on `stackpress-framework-view` for the preferences/flash composite.
It forwards only `web-preferences` and `web-flash`, without selecting Leptos or changing defaults.
Enable `web-preferences` and/or `web-flash` for independently selected capabilities.
Facade equivalents are `framework-view-web-preferences` and `framework-view-web-flash`.
Pure `preferences`/`flash` contracts still compile for the browser without a server.
Neither a feature nor configuration selects a plugin or writes cookies/files.

Select ordinary functions with the existing `ServerBuilder::plugin` method:

```rust
use std::sync::Arc;
use stackpress_framework_view::{plugin, preferences};
use stackpress_lib::{Flow, Result};
use stackpress_server::{ConfigLayers, ServerBuilder, StartupContext};

// This example deliberately has no session; it does not authorize domain operations.
async fn host_admission(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new(preferences::Admission::sessionless()))?;
    Ok(Flow::Continue)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut layers = ConfigLayers::new();
    layers.value("application", serde_json::json!({
        "view_preferences": {
            "origin": "https://app.invalid", "base": "/demo",
            "locales": ["en", "fr"], "default_locale": "en", "aliases": {},
            "settings_path": "/settings", "confirmation_path": "/settings/confirm",
            "serve_theme_bootstrap": true
        },
        "view_flash": {
            "origin": "https://app.invalid", "base": "/demo", "pages": ["/preferences"]
        }
    }))?;
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.on_config(host_admission)?;
    app.plugin("presentation", plugin::register)?;
    // This constructs the selected services/routes; it opens no listener or file.
    let server = app.build().await.map_err(stackpress_lib::Error::from_source)?;
    assert_eq!(server.plugins().count(), 1);
    Ok(())
}
```

`plugin::register`, `preferences::register` and `flash::register` each have signature
`fn(&mut ServerBuilder) -> stackpress_lib::Result<()>`. The composite calls preferences
then flash, retaining their own validators, services and startup phases. Select either
constituent directly for a smaller integration. Selecting the composite and a constituent,
or selecting the composite twice under different IDs, rejects duplicate typed sections.
Application composites can call `plugin::register` directly in their selected callback:

```rust
use stackpress_framework_view::plugin;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, StartupContext};

async fn preferences_page(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.html("Application preferences")?;
    Ok(Flow::Continue)
}

async fn pages(start: &mut StartupContext) -> Result<Flow> {
    start.get("/demo/preferences", preferences_page)?;
    Ok(Flow::Continue)
}

pub fn application_presentation(app: &mut ServerBuilder) -> Result<()> {
    plugin::register(app)?;
    app.on_route(pages)?;
    Ok(())
}
```

With the same trusted sections and Admission setup from the first example, select
`application_presentation` in place of `plugin::register`. All contributions retain the
application composite's plugin identity. Loaders, renderers and protected-operation policy
remain application choices. Direct `ServerBuilderViewExt::page`, pure settings/flash APIs
and native file adapters do not require the composite.

Do not call `app.plugin` from a selected callback: selection is already sealed there.
Plugin identifiers/order/prerequisites use the existing ServerBuilder/PluginSpec API.
The function participates only in runtime construction. It has no generation hook, parser,
database installation/migration/seed, renderer initialization or network-host startup.
The final native app and browser continue to select their own dependencies separately.

Supply trusted `ConfigLayers` sections before building:

| Section | Typed configuration | Contribution |
| --- | --- | --- |
| `view_preferences` | `preferences::Config` | Validated locale registry, canonical origin/mount, distinct literal settings and confirmation POST paths, optional immutable theme asset GET |
| `view_flash` | `flash::Config` | Canonical origin/mount and 1–128 unique literal application-relative GET page paths |

Both configurations deny unknown fields. Missing data, invalid origins/paths/locales,
duplicate sections/services and missing admission fail startup before a Server is returned.
Paths have no captures/query and are mounted through checked BasePath. These integrations
reject conflicts within their own declared inventories. Multiple handlers may intentionally
process the same route: priorities order execution, equal priorities retain FIFO order,
and cooperative abort (returning false) can stop later handlers. Application and plugin
authors own their route inventories and decide how handlers cooperate. Every protected
handler enforces its own policy; shared routing and public UI hints grant no authority.

Preferences additionally requires an explicit `preferences::Admission` service in Config,
registered before its configure callback. `Admission::new(fn(&Request) -> bool)` evaluates
fresh application-owned session/CSRF facts on every write and confirmation. Denial stages
no cookie. `Admission::sessionless()` is an explicit choice for sessionless hosts; exact
canonical Origin validation still applies. Admission and public allowed-action hints never
supply protected domain-operation authorization: use the independent fail-closed HostAccess
at that operation boundary.

Loaders borrow `preferences::Service::action()` and `flash::Service::notice(request)`
through ServerInvocationExt. Notice projection returns checked public text; escape it when
rendering. The flash response hook clears only configured successful HTML GET responses,
with the issuing Path/flags, before adapter commitment. Unrelated routes, operations, errors
and non-HTML responses preserve the cookie. `Service::issue` returns a header for explicit
application issuance after success. Consumption is best effort across concurrent tabs.

`serve_theme_bootstrap: true` contributes the same 248-byte immutable asset used by the
renderer, with one-year public caching and nosniff. False leaves ownership to an explicit
application asset bundle. The ui-preferences finite host selects the route; review-preview
composes its existing checked AssetBundle. Neither route startup nor the asset writes state.
Native file persistence remains a separate opt-in adapter; no native/Tauri plugin is inferred.
