# Slice 10 configuration and endpoint contracts

Draft / Not Frozen; [G43](slice-10-design-decisions.md) is binding. Existing foundation/host representational budgets continue; no new API inventory cap, key-name denylist, exposure allowlist, destination restriction or default-deny behavior is added.

## Every source config key

| Source key | Rust mapping now | Source default / mechanics |
| --- | --- | --- |
| `api` | `Option<ApiConfig>` section registered by selected function | None means disabled, no required resources or callbacks. Typed present data must be representable; invalid type is a mechanics diagnostic, not automatic enabling. |
| `api.expires` | Optional millisecond number | Only OAuth POST consumes it; omitted defaults to `1000*60*60*24`. Source comments say never expire, but page code is authority. Keep zero, fractional and negative representable values; no positive TTL rule or separate code/access lifetime. |
| `api.webhooks` | Ordered Vec<ApiWebhook> | Omitted empty. Listen registers each supplied event at -200. Endpoint list does not gate webhook registration. |
| `webhooks[].event/uri/method/validity/data` | Event string, configured URI/method, nested predicate/fixed data | Method/URI forwarded to fetch; data has `|| {}` fallback. validity is required by the source type and traversed as supplied; no signature/retry policy added. |
| `api.scopes` | Record<String,ApiScope> | Omitted empty; used to describe requested scopes in view. Endpoint/app/session scope checks use their own arrays, not this registry as a whitelist. |
| `scopes.<id>.icon/name/description` | Optional icon and text | Same presentation; Lucide name mapping is a rendering mechanics change, not a scope restriction. |
| `api.endpoints` | Ordered Vec<ApiEndpoint> | Omitted/empty means no endpoints and no OAuth routes/view. JS non-array check becomes typed-config decoding for malformed present values. |
| Endpoint `name/description/example` | Optional inert metadata | Keep supplied values; these do not confer permission or select a special response policy. |
| Endpoint `method/route` | Existing MethodPattern/RouteSpec | Explicit source method/path, including ALL/captures; no GET→HEAD exemption. No method/path exclusivity. |
| Endpoint `type` | ApiType public/app/session | Required typed variant; unknown value's inability to select an adapter is reported by typed decoding, not treated as public. |
| Endpoint `event` | Runtime configured event name | No added compiled exposure allowlist; ordinary static event listeners are compiled/selected separately. Config points to an event; it does not import a Rust module. |
| Endpoint `priority` | Source numeric priority mapped to server signed priority | `priority || 0` semantics: omitted/zero→0; negative nonzero retained. Unrepresentable numeric priority gets a mechanics error, never silently truncated. |
| Endpoint `data` | Source top-level set overlay | Source uses `endpoint.data || {}`; absent typed default empty. Each top-level key replaces that value; no recursive fixed/caller merge policy or reserved authority-key rejection. |
| Endpoint `scopes` | Optional Vec<String> | `|| []`; app/session need at least one overlapping element. Empty or omitted arrays deny 401. Public ignores scopes. Unknown registry names are not rejected if actual arrays overlap. |
| Endpoint `cors` | false/true/string/list-compatible enum | False/omitted: no CORS callbacks. true becomes wildcard. String wraps in list. Empty list still creates callbacks; no origin is matched. |
| `view.base/props/notify`, `brand.name/logo/icon/favicon`, language | Existing presentation config | OAuth copies selected view/api/brand fields with source `||` fallbacks; no new redaction/projection rule. Keep all props the page actually selects. |
| Model defaults/validators/admin config | API-owned schema + selected SQL/admin adapter | Existing database/populate/admin capabilities retain separate application configuration. API registration never runs install/seed itself. |

Dates use an explicit Rust source-Date carrier: millisecond arithmetic plus source wire serialization, with absent/invalid values represented where source permits them. A typed arithmetic/serialization failure must be reported honestly as a mechanics limitation; it is not permission to impose a new lifetime or reject user-supplied state flags. M0 tests zero/negative/fractional and Date range edges against the source oracle.

## Handler-by-handler execution

`authorize` reads the same Authorization header abstraction, calls split(' ') and uses segment 1, then split(':') and the first ID/secret. It does not validate the scheme, collapse arbitrary whitespace or enforce one colon. It checks trim-nonempty ID/token and, for non-GET, trim-nonempty secret; returns trimmed token/ID/secret or sets Unauthorized 401. Do not replace exact equality with a different accepted-input set. Approved subtle/ring availability does not mandate extra verification.

`open`: set configured data at top level, raw-emit configured event using original request/response, then Continue. It adds no auth/can/CSRF/prior-response guard. Each target/later handler owns its own checks.

`app`: authorize; resolve application-detail on a fresh request containing `{id}`; if no results, Unauthorized; compare supplied secret to stored secret only when uppercased method is not GET; require `.some(scope => permits.includes(scope))`; set fixed data; emit on same original request/response. Do not add expiry/active/client-role checks beyond what the selected lookup event does.

`session`: same sequence against session-detail, then set fixed data followed by `profileId = session.profileId`; raw-emit same request/response. It neither loads a JWT from this API row nor writes roles/session/caller metadata into a new slot. Input profileId and fixed profileId both lose to the selected row's profileId in this adapter; app/open retain the data they already carry.

Typed `resolve` mechanics creates Request/Response values for object input and emits a nested event; original Request input reuses it; an explicitly supplied response reuses it. Preserve host service access/ancestry without a new request/response lifecycle. Unlike `delegate`, raw `Invocation::emit` does not turn NoListener into a new error. API wrapper awaits then ignores nested Completed/Aborted/NoListener just as source ignores emit's return. Source runtime/handler errors still propagate normally.

Unauthorized sets response error then returns from this handler, represented as Continue, not a new automatic Abort. Likewise successful wrappers return Continue. Other listeners can inspect response/abort as they choose; API does not install a universal completed-response guard on endpoints. OAuth/token page entry guards remain only where source has them. Existing output commitment cannot be reopened; the Rust transport contract is not rewritten.

## Exact source CORS behavior

Create OPTIONS and actual-method callbacks at priority 1000 when `cors` is an array after normalization. Wildcard list emits Access-Control-Allow-Origin `*`; otherwise compare list entries with `req.url.origin`, not incoming Origin, and emit that URL origin when matched. If unmatched, return without headers. This does not reject or authorize the actual request.

When matched, set response `Access-Control-Request-Method` to endpoint.method, or `*` for ALL; `Access-Control-Allow-Methods: OPTIONS, GET`; `Access-Control-Allow-Headers: *`. Do not correct the response header name, add credentials, Vary, advertised methods or extra origin validation as an API default. These exact lines are part of G43 parity.

Preflight calls the same helper then sets status 200 unconditionally and returns Continue. It does not abort the chain. An ALL endpoint can therefore also see OPTIONS and follow its normal non-GET credential rule; do not add the earlier draft's business-dispatch skip. Multiple CORS/endpoint declarations may share a path; source registration/priority/FIFO determines header mutations and invocation order, with no policy union or framework arbitration.

Public/cookie/header caller behavior, scope risk and CORS limitations are documented developer responsibilities with parity tests. Optional application/plugin handlers may enforce different rules explicitly, but the API port does not select them automatically.
