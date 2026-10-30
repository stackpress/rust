# Preserved source behavior and open questions

G43 treats the original Stackpress logic as intentional. Plugins secure the app only as configured; route/event processes choose their own checks. This page records current behavior and Rust mechanics boundaries, without selecting a policy change.

## Source provenance

The read-only upstream package is `packages/stackpress-session` in the original Stackpress checkout. Its `src/session/Session.ts`, `src/session/plugin.ts`, `src/auth/{plugin.ts,events/,pages/,helpers.ts}`, `src/session/pages/` and `schema.idea` supply the identity, auth, account and data contracts. Source aggregate order comes from its `src/plugin.ts` and `packages/stackpress/src/plugin.ts`.

[JWT vectors](../tests/jwt-vectors.json), [column/record vectors](../tests/records.json), [auth traces](../tests/auth-flows.json), [account traces](../tests/account-flows.json) and the [joined removal oracle](../tests/account-removal-join.json) retain source-derived evidence. Their reproducible loaders live in [proofs](../proofs/). Email's [source-page oracle](../../framework-email/tests/fixtures/source-page-boundaries.json) records the shared-response integration. Source implementation remains reference material; real Rust SQLite/browser tests supply separate execution evidence.

## Identity and access

The source Session is a JWT from session.key/seed, not a signed revocable database reference. It accepts the short source default seed, optional expiry and null-on-invalid verification, caches once per Session instance and exposes raw token/permits through authorization. `can([])` is true; roles and regex matching preserve source truthiness/coercion/order. [Native tests](../tests/main/native.rs) and [identity source](../src/session.rs) pin these rules.

No session config means no identity. No session.access means no global request guard; an empty map installs the source permissive guard, including its zero-status error offer. Auth events remain independently reusable; no auth config means no auth process routes. [Host tests](../tests/main/host.rs), [auth registration traces](../tests/main/auth_flows/registration.rs) and [composite tests](../tests/main/composite.rs) cover these modes. Several handlers may serve one route/event by priority then registration order; any handler can abort. No route ownership arbitration is introduced.

## Flags, errors and effects

[Detailed auth behavior](auth.md#config-flags-errors-and-redirects) retains truthy request password, strict 2fa=false, source error fields/statuses, unchecked GET branches, challenge consumed rotation, literal session signout cookie and redirect precedence. No challenge TTL, account-active, CSRF, enumeration or revocation check is inserted beyond the source process. [Source auth trace test](../tests/main/auth_flows/oracle.rs) and [real events](../tests/main/auth_flows/events.rs) pin these branches. G44 explicitly permits missing phone on signup only.

Signup and account mutations keep source sequential partial writes; batch separately retains caught item statuses even after rollback/commit failure. Existing identifier updates do not demand a current password, while adding a missing method does. Password change/enrollment preserve post-write source flash/redirect decisions. Confirmed 2FA removal can run on GET without csrf.valid. [Account traces](../tests/main/account_flows.rs) and [SQLite account tests](../tests/main/account_flows/sql.rs) cover them; generic SQL event adapters add no authorization.

The deterministic storage codec is original AES-256-CBC and SHAKE256; no integrity envelope or password-format migration is added. Token uniqueness spans all auth types and inactive records. Native services/seeds stay outside automatic public projection, while source enrollment secrets/CSRF inputs remain deliberate page data. See [storage](storage.md), [account](account.md) and [projection tests](../tests/main/account_ui.rs).

## Open owner questions

Every item below is **open question — current behaviour follows upstream**. None is resolved by documentation or by substituting different defaults.

| Question | Current behavior and source anchor | Rust evidence |
| --- | --- | --- |
| Account delete keeps profile | `session/pages/remove.ts:28,69` shares the loaded parent body; `stackpress-sql/src/transform/events/remove.ts:79–81` skips on truthy body. Auth rows removed, profile active, success flash and signout redirect. | [Joined oracle](../tests/account-removal-join.json), [SQL removal test](../tests/main/account_flows/sql.rs) |
| SQLite ILIKE | Generated `ProfileStore.ts:491,561` and `AuthStore.ts:659` emit literal ILIKE; SQLite q/like execution fails. | [Query compiler](../src/storage/query.rs), [predicates test](../tests/main/actions.rs) |
| Auth create-return | `auth/AuthActions.ts:248` calls store.unserialize(raw) (schema decoding, not SQL unscalarize): ciphertext token remains, raw profile_id omitted. Ordinary find decodes normally. | [Create branch](../src/storage/actions.rs), [source scalar goldens](../tests/main/actions.rs) |
| Positive hasnt | `profile/ProfileStore.ts:519–522` and `auth/AuthStore.ts:617–620` compile hasnt through the same positive membership as has; no negation added. | [Query compiler](../src/storage/query.rs), [predicate execution test](../tests/main/actions.rs) |
| Optional phone on update | `session/pages/update.ts:161` passes absent phone to normalizePhone; `auth/helpers.ts:6` calls trim. Update keeps the error; signup alone accepts missing phone under G44. | [Update process](../src/account/profile.rs), [account oracle](../tests/account-flows.json) |
| Email sign-in skips send | `auth/pages/signin/email.ts:158,223,284,343` shares the auth-update response with email-send; `stackpress-email/src/events.ts:10–19` skips its truthy body before constructing a transport. | [Real email integration](../../framework-email/tests/cases/integration/mod.rs), [source boundaries](../../framework-email/tests/fixtures/source-page-boundaries.json) |
| Moment admin dates | Session `schema.idea:85–95,183–208` uses `m d, Y h:iA`. Moment interprets minute/weekday/year/12-hour/literal i; the API shared adapter retains that interpretation, including UTC `0 5, 2100 12:iAM`. This is admin schema metadata, separate from account ISO/Date.toString helpers. | [API date adapter](../../framework-api/src/views/roles.rs), [accepted API date evidence](../../../../.agents/specs/00001-stackpress-rust-foundation/tasks/00021-slice-10-m5.md) |

## Rust mechanics and evidence limits

Typed configuration, owned Send futures, explicit services and Leptos replace dynamic JavaScript/import/React mechanics. Rust regex cannot represent every ECMAScript lookaround/backreference; unsupported syntax returns an explicit error. Native Date ranges/accepted parser forms are finite and local offset/zone are supplied explicitly. Source error fields/causes use the foundation's typed response/diagnostic shape rather than fabricated V8 object identity.

QR URI, correction level, margin, scale and displayed payload follow source; approved Rust QR/PNG compression internals are not claimed byte-identical to Node. Ring supplies native entropy in place of engine Math.random, with source fraction/format retained through an injectable seam. These are mechanics boundaries, not new application security policies. [Auth source traces](../tests/main/auth_flows/oracle.rs), [native tests](../tests/main/native.rs), [account tests](../tests/main/account_flows/sql.rs) and [browser proof](../proofs/browser.cjs) are the concrete anchors.

SQLite runs against real files. PostgreSQL/MySQL have DDL/insert rendering and scripted branch evidence only; live backend/JSON semantics are unverified. Email transport capabilities and provider-or-named-error limits belong to the [email reference](../../framework-email/docs/source-behavior.md). Headless Chromium hydrate/CSR proofs fulfill requests in memory and do not establish installed Tauri desktop/mobile behavior, TCP/TLS flush or remote delivery acceptance.
