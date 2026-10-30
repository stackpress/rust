# Slice 9 source-parity execution contract

Status: Revised design only under G43. All original auth/account pages, flows and SQL adapter are now in scope. The framework secures processes only where configured/source code does; it adds no protection beyond that logic. Reasons below follow comments/call flow, with mechanical gaps stated explicitly.

## Session and lifecycle

A configured identity Service models the original Session class: key defaults session, seed abc123, access {}, expires=0. Rust replaces a JS class/global with a per-host shared runtime and synchronized per-instance cache. This is a mechanics adaptation for ownership/Send/concurrent hosts, not a changed policy. Native configure/getters and expires setter remain available; arbitrary source claims are retained.

Create signs an object using HS256, protected header alg only, issued-at set to current seconds. With expires=0, do not add exp (retain an input claim if provided); with nonzero expires, set that absolute NumericDate. Do not infer a duration, require exp/iss/aud/sub or accept a JWT library's different default clock tolerance. Verify as original jose jwtVerify with the configured UTF-8 seed; errors become cached null. Source temporal claim/type/header/algorithm semantics are a library-adapter proof obligation, not a new hardening policy.

Token extraction uses only normalized req.session[key], despite the header wording in its comment. Load explicit token or request→new Session. Missing token becomes ''. data caches verification once; guest is data===null. authorization merges {id:0,roles:[GUEST]}, data, token and permits in that order. Browser/native consumers can deliberately receive the raw token as source specifies. No live account lookup, revocation/epoch check, key rotation policy, cookie refresh or automatic signin fixation defence is added.

Role permits come from payload roles or GUEST on the original truthy fallback. Empty [] is truthy and yields no roles/grants. Dedup strings by value and object grants by identity. can([])=true; otherwise every requested event/route must match at least one grant. authorize returns true immediately for an empty access map; otherwise it prepends the current route to the supplied mutable permits, produces original 401 Unauthorized on failure or original authorization results on success. No new 403 split or empty-protected-request rejection.

Registration mirrors exact gates: root session composite auth→email→session; no session section deactivates the session subplugin; absent session.access means no global request listener; present empty map installs a permissive authorize guard. No auth config means no auth routes/views, but auth's Listen events remain. CSRF is an explicitly selected default mechanism without its own missing-config guard. Email config absence/falsiness disables its integration. Each normal return remains Continue; a source failure/explicit stop is mapped through existing lifecycle/error mechanics, not an added framework Abort.

## Auth/Profile storage and source codecs

Implement the original Profile/Auth schemas, columns, stores/actions and SQL adapter. These are account/credential records, **not** persisted JWT sessions. API Application/API Session SQL records belong to task 00021 and are separate. Reuse existing SQL transaction affinity/dialects/query/serialization, while retaining original operation boundaries.

| Contract | Exact source parity / why |
| --- | --- |
| Profiles | Generic person/company/etc.; id,image,name,type,roles,tags,references,active,created,updated, auth relation. Do not narrow generic action inputs to a new user DTO. |
| Credentials | id,profileId,type,token,secret,verified,consumed,active,created,updated. Global token uniqueness and deterministic encrypted lookup, not method+identifier uniqueness or new blind index. |
| Active filter | Store.where defaults active=true unless explicitly supplied; active=-1 removes that filter. Preserve actual joined-query behaviour rather than add a new account-active gate on JWT/can. |
| Required/default/coercion rules | Preserve source column default/filter/populate/serialize/unserialize, unset/null/empty distinctions and String/Date/list conversions. Core scalar rules are reused only when equivalent. |
| hash | Source framework-schema/helpers.ts:104–108: SHAKE256, 32 output bytes, lowercase hex. Challenge and password comparisons depend on deterministic output. No silent salted Argon2 substitution. |
| encryption | helpers.ts:64–98: SHA256(seed)→base64 text first32 characters as AES-256-CBC key bytes; first16 key characters as IV, source padding and hex encoding. Same read/write/equality contract, not AEAD replacement. |
| Date/hash input | consumed is updated then reloaded before constructing challenges. Match source Date.toString() and store Date conversion/timezone, not a convenient ISO string. A Clock/date compatibility adapter and exact vectors are required. |
| install/upgrade/purge/uninstall | Full direct APIs now; application supplies Engine/driver and calls them explicitly. Source plugin registration does not run these operations automatically. |
| Generic auth/profile events | Preserve generated action events and nested resolve/emit; no private-only replacement to enforce new policy. Each event/plugin decides its process. |

Rust library fit for SHAKE256/CBC/Date/JS-regex is an M0 technical gate. The approved set does not automatically supply every source primitive (ring has no SHAKE256/AES-CBC). Reuse a verified provider through the existing codec contract and coordinator dependency reconciliation; do not claim Argon2 changes are forced by Rust, write a home-grown cryptographic primitive, or mark full storage parity proved before that fit is resolved. No additional library choice is presented as a decided default here.

## Auth process transitions and rationale

| Trigger | Original sequence / result to retain |
| --- | --- |
| Signup | Merge auth.roles first then request data; normalize phone; assert rules; create profile→email→phone→username without new outer transaction; field-specific uniqueness errors; fire email-email-send/auth-phone-verify hooks without awaiting; strip known secret fields then return results. This reuses generic stores and extension events. |
| Password signin | Default type=username; Boolean(password) controls comparison; lookup type/token via String(input[type]); missing→404 User Not Found, wrong secret→401 Invalid Password; update consumed and reread. No uniform failure/dummy hash/attempt budget. |
| Signin event 2FA branch | Unless req.data['2fa'] is exactly false, find profile/auth/consumed, search 2fa row, redirect with hash(consumed.toString()) before issuing JWT when present. `password` falsey bypasses password; `'2fa':0` does not equal strict false. Preserve page overrides separately. |
| Signin completion | Delete results.secret, set results, issue cookie with configured session key and selected profile id/name/image/roles (nonempty array else GUEST). Do not issue an opaque record or rotate/revoke another JWT. |
| Username/phone/email password pages | Page sets password=true and type explicitly, after original CSRF sequence. Generic event still accepts its flags because later passwordless stages intentionally reuse it. |
| OTP issuance | Trim email only here; lookup and original errors; update consumed→reload; floor(Math.random()*1e6)→pad six; hash(consumed.toString()+code); source host/base/link construction; missing sender/email config no-op; send then clear/redirect. RNG host replacement is mechanical; do not claim a new CSPRNG/TTL guarantee. |
| Magic issuance | Same consumed rotation/reload; hash timestamp only; send template; preserve success ending without OTP's clear/redirect. No new same-browser/fragment policy. |
| OTP answer | Guest flow, auth/challenge params; GET returns secret-stripped row; POST csrf.valid; trim/pad, require padded.length===6, compare hash; redirect to shared 2FA. No atomic one-use/attempt table. |
| Magic link | Guest flow checks timestamp hash and redirects to 2FA, including GET. Existing signed-in identity redirects earlier. No new POST confirmation or exp clock. |
| 2FA | Check profile/auth/challenge; locate 2fa record. If none, set password=false/type/token/2fa=false and emit signin immediately (even GET). If present, POST CSRF+TOTP then same mutation/emission. Signin's consumed update invalidates the earlier challenge; no added matched-step replay state. |
| Signout | GET page resolves auth-signout, which deletes literal session and sets 200; preserve original redirect precedence. Custom key behaviour is not silently fixed or replaced by DB revocation. |
| Account update | Load JWT/profile; GET projects; POST requires current password only when adding a missing method; profile write then sequential auth-field writes, skip empty identifier updates, stop on first returned error. No new universal reauth/CSRF/transaction. |
| Password change | Original rules plus current required/different; search auth rows, hash/compare current, sequentially update every auth row; same flash/redirect and result handling. No JWT invalidation added. |
| Account removal | JWT/profile, POST+confirmed, current password; sequential auth-remove then profile-remove; source status/error stops, flash then signout redirect. No new transaction/CSRF/generation transition. |
| Account export | JWT/profile and download flag; source profile CSV formatting and = + - @ prefix, source attachment name. Do not add credentials, extra admin CSV rules, reauth or new export caps. |
| TOTP enrollment | Secret from request or stored row unless regenerate or generated; issuer/label/otpauth URI and QR; original POST CSRF/code validation, update/create type 2fa, source result/flash/redirect. Existing/generated/request secret is deliberately displayed with QR; no private provisioning replacement. |
| TOTP removal | JWT/profile; truthy confirmed branch with scoped auth id/profileId/type and source removes/flash/redirect; no added method/CSRF/current-password gate. |

Preserve exact error calls, code omissions, field errors, casing, previous-body/non-200 skips and nested response propagation. Rust Result replaces throws/callback rejection mechanically; it must not invent a status when the source relies on Response.setError defaults. Partial completed writes remain observable unless the original explicit batch transaction rolls back.

## CSRF and email

CSRF generate uses 32 random bytes→hex and stages both response session and {name,token} data. Page generate-before-valid compares incoming req.session with req.data, not the newly generated response state. valid accepts any equal nonempty strings, checks UTF-8 byte lengths, then timing-safe equality; it performs no signature/Origin/method/expiry/session-binding check. Failure sets configured message and 419 Page Expired. clear removes req.data, response session and response data only. Protection exists only at the original explicit call sites or app-added plugins.

Email-send preserves source body/non-200 skip; absent config returns. It creates a transport from config per invocation, passes the original options, awaits callback success/error, sets provider info as results or propagates error. Rust factory/Send futures/native adapters replace Node objects; preserve SMTP/pool/JSON/stream/sendmail/SES/string and file/URL/content/header extension capabilities. Do not impose authorization/egress/recipient/header/timeout/retry policy the source lacks. Library framing/encoding and memory-safe IO are mechanics obligations, not permission to silently drop options.

Fire-and-forget signup hook intent needs a supervised Rust task/spawn boundary because dropping an unpolled future does not execute it. Preserve parent non-awaiting/result behaviour, independent request/response payload and registration order; do not borrow mutable parent state into a 'static task or change the hook name. Cancelling one root must not leak locks/resources. External sends and sequential SQL operations have the source partial-effect semantics; do not claim exactly-once delivery, automatic rollback or remote acceptance from local task completion.

## Browser data and visual acceptance

Leptos replaces React/Frui; current Rust theme/locale/ownership remains G39–G41. Source-selected response data and account fields remain, including SessionTokenData.token, CSRF, enrollment secret/QR and source form input binding. Do not apply the rejected blanket rule that removes all such values from SSR. Actual signing/database/SMTP configuration and native handles are not automatically serialized; test against unintended resource/secret leakage separately from intentional page data.

| Control | Source-parity UI verification now |
| --- | --- |
| S9-UI01 | Signup/method chooser/username/email/phone/OTP/link/2FA display source fields, choices, error messages, input binding and redirect branches; no new confirmation flow. |
| S9-UI02 | Account/read/update/security/password/export/removal/2FA detail/removal all work with SQL-backed source actions and exact conditional checks/partial effects. |
| S9-UI03 | Leptos keyboard/labels/focus, 375px/1280px, 200% zoom and long content preserve usability; project Lucide family replaces source icon mechanics without changing process policy. |
| S9-UI04 | SSR→hydrate and separate CSR use source-shaped data; G39 Light/Dark/unset/no-JS Light behaviour and existing escaping/CSP mechanics are preserved. |
| S9-UI05 | Enrollment renders source url/secret/profile/authId and regenerate/request-secret branches; tests distinguish these intended values from accidental native config/handle exposure. |
| S9-UI06 | Shared-route priority/FIFO, plugin augmentation/abort, source missing-config modes and direct events behave identically; no framework default-deny is asserted by visual tests. |

Code verification is not owner visual/provider/installed-platform acceptance. No listener, repository build or browser run is performed during this design revision.
