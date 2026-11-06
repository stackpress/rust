# Preserved source behaviour and G46 decisions

G43 treats the original Stackpress logic as intentional. Plugins secure the app only as configured; route/event processes choose their own checks. G46 resolves the ten source-behaviour questions, with one rendering change: PostgreSQL retains ILIKE; SQLite and MySQL use LIKE.

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

## Owner decisions G46

All ten items are **resolved under owner decision G46**. The original G46 resolutions remain recorded here; G54 supersedes positive hasnt with exclusion. Search uses the dialect's supported operator. These decisions add no global guard or new account policy.

| Resolved item | Current behaviour and source anchor | Rust evidence |
| --- | --- | --- |
| Account delete keeps profile | Intended soft delete via @active: removed records are deactivated, queries filter active rows by default, and active=-1 is the escape. The account process retains Profile: `session/pages/remove.ts:28,69` shares the parent body; `stackpress-sql/src/transform/events/remove.ts:79–81` skips profile removal on truthy body. No code change. | [Joined oracle](../tests/account-removal-join.json), [SQL removal test](../tests/main/account_flows/sql.rs), [active-filter tests](../tests/main/actions.rs) |
| SQLite ILIKE | Owner: use LIKE. Generated source `ProfileStore.ts:491,561` and `AuthStore.ts:659` use ILIKE; G46 keeps that spelling only on PostgreSQL. SQLite/MySQL render LIKE; SQLite matches ASCII case-insensitively by default, MySQL relies on default _ci collations. | [Dialect adapter](../../framework-sql/src/source/statement.rs), [rendering and real SQLite tests](../tests/main/actions.rs) |
| Auth create-return | Follow original source, unchanged: `auth/AuthActions.ts:248` calls store.unserialize(raw), so ciphertext token remains and raw profile_id is omitted. Ordinary find decodes normally. | [Create branch](../../framework-sql/src/source/native_actions.rs), [source scalar goldens](../tests/main/actions.rs) |
| Positive hasnt | Superseded by G54: exclude every requested member; SQL NULL lists and missing JSON paths match. Original `stackpress-sql/src/transform/store/select.ts:680–682` calls whereJsonContains with whereJsonNotContains commented out. | [Shared compiler](../../framework-sql/src/source/membership.rs), [shared cases](../../framework-sql/tests/support/g54.rs), [predicate execution test](../tests/main/actions.rs) |
| Optional phone on update | Follow original source, unchanged: `session/pages/update.ts:161` passes absent phone to normalizePhone; `auth/helpers.ts:6` calls trim. Update keeps the error; G44 optional signup phone remains separate. | [Update process](../src/account/profile.rs), [account oracle](../tests/account-flows.json) |
| Email sign-in skips send | Follow original source, unchanged: `auth/pages/signin/email.ts:158,223,284,343` shares the auth-update response with email-send; `stackpress-email/src/events.ts:10–19` skips truthy body before constructing a transport. | [Real email integration](../../framework-email/tests/cases/integration/mod.rs), [source boundaries](../../framework-email/tests/fixtures/source-page-boundaries.json) |
| Moment admin dates | Follow original source, unchanged: session `schema.idea:85–95,183–208` uses `m d, Y h:iA`. Moment retains minute/weekday/year/12-hour/literal i interpretation, including UTC `0 5, 2100 12:iAM`. Separate from account ISO/Date.toString helpers. | [API date adapter](../../framework-api/src/views/roles.rs), [accepted API date evidence](../../../../.agents/specs/00001-stackpress-rust-foundation/tasks/00021-slice-10-m5.md) |
| PGlite | Not required. The three standard engines are MySQL, SQLite and PostgreSQL; no PGlite connector is planned. | [Resolved aggregate ledger](../../framework/proofs/coverage-map.json) |
| License | Owner-owned. | [Resolved aggregate ledger](../../framework/proofs/coverage-map.json) |
| SQLite purge | Keep DELETE FROM "table"; no TRUNCATE or added WHERE 1 is needed. | [SQL renderer](../../../sql/sql/src/table.rs), [SQLite DDL tests](../../../sql/sql-sqlite/src/tests/m2/ddl.rs) |

## Rust mechanics and evidence limits

Typed configuration, owned Send futures, explicit services and Leptos replace dynamic JavaScript/import/React mechanics. Rust regex cannot represent every ECMAScript lookaround/backreference; unsupported syntax returns an explicit error. Native Date ranges/accepted parser forms are finite and local offset/zone are supplied explicitly. Source error fields/causes use the foundation's typed response/diagnostic shape rather than fabricated V8 object identity.

QR URI, correction level, margin, scale and displayed payload follow source; approved Rust QR/PNG compression internals are not claimed byte-identical to Node. Ring supplies native entropy in place of engine Math.random, with source fraction/format retained through an injectable seam. These are mechanics boundaries, not new application security policies. [Auth source traces](../tests/main/auth_flows/oracle.rs), [native tests](../tests/main/native.rs), [account tests](../tests/main/account_flows/sql.rs) and [browser proof](../proofs/browser.cjs) are the concrete anchors.

SQLite runs against real files. The [opt-in live searches](live-search.md) execute Profile q/like and the Auth-to-Profile join on local PostgreSQL and MySQL, with real installation and source codecs. G48 selects explicit ordered non-atomic MySQL install/upgrade; PostgreSQL/SQLite retain atomic setup, and ordinary transaction contracts stay unchanged. This is bounded ASCII-search evidence, not arbitrary collation, JSON-query or aggregate-provider acceptance. Email transport capabilities and provider-or-named-error limits belong to the [email reference](../../framework-email/docs/source-behavior.md). Headless Chromium hydrate/CSR proofs fulfill requests in memory and do not establish installed Tauri desktop/mobile behavior, TCP/TLS flush or remote delivery acceptance.
