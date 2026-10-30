# Slice 9 full-scope milestones under G43

Status: Revised design only; stop after this revision. No production work is released. G43 settles source behaviour and full scope; [decisions](slice-9-design-decisions.md) retains only JWT selection and a concrete optional-phone question.

## Scope and ownership

All original Session/Auth/Profile APIs, generic SQL adapter/storage/maintenance surface, signup/signin/signout, username/email/phone password, email OTP/link, 2FA signin/detail/removal, account read/update/security/password/export/removal, every view/helper, CSRF and complete email package transport/config variants are Now. Existing extension-only hooks remain hooks, not deferred source code. API records/OAuth consent/SQL adapter are Now in parallel task 00021, not work removed from that slice.

| File set | Single proposed owner | Boundary |
| --- | --- | --- |
| crates/framework/framework-session/** | Lane A: session/auth/profile/storage/UI | All source-parity implementations, schema/Store/Actions, SQL adapter, optional host/public/Leptos modules and own tests/docs. No unrelated demo Profile or frozen metadata edits. |
| crates/framework/framework-csrf/** and framework-email/** | Lane B: CSRF/email | Original CSRF/default gates, all email options/transport factories/adapters, own tests/docs; no imported signed-CSRF/transport restriction design. |
| examples/session-security/** (proposed finite parity host/consumers) | Lane A integration, after contracts reviewed | Name describes test scope, not a new security-policy guarantee. Exercises all configured/missing-config/source unchecked paths with actual SQL actions and fake providers, no listener required. |
| Root Cargo.toml/Cargo.lock/src/lib.rs | One coordinator-assigned integration implementer, serial grant | Reconcile dependencies/features with API lane first. Neither lane writes shared files opportunistically. Defaults remain unchanged; no commits. |
| stackpress-lib Request native carrier files | stackpress-lib owner, coordinator-released implementer only | Proposed request.rs plus focused type-map module; no framework deps. Freeze fill/read/transfer contract with API and scope exact files before release. Slice lanes otherwise have no shared-code grant. |
| Existing view/lib/server/SQL/admin code, review-preview/ui-preferences | Keep unless coordinator grants a narrow mechanics change | No automatic default-policy migration or fixture replacement. Existing explicit application policies stay as they are. |
| Spec task status/verification/historical G ledger | Coordinator | Implementer revises these new design files/assigned implementation notes only. G43 is read from authorized main path; no historical record edited. |

Two implementation lanes are proposed, not spawned. API lane has separate ownership; caller/DTO/carrier/root-manifest contracts are frozen before concurrent writes. Coordinator reviews/verifies/commits; owner decides product/remaining dependency/defect questions.

## Gated plan

| Gate | Lane A | Lane B | Stop / required evidence |
| --- | --- | --- | --- |
| Revised design (now) | Full source intent/parity map, G43 defaults, all SQL/page scope, single API interface | Approved dependency/transport/CSRF fit integrated here | Stop for coordinator review. One JWT dependency question; Q-DEF1 does not change the default. No scaffold/build in repository. |
| M0 mechanics and riskiest parity proof | Real JWT Session types with optional-config/expiry/cache/me/can; SQL source-codec/date/regex/coercion fit; request carrier only under grant | Real random/variable-string CSRF valid and per-call Send transport factory; source option/default mapping, task supervision | Reconcile API identity paths first. Prove source defaults, short seed, null-on-invalid, missing versus empty access, consumed-hash vectors and Send ownership. Unresolved provider fit stops broad implementation, not a silent behaviour swap. Stop for independent rerun. |
| M1 direct primitives and storage | Complete Auth/Profile schema/columns/Store/Actions/SQL adapter, generic CRUD/batch/install/upgrade/purge/uninstall, global token uniqueness and source codecs | Full CSRF generate/clear/valid/419; email option/result types, transport variants and direct calls | Real SQLite tests and golden source values; explicit batch versus sequential partial effects; no revocation/pending-proof schema. Stop. |
| M2 reusable events and auth processes | me/authorize/request guard, auth signup/signin/signout, password flows, email OTP/link/2FA handoff and all source flags/errors/redirects | email-send exact skips/results/errors; nonawaited signup hook support with owned/supervised tasks, all per-call transports | Source-derived invocation traces, missing-config modes, same route/event ordering, direct event reuse and source absence-of-check cases. No new global gate. Stop. |
| M3 every account page and Leptos view | All account/security/update/password/export/removal/TOTP enrollment/removal, every auth form/head/helper and native loaders; full SQL-backed finite host | Cross-crate CSRF/email/render/error integration; SMTP/pool/JSON/stream/sendmail/SES/string option fixtures | Complete page/export/type inventory coverage; source-intended token/QR/secret data and branch behaviour; S9-UI01–06 checks. Stop before refactor rounds, per-crate coverage reported. |
| M4 integration/feature/consumer gate | Explicit facade/direct/plugin consumers, source aggregate/composite order and API/carrier reconciliation | All adapter feature graphs, direct/event/disabled consumers and docs observations | Full compatible workspace/target/browser checks, >=70% per changed crate; provider/backend/installed limits explicit. No implementation omitted as a later hardening/provider slice. Stop. |
| Rounds 1–4 | Complexity, responsibility, consistency/Rustdoc/comments and Rust safety/soundness | Same distinct test-verified rounds | Meaningful fixes or justified no-edit review; source parity remains contract. Coordinator re-verifies. |
| Docs follow-up | Session/Auth/Profile direct/plugin/getting-started/concepts/guides/API/source behaviour | CSRF/email equivalents and all transport/config examples | chrisai-docing, external consumer compile checks, claim/link checks; no hidden source changes during docs. Owner UI acceptance remains separate. |

The previous signed-reference/revocation/default-deny milestones are superseded. Scope is not split into a narrow safe core and later source flows. Source-specific extension hooks or a source placeholder lacking a provider do not justify implementing a new provider mandate; all existing package functionality is still covered now.

## Verification execution after release

Use one warm effort target per lane, separate native/browser and disposable consumer directories, sccache and bounded jobs. At fresh M0 effort, record required cargo clean and llvm-cov cleanup once; no repository cleanup/build is run at this design gate. Cap effort targets at 8 GiB and record observed peak; preserve shared benchmark store.

Narrow edit-loop nextest/direct dependents, then full gate fmt/workspace tests/doctests/Clippy/Rustdoc/valid feature matrix/coverage. Every parity R01–R30 and implementation I01–I08 has evidence or an explicit nonverified target limitation. Deadlock-class tests run in externally supervised processes with absolute deadlines/kill/reap; no scheduling sleeps or unbounded waits. Full source UI and SQL integration are required, while a real provider or installed platform is not claimed from a fake/finite test.

No dev listener is needed for unit/in-process/SQLite/finite tests. Any later requested preview must use devmetrics with managed purpose/port/expiry; no bare server. Live SMTP/SES/sendmail and PostgreSQL/MySQL acceptance need explicit configuration/permission and receipts. No commits, pushes, installs or historical edits from either implementation lane without release.

Final receipt includes inventory/parity/mechanics/UI mapping, tests/features/platform coverage, per-crate line coverage, exact benchmark records/attributable count delta, command time separately from Cargo-stage subtotal, resource cleanup and remaining owner questions. Design/probes/verification do not themselves release M0 or accept the UI.
