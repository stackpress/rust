# Task 00054 — no-panic framework/data receipt

Status: local verification passed with retained baseline failures and the coverage diagnostic below; foundation integration remains for the coordinator. Coordinator merges and reviews; owner accepts.
Branch `hardening/no-panic-framework`, base `7213e77`. Scope: SQL, framework SQL/schema/session/API and aggregate.
Authority: G62, G42/G43; coordinator's constant-literal exception and cross-lane instructions.

## Changes and public signatures

- All six library roots deny unwrap/expect/panic/unimplemented/todo/unreachable Clippy lints.
- Framework SQL now rejects malformed handwritten relation/unique metadata with existing typed errors. Valid generated catalogs, statement ordering, unknown-path retention, and external-model skips remain unchanged.
- `SourceStore::create()` → `Result<Create, Self::Error>`; `NativeStore::create()` → `SourceResult<Create>`; `paths`, `selectors`, `joins` → `SourceResult<Vec<…>>`. Owned implementations/callers propagate errors.
- Adapted the eight handwritten blog storage callers to propagate preparation failures before deriving table definitions.
- Aggregate emitted client `definition()` → `stackpress_lib::Result<Vec<TableDefinition>>`, wrapping associated-store and SQL errors. This follows the fallible store contract.
- API `views::management::import_csv` accepts `impl AsRef<[u8]>` and calls `transfer::parse(csv.as_ref(), …)`. Native admin import passes borrowed original upload bytes; UTF-8 input follows the same parser path. Existing JSON row batches bypass the raw-body fallback. End-to-end tolerant decoding remains for the coordinator's combined admin/API gate.
- Schema emitters propagate codec/metadata errors and build fixed property objects directly. Generated columns retain source declaration order without runtime lookup expectations.
- Datetime retains its checked instant; canonical wire text, debug output and nanosecond values remain stable. Migration plans retain the checked dialect. Session/API code uses typed matching/records and infallible string construction.
- No Cargo dependency/manifest/lock, original/Node golden, oracle, vector or `.stderr` change. Driver test fixtures/guides were additionally changed only under the coordinator's explicit cleanup-bug authorization; no driver production code changed.

## Tests and refactoring rounds

- Crash regression: missing local/foreign relation keys return 500 errors during DDL preparation and path resolution; missing unique columns return an ordinary internal error.
- Compiled emitted `describe()` model/catalog values round-trip the composed source graph. The test invokes official freshness checks and compares stored-column order as well as descriptors.
- Non-UTF-8 request-body CSV bytes are borrowed unchanged (including buffer identity); a second regression preserves JSON row batches with buffered bodies. Default language and session-asset constant constructors have direct unit proofs.
- Datetime tests cover canonical wire/debug output, offsets, pre-epoch nanoseconds and supported year edges. Other removed sites use guarded/private invariants or fixed shapes; existing invalid-schema tests still reject malformed input before emission.
- Initial six-crate baseline: 468 tests; 465 passed, three PostgreSQL search cases exceeded their existing 2s deadline under parallel load. All three passed an exact serial retry without changing deadlines or expectations.
- Round 1 (complexity): named traversal result and a single relation lookup; 65/65 targeted tests.
- Round 2 (responsibility): shared encoding error conversion and checked dialect ownership; 71/71 targeted tests.
- Round 3 (consistency/docs): public error docs, fixed property ordering and cache explanation; 72/72 targeted tests. The two added literal-default proofs subsequently passed 2/2. Final gate includes the datetime debug-compatibility assertion.

## Approved exceptions / foundation merge

One template-level exception entry for the coordinator to add to `.agents/context/no-panic-exceptions.md`:
`crates/framework/framework-schema/src/codegen/templates/{source-model,source-catalog}.rs.tpl`, generated `describe()`: checked `Model`/`SourceCatalog` serialized into a constant JSON literal, guarded by official freshness plus emit/compile/describe/compare proof. Narrow `#[expect(clippy::expect_used, reason = …)]` is emitted by the templates.

Two local compatibility sites remain under the coordinator's approved temporary constant-literal exception; replace at foundation merge and remove their temporary expectation attributes:

- API `src/views/mod.rs`, `Consent::new`: `Language::default()`.
- Session `src/presentation.rs`, `Presentation::default`: `PublicLocation::session_asset()`.

Foundation applies the following fallible `Nest::entries()` callers at merge. These exact lines are unchanged here so git takes the foundation versions:

- Session `src/auth/bridge.rs:49`: `.entries()?`.
- API `src/helpers.rs:79`: `for (key, value) in fixed.entries()? {`.
- Session `tests/email/integration/signup.rs:13`: `.entries()?.clone()`.
- Session `tests/email/integration/mod.rs:183`: `serde_json::to_value(req.data.entries()?)?`.
- Session `tests/main/account_flows.rs:30`.
- Session `tests/main/auth_flows/oracle.rs:48` and `:270` (the latter unwraps in a unit-returning test).
- Session `tests/main/auth_flows/events.rs:334`.

Already applied byte-identical, independently compatible fixes: API `src/helpers.rs` uses `target.len()`; API `tests/endpoints.rs` uses `req.data.len()`.
The bridge file also has an unrelated percent-encoding edit; the `entries()` line itself is unchanged.

## Gate, coverage and evidence

- Default workspace: 1,086/1,087 main tests passed; one PostgreSQL driver lifecycle pool timeout passed an unchanged isolated retry; TCP watchdog passed. Default framework-search filter selected zero cases (not applicable).
- Compatible workspace: 1,316 main + one TCP + three serial PostgreSQL search cases passed. After final API review, all 108 API cases passed in the coverage replay, including the added JSON row-batch regression.
- Formatting; six standalone all-target Clippy checks; workspace Clippy default/compatible; doctests and Rustdoc `-D warnings` default/compatible passed. Scoped Clippy followups fixed a generic borrow and test module/guard lint findings without allowances.
- Aggregate/session native/WASM graphs and all 12 standalone consumers passed, including build-host `--check`; all four model generators, generated-workspace/admin and API checkpoint freshness checks passed. `check_packages.py` passed.
- Blog first run exposed eight handwritten `create()` callers. After adapting them, native checks/docs, both browser modes/public matrix, copied-out nickname preparation/freshness/browser and locks pass. Two additional tests compare emitted-client DDL with explicit setup across three dialects and exercise generated Article typed fields/storage round-trip; fresh native coverage executes 92/92 tests. The expanded generated-source denominator initially put blog-web at 68.25%; those real generated-path proofs bring it to 70.68% without exclusions. Original failed logs are retained.
- Ordinary live compatible coverage: 1,319/1,320 initially passed; the unchanged isolated instrumented PostgreSQL lifecycle retry passed. All owned cases passed; API replay 108/108 passed. Final accumulated report passed all six 90% floors; no added exclusions or custom export.

| Crate | Covered / executable source lines | Percentage |
| --- | --- | --- |
| SQL | 3292 / 3394 | 96.99% |
| Framework SQL | 7844 / 8667 | 90.50% |
| Schema | 6311 / 6860 | 92.00% |
| Session | 3704 / 3968 | 93.35% |
| API | 2263 / 2497 | 90.63% |
| Aggregate | 1176 / 1232 | 95.45% |

Coverage uses cargo-llvm-cov 0.8.6 / rustc 1.96.0 / LLVM 22.1.2, aarch64-apple-darwin, the exact `common.COMPATIBLE` set and both approved local databases. Counts summarize first-party `src` rows from the ordinary report. Generated code is additionally proved by compiled descriptor round-trip, consumer execution, official freshness and the blog gate. Other runtime platforms are not measured.
Evidence: `target/00054-framework/evidence/{gate,continuation,fixture-followup}.json`, `evidence/blog-{followup,generated-path,clean-coverage,fresh-finish}/results.json`, `evidence/packages.json`; ordinary profiles/reports under `target/00054-coverage`. Final six-crate report SHA-256: `02723ddc6807bde921742bf81fa3b3ebe5aae123e4541653fbc82b16c4f55665`.

The official blog LLVM exporter warns **68 functions have mismatched data** on the final fresh profiles (the original ordinary export already warned 92). This diagnostic is retained; no source/file/line exclusions were added. The final official report still contains the same 3,121 blog-web executable lines and passes all six native blog crate floors; the thin browser client retains its existing G56 exemption.

Observed main target peak **12.17 GiB briefly exceeded the 12 GiB cap**. The fresh blog coverage rebuild also reached **12.08 GiB**, stopped at the budget guard after 92 passing tests, then resumed after retiring completed native executable/docs output. Completed task-owned executable/trybuild/generator/consumer artifacts were retired; library artifacts stayed warm. Final allocated sizes: main 11.52 GiB, coverage 7.46 GiB, blog 10.84 GiB.

The root lock is byte-identical to HEAD (699 registry tuples); no manifests/other locks changed. `git diff --check`, ownership/protected-output review and all 46 registered test copies pass. The local-only Clippy configuration is removed from the final diff.

## Follow-up: uncertain CREATE cleanup

The three baseline G46 PostgreSQL CREATE requests hit the existing 2s deadline after server-side creation. Because cleanup was armed only on acknowledged success, no DROP ran. The encoded name timestamps/order map them to aggregate live_setup, API live_search and session live_search respectively. All were idle; exactly these three were dropped and their absence verified:

- `stackpress_g46_postgres_a9f2_18e732004ee40478` (04:32:33.435387 UTC).
- `stackpress_g46_postgres_b447_18e732020b1d9ef0` (04:32:40.888246 UTC).
- `stackpress_g46_postgres_d902_18e7320b289e2590` (04:33:20.037914 UTC).

Cleanup now reserves the unique name before sending CREATE, attempts and records exact `DROP DATABASE IF EXISTS` on errors/unwind, and preserves the original CREATE error. Scope: canonical session `tests/fixtures/live_support.rs` plus byte-identical API/aggregate copies; SQL `tests/network.rs`; PostgreSQL/MySQL `tests/m0.rs`, `src/tests/{m1,m2}.rs` and testing guides; generated-workspace `driver_stores/{postgres,mysql}.rs`; copy manifest hashes. `crates/sql/test-support/harness.rs` has no database ownership and required no change.

Live proof: 15/15 G46 cases including injected post-CREATE error and exact catalog absence on both servers in all three copies; 22/22 SQL/driver cases covering every changed fixture; 3/3 generated-store cases. Compatible Clippy/fmt/copy/package followups pass. Audit records 36 captured CREATE/DROP pairs plus the three manual cleanup pairs. Final PostgreSQL and MySQL `stackpress_*` catalogs are both empty: `evidence/database-audit.json`. No prefix sweep or unrelated database deletion occurred.

All Cargo/proof commands use the shared benchmark recorder, sccache, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, two bounded build jobs and task-owned targets. Benchmark store:
`/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`.

Selected records:

- Baseline: `20261112T043112.514499Z-01f7a18a275b`; exact serial retry `20261112T044036.276485Z-2f0ed1279708`.
- Round 1: `20261112T044147.162035Z-8d3a94d0e0f1`.
- Round 2: `20261112T044400.207782Z-414b08416344`.
- Round 3: `20261112T044631.600646Z-e513b9936ef5`.
- Constant proofs: `20261112T045046.022922Z-d85edf4953c9`.
- Official primary generation: `20261112T042251.463059Z-7dd2912e414e`; consumer/checkpoint generation `20261112T044653.463649Z-ac93386e894e`.

Additional gate records:

- Broad gate: `20261112T045207.701282Z-5365b03ac9e1` (original failures retained).
- Isolated PostgreSQL / session graphs / format: `20261112T053556.519486Z-56ddbebd3dd2` (intermediate Clippy failure retained).
- Final compatible Clippy / format: `20261112T055706.585508Z-377c511baa71`.
- Full coverage: `20261112T052135.157119Z-43a2efdf0c44`; complete-workspace ordinary report: `20261112T055333.968503Z-6e48ec01fce8`; API replay: `20261112T060516.242858Z-bc41c2d48a24`.
- Blog followup: `20261112T062250.413820Z-60b3c17fa549`; generated-path followup: `20261112T065552.859682Z-9dd9c311371e`; fresh 92-test profiles: `20261112T065804.442915Z-a0d14429626e` (budget stop after tests); final quiet PostgreSQL coverage: `20261112T064733.854356Z-9b28b54466ef`.
- Final fresh blog export: `20261112T065927.704584Z-1d681e8cdb79`; resumed the completed fresh-profile run after budget cleanup, without repeating tests.
- Exact three-name cleanup: `20261112T064739.057439Z-3b964c4e1786`; fixture followup: `20261112T065311.256605Z-9e1bf66d74a1`; final create/drop audit/catalogs: `20261112T065628.537485Z-674f2f9a14e9`.

The initial selected check used a freshly cleaned target with warm sccache but was incorrectly labeled cache=warm; subsequent instrumented coverage was labeled cold. The recorder preserves failed attempts. `command_seconds` measures end-to-end time; observed `cargo_build_seconds` is a partial Cargo-stage measure where outer scripts capture output. No aggregate compile-time claim is made. This worktree added 59 completed recorder attempts (41 passed, 18 failed; none active); preserved totals are 11,116.26 command-seconds and 6,966.51 observed Cargo-stage seconds. These overlapping totals are not elapsed time or complete compilation time.

Agent workspace validation: 146 inherited errors and 44 warnings, chiefly missing historical TypeScript source links; final output is byte-identical to the initial baseline. No commit, push, install or listener was performed.
