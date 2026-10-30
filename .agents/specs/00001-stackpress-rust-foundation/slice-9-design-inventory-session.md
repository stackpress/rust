# Slice 9 session inventory under G43

Status: Revised design only. **Now** means full Slice 9 scope after implementation release; source parity is binding. `S/` denotes upstream `packages/stackpress-session/src/`; [provenance](slice-9-design-sources.md) pins files. Reasons marked “call-flow interpretation” explain use, not undocumented owner intent.

## Root and package exports

| Every relevant source surface | Disposition / why | Rust mapping and mechanics |
| --- | --- | --- |
| `index.ts` Session (default class name SessionServer); `./plugin`; `./types`; type-star from auth/profile/session | Now: reusable identity and framework wiring | Native Session object and configurable Service; ordinary plugin function and explicit re-exports. No revocation store. |
| `actions = AuthActions`; StackpressSessionException; `./Exception` | Now: reusable actions/errors outside pages | Native Actions/typed source-shaped error envelope and retained cause. Alias need not reproduce a JS constructor object. |
| matchAnyEvent, matchAnyRoute, matchEvent, matchRoute, isRegExp | Now: permission patterns are part of configured policy | Preserve exact fast path, derived regex, flags and method rules. Do not silently substitute lib's case-sensitive escaped-literal matcher. |
| `./auth/views/*`, `./session/views/*` and esm/cjs aliases | Now all behaviours | Leptos equivalents for every named form/body/head/page/helper in [page inventory](slice-9-design-inventory-pages.md). React/npm module packaging is the mechanics-only non-port. |
| styles.css, package plugin/build/typeVersions/npm metadata | All presentation intent now; packaging not ported | Existing Rust UI ownership/styles plus Leptos; Cargo features/compiled registration replace npm discovery and dynamic import. |

## Session types and methods

| Source item / lines in Session.ts unless stated | Why / exact behaviour | Rust mapping |
| --- | --- | --- |
| SessionConfig {key?,seed,access?} (`session/types.ts:11–20`) | Configurable identity policy; implementation supplies key=session, seed=abc123, access={} | Optional typed section with presence retained separately from values; same defaults, no minimum key strength or secret-service-only replacement. |
| SessionRoute {method,route}; SessionPermission string-or-route; SessionPermissionList role map | Same permission representation for events and routes | Tagged Rust enum/ordered role map with extension serialization; source request and grant forms are not replaced by a new protected-permit language. |
| SessionData {id,name,image?,roles,...any} | JWT payload can carry application fields | JSON object plus typed accessors; retain arbitrary claims and source missing/empty values. No new required-claim policy at verification. |
| SessionTokenData adds token and permits | Consumers such as me/OAuth/view can read authorization data | Serializable source-shaped DTO, including raw token and guest id=0. Do not replace with a nonserializable-only caller type. Native caches remain separate. |
| SessionServerConstructor / SessionPlugin; access/seed/key getters; configure (31–47,95–104) | Reusable configured constructor | Per-host Arc runtime replaces a mutable JS class/global mechanically; direct configure/getters remain available. Configuration mutation affects subsequent uncached loads; cached data stays cached. |
| expires setter defaults 0 (22,52–54) | Optional expiry, caller chooses absolute time | Preserve 0=no added expiry; nonzero NumericDate passed as exp. No ttl_seconds, expiry requirement or sliding window. Plugin does not configure expires. |
| create(data) (109–117) | Sign identity without SQL lookup | JWT HS256, protected header only alg, issued-at overwritten with current time, all other claims retained; set absolute exp only when configured. Rust library must disable its default typ header and unwanted claim defaults. |
| token(req) (123–127) | Reads normalized request cookie/session state | Only req.session[configured key]; comment mentions header but implementation does not. No Authorization fallback, duplicate-cookie policy or new cookie prefix. |
| load(string|Request), constructor(token), readonly token (133–149) | Direct token and request use share one object | Own token and runtime reference in Session. Missing request token becomes empty string. Native explicit-token APIs remain available without plugins. |
| data() and _data cache (166–179) | Verify at most once per instance and treat verification failure as unauthenticated | Async-shaped wrapper over local library verification; cache None on any JWT verification/parse failure. No DB read, unavailable→503 policy or live role/revocation recheck. Synchronization is Rust mechanics, not new authorization. |
| authorization() (152–160) | Stable guest defaults merged with whatever valid claims contain | Merge id=0, roles=[GUEST], then data, then token and permits; preserve arbitrary claim values and source order. |
| guest() (185–188) | Whether data verification returned null | None=true; a verified object with missing id is not automatically Guest. |
| permits() (219–228) | Aggregate grants for token roles; Guest grants when roles absent/data null | Source JS truthiness matters: [] is truthy and grants none, unlike missing roles. String grants deduplicate by value; route objects by reference identity. Represent grant identities/Arc entries so repeated roles do not add the same object twice, while separate equal route objects remain separate. |
| can(...permits) (193–213) | Every requested capability must have some grant | Empty requested list=true; otherwise all-of over event/route helpers. No new empty-protected-declaration rejection, authenticated-only prerequisite, expiry refresh or row-policy rule. |
| authorize(req,res,permits) (60–89) | Optional coarse route check that can include extra event permits | Empty access immediately true, without loading token, mutating permits or setting result. Otherwise prepend current uppercase method/path to mutable Vec, call can, set original 401 Unauthorized on false, authorization results on true. Preserve input mutation. |
| save(res) (234–236) | Stage token in outgoing cookie-backed session map | res.session.set(configured key, token), retaining existing global/per-key cookie handling; no automatic Secure/HttpOnly/rotation/TTL flags. |

## Matching semantics and intent

| Source surface / line | Why / behaviour | Port requirement |
| --- | --- | --- |
| helpers.ts:6 isRegExp | Recognizes explicit slash-delimited patterns/JS flags | Port recognizer as is_regexp; do not classify every pattern as an escaped literal. |
| helpers.ts:12–44 matchAny* | Exact/some-grant helpers, distinct from Session.can all-of | Any matching grant suffices for each requested permit. Empty grant list=false. |
| helpers.ts:50–78 matchEvent | Hyphen-segment wildcard language or explicit regex | Exact shortcut; otherwise replace * then paired stars to **, anchor derived pattern and use ig. Existing regex punctuation remains meaningful. No automatic escaping/security narrowing. |
| helpers.ts:85–126 matchRoute | Method test plus slash-segment patterns | ALL or exact method string; exact route shortcut; :param→*, * one slash segment, ** spans; derived regex ig, explicit regex uses supplied flags. No new trailing-slash tolerance from lib router. |
| Existing Rust regex/backend grammar | Rust mechanics fit gate | Use source-compatible derivation over existing regex facilities where equivalent; g has no persistent cursor because source creates a fresh RegExp each call, y must preserve sticky-start semantics. Existing Rust backend lacks some JS constructs; test and report unsupported syntax rather than silently weakening/changing it. No new matcher dependency is approved here. |

## Registration and events

| Source anchor | Why / source behaviour | Rust contribution |
| --- | --- | --- |
| root plugin.ts:13–15 | Groups auth→email→session, including email reached through session | Composite calls those ordinary registrars in that order. No nested app.plugin; direct constituents remain usable. Do not replace with identity-only registration. |
| session/plugin.ts:12,14–19 | No configured session means no identity capability | Optional typed section checked before adding source session contributions. Missing config is a no-op, not a startup error; default getters remain available for explicit direct use. |
| session/plugin.ts:22–24 | Register reusable me and authorize only when session active | Same Listen events; no mandatory per-event role/CSRF layer added. me returns original authorization DTO. |
| session/plugin.ts:26–31 | Access list presence opts into global request evaluation | Missing access: no request listener and all routes allowed by this mechanism. Present {}: listener exists but authorize allows all. Preserve that distinction. |
| session/pages/authorize.ts:6–16 | Cooperative request guard respects prior response, delegates authorize, forwards failures/error | Preserve body/non-200 skip, copied status and explicit error event. Do not add Abort/default-deny when source returns normally; G42 lets each handler choose to abort. |
| session/events/authorize.ts:6–9; events/session.ts:6–10 | Small adapters around direct APIs | Preserve outputs and error sequencing; no new dispatcher or public projection filter. |
| session/plugin.ts:34–89 | Account routes depend on session integration and auth config | With no session config the subplugin registered nothing; with session but no auth, no account routes. All listed routes/views ship now. |
| auth/plugin.ts:9–17 | Auth Listen events are separate from its auth-config Route gate | Keep auth-signup/signin/signout listeners even when auth config is missing; route absence does not remove those reusable events. Missing runtime dependencies fail when source would consume them, not through new eager policy. |
| aggregate stackpress/src/plugin.ts:16–24 | Dependency wiring order | server→schema→language→csrf→sql→view→session→api→admin; email through session. Final aggregate package belongs to its scheduled group, but this slice provides all its constituent behaviour now. |

No test files exist in the three maintained source package trees at capture; package scripts naming tests do not prove a parity suite. Verification must use source-derived fixtures, not the rejected hardened design as oracle.
