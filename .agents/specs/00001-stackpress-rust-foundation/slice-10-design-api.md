# Slice 10 public API sketch

Draft / Not Frozen; proposed Rust spelling, not currently implemented/compile-checked. [G43 behavior](slice-10-design-decisions.md), [configuration](slice-10-design-configuration.md) and [original session interface](slice-10-design-boundaries.md#original-caller-and-session-flow) govern these APIs.

## Public surface

| Proposed item | Contract |
| --- | --- |
| `ApiConfig`, `ApiEndpoint`, `ApiType`, `ApiScope`, `ApiWebhook`, `Scopes` | Source config/type equivalents; optional section disables plugin. No Exposure, ScopeRequirement or new OAuth policy schema. |
| `Application`, `ApplicationInput`, `ApplicationExtended`; `Session`, `SessionInput`, `SessionExtended` | Source model DTOs with ordinary stored secrets and joined profile/application where selected. Qualified API Session avoids collision with framework-session JWT type. |
| `ApiConfigProps`, `ApiOauthInputProps`, `ApiOauthFormProps` | Same selected config/request/form metadata for Leptos presentation. Preserve source-selected fields instead of a newly narrowed public projection. |
| `ApiError` | Existing error/status interoperability, Unauthorized 401. No global uniform failure remapping or automatic WWW-Authenticate/403 rule. |
| `helpers::authorize(&Request,&mut Response) -> Result<Option<Authorization>>` | Source header split/trim + GET/non-GET presence checks; typed Option replaces JS false. Authorization has token/id/secret. No store lookup inside helper. |
| `helpers::unauthorized(&mut Response) -> Result<()>` | Set existing API Unauthorized error/status; does not abort future listeners itself. |
| `helpers::valid_data(assert,data) -> Result<bool>` | Source recursive predicate/scalar strict equality/list includes; container identity as Q10-REF. Inherited representational errors remain mechanics errors. |
| `plugin::register(&mut ServerBuilder) -> Result<()>` | Optional api section, Config services only if active, Listen webhooks, Route endpoints plus source-gated OAuth/view. No file/DB install/network during registration. |
| `routes::{cors,session,app,open}(&mut ServerBuilder, ApiEndpoint) -> Result<()>` | Individually callable source route helpers, same callbacks as plugin. `cors` only registers headers/preflight; no separate dispatcher. |
| `endpoints::routes(&mut ServerBuilder, &[ApiEndpoint]) -> Result<()>` | Direct source loop including CORS and selected adapter; mounting OAuth follows the plugin's nonempty-list rule when using full `api::routes`. |
| `pages::{oauth,token}(&mut Request,&mut Response,Invocation) -> Result<Flow>` | Named async page handlers port source branches; no standard OAuth SDK/exchange service/new grant types. |
| `views::{ApiOauthForm,OAuthBody,ApiOauthHead,ApiOauthPage,Head}` | Equivalent Leptos pieces now, including default full consent page and head. |
| `sql::{Adapter,Dependencies,register,operations}` | Real Application/Session storage adapter and original hyphen-named model events now. App supplies open Engine, Profile mapping and default providers; registration starts no database. |
| `admin::{register,routes}`, `codegen::register` | Select/source-generate both models, management pages and related record views. Generator/runtime remain separate. Source has no independent rotate/revoke/issue_application credential protocol. |
| `webhooks::{register,send}`, `Fetch` / `ReqwestFetch` | Register late event listeners and await one source envelope send. A transport seam exists to replace JS fetch and test without I/O; no queue/store/worker abstraction. |

Traits are limited to real adapter needs (Fetch and existing SQL/value providers). Use ordinary structs/functions for source checks. Box a Send future only for an object-safe adapter boundary; no async-trait dependency is requested. Document public/private functions, ordering, errors and source-specific data mutation per Rust style.

## Function plugin DX

All API names below are proposed. Existing lib/server calls follow #14. This example shows the same public surface as source and deliberately adds no exposure allowlist.

```rust
use stackpress_framework_api::plugin;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ConfigLayers, ServerBuilder};

async fn article_search(
    _: &mut Request,
    res: &mut Response,
    _: Invocation<'_>,
) -> Result<Flow> {
    res.results(&["Published article"])?;
    Ok(Flow::Continue)
}

pub fn application() -> Result<ServerBuilder> {
    let mut layers = ConfigLayers::new();
    layers.value("application", serde_json::json!({
        "api": {
            "endpoints": [{
                "method": "GET", "route": "/api/articles",
                "type": "public", "event": "article-search",
                "cors": true, "data": {"status": "PUBLISHED"}
            }]
        }
    }))?;
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.on("article-search", article_search)?;
    app.plugin("api", plugin::register)?;
    Ok(app)
}
```

Nonempty endpoints also install OAuth/token/page bindings as source does; invoking those paths needs their ordinary selected session/model/view resources. Do not fail public-only startup just because a currently unused source event has no listener. Missing api config leaves this selected registrar inert. `build().await` never binds a socket.

For a full app, application Config supplies an already opened Engine and shared Profile/model/provider mapping; selects the SQL adapter/model event integration, Slice 9 session integration, API and generated management in source prerequisite order. The ordinary functions can be composed in an outer selected callback. `PluginSpec::requires` may express application-owned selected IDs/order where real prerequisites apply; API does not globally require the literal plugin ID session or force session/access config on every endpoint.

## Direct use

```rust
use stackpress_framework_api::{routes, ApiEndpoint};
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

pub fn public_endpoint(app: &mut ServerBuilder, endpoint: ApiEndpoint) -> Result<()> {
    routes::cors(app, endpoint.clone())?;
    routes::open(app, endpoint)?;
    Ok(())
}
```

Direct library consumers can also call helpers::authorize, pages::oauth/token with a real host Invocation, SQL Adapter operations, or webhooks::send with a supplied Fetch. They receive source behavior; no plugin invocation or ambient caller is manufactured by a helper. The app registers target events independently; a config name cannot dynamically import a crate.

## Native features and composition

Keep crate/facade defaults unchanged. Proposed `server` exposes runtime registration; `sql` supplies the full adapter; `ui` enables consent/admin presentation on existing Leptos/view facilities; outbound reqwest is an explicit native dependency of selected webhook fetch integration. The full registrar is compiled with its complete source page bindings; smaller direct support APIs remain independently usable. Native feature combinations and shared/browser DTO builds are proved before accepting exact wiring. This is target packaging, not deferring API features or adding independent OAuth activation switches.

Select only the outer composite; inside it call ordinary register functions, never `app.plugin`. Config/Listen/Route remain existing phases and every contribution retains parent plugin identity. Retain direct generated operations/routes alternatives and G42: no global route uniqueness, and any listener may stop its own chain. A failed registration discards the builder; no partial Server publishes and earlier external effects are not magically rolled back.
