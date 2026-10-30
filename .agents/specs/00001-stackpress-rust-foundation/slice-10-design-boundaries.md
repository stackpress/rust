# Slice 10 overlaps and original session interface

Draft / Not Frozen; [G43 checkpoint](slice-10-design-decisions.md). This page is the sole session-interface assumption section. It replaces the prior opaque-caller/mandatory-can bridge proposal.

## Ownership and overlap

| Existing owner / source | API use and boundary |
| --- | --- |
| `crates/lib/src/{request,response}.rs`, Nest/headers/events | Reuse independently constructible values and raw emit. API row IDs/secrets/profileId remain ordinary source data; no new identity type enters lib. Preserve top-level source set semantics rather than using a deep merge blindly. |
| `crates/server/server/src/{config,startup,services,plugin,builder}.rs` | Function selection, optional typed section, Config→Listen→Route and frozen services. Disabled config requires no API store/renderer/transport resources and contributes no behavior. |
| `crates/server/server/src/{registration,router,context,lifecycle}.rs` | Existing RouteSpec, G42 ordering and raw Invocation::emit. Fresh request helpers must not copy cookies/session merely because invocation ancestry/host metadata is inherited. No nested top-level handle. |
| `crates/server/server/src/http*`, output and `crates/server/http` | Existing request/response conversion, owned inbound I/O/TLS/cancellation. No new API listener/parser/host or stricter network policy. Reqwest is outbound fetch mechanics only. |
| `framework-sql` Engine/stores/schema/codegen/operations | Implement Application/API Session SQL adapter and normal source-named actions now. Existing strict Rust public-operation DTOs are not substituted where they would reject the source's protocol/model input. Use low-level stores/Engine and API-owned compatibility extraction; keep unrelated Rust operation policy unchanged. |
| `framework-admin` generated pages and codegen | Select both source API models for generated management now, preserve field metadata including secret displays and related records. API owns schema/selection and host glue; no hand edit to emitted output. |
| `examples/generated-admin/web/src/host.rs` | Regression comparison for generic CRUD/transport behavior, not the implementation of API credentials. Its fixture x-admin-actor/CSRF policy remains example-owned and is not copied into API. |
| #14 SQL/select-plugins, admin/pages-host, view/presentation-host docs | Follow ordinary register functions, direct alternatives, prerequisite service order and separate generation/runtime. No mandatory hardcoded session plugin ID. |
| `framework-view`, `stackpress-leptos`, language | Port source consent pieces/brand/styles/translation with existing presentation. G39–G41 remain intact. View imports do not select API or a new theme mechanism. |
| Slice 9 implementer | JWT SessionServer/roles/can, me event, global request guard only as configured, Profile/Auth schema, CSRF/account/email. API only consumes original contracts below. |
| Coordinator | Owns root workspace/facade/lock files and any explicitly released shared lib/server/schema/admin edits; later aggregate composition. API does not edit another lane's shared files on inference. |

## Original caller and session flow

Source paths: API `plugin.ts:104–185`, OAuth `pages/oauth.ts:49–75`, token `pages/token.ts:27–49`; session `session/events/session.ts:6–10`, `session/Session.ts:123–179,193–228`; Ingest `Router.ts:382–396`.

| API path | What original obtains/passes | Minimal Rust need |
| --- | --- | --- |
| Public endpoint | No caller lookup; fixed data on original req then emit(event,req,res) | Existing mutable Request/Response and nested raw emit. |
| App endpoint | Header ID/optional secret; fresh `{id}` to application-detail; source checks; fixed data then original req/res emit | API SQL action compatibility DTO + fresh lookup request; no JWT conversion, new Caller or roles. |
| API Session endpoint | Header ID/optional secret; fresh `{id}` to session-detail; source checks; fixed data then profileId on original req/res | Existing Request.data.set handles the complete forwarding contract. API Session record remains API-owned. |
| OAuth browser caller | Resolve me with the same original Request; session event loads JWT from req.session[configured key] and returns authorization `{id:0,roles:[GUEST],...data,token,permits}` | Slice 9 supplies the session service/me event. API tests `results.id` truthiness, then passes its ID as profileId to session-create; no synthesized API caller. |
| OAuth application lookup | Resolve application-detail with new `{id}` Request and the current response | Fresh input contains only id; shared response exposes the selected application result/code for the page. Do not inherit browser cookies into lookup input. |
| OAuth creation | Fresh request with `{...req.data(),applicationId:id,profileId:me.id,expires:now+expires}`; fresh response | Explicit source copy/override semantics, model-owned filtering/defaults. Current browser JWT is not saved into API Session. |
| Token exchange | Fresh `{id:code}` session-detail lookup; uses joined profile to return user subset | API adapter returns SessionExtended; no call to me/can and no JWT authorization service inside this page. |

SessionServer.load(Request) reads the configured JWT cookie/session value; it does not read API ID:secret from Authorization. SessionServer.can is owned by Slice 9, with original role/pattern semantics. API app/session scopes do not call can automatically. A configured session request guard or developer-selected route/event may check permissions; absent session/access configuration is not replaced by API protection. The two identity sources remain distinct even if both appear on one HTTP request.

No API request-local caller slot is needed. The earlier SQL-policy/signature blockers arose from the rejected hardening design and are removed. Existing OperationPolicy remains application-selected for existing Rust generated operations; it is not an authorization layer inserted into API's source-compatible model handlers.

If Slice 9 chooses a request-local JWT cache to implement original per-Session-instance memoization, ownership is: Slice 9 implementer owns framework-session cache/accessor types; coordinator owns/release-assigns `crates/lib/src/request.rs` and its tests if a per-request slot is necessary. Coordinator similarly owns `crates/server/server/src/{context,lifecycle}.rs` if its adapter slot must be used. The API implementer owns none of those files. A cache must not make a freshly constructed lookup request inherit the browser JWT; do not infer identity from root invocation context or cache across callers.

## Target and dependency boundaries

Shared DTO/config/consent types can compile for browser/WASM; native runtime/SQL/outbound fetch are explicit features and never follow a browser import automatically. Runtime plugin offers the full original behavior when selected with api config; Cargo packaging may separate server/sql/ui for target mechanics but may not silently drop configured OAuth/token/management pages. No outbox, OAuth-server SDK or new role engine is required.

The API SQL adapter reuses the Profile schema supplied by Slice 9 and owns only Application plus API Session schema/tables/relations/default providers. Both slices share approved ring HMAC/randomness utilities by dependency/use pattern, not a new common security crate. Root aggregate order remains source order; aggregate implementation is coordinator/later task ownership, not a deferral of any API feature.
