# Slice 9 M1 Lane A implementation receipt

Status: M1 complete; stopped for coordinator verification. Base `ffdebc4`, confirmed by `git log -1`; worktree was clean. G42–G45 remain binding. This receipt does not release M2 or accept later page/provider/platform work.

## Stable Profile contract for Slice 10

`stackpress_framework_session::profile::ProfileSchema` (`native`) is the public schema path. `ProfileSchema::new(seed)` dereferences to `columns::RecordSchema`; `model()` returns an owned `stackpress_framework_schema::Model` with Profile's ten stored fields and Auth/Application/Session relation metadata. The Rust member for source `type` is `type_`. The native generated ProfileStore has no reverse eager relations, exactly like source.

`profile::{Profile, ProfileInput, ProfileExtended}` are native source record aliases; records map names to `columns::Value`. `Value::Date` is a distinct variant: arbitrary JSON objects have no reserved marker keys, and Serde writes Date as the original UTC ISO JSON string. `columns::{date, date_with_display, as_date, to_json}` are available for direct callers. Schema `columns`/`column` expose defaults, assertions and codecs. Source coercion/UTF-16 assertions belong to the native schema; shared descriptors describe shape and relations, with presentation metadata following M3.

With `sql`, `profile::{ProfileStore, ProfileActions}` and `auth::{AuthStore, AuthActions}` expose direct primitives. The same types are re-exported under `storage`. Direct Actions use the app's `Arc<Engine>`; `_in` variants exclusively borrow `DataSession`. API code can use the Profile descriptor without server integration. No stackpress-lib carrier, root facade, API, CSRF, email or shared schema provider file changed.

## Implemented source parity

| Surface | Evidence and behavior |
| --- | --- |
| Complete Auth/Profile columns/schema | Ten columns each, source defaults and fresh CUID/clock values; assert/filter/populate/serialize/unserialize; missing/null/empty retained. Source Date versus number, list-element coercion, Zod first messages, references and Boolean rules have original-source golden coverage. |
| Store/SQL adapter | Source VARCHAR(255)/JSON/Boolean/Datetime definitions, indexes, Auth global token UNIQUE, Profile FK CASCADE/RESTRICT, selectors/wildcards/JSON paths, inner Profile join, filters/OR groups, sort/paging, scalarize/unscalarize, checked detached builders and alterations. SQLite omits unsupported DDL comments mechanically. |
| Generic CRUD | count/find/find_all/create/update/upsert/delete/remove/restore, direct and explicit-executor paths. active=true default; eq.active=-1 removes it. Pre-read merged update return uses sanitized data, while original input is written and updated is mutated. No requery/hidden transaction or JWT revocation. |
| Source native insertion branches | Returning rows go through schema.unserialize, preserving encrypted Auth token and omission of raw profile_id. No-row branch retains insert-ID lookup/input fallback. MySQL ignores RETURNING through insert_for; no invented authoritative-row guarantee. |
| Source codecs/uniqueness | Reuse approved shared CUID/AES-CBC/SHAKE helpers. Token encryption/query encoding occurs at SQL boundary; secrets hash there. Uniqueness is global across auth types, with active-only preflight plus authoritative SQL constraint. Inactive duplicate reaches native error, preserving source distinction. |
| Auth actions | Ordered password checks with UTF-16 lengths and source special-character whitespace; equivalent finite scanner for unsupported JS email lookaround. signin retains 404/401 errors, optional comparison, consumed update/reload, no extra Profile-active guard. signup creates Profile then email/phone/username sequentially; missing phone succeeds G44; later failure retains earlier records. make applies source seed fallback after host resolves engine/config; direct options keep empty seed. |
| Batch/maintenance | Exact borrowed transaction for all items; caught item failure rolls back after continuing; attempt statuses remain. Missing update selector adds an error item without rollback. Explicit install/upgrade/purge/uninstall use existing SQL plans; PostgreSQL purge cascade retained, SQLite/MySQL ignore the flag like source. |
| G42/configured policy | No route ownership, new handlers/global guards or default-deny policy introduced. Existing identity registration unchanged. Individual process policy and original unchecked paths remain their source responsibility. |

Real file SQLite tests prove CRUD, persisted versus returned values, query/project/join behavior, encryption/hash bytes, cross-type and inactive uniqueness, FK cascade, batch rollback and sequential partial effects, source signin errors/consumed reload, optional phone, and maintenance. Golden generator imports the original read-only TS classes through its installed TypeScript compiler: 380 column cases, 17 email cases, scalar records, source query operators and returned-row branch. No dependency fetch/install occurred in M1.

## Verification and mechanics limits

| Component | Final result |
| --- | --- |
| Full workspace | fmt; Clippy -D warnings; **822/822 nextest tests**, 84 skipped; **951 doctests**; warning-clean workspace Rustdoc. Every workspace nextest command explicitly excludes `tcp_watchdog`, coordinator-owned. |
| Final owned suite | **27/27 tests**, one deliberately ignored child executed by the external watchdog; both session doctests, Clippy and Rustdoc pass. |
| Features/consumer | default/native/sql/server native checks; default and native,sql,server WASM checks and normal dependency graphs exclude native crypto/server/SQL. Independent consumer with an empty dedicated target and warm sccache compiles the public Profile/schema/Store/Actions/value APIs. |
| Coverage | cargo-llvm-cov nextest, aarch64-apple-darwin, session/server,sql: **2273/2544 = 89.35%** executable lines. Only tests/examples excluded, no production exclusions. Meets development floor; no publication claim. |
| Source/lock/scope | Original TS oracle reproduced unchanged; **667 existing external package blocks byte-identical, zero new/removed packages**. Source manifest SHA256 `0832445d092cadb84f0b9c21baada459b45f306eb4188fdbbfa81792ac586e33`. |
| KB | Existing baseline **146 errors / 27 warnings**; receipt validation adds zero new diagnostics. These inherited upstream layout/path diagnostics are not reported as clean code verification. |


One new compile-fail fixture rejects overlapping mutable executor borrows for the intended E0499 reason; four M0 compile-fail expectations remain unchanged and pass. Actual futures are checked Send across an exclusive transaction borrow. The external parent watchdog now also runs a real SQLite batch rollback: absolute 15s including startup, 2s entry ack, kill/reap on failure; its permanently blocked negative control is killed at 200ms. No scheduler sleeps order database operations.

Rust mechanics: explicit Arc/executor ownership, native Date variant and host offset/display replace JS runtime types/process timezone. Native time range/offset failures are fallible; supported textual date inputs include ISO/RFC3339, source local SQL text, date-only and V8 numeric month/year spellings. Implementation-specific legacy Date.parse strings require a native Date value. Existing checked/bound SQL identifier/path and execution budgets remain; unsupported SQL DDL operations fail explicitly. PostgreSQL/MySQL DDL/insert rendering is checked, but live backend/provider/platform and backend-specific JSON acceptance are not claimed. No browser/SSR/account-page implementation is claimed at M1.

Rounds: (1) logic review retained source ILIKE, list defaults, native insert branches and MySQL RETURNING adaptation; targeted tests passed. (2) responsibility/ownership separated schemas/columns, detached Store/query, CRUD, batch and account actions; transaction/Send/watchdog/compile-fail tests passed. (3) consistency/Rustdoc/comments/direct reference, fmt/Clippy/tests/doctests/docs passed. Follow-up ownership review replaced a colliding temporary Date tag with a native enum and corrected type_; all original goldens and the new arbitrary-object/Serde regression passed, followed by a fresh full gate. These are M1 reviews, not the later full-slice rounds.

## Source questions for owner

The defaults below remain source parity; no fix is selected implicitly.

- Literal ILIKE: ProfileStore.ts:491,561 and AuthStore.ts:659 emit it; original-source SQL fixture retains it, and real SQLite rejects q/like searches. Should this cross-dialect source failure be changed in a separately released correction? Recommendation: retain current parity until directed.
- AuthActions.ts:248 returns schema.unserialize(rows[0]); returning-row Auth token remains ciphertext and raw profile_id is omitted. Should the create return deliberately differ from ordinary find? Recommendation: retain this tested branch until the owner chooses otherwise.
- ProfileStore.ts:643 / AuthStore.ts:741 make hasnt positive contains (negative call commented immediately afterward). Is the positive alias intended for the Rust API too? Recommendation: keep this exact behavior unless explicitly changed.

## Benchmarks, resources and scope

Recorder store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Attributed M1 snapshot: **32 completed records (19 passed, 13 failed)**, not other agents' concurrent attempts. Recorded Cargo activity totals: {'compiling': 131, 'checking': 242, 'documenting': 54}. Command time **1010.219327s** and observed Cargo-stage subtotal **393.13s** overlap and are not additive or continuous end-to-end elapsed time. All Cargo/proof/clean commands use the recorder; outer scripts are wrapped once. Completed final receipt validation is included in this snapshot.

One retained warm target `target/slice9a`; edit loops unset incremental, gates set CARGO_INCREMENTAL=0; sccache enabled, jobs=2, debug=0. No repeat initial clean (M0 remains the same effort). At the original 8 GiB cap, recorded cleanup removed only debug/incremental (423,899,400 logical bytes). Coordinator then raised the cap to **12 GiB** for the rest of the effort; `/private/tmp/slice9a/run.py` now enforces 12 GiB. Observed M1 target peak **8,060,241,131 bytes (7.507 GiB)**; this is sampled filesystem size, not process memory. Consumer target is target/slice9a/consumer-m1, coverage uses the same effort tree. Raw LCOV/summary, source manifests and helper scripts stay under /private/tmp/slice9a/m1; reproducible fixture/loader scripts remain in the crate.


| Record ID | Label / status | Command s | Cargo s |
| --- | --- | ---: | ---: |
| `20261030T043021.271050Z-a74aadd1bd07` | edit loop Profile schema early path; passed | 5.296150 | 3.15 |
| `20261030T043614.463028Z-7968455e6e98` | edit loop complete SQL primitives compile; failed | 16.790341 | — |
| `20261030T043725.612592Z-5bf41560c1e3` | original Node schema and storage goldens; failed | 1.616476 | — |
| `20261030T043745.062424Z-3e305ed0ac01` | edit loop SQL compile after join syntax; failed | 2.917557 | — |
| `20261030T043823.533500Z-b82a1654938b` | original Node goldens using upstream TypeScript emit; failed | 0.752126 | — |
| `20261030T043857.606558Z-6c38f4004e25` | original Node records goldens; passed | 4.149420 | — |
| `20261030T043909.381667Z-c686b8ccdee1` | edit loop SQL primitives compile fixes; passed | 2.755142 | 2.13 |
| `20261030T044039.748187Z-58790e5507a8` | edit loop golden columns; failed | 19.209731 | 5.55 |
| `20261030T044233.586906Z-7dfa733a8e3c` | edit loop SQLite actions and golden columns; passed | 12.840791 | 4.95 |
| `20261030T044544.193904Z-c1eb43b52ea1` | round 1 source logic operators defaults and insert fallback; passed | 10.655007 | 4.54 |
| `20261030T044758.580561Z-5c54aa6e53ce` | source goldens Date undefined and source SQL operators; passed | 0.901478 | — |
| `20261030T044902.518584Z-a785f4036bff` | round 2 ownership schema descriptors and batch separation; failed | 9.611328 | 2.98 |
| `20261030T045047.820978Z-dc033a801d1d` | round 2 Date goldens and externally bounded transaction proof; passed | 9.707568 | 3.40 |
| `20261030T045204.859075Z-944e8357bd97` | round 3 consistency Clippy and Rustdoc baseline; failed | 4.859551 | — |
| `20261030T045559.254437Z-7cfb3e5d219e` | round 3 source docs and consistency verified; failed | 2.560155 | — |
| `20261030T045706.540207Z-2718976a770d` | round 3 final consistency docs and source membership; failed | 5.972896 | — |
| `20261030T045846.974001Z-b25da0a9d7a2` | source final records fixtures and native regex cases; passed | 0.800917 | — |
| `20261030T045925.156996Z-00a95aaa9d8b` | round 3 complete lint tests docs and native parity; failed | 11.976671 | 5.43 |
| `20261030T050012.534214Z-3442d5ce2aa0` | round 3 verified source membership renderer and docs; passed | 16.452957 | 8.66 |
| `20261030T050043.044042Z-6236148e7e01` | gate full workspace excluding coordinator tcp_watchdog; passed | 605.958054 | 274.46 |
| `20261030T050226.096424Z-b41a590c4e16` | gate source reproduction byte-identical lock and scope audit; passed | 2.073434 | — |
| `20261030T050510.261372Z-525b3cee16b7` | gate trim exact incremental at target cap; passed | 0.542632 | — |
| `20261030T051046.541156Z-ad0d9c529e7c` | edit loop native Date value boundary; failed | 5.184750 | — |
| `20261030T051123.915471Z-60a4e1e12e6b` | edit loop native Date values and unchanged object keys; failed | 6.298935 | — |
| `20261030T051253.877381Z-f2f1743d0346` | edit loop Date enum object-key regression; failed | 14.082725 | 6.80 |
| `20261030T051404.921118Z-6eb302eba06d` | follow-up native Date ownership and wire round; passed | 17.280432 | 8.73 |
| `20261030T051550.350519Z-e6515caddbda` | gate final workspace native Date schema contract; passed | 145.427313 | 12.96 |
| `20261030T051829.851785Z-cac968ff8513` | gate native feature matrix and WASM graph isolation; passed | 7.266438 | 4.30 |
| `20261030T052029.922952Z-df2b10d60504` | gate empty independent Profile consumer warm sccache; passed | 36.102829 | 35.34 |
| `20261030T052119.753762Z-4b6ade33c394` | gate session native SQL line coverage; passed | 22.635772 | 9.75 |
| `20261030T052417.581625Z-b55975962575` | gate final fixture lock scope and KB baseline audit; passed | 5.253703 | — |
| `20261030T052804.288814Z-809f4ff66241` | gate receipt baseline diagnostics and final source hashes; passed | 2.286048 | — |

Failed attempts remain recorded: source type-import/dialect loader fixes, SQL type/syntax adaptation, intentional golden failures exposing V8 date/nested Date behavior, Clippy findings, checked selector rendering, and native Date enum integration. Every relevant correction has passing final evidence. No failed receipt/source verification is represented as passed.

No repository/provider/facade changes outside the Lane A grant, historical Agent File edits, commits, pushes, installs, GitHub writes, upstream edits, or development listeners occurred. The only root change is the session package's internal framework-sql dependency edge in Cargo.lock; all external package blocks are byte-identical and no packages were added or removed. Shared helpers did not need extension. Warm artifacts remain for verification; test database engines close and externally supervised children are reaped. Stop at M1; coordinator verifies and commits.

## Coordinator verification

Coordinator brought trial/3 (Lane B M0 and Slice 10 M0/M2/M4) into the branch at `fab749c`. The lock is the union of both sides: no external tuple changed or was removed. Then reran on the merged tree at gate tier (`sccache`, non-incremental, empty target), recorded as `s9am1-verify gate *`. All passed:
- fmt; workspace Clippy `-D warnings`; workspace Rustdoc `-D warnings`.
- `cargo nextest run --workspace --all-targets --locked --no-fail-fast` with no exclusions: 822 passed, including `tcp_watchdog`.
- Workspace doctests: 951 passed.
- Coverage: framework-session with `sql,server` 89.35% lines. framework-schema with `providers,codegen` 70.25% lines; the provider files stay at 96–100%.

The three source questions above remain open with the owner; source parity is retained until they decide. Merged into trial/3 on 2026-10-30 (`bae2c26`); Lane A M2 released.
