# Slice 11 targets, configuration and dependencies

Status: Draft / Not Frozen. Proposed feature names are not manifest changes; [existing features](slice-11-design-constituents.md) remain intact. Renderers are separate artifacts, never --all-features.

## Proposed aggregate selection

Keep default=[] for aggregate and root. No-feature aggregate exports portable contracts, without an empty register function. Native registration exists only when an actual contribution is selected. `full` is explicit; applications can select constituent leaves with default-features=false.

| Proposed feature | Selection / boundary |
| --- | --- |
| native | Native lib/server registration contracts, cfg(non-WASM); alone not an empty plugin. |
| server-commands | native; actual develop/emit/serve adapters, explicit app IO bindings, no startup watcher/listener. |
| schema-client | native + portable schema; compiled client service/generate bridge, no parser/codegen in runtime graph. |
| language | native source cookie/prefix adapter; existing preference validation/Leptos context independent. |
| csrf | native + framework-csrf/server explicitly. |
| sql | native source command/client adapter + SQL contracts; no default backend/pool. |
| generated-sql | sql + framework-sql/server; existing explicit Engine/Policy/Contributions integration, not implied by full. |
| view | Portable contracts + native source config/provider bridge; missing config still deactivates. |
| presentation | view + framework-view/presentation; explicit preferences/flash sections/Admission, no renderer. |
| session | native + framework-session/auth-server; includes email/server through the existing composite. |
| api | native + framework-api/native,sql for complete persisted source flow; catalog selects API SQL registration at SQL stage. |
| admin | Shared admin support and compiled-client admin traversal; no universal routes or required generated client. |
| full | server-commands,schema-client,language,csrf,sql,view,session,api,admin; no renderer/backend/generated-sql/presentation/network host/codegen guessed. |
| ssr | Native selected view/session/API/admin renderer forwarders; no hydrate/CSR. |
| hydrate / csr | Separately built portable UI graphs; no native registrar, command, SQL, transport or codegen edges. |
| codegen | Separate native build-host schema/SQL/admin transform helpers + API schema composition; not in full/ssr/browser. |

Use weak optional edges or separate UI feature groups so a renderer cannot re-enable an omitted runtime plugin. Browser UI does not imply session/auth-server or API/native. M0 freezes exact TOML only after isolated graphs prove these boundaries. native/full + hydrate/CSR is unsupported as one artifact; incompatible modes fail explicitly, not by picking a winner.

API/codegen currently brings native dependencies: allowed in its separate build tool, not a reason to select it in final runtime/browser graphs. Report any required existing-crate mechanics fix for a coordinator grant, rather than silently altering a constituent.

## Facade

Add optional stackpress-framework and feature/export framework after release. Aggregate forwarders use **framework-aggregate-<feature>** to avoid changing existing framework-session*, framework-admin*, framework-view* and server-http* semantics. Constituent exposure does not silently select an aggregate plugin.

Add missing optional framework_api/framework_csrf/framework_email exports and useful leaves; add current missing view presentation/schema-SQL-admin leaf forwarding only through a serial shared-file grant. Root stays a re-export layer over directly usable crates. Default normal root graph must still contain stackpress-lib only among first-party packages.

## Configuration owners

Upstream src/client/types.ts Config is UnknownNest plus these optional sections, not an aggregate enabled-plugin list. Existing ConfigLayers/typed sections and explicit cross-plugin native mappings remain.

| Source key | Rust mapping |
| --- | --- |
| brand | Public Presentation defaults and native auth Environment brand mapped explicitly. |
| terminal | CLI/app command config; no implicit TTY or prompt. |
| server | App host/mode/command settings; port does not bind IO. |
| client | Static CompiledClient/build identity and nullable access; module/build paths belong to tool mechanics. |
| cookie | Existing lib CookieOptions, no extra flags or lifetime. |
| admin | Generated/app host config/Mount; app policy retained. |
| api | Existing ApiConfig gates, endpoints/scopes/CORS/OAuth/webhooks. |
| email | Existing source EmailSection/providers/Hooks ownership. |
| language | Native source adapter key=locale, locale=en_US, languages={}; distinct from view_preferences policy. |
| database | App Engine/provider/seed; no inferred backend or table installation. |
| view | Source native config/provider/assets; optional preference/flash settings remain distinct. |
| auth | Existing AuthSection/routes/flags/menus/password; missing auth retains three reusable auth events. |
| session | Existing SessionConfig; missing access means no global guard, empty access preserves source shortcut. |

CSRF retains its own source default/optional section contract; it is not a new required Config field. Aggregate must not add default auth/session/API protection. Exact section fields are delegated to their current owning config types, not copied into a second mutable master config.

## Target artifacts

Native full host explicitly supplies Engine, compiled catalog, providers and lifecycle shutdown. Website adds ssr, real renderer/assets, SSR executor initialization, optional presentation admission/resources and Hooks bind/shutdown. Build tool separately selects codegen and owns normalization/output publication. Runtime import never runs generation/install/migrate.

Hydrate and CSR consume public views/contracts in separate modes. Their normal graphs exclude native server/SQL/CLI/TTY/SMTP/HTTP host, JWT/crypto/providers, parser/transformer/codegen. No service/catalog/provider configuration is automatically projected to PageSnapshot. Deliberate source enrollment/CSRF form values remain permitted. Library-owned G41 checks are preserved rather than centralized or weakened.

Cargo features are additive: omitting session registration does not prove session code absent when API/native needs its schema. Report registration and dependency reachability separately. Use an **API-disabled** consumer for the strong absent-crate graph proof. Runtime config cannot unload code; disabling production plugins requires rebuild/redeploy.

## Lock and resource policy

No new external tuple requested. After release, new internal edges preserve existing (name,version,source,checksum) tuples; never cargo update/generate-lockfile. No root/consumer lock changes now. New consumer locks need a separate one-time offline-resolution grant: seed root copy, resolve once without --locked only when granted, immediately assert subset and unchanged root bytes; then every command --locked --offline. Stop on a new tuple.

At a fresh M0 effort, use one recorder-wrapped initial cargo clean/llvm-cov clean and an agreed capped target, then warm sccache/bounded jobs. Do not reuse stale target/slice9a evidence for this checkpoint. No build/clean/tree, install, listener, commit/push or GitHub action occurs here; optional tree inspection would be locked/offline and recorded.
