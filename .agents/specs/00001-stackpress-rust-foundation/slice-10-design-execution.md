# Slice 10 OAuth, SQL, management and webhooks

Draft / Not Frozen; [G43](slice-10-design-decisions.md) replaces the first draft's separate token grants, outbox, security policies and deferrals. All original API capabilities land now. Scope/defaults are established by source, not by the prior OAuth BCP research.

## OAuth page

Port `pages/oauth.ts` in order. Entry returns Continue when res.body is truthy or res.code is nonzero and not 200; no broader completed-response guard is invented. Read view/brand/api config, place `{scopes,endpoints}` in response.data.api, view base/props/notify and brand fields with source `||` defaults. Keep selected application results available to the page; do not newly remove its secret or fields from these props.

Require truthy client_id and redirect_uri, else Unauthorized 401. State is optional free caller data. Resolve me using the original Request; when results.id is falsy, redirect to `/auth/signin?redirect_uri=` plus encodeURIComponent(pathname+search). Resolve application-detail using fresh `{id}` and the current Response. If current response code is not 200, set Unauthorized. No registration of allowed callback URIs, client scope maximum intersection, explicit CSRF or can check is added by API; developer-selected processes may provide their own.

Only exact method POST creates a Session. Copy all req.data, then override applicationId from client_id, profileId from me.results.id and expires with new Date(now + expires), using the destructuring default 86400000 ms only when expires is absent/undefined. Model filtering/defaults/assertions decide stored fields; API does not reject supplied id/secret/scopes/active/created/updated. Zero and negative expires remain source inputs. Preserve a distinction between missing and present values wherever JavaScript conversion affects this expression; do not accidentally implement a truthy/positive-TTL fallback. The next branch checks the current response code and presence of the fresh creation response.results, not the creation response code. Preserve that distinction.

Split redirect on `?`, use the first URI and first query segment, replace/add code with created Session.id and state only if truthy, then redirect with the source helper. Do not switch to a standardized 303/no-store or registered-destination rule without owner instruction. No code record, PKCE verifier, consumption flag, token-family revocation or separate access-token TTL exists.

## Token page

Port `pages/token.ts:7–50` as an ALL-method page, preserving the same entry guard. Call authorize, require code plus nonempty returned secret even on GET, resolve session-detail on fresh `{id:code}`, require a result whose applicationId equals header ID, and reject only when a present expires Date is strictly earlier than now. Equality passes. There is no client-secret equality lookup or consume/update call in the token page. A repeated valid request returns the same stored credentials.

Set results exactly to token_type Bearer, access_token=data.id, access_secret=data.secret, expires_in=floor((expires-now)/1000) or 0, and user `{id,name,image,created}` from joined profile. Preserve source result/body/status conversion rather than forcing a new OAuth envelope/error/header scheme. JavaScript Date becomes an explicit Rust timestamp/wire carrier; malformed/missing joins retain the equivalent error branch or mechanics error, not a fabricated authenticated user.

## Consent view and proposed acceptance

Port ApiOauthForm, OAuthBody, ApiOauthHead, ApiOauthPage/Head/default now in Leptos. The source form uses POST, switches named scopes[], Grant submit and Deny anchor. It uses requested scope.split(' ') to display only known registry entries; unknown IDs do not become a request error. With no displayed items show the source generic access request sentence. Build denial URI through the same query split and error=denied. Preserve source title Grant Access and body heading Sign Up unless the owner separately changes copy.

Brand logo/name/fallback, favicon MIME selection, `/styles/global.css`, extra styles, LayoutBlank equivalent and scoped consent styles remain. Lucide is the project rendering replacement for source Font Awesome; translation/control/layout differences are Leptos mechanics. G39/G40 Light/Dark/initial OS/no-JS behavior is consumed, not reimplemented. React and Leptos text escaping are equivalent rendering mechanics; do not insert new URI admission or model-field masking into API defaults.

| Proposed acceptance control | Owner-observable evidence |
| --- | --- |
| 00021-001 | Missing browser user follows the original sign-in continuation; a present user sees source-selected app/config and known requested scope labels. |
| 00021-002 | Grant POST sends scopes[] and other ordinary form data; Deny navigates to source error=denied URI. Both work without hydration and by keyboard. |
| 00021-003 | Source Unauthorized/prior body/non-200/create-result branches render/redirect exactly; token endpoint shows source JSON credentials and user fields rather than a new HTML token view. |
| 00021-004 | At 320px/tablet/desktop, long app/scope labels wrap with no horizontal overflow, visible focus and associated control labels. |
| 00021-005 | Existing Light/Dark, first paint and no-JS Light remain; translated source phrases/head/favicon/styles are present. |
| 00021-006 | Application/Session generated management supports original displayed secrets/scopes/dates/relations and create/search/detail/update/copy/remove/restore/import/export as the selected model inventory permits. No substituted key console. |

These are proposed UI criteria pending coordinator/owner review, not completed visual acceptance. Icon implementation loads iconography skill; any later preview uses devmetrics. This stage starts no listener.

## SQL adapter and original app/key management

Ship API-owned Application and API Session schema, typed stored/input/extended records and a real SQL adapter using existing Engine/backend contracts. Application has owner profileId/profile, ID, name/logo/website, secret, scopes, active, expires, created/updated and sessions; Session has ID/applicationId/profileId/secret/scopes/active/expires/created/updated plus joined application/profile. Slice 9 owns Profile; do not create another JWT/session-identity table named Session for it.

Reproduce source defaults cuid() for ID/secret, [] scopes, true active, now creation, update timestamp, required name/profile/application/expiry and declared relations/query metadata. Store plaintext secret because source later compares/displays/returns it; do not hash it or change it to a one-time reveal. Input-provided model fields follow source store filter→populate→serialize/unserialize→assert behavior. Extra protocol fields from OAuth spread are model-filtered away rather than causing strict DTO rejection; existing supported model fields such as active remain eligible. Q10-CUID is the unresolved provider/dependency mechanism, not a deferral of storage.

Register original hyphen-named application/session batch/create/detail/get/purge/remove/restore/search/update/upsert events using existing event host and operation selection. Reuse generated stores/SQL builders but add API-owned compatibility handlers where Rust public-operation DTO envelopes/policy would change source inputs. Source generated events have their own body/non-200 and missing-database pass-through checks. Do not silently install new role/can rules through these handlers; custom handlers can enforce their own checks and stop chains.

Source SQL `transform/store/select.ts:515–548` supplies default active filtering and `eq.active=-1` escape in its selection builder. Adapter mirrors the actual selected action/query semantics, not a blanket promise that all inactive rows are visible or that API universally rejects them. Endpoint code itself adds neither active nor expiry check; a custom detail event can return a different row. Pin default detail/search/get/update/remove/restore behavior with real database tests, including relation selection `*`, profile.* and application.* and not-found/validation/error responses.

Keep backend concerns in existing SQL adapters. Verify SQLite with an actual temporary database, and PostgreSQL/MySQL query/row conversion in their existing opted-in lanes when available; do not claim live backend acceptance from rendered SQL alone. API schema compilation and packaged SQL adapter are complete scope now even if live credentials for a backend are absent. Registration takes an already opened Engine/Profile/provider mapping and never installs, migrates or seeds automatically. Explicit app migration/population remains the normal mechanism.

Generated admin selection includes Application and Session metadata exactly: list.string/view.string secrets, name/search, relation chooser URLs, scopes tags, active/filter, dates/sort, profile/application embeds, related sessions. Source uses generic admin, not dedicated application/key pages in API src. Reuse current framework-admin page families and app host glue, with API-owned codegen selection; regenerate owned outputs, never hand-edit output. Population via application-create, editing a secret through update and active/remove/restore through normal model actions implement original management now. Do not add mandatory rotate/revoke confirmation, mask or new authorization protocol.

Source admin/SQL/auth/view remain composable independent plugins. API runtime should not seize admin route ownership or create a second dispatcher. Actual selected routes keep each process's explicit checks; unrelated existing generated-admin fixture policy is unchanged. Any shared framework-schema/admin generator gap is reported to the coordinator with an exact file grant before implementation; it is not solved by dropping management scope.

## Inline outgoing webhooks

After api config activates, Listen registers each webhook.event at -200. On invocation, skip unless res.code is exactly 200, derive data=webhook.data||{}, params=req.data(), results=res.toStatusResponse().results, and skip when validData fails. Await fetch(uri,{method,headers:{Content-Type:application/json},body:JSON.stringify({data,params,results})}). Capture the same source phase/priority; status 200 alone is not an outbox/commit guarantee, and no extra success/error/body/approval predicate is added.

Port validData: every expected key must exist, expected array needs actual array and includes every expected element, expected object recurses, other values use strict equality. Missing differs from explicit null; numeric/string/bool distinctions remain. Object/array elements in contains use reference identity (Q10-REF), never silent deep equality. JSON serialization retains omitted/missing behavior and Date-to-wire conversion; no webhook field redaction or approved public projection is inserted.

Source awaits one fetch, ignores a resolved HTTP error status, and lets fetch rejection propagate through the existing event error path. No framework retry, delivery ID, deduplication, signature, queue, lease, persistent attempt state or remote-acknowledgement claim is provided. Cancellation drops the active Send future; already accepted external effects are not undone. A caller may invoke the event again, potentially sending another webhook; applications/receivers own any idempotency they need.

Use approved reqwest behind a small Fetch seam to reproduce ordinary fetch methods/body/headers, configured HTTP/private URIs, and redirect behavior. Fetch standard redirect limit is 20; reqwest's different default must be adjusted as a mechanics compatibility setting. Do not add HTTPS-only, address allowlists, DNS pinning, private-address blocking, no-redirect or API-imposed deadlines/body caps. Inherited host/runtime/network errors still exist and must be described honestly. GET/HEAD body rejection, redirected method/header changes, response/body lifecycle and cancellation are part of the adapter proof, not reasons to restrict source configuration preemptively.

Ring HMAC probe demonstrates the approved shared primitive for Slice 9 and direct supporting use; the API source does not sign webhooks. No signing default or signing config is introduced merely because ring is available.
