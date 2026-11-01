# Slice 11 Rust constituents and overlap

Status: Draft / Not Frozen. Existing surfaces are inspected at af9cc01; new adapters are proposed. Read the [entry](slice-11-design-decisions.md) for authority and [target map](slice-11-design-targets.md) for selection.

## Actual runtime and build entry points

Runtime functions use fn(&mut ServerBuilder) -> lib::Result<()> unless stated. A package name alone does not imply a plugin.

| Source stage | Current Rust | Boundary / missing work |
| --- | --- | --- |
| server | stackpress_server::ServerBuilder, ConfigLayers, call/resolve/delegate; CLI run/main_with/main_with_args | No stock server registrar. Native HTTP host is separate. Need real develop/emit/serve command bridge, not an empty callback. |
| schema | framework_schema contracts, codegen::register(registry,ContextTransform), native providers | No runtime registrar. Compiled-client access and generate command need an aggregate adapter; parser/output publication stay build-host. |
| language | framework_language::LocaleRegistry/PreferenceError; stackpress_leptos::language; preference routes under framework-view | No runtime registrar. Checked preference registry is not source cookie/locale-prefix dispatch. New source adapter stays in aggregate; library remains pure. |
| csrf | stackpress_framework_csrf::plugin::register, Service | Registrar requires optional **server** feature explicitly. No global guard. |
| sql | framework_sql::plugin::register; Contributions/RegisterOperations/Service; direct stores/DataSession/migrations; codegen::register | Generic plugin requires existing Engine/Policy before Config, 1–128 Contributions before Listen. Not the source command/client plugin by itself. |
| view | framework_view::plugin::register calls preferences then flash; independent module registrars and direct PageRenderer APIs | Composite requires **presentation**, explicit view_preferences/view_flash and Admission. Selects no renderer. Source view/provider/build adapter is distinct. |
| session | framework_session::composite::register; auth::register; identity plugin::register; account::events::register | Composite requires auth-server and calls auth/email/identity. SQL collaborators explicit. CSRF/server registrar is not selected merely by auth-server. |
| api | framework_api::register / plugin::register; separate sql::register; models/actions; codegen::definitions/config/normalize; views | No api::codegen::register. Default native endpoint registrar exists; sql/codegen/UI are separate selections. SQL registration queues Listen. |
| admin | framework_admin shared Draft/pages/presentation/roles/transfer; codegen::register/register_pages; generated app pages_host::register | No universal runtime registrar. Generated Mount/host handlers/SQL Contributions stay native and app-owned. |

Sources: [server](../../../crates/server/server/src/lib.rs), [CLI](../../../tools/cli/src/lib.rs), [schema](../../../crates/framework/framework-schema/src/lib.rs), [language](../../../crates/framework/framework-language/src/lib.rs), [CSRF](../../../crates/framework/framework-csrf/src/plugin.rs), [SQL](../../../crates/framework/framework-sql/src/plugin.rs), [view](../../../crates/framework/framework-view/src/plugin.rs), [session](../../../crates/framework/framework-session/src/composite.rs), [API](../../../crates/framework/framework-api/src/lib.rs), [admin generation](../../../crates/framework/framework-admin/src/codegen/pages.rs). Existing [SQL plugin guide](../../../crates/framework/framework-sql/docs/guides/select-plugins.md) and [presentation guide](../../../crates/framework/framework-view/docs/presentation-host.md) remain authoritative.

## Current constituent features

| Crate | Features |
| --- | --- |
| server | default, http, http-stream |
| HTTP host | default, tls; supplied-IO host is not buffered conversion |
| stackpress-leptos | default, ssr, hydrate, csr, async, stream, rich, format |
| schema | default, providers, codegen |
| language | none |
| csrf / email | each: default, server |
| framework-sql | default, server, codegen, migrations, sqlite-maintenance |
| view | default, cookies, preferences, flash, web-preferences, web-flash, presentation, native-file, server, leptos, leptos-async, leptos-ssr, leptos-hydrate, leptos-csr, leptos-stream |
| session | default, native, sql, server, auth-server, leptos, leptos-ssr, browser, leptos-hydrate, leptos-csr |
| API | default, native, sql, codegen, ui, ui-ssr, ui-hydrate, ui-csr |
| admin | default, format, ssr, hydrate, csr, codegen |

Manifests are the source of truth for transitive edges. Renderer modes are separate artifacts; selecting a crate/feature never registers a host.

## Current root facade

[Cargo.toml](../../../Cargo.toml) and [facade](../../../src/lib.rs) have default=[] and only stackpress-lib as a first-party default normal dependency.

| Existing feature family | Export / meaning |
| --- | --- |
| server; server-http; server-http-host; server-http-host-tls | server, buffered HTTP, separate server_http_host, optional TLS. No bind. |
| idea-parser; idea-transformer; idea | Independent parser/transformer/tool wrapper. |
| sql; sql-sqlite; sql-postgres; sql-mysql | Query/selected backend crates; no PGlite. |
| framework-schema; framework-sql | Contract/store support; no current codegen/server forwarders. |
| framework-admin; framework-admin-format | framework_admin and formatting; no current renderer/codegen forwarders. |
| framework-session plus suffixes | -native,-sql,-server,-auth-server,-leptos,-leptos-ssr,-browser,-leptos-hydrate,-leptos-csr; same crate export, no registration. |
| framework-view plus suffixes | -preferences,-flash,-native-file,-server,-web-preferences,-web-flash,-leptos,-leptos-ssr,-leptos-hydrate,-leptos-csr,-leptos-async,-leptos-stream; no -presentation. |
| framework-language | Pure framework_language export. |
| leptos plus suffixes | -ssr,-hydrate,-csr,-async,-rich,-format,-stream. |

No root framework/framework-api/framework-csrf/framework-email export or feature exists yet. Proposed additions are optional and preserve all current meanings/defaults. New aggregate leaves use framework-aggregate-* so root framework-session does not silently become a plugin switch.

## Move, reuse or keep

| Piece | Proposed disposition |
| --- | --- |
| Server scripts | Aggregate native adapter over existing command/event/host APIs; explicit app IO bindings, no second host. |
| Compiled client | Aggregate typed catalog/service instead of dynamic import; reuse schema metadata and compiled generated registrar functions. |
| Language prefix hook | Aggregate native source adapter; keep existing locale preference validation, cookies/actions and translation components in their owners. |
| CSRF/session/API | Reuse real ordinary registrars; no policy, flag, lifetime, response or global-guard wrapper. |
| SQL commands/catalog | Aggregate operator/client adapter plus optional existing generated-SQL integration; no implicit Engine/install/migrate; accepted G35/G37/G38 review/sequence contracts stay. |
| Source view and preferences/flash | Source provider adapter plus separately selected existing presentation composite; no invented renderer/Admission. |
| Admin | Actual generated/app registrar last through catalog; no generic shared admin routes or duplicate Contributions. |
| Facade | Re-export underlying modules, not copied algorithms or hundreds of flat JS aliases. |

If M0 proves an existing crate needs a startup-only helper or target fix, report exact files/signatures and stop that dependent work for a coordinator grant. No other crate is silently edited by the aggregate effort.
