# Slice 10 upstream inventory and intent

Draft / Not Frozen; binding [G43 dispositions](slice-10-design-decisions.md). Source prefix `A` is canonical `packages/stackpress-api/`; `T` is `templates/blog/`; all source stays read-only. Everything behavioral below is Now. Only language/distribution mechanics are replaced, not deferred features. Intent interpretations explain a coherent use without claiming private knowledge of the author.

## Exports and types

| Export / source | Why it exists | Rust mapping now |
| --- | --- | --- |
| Root `index.ts:1–21`, package root export | Stable public API/types separate from plugin setup | Explicit crate reexports and `stackpress::framework_api`; importing types registers nothing. |
| `ApiType = public/app/session`, `types.ts:13` | Different callers need different local admission, not one universal policy | `ApiType::{Public,App,Session}`; retain source serialized names. API Session is a DB record, distinct from JWT session service. |
| `ApiConfigProps {language,view,brand,api}` | Server-configured consent presentation | Typed props with the same selected config and application response, no newly filtered public projection. |
| `ApiOauthInputProps {client_id,scope,state,redirect_uri}` | Request carries chosen app, desired scope display and callback correlation | Same named fields; no response_type/code_challenge requirement added. |
| `ApiOauthFormProps {appName,revert,items{id,icon,name,description}}` | Reusable Grant/Deny form and permission display | Leptos props/controls preserve names, scope[] submission and denial link. |
| `Scopes`; `ApiScope {icon?,name,description}` | Human descriptions for requested permission IDs | Registry of display metadata; not a new scope whitelist. |
| `ApiEndpoint` all fields, `types.ts:46–60` | Declarative adapter from a method/path to an existing event | Every field is mapped in configuration; no new endpoint ID/exposure capability table. |
| `ApiWebhook`, `types.ts:68–76` | Event/result-filtered external notifications | Event, URI, method, validity and data; no extra signing/queue configuration. |
| `ApiConfig`, `types.ts:79–91` | Optional activation, lifetime, endpoints, descriptions and subscriptions | Optional typed section with expires in milliseconds; defaults remain at each source consumer. |
| `Application`, `ApplicationExtended`, `ApplicationInput`, `types.ts:96–120` | App metadata + reusable ID/secret and editable model input | Source shapes, including plaintext secret/scopes/active/expires/timestamps. Schema adds owner profileId/profile absent from handwritten type. Extended remains the alias unless relation projection is requested. |
| `Session`, `SessionExtended`, `SessionInput`, `types.ts:122–147` | User/app authorization record; Extended joins application/profile for token page | Keep API Session name in API module, joined relation DTO and model input semantics; no grant/code/token split. |
| `StackpressApiException` root alias; `Exception.ts:8` default subclass | API errors carry existing framework response shape | `ApiError` compatible with lib error/status; Unauthorized remains 401, no forced 403/Bearer challenge. |
| `plugin.ts` default plus named `cors/session/app/open` | Composite setup plus individually reusable route helpers | Ordinary function register and direct route helper functions, existing server phases/dispatcher. |
| `helpers.ts` `authorize/unauthorized/validData` | Shared local header parsing, response error and predicate | Equivalent functions and JS comparison branches represented in Rust; Q10-REF tracks reference representation only. |
| `pages/oauth.ts` default; `pages/token.ts` default | Consent/session creation and exchange using normal events | Named async page handlers with existing request/response/invocation signature. Token has no separate HTML view upstream; port its handler now, not an invented token screen. |
| `views/oauth.tsx` ApiOauthForm/OAuthBody/ApiOauthHead/ApiOauthPage/Head/default | Form/body/head/layout are reusable presentation pieces | Equivalent Leptos parts and page registration; labels, fallbacks, brand/favicon/styles preserved. |
| `styles.css` instructions/scope/container/description/deny rules | Consent layout and familiar auth styling | Port with existing Rust presentation tokens/Leptos controls; Lucide replaces icon markup under project family rules. |
| `schema.idea`, generated admin/model actions | Persistent records, default keys, relation selectors and admin management | Full SQL adapter/schema/operations and management pages now; no key masking, hashing, one-time display or new rotation protocol. |
| CJS/ESM paths, declarations, TS configs/build scripts/typesVersions | Node distribution of the same source | Mechanics replacement by Cargo/features/Rustdoc; these artifacts themselves are not Rust runtime features. Preserve package/license responsibility in docs. |

## Model fields and management

| Source model/field | Source intent and Now mapping |
| --- | --- |
| Application `id` (`schema.idea:9–14`) | Default cuid() ID is both model identity and read credential; keep default, optional input override and list/detail display. |
| Application `profileId/profile` (`16–32,111–112`) | Owner relation and profile chooser; schema is more complete than handwritten Application type. Join Profile from Slice 9, not a duplicate profile store. |
| Application `name/logo/website` (`34–56`) | Display/branding for three-party consent and admin search; website is ordinary metadata, not a newly imposed redirect registry. |
| Application `secret` (`58–63`) | Source description explicitly says write-operation credential; cuid() default and list.string/view.string are intentional management affordances. Store and display it as source; updates use normal model operations. |
| Application `scopes/active/expires/created/updated/session[]` (`65–109`) | Editable service labels, active/list filter, date and audit metadata, reverse sessions. Keep declared defaults/validators/timestamps and query selection; API adds no additional active/expiry/revocation check. |
| Session `id/applicationId/profileId` (`121–162`) | Persisted code/read credential bound to app and user, relation choosers; same fields and defaults. |
| Session `secret/scopes/active/expires/created/updated` (`164–212`) | Write credential, granted scope values, row state/date/timestamps; source is editable through generated management, not an immutable/hashed token vault. |
| Session `application/profile` (`214–218`) | Extended detail used by token response, including joined public user subset selected there; SQL adapter supplies equivalent relations. |
| Both Input types' optional id/secret/scopes/active/timestamps | Source generated model filtering/population/assertion decides model writes. API does not reject those flags by name. Unrelated protocol fields are filtered by the model store, not rejected as mass assignment by API. |
| Labels/icons/display/query/searchable/sortable/relation/UI metadata | Generate current Rust equivalent of all source administration. Apps and Sessions menu entries `T/config/common.ts:52–61` lead to `/admin/application/search` and `/admin/session/search`. |
| Generated action inventory | SQL `transform/events/index.ts:18–29,126–128`: batch/create/detail/get/purge/remove/restore/search/update/upsert, with restore selected by restorable metadata. Register original hyphen event names for both models. Existing Rust stores may be reused internally; do not expose only the narrower admin transport subset. |
| App/key provisioning usage | `T/config/common.ts:280–285,419–424` and website `plugins/store/populate.ts:20–25` create Application by event. Keep direct population/create and schema-selected generated administration. No dedicated key-rotation/revoke page exists in API src; ordinary update secret/active/remove/restore is the original management mechanism. |

## Behavioral rationale and parity obligations

| Source behavior / lines | Why it exists (comment/usage or interpretation) | Now contract |
| --- | --- | --- |
| `plugin.ts:19,21–48` absent config, separate phases | Explicit configuration selects effects; endpoints and webhooks have distinct lifecycle jobs | Missing api contributes nothing; present api registers Listen/Route; empty endpoints still permits configured webhooks. |
| `plugin.ts:49–51` OAuth tied to endpoints | Interpretation: enabling API endpoint surface also enables its credential flow | ALL `/auth/oauth/token`, ALL `/auth/oauth`, view -100 only after nonempty endpoint array. |
| `helpers.ts:14–33`, schema secret descriptions | Read credentials and stronger write credentials are distinct source concepts | Ignore scheme string; split on literal space then colon; use first ID/secret segments and source trim order; GET may omit secret. HEAD/OPTIONS are non-GET. |
| `plugin.ts:115–134,151–169` local detail/scopes/data | Reuse replaceable data events and bind session data to the user before business dispatch | Fresh `{id}` lookup, any-match scope check, fixed top-level data, then session profileId overwrite, same request/response emitted to target. |
| `plugin.ts:134,169,181` ignored emit return | Interpretation: nested business chain and outer route chain remain separate processes | Child Abort stops its child chain; wrapper returns Continue because source ignores the returned value. No-listener does not become a new API error. |
| `plugin.ts:66–101` CORS | Per-endpoint convenience headers are separate from admission | Preserve URL-origin test, response Access-Control-Request-Method, `OPTIONS, GET`, wildcard headers, status 200 and Continue; no Vary/Origin-header rewrite. |
| `pages/oauth.ts:16–18,21–35,49–63` | Respect earlier response, share selected config, reuse browser identity and app detail | Same guard and config defaults, same me request, sign-in continuation and shared application response. |
| `pages/oauth.ts:65–86` source input spread and code | Normal session creation remains a plugin-owned model operation | Full copied data followed by app/profile/expiry overrides; expiry default 86400000 ms, optional state echoed; persisted Session ID is code. Preserve checking current res.code after creation. |
| `pages/token.ts:14–49` | Exchange joins app/user session into stable read/write credentials and user info | Secret presence required, not equality to client secret; app ID and strict-past expiry checked; same code can be exchanged repeatedly. Payload unchanged. |
| `views/oauth.tsx:69–113,130–172` | Requested known scope labels aid consent; custom callbacks/branding are application choices | Unknown requested scope labels omitted, scopes[] switches, Deny error=denied URL, Sign Up body heading/Grant Access title, existing head/style/layout defaults retained. |
| `plugin.ts:24–41` webhook side effect | Late-priority listener observes successful operation results and awaits notification | Code exactly 200, validity filter, exact `{data,params,results}`, one inline fetch at -200. No source retry/signature/idempotency guarantee. |
| `helpers.ts:45–58` predicate | Object subset plus contains-all list membership selects notifications | Missing fails, arrays must be arrays and includes every expected element, nested object recurses, scalar strict equality; complex array references follow Q10-REF. |
| Aggregate `packages/stackpress/src/plugin.ts:16–24` | Ordered capabilities compose rather than a new application framework | Preserve server→schema→language→csrf→sql→view→session→api→admin in later aggregate task; current crate offers direct functions and typed resources now. |

No `stackpress-api/tests/` directory was present; manifest test glob alone is not parity evidence. Every behavior above has a planned source-parity case in verification. Mechanically necessary replacements are enumerated in configuration/boundaries/assessment, with no security improvement claimed.
