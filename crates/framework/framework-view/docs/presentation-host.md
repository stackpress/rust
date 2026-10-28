# Explicit native presentation contributions

Enable `web-preferences` and/or `web-flash` on `stackpress-framework-view`.
Facade equivalents are `framework-view-web-preferences` and `framework-view-web-flash`.
Pure `preferences`/`flash` contracts still compile for the browser without a server.
Neither a feature nor configuration selects a plugin or writes cookies/files.

Select ordinary functions with the existing `ServerBuilder::plugin` method:

```rust,ignore
app.on_config(host_admission)?; // registers Arc<preferences::Admission>
app.plugin("preferences", preferences::register)?;
app.plugin("flash", flash::register)?;
```

Each function has signature `fn(&mut ServerBuilder) -> stackpress_lib::Result<()>`.
Application composites can call both functions directly in their own selected callback.
Do not call `app.plugin` from a selected callback: selection is already sealed there.
Plugin identifiers/order/prerequisites use the existing ServerBuilder/PluginSpec API.
There is no `framework_view::plugin::register`; the generic plugin work is issue 14.

Supply trusted `ConfigLayers` sections before building:

| Section | Typed configuration | Contribution |
| --- | --- | --- |
| `view_preferences` | `preferences::Config` | Validated locale registry, canonical origin/mount, distinct literal settings and confirmation POST paths, optional immutable theme asset GET |
| `view_flash` | `flash::Config` | Canonical origin/mount and 1–128 unique literal application-relative GET page paths |

Both configurations deny unknown fields. Missing data, invalid origins/paths/locales,
duplicate sections/services and missing admission fail startup before a Server is returned.
Paths have no captures/query and are mounted through checked BasePath. These integrations
reject their own conflicting paths; existing ServerBuilder semantics for unrelated routes
remain unchanged, so the application must not register additional handlers at these paths.

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
