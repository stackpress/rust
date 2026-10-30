# Slice 9 design checkpoint: G43 source parity

Status: **Revised design only; not released for M0**, 2026-10-30. Entry for [task 00020](tasks/00020-slice-9-session-csrf-email.md). Binding authority is G43 read directly from `/Users/cblanquera/server/projects/stackpress/rust/.agents/specs/00001-stackpress-rust-foundation/decisions.md:82` on trial/3 at `3cf407f`. That explicit owner direction supersedes this checkpoint's former recommendations; it does not authorize production code or edits to the historical ledger.

## Reading map

| Need | Read |
| --- | --- |
| Exact source evidence, hashes and scope | [Provenance](slice-9-design-sources.md): current source inventory, G43 authority and revision coverage. |
| Session exports, JWT, role checks and lifecycle | [Session inventory](slice-9-design-inventory-session.md): source intent and Rust mapping for every relevant item. |
| Auth/profile types, operations and SQL storage | [Auth inventory](slice-9-design-inventory-auth.md): full current scope, codecs, uniqueness and sequential operations. |
| Every original route, view and helper | [Pages inventory](slice-9-design-inventory-pages.md): exact paths/methods and Leptos mapping. |
| Every config key, CSRF and email mechanism | [Config inventory](slice-9-design-inventory-config.md): activation, defaults and transport contracts. |
| Existing Rust overlap and proposed shared change | [Overlap map](slice-9-design-overlap.md): keep/reuse decisions and coordinator-owned request-local carrier. |
| Caller/permission contract shared with API | [API sketch](slice-9-design-api.md): the single API-facing section, direct calls and native slot ownership. |
| Runtime plugin usage | [Registration DX](slice-9-design-registration.md): ordinary functions, composite order and optional config. |
| Flow ordering, lifetimes, storage and public data | [Execution contract](slice-9-design-execution.md): source behaviour with reasons and mechanical adaptations. |
| Approved dependencies, JWT recommendation and probes | [Assessment](slice-9-design-assessment.md): pinned features, retained historical evidence and G43 probe. |
| Accepted behaviour and implementation risks | [Verification](slice-9-design-verification.md): parity tests versus Rust correctness prevention, supervision and coverage. |
| Full-scope implementation order | [Milestones](slice-9-design-milestones.md): two lanes, exclusive ownership, release gates and complete account/page/SQL scope. |

## Decided under G43

Assume each original line is intentional. Plugins provide only their configured mechanisms; routes/events secure their own processes. The reason for a behaviour below is either explicit in comments/call flow or a labelled interpretation of that call flow, not a claim about undocumented owner intent.

| Former item | Binding decision / revised mapping |
| --- | --- |
| D01 | JWT identity from original Session.ts and session.key/seed; **no opaque reference, revocation repository, security generation or live-role lookup**. Auth/Profile SQL storage is required and is separate from the JWT. |
| D02 | No session config disables session; missing session.access installs no global request guard. Present empty access still permits everything through Session.authorize. No default-deny/public-mode switch added. |
| D03 | Original lifetime semantics: JWT expiry defaults to zero/unset; nonzero value is absolute NumericDate. No new TTL, challenge expiry, attempt budget or sliding refresh. |
| D04 | **Everything now:** all original account/auth flows, every page/view/helper and SQL adapter. Source extension hooks/placeholders remain hooks/placeholders, not reasons to defer existing code. |
| D05 | Preserve request-data password/2fa flags, JS truthiness versus strict false distinctions, request-over-config signup roles and each page's own overrides. |
| D06 | Original random CSRF cookie/session/form comparison, 419 failure and explicit call sites; original GET signout and page-specific method checks. No global Origin/signed-token/POST-only policy added. |
| D07 | Original status/message/field errors; no uniform enumeration response or dummy hashing added. |
| D08 | Approved: argon2, lettre, totp-rs, qrcode; reuse ring, subtle, zeroize, base64, url, reqwest, with pinned minimal features. JWT crate is the single remaining dependency question below. Approval of a library is not permission to change source logic/codecs. |
| D09 | Include the SQL storage adapter and original install/upgrade/purge/uninstall APIs now. Registration does not open a listener; retain the source separation between plugin wiring and explicit storage operations, not a new revocation schema. |
| D10 | Original TOTP enrollment displays the selected/generated/stored secret and QR in page results; no separate one-time provisioning protocol replaces it. |
| D11 | Full email package behaviour/config transport union now, using compiled native adapters. Do not add recipient/egress/header-policy caps or forbid a source transport as a new framework policy. |
| D12 | Original password rules and JS length/coercion, phone normalization, identifier matching and codec semantics. No minimum-12/E.164/new case canonicalization or automatic Argon2 format migration. |
| D13 | Original globally unique Auth.token, not uniqueness on method+identifier. |
| D14 | Original link and 2FA URL parameters, redirects, timestamp challenges and GET/POST behaviour; no fragment/same-browser/rebind/POST-confirmation replacement. |

G39–G41 continue to govern existing theme/preferences/Leptos ownership. G42 is unchanged: several handlers per route/event, priority then registration order, any handler can abort; no route ownership or conflict arbitration. Config absence deactivates the source capability at its actual gate. In particular, the CSRF package itself registers its default mechanism without a csrf section, and auth's Listen events are registered before its auth-config Route gate; those concrete source distinctions are preserved.

## Short source-parity table

Paths below are under upstream `packages/stackpress-session/src/` unless prefixed with another package. Full inventories include the remaining methods/exports and rationale.

| Behaviour / why it exists | Exact upstream line | Rust mapping / mechanics deviation |
| --- | --- | --- |
| Optional session integration permits hosts without identity | `session/plugin.ts:12` | Optional typed section; missing section contributes no session service/events/guard/account routes. Typed config is a Rust mechanics adaptation. |
| Optional route-wide guard, separate from individual route processes | `session/plugin.ts:26–31`; `session/Session.ts:66–68` | Preserve missing-vs-empty access and allow-all empty authorize branch; no new default policy. |
| JWT stores account claims without a session-table lookup | `session/Session.ts:109–117,123–137` | HS256 signing, issued-at, configured cookie input, optional absolute exp; synchronous Rust JWT implementation may sit behind async source-shaped methods. |
| Invalid token acts as Guest; token data can still be returned | `session/Session.ts:152–179` | Cache null on verification error; authorization merges id=0/GUEST, claims, token and permits in that order. |
| Caller asks for every listed permission; authorize adds current route | `session/Session.ts:72–75,193–228` | Preserve all-of, empty true, role grants and source dedup semantics; Rust Vec mutation replaces JS unshift. |
| Passwordless stages reuse normal signin and skip repeating 2FA | `auth/events/signin.ts:17–34`; `auth/pages/signin/2fa.ts:103–115,145–153` | Preserve public data flags and page overrides, not a sealed-proof replacement. |
| Persisted consumed time links email/2FA steps and supersedes old issuance | `auth/AuthActions.ts:415–425`; `auth/pages/signin/email.ts:155–192,281–314` | Same updates/reload/hash inputs/sequence; explicit Date/string adapter is a mechanics requirement. |
| Email/password identity failures provide field-specific feedback | `auth/AuthActions.ts:395–412` | 404 User Not Found versus 401 Invalid Password, original error envelopes. |
| Signup allows caller data to override configured defaults | `auth/events/signup.ts:17–22`; `auth/AuthActions.ts:438–452` | Preserve object spread precedence and profile→email→phone→username sequence. |
| CSRF generates form data then compares the incoming submitted value | `stackpress-csrf/src/plugin.ts:20–65` | 32 random bytes→hex; incoming req.session and req.data scalar/equal-length/timing-safe comparison; no new protocol. |
| Enrollment needs QR/secret to configure an authenticator | `session/pages/2fa/detail.ts:60–74,132–140` | Same result fields in the Leptos page snapshot; React→Leptos rendering is mechanical. |
| Email event creates a configured transport and preserves earlier body/error | `stackpress-email/src/events.ts:10–29` | Same invocation skip/send/result/error sequence; compiled lettre/native adapters replace NodeMailer objects. |
| API consent uses cookie identity, API endpoints use API SQL credentials | `stackpress-api/src/pages/oauth.ts:48–58`; `stackpress-api/src/plugin.ts:106–134` | Preserve `me` result separately from session-detail/scopes/profileId; [one interface section](slice-9-design-api.md) specifies native carrier ownership. |

## Questions for the owner

Only the JWT dependency and a concrete source-contract inconsistency remain questions. Accepted developer responsibilities are not reopened as defect questions. Defaults stay source-parity while questions are reviewed.

| ID | Question / evidence | Recommendation without changing the default |
| --- | --- | --- |
| Q-JWT | Approve **jsonwebtoken 11.1.0**, default-features=false, rust_crypto, from the probes? | Recommend it over the older probed 9.3.1. Keep the source claim/expiry adapter explicit; library defaults do not match jose. Probe and provider-feature costs are in the assessment. |
| Q-DEF1 | Must direct auth-signup supply an empty phone string, or should an omitted optional phone be accepted? `auth/events/signup.ts:22` always calls normalizePhone(input.phone); `auth/helpers.ts:6` calls phone.trim(), while `auth/types.ts:173` declares phone optional. | Ask whether the apparent optional-input mismatch is intentional. Keep the source missing-input failure until answered; do not insert a default/guard or label it a security fix. |

## Mechanics gates for coordinator release

The source SHAKE256/CBC codecs, JS regex/date/coercion behaviour and NodeMailer adapters must be proved against source fixtures before implementation claims parity. Approved Argon2 cannot silently replace deterministic source hashes. These are implementation-fit gates, not extra owner product decisions or permission to write custom cryptographic primitives. Any necessary shared request carrier belongs to stackpress-lib and needs a narrow coordinator grant; this design authorizes no shared-code edit.

Stop here for coordinator review. No production code, repository Cargo build, lockfile change, historical-record edit, commit/push or listener is released by this revision.
