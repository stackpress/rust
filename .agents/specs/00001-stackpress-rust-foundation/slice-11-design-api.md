# Slice 11 composition and public API sketch

Status: Draft / Not Frozen. New names/examples below are **uncompiled design sketches**. Existing symbols are in the [overlap map](slice-11-design-constituents.md). No source stage is represented by a dummy registrar.

## Minimal shape

Use ordinary compiled functions, a static client descriptor and existing host types. Cargo features fix the available constituent set before compilation.

```rust
pub type Register = fn(&mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()>;
pub struct CompiledModel {
    pub name: &'static str,
    pub listen: Option<Register>,
    pub admin: Option<Register>,
}
pub struct CompiledClient {
    pub models: &'static [CompiledModel],
    pub build: &'static str,
    pub describe: fn() -> Vec<stackpress_framework_schema::Model>,
}
pub fn register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()>;
pub fn register_with(
    app: &mut stackpress_server::ServerBuilder,
    client: &'static CompiledClient,
) -> stackpress_lib::Result<()>;
```

Stock register has no generated client, matching nullable source import. register_with uses a statically linked catalog assembled/emitted before application compilation. Missing model listen/admin methods are represented by None, not a runtime enable flag or empty callback. No catalog is built from caller/config strings; no uncompiled function is loaded. Supplied callbacks contribute real registrations and are independently callable.

## Nine-stage algorithm

Call these synchronously at selection, returning the first registration error. Existing ServerBuilder owns Config → Listen → Route and publication. Phase priority still precedes registration order; source schema client Config priority **10** must not be flattened to zero.

| Order | Contribution | Required parity |
| --- | --- | --- |
| 1 server | Proposed source command adapter | Listen registers develop, emit, serve. Actual IO only when explicitly invoked via app binding; no listener at startup. |
| 2 schema | Proposed compiled-client/schema adapter | Config priority 10 publishes client access; nullable/nonnullable distinction; Listen generate; separate build-host transform. |
| 3 language | Proposed source native adapter | Optional language config/service/request hook; data locale then prefix, cookie staging, raw nested method/path dispatch. |
| 4 CSRF | Existing plugin::register | Explicit csrf/server; source default service; no global guard. |
| 5 SQL | Proposed command/client adapter; explicit generated-SQL option | install,migrate,purge,push,query,populate,uninstall,upgrade in source order; model listen callbacks in catalog order. API data registrar here. |
| 6 view | Source renderer/provider adapter; optional presentation composite | Missing view disables source contribution; optional existing preferences/flash require their own config/admission. |
| 7 session | Existing composite::register | auth → email → identity, once; source session/auth/access gates. |
| 8 API | Existing framework_api::register | Endpoint/CORS/OAuth/webhooks; SQL selected at stage 5; codegen/UI explicit. |
| 9 admin | Compiled model admin callbacks | Real generated/app registrars and checked Mount/SQL dependencies; absent client/method contributes no routes. |

Disabling a constituent removes its stage without rearranging survivors. Config deactivation leaves existing callback priorities intact. Multiple same-route/event contributions and any-handler Abort remain; no aggregate route owner or conflict resolver.

## Application registration and direct use

This sketch binds a static catalog to the ordinary #14 function. Metadata uses existing owned Model descriptors, so no private normalized Schema constructor or generated-code import is guessed. The example selects real account data events; the full consumer must add API models and its real generated admin callback.

```rust
use stackpress_framework::{CompiledClient, CompiledModel};
use stackpress_framework_schema::Model;
use stackpress_framework_session::{auth::AuthSchema, profile::ProfileSchema};
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

fn describe() -> Vec<Model> {
    vec![ProfileSchema::default().model(), AuthSchema::default().model()]
}
static MODELS: &[CompiledModel] = &[CompiledModel {
    name: "account",
    listen: Some(stackpress_framework_session::account::events::register),
    admin: None,
}];
static APPLICATION_CLIENT: CompiledClient = CompiledClient {
    build: "application-v1",
    describe,
    models: MODELS,
};
fn application_framework(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework::register_with(app, &APPLICATION_CLIENT)
}
fn select(app: &mut ServerBuilder) -> Result<()> {
    app.plugin("framework", application_framework)?;
    Ok(())
}
```

Direct composition calls application_framework(&mut app). A host without generated client selects app.plugin("framework", stackpress_framework::register). A facade consumer uses stackpress::framework::register_with under opt-in facade features. Independent constituent consumers call their existing registrars and need no aggregate dependency. Never call app.plugin from inside a selected callback: selection is sealed there. Contributions inherit aggregate/application provenance as existing composites do.

The full consumer catalog supplies actual account SQL functions, API sql::register and real generated admin pages_host::register, with the app's real schema metadata. No no-op admin or fake SQL listener counts as proof of stages 5/9. Catalog entries may group existing native contribution families without renaming Profile/Auth/Application/Session storage.

## Phase and resource reconciliation

Compiled listen callbacks accept ServerBuilder because current API SQL queues Listen while session account SQL currently calls app.on immediately. M0 compares actual phase/event inventories to source. If immediate account registration breaks source ordering, request an exact-file startup helper from the coordinator; do not duplicate its seven-handler list or change other crates without release.

Generated admin Config must still see Engine/Policy/SQL Service/Mount and can supply SQL Contributions before Listen. `generated-sql` selects this existing contract explicitly; it is not forced by missing client or full. Missing source client/models must remain an allowed mode without an Engine. Source SQL operator handlers acquire resources only when invoked; startup performs no query/install/maintenance.

Client service access returns None for nullable missing lookup and a native error for required missing lookup. Source client.module becomes build/catalog identity rather than runtime import. Catalog metadata owns no DB/SMTP/signing secrets and is not a browser DTO.

Language retains data-selected locale followed by recognized prefix, cookie staging and raw nested dispatch to the stripped path with the same response. Preserve query/post/headers/body/resource/session behavior using existing Request ownership. Uncloneable native IO/replay semantics require a disposable proof and explicit mechanics boundary before claiming parity; never serialize a resource to clone it. No redirect, new normalization, second top-level lifecycle or guard is substituted. Unknown locales retain source behavior, not preference-registry policy.

## Generation and command boundaries

Separate aggregate codegen helpers call current schema/SQL/admin registration functions with the application's real ContextTransform callbacks. API supplies codegen::config/definitions/normalize; there is no api::codegen::register. Use current Registry ownership/dependencies, app definitions/diagnostics and explicit OutputWorkspace check/install. stackpress.idea's API include maps to explicit API schema composition, not a runtime generator call.

Source server/schema/SQL events and script functions map to explicit native command adapters over existing CLI/host/planner/Actions contracts. Real develop/serve/build IO bindings remain app-owned; tests use finite provider seams and never start watchers/listeners. Preserve source input/result/error/skip order at the adapter boundary. Node-only loader/build packaging is separately inventoried; command registration is never claimed to prove a live Node/Vite toolchain.
