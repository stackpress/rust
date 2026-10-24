# Slice 2 design: host API and ownership

API contract for [task 00005](tasks/00005-slice-2-server-and-cli.md), approved with G12–G16 and the coordinator DX changes. Load for layout, plugins, startup, typed services/config and the additive lib boundary. Only M0 is released; signatures for later milestones remain sketches. The task receipt separates compiled M0 evidence from later behavior.

## Crates and modules

```text
crates/server/                  stackpress-server; default features = []
  src/lib.rs                    explicit public exports and crate example
  src/builder.rs                selected plugins and consuming startup
  src/plugin.rs                 identity/dependency/registration descriptors
  src/config.rs                 bounded JSON layers, section decoders/validation
  src/services.rs               typed immutable service registry
  src/context.rs                HostContext, CallContext, Invocation extension
  src/lifecycle.rs              top-level state machine only
  src/report.rs                 stage/correlation/failure ownership
  src/router.rs                 route wrappers, capture injection, descriptors
  src/http.rs                   optional buffered HTTP adapter
  tests/                        host/lifecycle/config/plugin/HTTP/proof cases
tools/cli/                      stackpress-cli, reusable adapter library
  src/lib.rs                    run/Terminal public API
  src/args.rs                   operator options and target argv boundary
  src/control.rs                writer, brand and color policy
  src/prompt.rs                 pure prompt state/validation/render logic
  src/tty.rs                    optional exclusive native TTY driver
examples/capability-host/        compiled profiles plugin and app-specific stackpress bin
```

Root facade adds optional `server` and `server-http` features and `pub use stackpress_server as server`. Keep current lib imports/defaults compatible. No server/CLI depends on the root facade, database, renderer or generator. Introduce only crates with real code. CLI reusable library does not enter the facade's default dependency path.

## First success

An empty application needs a normal function plugin and a named handler. With ordinary imports from `stackpress::{Flow, Invocation, Request, Response, Result}` and `stackpress::server::{Server, ServerBuilder}`:

```rust
async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"hello")?;
    Ok(Flow::Continue)
}
fn plugin(app: &mut ServerBuilder) -> Result<()> {
    app.on("greet", greet)?;
    Ok(())
}
async fn application() -> stackpress::server::BuildResult<Server> {
    let mut app = ServerBuilder::new();
    app.plugin("greetings", plugin)?;
    app.build().await
}
```

Application binary glue will parse Terminal and call `stackpress_cli::run(&server, terminal, sinks).await`; `stackpress emit greet` will print the public response envelope after the CLI milestone. G14 selects an application binary only, with no stock/empty binary in tools/cli. [Adapters](slice-2-design-adapters.md) specifies the compiled application seam.

## Builder and plugin contract

```rust
pub type BuildResult<T> = std::result::Result<T, BuildError>;
pub struct ServerBuilder { /* owned, non-Clone, no runtime sharing */ }
pub struct Server { /* immutable EventEmitter and Arc<HostContext>; non-Clone */ }
pub struct PluginSpec { /* ID, required IDs, optional-present ordering IDs */ }

ServerBuilder::new() -> Self
ServerBuilder::plugin(&mut self, id: &str,
    register: fn(&mut ServerBuilder) -> Result<()>) -> Result<&mut Self>
ServerBuilder::plugin_with(&mut self, spec: PluginSpec,
    register: fn(&mut ServerBuilder) -> Result<()>) -> Result<&mut Self>
ServerBuilder::build(self) -> impl Future<Output = BuildResult<Server>> + Send
PluginSpec::new(id: &str) -> Self
PluginSpec::requires(self, ids: &[&str]) -> Self
PluginSpec::after_if_present(self, ids: &[&str]) -> Self
Server::plugins(&self) -> impl Iterator<Item = &PluginDescriptor>
```

`plugin` stores a compiled registration function; build preflights the complete selected list before calling any function. Validate unique bounded IDs, missing dependencies, self-dependencies, cycles and order violations. Preserve supplied plugin order: a required or present optional prerequisite must precede its dependent. Reject wrong order with both IDs; do not silently topologically reorder priority-sensitive composition. Optional absent dependencies are not errors and do not load code. Dependencies express startup presence/order, not semver solving or runtime isolation.

Registration functions run sequentially in selected order, under explicit contributing-plugin metadata. Nested plain helper functions retain that identity; separately identified plugins must be in the selected list before build. Reject adding plugins/phase callbacks after their collection is sealed. Duplicate IDs fail, even for the same function. No hot enable/disable, native libraries, JS import or config plugin strings.

Direct builder contributions happen before queued plugin functions execute; among equal priorities actual registration order remains the lib tie rule. Prefer one application plugin for explicit complete ordering. Every forwarding registration captures its actual caller synchronously with `#[track_caller]`; labels do not substitute for call sites.

## Startup and runtime registration

```rust
pub enum StartupStage { Config, Listen, Route }
pub struct StartupContext { /* exclusive builder/service state for one step */ }
pub struct StartupSpec { /* stage, priority, label; plugin/site automatic */ }

ServerBuilder::on_config(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()>
ServerBuilder::on_listen(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()>
ServerBuilder::on_route(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()>
ServerBuilder::startup(&mut self, spec: StartupSpec,
    step: impl TaskHandler<StartupContext>) -> Result<()>
StartupContext::stage(&self) -> StartupStage
StartupContext::config<T: Send + Sync + 'static>(&self) -> Result<&T>
StartupContext::register<T: Send + Sync + 'static>(&mut self, service: Arc<T>) -> Result<()>
StartupContext::service<T: Send + Sync + 'static>(&self) -> Result<&T>
```

Use existing `TaskQueue<StartupContext>` for each phase: higher priority first, stable registration ties. Empty phases succeed. Each consumed task has plugin/label/site metadata in `BuildError`. Decode and validate all config before config tasks; freeze services after Config; Listen adds operations; Route adds HTTP exposure. The same callback signature serves all phases, but mutation methods check phase and reject service insertion outside Config or registration in the wrong phase. Direct plugin registration may attach handlers immediately; async resource initialization belongs in Config.

Any task error or abort fails build; no partial Server escapes. Dropping build consumes the attempted task; there is no resumable/retry builder. Dropping constructed resources is ordinary RAII; external side effects and asynchronous cleanup remain application responsibilities. No implicit spawned startup work or background cleanup promise.

Both ServerBuilder and permitted StartupContext phases forward these existing lib capabilities:

```rust
on(&mut self, event: &str, action: impl Handler) -> Result<ListenerId>
on_priority(&mut self, event: &str, priority: i64, action: impl Handler) -> Result<ListenerId>
listen(&mut self, spec: ListenerSpec, action: impl Handler) -> Result<ListenerId>
route(&mut self, method: MethodPattern, path: &str, action: impl Handler) -> Result<ListenerId>
route_with(&mut self, spec: RouteSpec, action: impl Handler) -> Result<ListenerId>
get/post/put/patch/delete/head/options/connect/trace/all(/* path, action */) -> Result<ListenerId>
replace(&mut self, id: &ListenerId, action: impl Handler) -> Result<()>
remove(&mut self, id: &ListenerId) -> Result<bool>
```

`RouteSpec` carries method/path/priority/label and host capture policy. Duplicate event/route matchers are independent contributions, not conflicts; preserve all-matching priority and before/action/after rules. IDs remain the replacement/removal authority. Host route replacement re-wraps capture policy; it must not accidentally install a raw handler. Builder composition moves contributions with original sites/IDs and explicit owner metadata; no public mutable runtime emitter/service map.

## Typed config and service access

```rust
ServerBuilder::config_layers(&mut self, layers: ConfigLayers) -> Result<&mut Self>
ServerBuilder::config_section<T>(&mut self, key: &str, defaults: T,
    validate: fn(&T) -> Result<()>) -> Result<()>
    where T: Serialize + DeserializeOwned + Send + Sync + 'static
ConfigLayers::new() -> Self
ConfigLayers::json(&mut self, label: &str, text: &str) -> Result<&mut Self>
ConfigLayers::value(&mut self, label: &str, value: serde_json::Value) -> Result<&mut Self>
HostContext::config<T: Send + Sync + 'static>(&self) -> Result<&T>
HostContext::service<T: Send + Sync + 'static>(&self) -> Result<&T>
HostContext::shared_service<T: Send + Sync + 'static>(&self) -> Result<Arc<T>>
```

Config is package-owned typed sections, not `Server<C>` requiring every third-party plugin to know the app's whole type. Bind each section name and concrete TypeId once. Defaults serialize as the lowest layer; explicit layers apply in supplied order; objects merge recursively, arrays/scalars replace, null is a real replacement (not deletion). Only object roots are accepted. This is config-specific; **do not change Nest::merge**, which remains a shallow top-level overlay.

After layering, deserialize each registered section once and validate before startup. Applications/packages choose serde field policy (recommend `deny_unknown_fields`); the host rejects unclaimed top-level sections. Absent optional sections stay absent unless their owning selected plugin registers defaults. Unknown sections never enable plugins. Explicit code overrides go in the last value layer; environment values are converted explicitly by the application, with no hidden process-env/cwd/file reads in the server crate. Errors name layer/section/type/plugin and field path where available; never dump the config document.

Config input/depth/entries/aggregate merged output use finite budgets, checked before publication; layer failure leaves the previous accepted accumulator unchanged. Reject duplicate section names/types. One config value per concrete type; newtypes distinguish roles. A missing service is a named typed lookup error, not a panic.

Services are keyed by TypeId; `register(Arc<T>)` explicitly shares ownership and rejects duplicates. Provide `register_named(name, Arc<T>)`, `service_named<T>(name)` and `replace_service<T>(Arc<T>)` for deliberately named instances/customization during Config. Replacing an absent/wrong-typed slot errors; retain contributor/replacement sites. A returned `&T` is read-only access, not a claim that T lacks interior mutability. Applications own locks, transaction semantics and resource cancellation. No registry lock survives await; no implicit service Clone.

Example plugin phases and handler (imports include `std::sync::Arc` and `stackpress_server::ServerInvocationExt`):

```rust
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    let limit = start.config::<ProfilesConfig>()?.limit;
    start.register(Arc::new(Profiles::new(limit)))?;
    Ok(Flow::Continue)
}
async fn profile_search(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let id = req.data.path_as::<String>("id")?.unwrap_or_default();
    let rows = inv.service::<Profiles>()?.search(&id).await?;
    res.rows(&rows, rows.len() as u64)?;
    Ok(Flow::Continue)
}
fn profiles(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("profiles", ProfilesConfig::default(), validate_profiles)?;
    app.on_config(configure)?;
    app.on_listen(listen_profiles)?;
    app.on_route(route_profiles)?;
    Ok(())
}
```

`listen_profiles` and `route_profiles` are named startup functions registering `profile-search` and `GET /profiles/:id`. The route handler delegates with `inv.emit`, propagates child Abort deliberately, and never starts a second lifecycle. The illustrative Profiles service/config are application-owned, not new framework domain types.

## Additive lib changes and context lifetime

The current [event implementation](../../../crates/lib/src/event.rs) has private immutable table/ancestry/captures and no host-state slot. Closing over a Server from its own handlers would form an ownership cycle; putting services in Request data would mix host authority with untrusted inputs. Neither is acceptable.

Propose only these generic, transport-neutral additions:

```rust
EventEmitter::emit_with_context<'a, T: Any + Send + Sync>(
    &'a self, event: &'a str, req: &'a mut Request, res: &'a mut Response,
    context: &'a T,
) -> HandlerFuture<'a, Result<DispatchOutcome>>
EventEmitter::resolve_with_context<'a, T: Any + Send + Sync, I: IntoRequest + 'a>(
    &'a self, event: &'a str, input: I, context: &'a T,
) -> HandlerFuture<'a, Result<Resolution>>
Invocation::context<T: Any + Send + Sync>(&self) -> Option<&T>
```

Invocation holds an optional borrowed `&(dyn Any + Send + Sync)`; nested emit/resolve pass the exact same borrow, including fresh child req/res pairs. Existing root lib emit/resolve supplies None and retains all behavior. No server dependency, type registry, global, request injection, unsafe lifetime extension or `Clone` is added to lib.

The server passes a concrete owned `CallContext` containing `Arc<HostContext>`, correlation ID, stage and optional `Arc<StageFailure>`. It owns no Server/emitter/callback, so no framework cycle exists. `ServerInvocationExt::{host,service,config,shared_service,correlation_id,stage,failure}` provides direct typed access; missing host context returns an Error. `inv.delegate(event, req, res)` maps Completed to Continue, Aborted to Abort, and NoListener to a named DelegationError without another lifecycle. Authors import the extension trait and never implement one. M0 proves service/config/delegation shortcuts and inherited metadata; failure context and top-level allocation remain M2.

M0 compiler/runtime probes confirm that external generic Handler/TaskHandler bounds can call the existing sealed supertrait methods. Route/startup wrappers therefore need **no forwarding additions** to lib and no unsealing/signature rewrite. Handler storage wrappers explicitly retain the existing `+ 'static` registration bound; this does not make borrowed dispatch futures static.

The [lifecycle](slice-2-design-lifecycle.md) defines reports/handle signatures, [adapters](slice-2-design-adapters.md) defines wire/CLI entry, and [verification plan](slice-2-design-verification.md) defines the M0 stop and shared-file ownership.
