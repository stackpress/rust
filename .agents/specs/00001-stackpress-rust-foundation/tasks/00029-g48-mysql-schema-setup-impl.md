# G48 MySQL schema setup — implementation receipt

Status: **Implementation and local verification complete; awaiting coordinator verification. No commit.**
Authority: [G48](../decisions.md), the coordinator release of releases 1–2 from [00028](00028-g48-mysql-schema-setup.md), and the 2026-11-03 CASCADE ruling read from the main checkout / trial/3 commit 31b7367. This receipt closes the MySQL install boundary reported in [00027](00027-g46-live-search.md); historical receipts remain unchanged.

Git integration: the clean implementation baseline was **2cc9b41**. `git merge-base --is-ancestor HEAD 31b7367` returned 1: 31b7367 does not include the coordinator's 2cc9b41 live-search/design commit. A fast-forward is impossible. No merge/reset/rebase/cherry-pick was attempted; all uncommitted work is preserved for coordinator integration. The binding decision was applied from its read-only text.

## Implemented contracts and source mapping

| Owner | Result / evidence |
| --- | --- |
| stackpress-sql | Public `Engine::execute_non_atomic(&self, &[Query]) -> Result<Vec<QueryResult>>`: validate the complete input before adapter IO, ordered owned results, valid empty input, stop on first error, no retry or rollback. [Core tests](../../../../crates/sql/sql/tests/engine/non_atomic.rs). |
| Error metadata | Public `PlanProgress` and `SqlError::plan_progress() -> Option<&PlanProgress>`: acknowledged count, zero-based failed index, total. Original kind/display/cause/cleanup stay intact. Progress is boxed to keep downstream error envelopes compact. Validation has zero acknowledgements; a failed in-flight statement can still have taken effect. Cancellation returns no report. |
| Shared policy | `stackpress_framework_sql::setup::{execute_plan, execute}` selects explicit non-atomic MySQL execution; PostgreSQL/SQLite retain atomic plan/transaction execution and rollback context. Whole-plan validation stays before execution. [Selector](../../../../crates/framework/framework-sql/src/setup.rs), [atomic tests](../../../../crates/framework/framework-sql/tests/setup.rs). |
| Source stores | Profile/Auth and Application/Session install/upgrade call that selector. CRUD, batch, identifiers, codecs and error envelopes remain at their existing owners. Live tests use the stores' actual DDL and alteration, mixed-case ASCII q/like, Auth-to-Profile join, and teardown with `cascade=true`. Children are uninstalled before parents. |
| Source commands | Aggregate install/uninstall/purge/upgrade/push use the selector. Source install uninstalls before installing; IF EXISTS drops remain reverse catalog order; skips and lazy resources, warning/force and revision insertion only after complete SQL success stay intact. Adjacent generated migrations apply through actual push→upgrade on both live engines. |
| Purge ruling | PostgreSQL retains parent-first TRUNCATE CASCADE; SQLite retains DELETE. MySQL callers omit CASCADE. Unrelated MySQL catalogs retain source-order TRUNCATE; catalogs declaring foreign keys use typed DELETE in reverse create order, children first. No runtime retry, constraint/session toggle or drop/recreate is added. App-authored Scripts keep their explicit query inventory. |
| Retained atomic APIs | `Engine::execute_plan`, ordinary Transaction implementation, aggregate public `transaction` and MySQL transaction-DDL refusal are byte-identical in effect/source to baseline (scope assertion). Live MySQL still refuses ordinary transaction DDL; PostgreSQL setup failure rolls back. The general MySQL CASCADE renderer refusal is unchanged. |

Source anchors inspected for setup: upstream `stackpress-sql/src/scripts/install.ts:18–44`, `uninstall.ts:17–25`, `purge.ts:17–26`, `upgrade.ts:18–56`, `migrate.ts:24–74`; `transform/scripts.ts:63–153`, `transform/helpers.ts:295–329`, generated install/purge/uninstall/upgrade action templates; `stackpress-session/src/profile/ProfileActions.ts:270–292,330–334` and AuthActions. Full input/order/skip/error inventory is preserved in 00028. G48 overrides the source MySQL CASCADE spelling (`inquire/src/dialect/Mysql.ts:662–666`). No startup IO, global guard, default-deny or route arbitration is added (G42/G43).

## Native results and owned database cleanup

Final live replay **53accb0b7bd7**: **9 passed / 0 skipped** (six actual backend scenarios, three external-watchdog forced-kill controls). The shared 00027 test-only fixture uses the existing adapter and process/discovery helpers. Gates restrict local host/port/identity; URLs are never printed. Every database uses unique CREATE DATABASE without IF NOT EXISTS; cleanup arms only on acknowledgement and drops that exact name on success/failure/unwind. Parent 30-second and administrative 9/10-second deadlines remain. No enumeration/adoption/sweep, server startup, role/global change or listener occurs.

Real MySQL: A/duplicate A/C leaves A durable and C absent, progress 1/1/3. PostgreSQL rolls back A. Suspended provider barriers prove unpolled execution, cancellation before submission, after an acknowledged prefix and after native execution without outer acknowledgement; no later statement/task is submitted. Source install failure does not publish a revision. Existing generated SQLite migration tests pass unchanged.

Initial source CASCADE probe (record **4fb40c6fdd14**, now superseded by the owner ruling) returned verbatim:

```text
ERROR 1064 (42000) at line 1: You have an error in your SQL syntax; check the manual that corresponds to your MySQL server version for the right syntax to use near 'CASCADE' at line 1
```

After omitting CASCADE, the live foreign-key control (record **53accb0b7bd7**) returned verbatim:

```text
error returned from database: 1701 (42000): Cannot truncate a table referenced in a foreign key constraint (`stackpress_g46_mysql_c13_18e45b95067e47f0`.`g48_child`, CONSTRAINT `g48_child_parent`)
```

The actual `Scripts::from_tables` + public `sql::purge` then clears child and parent with DELETE. An orphan insert still fails, proving constraints remain active. Repeated purge succeeds; PostgreSQL's actual parent-first CASCADE purge also succeeds. The current probe uses the real Engine/adapter directly; it needs no installed mysql CLI.

Every created database in all G48 attempts is listed below, with one confirmed drop per name (including the initial failed aggregate probe). Counts come from recorder logs, not server enumeration.

| Exact database — created / drop confirmed | Record |
| --- | --- |
| `stackpress_g46_mysql_68f7_18e3f3956b117420` — **1 / 1** | `20261101T145444.195406Z-5b8a0b772bcb` |
| `stackpress_g46_postgres_68ff_18e3f3957850db58` — **1 / 1** | `20261101T145444.195406Z-5b8a0b772bcb` |
| `stackpress_g46_postgres_68d4_18e3f395563dd1b0` — **1 / 1** | `20261101T145444.195406Z-5b8a0b772bcb` |
| `stackpress_g46_mysql_69d5_18e3f39592ba1658` — **1 / 1** | `20261101T145444.195406Z-5b8a0b772bcb` |
| `stackpress_g46_postgres_69eb_18e3f3959c4366e8` — **1 / 1** | `20261101T145444.195406Z-5b8a0b772bcb` |
| `stackpress_g46_mysql_68d5_18e3f395563fbde0` — **1 / 1** | `20261101T145444.195406Z-5b8a0b772bcb` |
| `stackpress_g46_mysql_11e67_18e3f4c1ecfbbe00` — **1 / 1** | `20261101T151623.855521Z-4fb40c6fdd14` |
| `stackpress_g46_mysql_11e97_18e3f4c23c3e4118` — **1 / 1** | `20261101T151623.855521Z-4fb40c6fdd14` |
| `stackpress_g46_postgres_11e66_18e3f4c1ecfb0668` — **1 / 1** | `20261101T151623.855521Z-4fb40c6fdd14` |
| `stackpress_g46_postgres_11ea1_18e3f4c249ab4800` — **1 / 1** | `20261101T151623.855521Z-4fb40c6fdd14` |
| `stackpress_g46_mysql_11eb5_18e3f4c25baa0ff0` — **1 / 1** | `20261101T151623.855521Z-4fb40c6fdd14` |
| `stackpress_g46_postgres_11eb6_18e3f4c25c4430f8` — **1 / 1** | `20261101T151623.855521Z-4fb40c6fdd14` |
| `stackpress_g46_mysql_c13_18e45b95067e47f0` — **1 / 1** | `20261102T224037.558392Z-53accb0b7bd7` |
| `stackpress_g46_mysql_c20_18e45b9531a1f3a0` — **1 / 1** | `20261102T224037.558392Z-53accb0b7bd7` |
| `stackpress_g46_postgres_c27_18e45b954580c630` — **1 / 1** | `20261102T224037.558392Z-53accb0b7bd7` |
| `stackpress_g46_postgres_c14_18e45b950664efa8` — **1 / 1** | `20261102T224037.558392Z-53accb0b7bd7` |
| `stackpress_g46_mysql_c43_18e45b956cac87a8` — **1 / 1** | `20261102T224037.558392Z-53accb0b7bd7` |
| `stackpress_g46_postgres_c45_18e45b956ff73e08` — **1 / 1** | `20261102T224037.558392Z-53accb0b7bd7` |

## Refactoring rounds

| Round | Finding, action and proving rerun |
| --- | --- |
| 1 — complexity | Reviewed complete preflight vs execution, every failure index, empty input and cancellation. **Nothing further found** after the natural two-pass implementation. Core engine suite: 12 passed; later external watchdog: three new tests plus forced-kill negative control passed. |
| 2 — responsibility/abstraction | One shared setup selector owns backend choice; stores retain model actions and aggregate retains command/source ordering. Shared test-only ownership is reused. **Nothing further found** requiring another abstraction. Initial focused run: eight passed including unchanged SQLite migrations; extension rerun: nine passed including the three-dialect/FK catalog table. |
| 3 — consistency/Rustdoc/comments | Added the public progress/setup references and current session/API/aggregate claim anchors; documented native evidence vs provider limits. A stale transaction claim was corrected and rerun. 436 scoped doctests passed; final full workspace doctests, docs claims, links and Rustdoc passed. |
| Targeted follow-ups | Workspace Clippy exposed an inline progress size regression: boxed metadata restores compact downstream errors; core 12 and Clippy reruns passed. The initial CLI probe's duplicate module import was removed; the new typed 1701 proof eliminates that CLI path entirely. Extension review kept the named catalog predicate and existing store flag rule; no new helper/API was justified. Source/API comments and docs now match the owner ruling. Final live and default/compatible workspace gates passed after these changes. |

## Commands and final gate

All Cargo/proof/clean commands use `.agents/scripts/build-benchmark.py` at the canonical main-checkout store, wrapping each outer script once. Cargo resolution/build/test/check/doc commands use `--locked --offline`, except the eight specifically authorized one-time offline consumer resolutions. No update/generate-lockfile/install/fetch was used. Nextest/libtest use two test threads; workspace nextest excludes tcp_watchdog; live URLs are unset for the workspace/coverage gate. fmt has no resolution flags. No renderer all-features was used.

Environment: target/g48, warm sccache, two build jobs, debug=0, incremental=0. Initial fresh-target cargo clean and llvm-cov clean were recorded once. Other effort targets are retained. Cap **24 GiB**, guard **23.4 GiB**. Completed owned consumer outputs are removed in the recorded consumer step; main debug dependency artifacts stay warm.

| Actual command family | Result |
| --- | --- |
| cargo fmt --all --check | Passed. |
| cargo clippy --workspace --all-targets [--features F] --locked --offline -- -D warnings | Default + compatible passed. |
| cargo nextest run --workspace --all-targets [--features F] --locked --offline --test-threads 2 --no-fail-fast -E 'not test(tcp_watchdog)' | Default: **939 tests run: 939 passed, 88 skipped**; compatible: **1057 tests run: 1057 passed, 95 skipped**. |
| cargo test --workspace --doc [--features F] --locked --offline -- --test-threads 2 | **963 default / 968 compatible passed**. |
| RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --features F --locked --offline | Passed. |
| Core engine nextest / external watchdog | 12 engine cases passed; three new cases under a 15-second external deadline plus SIGTERM-resistant forced-kill control passed. |
| Focused setup/catalog/SQLite migration nextest | Nine passed after the purge extension; existing SQLite MigrationRunner remains unchanged. |
| Live nextest (session + API live_search, aggregate live_setup) | Nine passed; real MySQL and PostgreSQL searches/setup/adjacent upgrade/purge plus controls. Explicit unset replay: three controls passed / six live cases skipped, never counted as passed. |
| consumers/graphs.py --check; consumers/run.py | Passed all eight patterns (build-host, full, direct, headless, api-disabled, disabled, facade, website). Exact direct/facade phase/event/priority/response inventory equals full. Separate WASM hydrate/CSR checks passed; no API in api-disabled normal graph, no codegen in runtime/browser graphs. Default facade first-party graph exactly {stackpress, stackpress-lib}. |
| session/docs/check.py; aggregate/docs/check.py; aggregate/proofs/check.py | 21 pages / 17 fences / 38 claims / 4 controls; 19 pages / 11 fences / 62 claims / 7 controls; full ledger passed. |
| Changed documentation | 11 Markdown pages / 151 local links pass; six existing runnable fences unchanged. New core no_run example compiled by doctests. No changed aggregate fence requires external recompilation. |
| KB validator | Actual coordinator main checkout: **0 errors / 33 warnings**. This worktree with canonical sibling links: **15 expired temporary-evidence errors / 28 warnings**; raw worktree: 161 errors / 28 warnings. All 15 canonical errors belong to four untouched historical receipts, corrected by 31b7367, which cannot fast-forward here. **Zero introduced errors**; no missing task-link error. Historical files were preserved. |

F = `framework-aggregate-full,framework-aggregate-ui-session,framework-aggregate-ui-api,framework-aggregate-ui-admin,framework-aggregate-ui-view,framework-aggregate-ssr,framework-api-codegen,framework-csrf-server,framework-email-server,stackpress-framework/codegen,stackpress-framework-schema/providers,stackpress-framework-sql/sqlite-maintenance,stackpress-framework-sql/codegen,stackpress-framework-sql/server`.

Coverage: installed cargo-llvm-cov/LLVM on Rust/Cargo 1.96.0, native aarch64-apple-darwin. Each command: `cargo llvm-cov -p PACKAGE [--features FEATURES] --locked --offline --fail-under-lines 70 --json --output-path /private/tmp/g48/PACKAGE-coverage.json -- --test-threads 2`. Framework-sql also selects `-p generated-workspace-app -p generated-workspace-tool` with the default app fixture: its generic stores and emitters are instantiated/executed by those existing real consumers. The first own-tests-only scope measured 3526/6306 = 55.92%, including 85.71% for new setup.rs; no source/exclusion/threshold changed to replace that incomplete test selection. An attempted combined matrix/names selection hit the intentional compile_error; it was corrected to the default fixture without touching that crate. The final table retains its full owning src denominator. Native live proofs above are separate from coverage with unset live gates. The outer warm record includes a warm unset-gate replay; its instrumented subtarget starts cold and is retained across the five measurements. Each successful report uses the normal fresh profile collection; no stale-source/no-clean merge is used.

| Changed implementation crate / features | Covered executable lines |
| --- | --- |
| stackpress-sql / `default` | 3222/3394 = **94.93%** |
| stackpress-framework-sql / `stackpress-framework-sql/server,stackpress-framework-sql/codegen,stackpress-framework-sql/sqlite-maintenance` | 5446/6306 = **86.36%** |
| stackpress-framework-session / `auth-server,leptos-ssr` | 5223/5711 = **91.46%** |
| stackpress-framework-api / `sql,codegen,ui-ssr` | 3709/4147 = **89.44%** |
| stackpress-framework / `full,codegen` | 1064/1100 = **96.73%** |

## Exact file and lock delta

Root Cargo.lock adds exactly two aggregate workspace dev edges: stackpress-sql-mysql and stackpress-sql-postgres. All **699 external package entries remain byte-identical**, same set/count. Activating the already-declared framework-sql dependency under aggregate sql changes no root lock text. The manifest also gives aggregate tests the existing framework-sql migrations feature and declares the finite custom live target. No new external dependency.

Each of the eight consumer manifests received one recorded offline resolution seeded from root bytes; registry subset and unchanged root bytes were asserted immediately after each command. Six locks add only stackpress-framework-sql to the aggregate workspace dependency list: api-disabled, direct, facade, full, headless, website. build-host and disabled remain byte-identical. Each consumer's external entries remain byte-identical. Later runs are locked/offline. **No expected-graph fixture changed**; only the ledger's current evidence wording changed.

Exact changed paths: **40 including this receipt**. Frozen baseline tracked files: 3549. Ordinary atomic functions/Transaction, MySQL adapter/renderer, SQLite generated runner, root manifest/facade, license files, .agents/context and historical records are unchanged. Nothing staged or committed.

- `.agents/specs/00001-stackpress-rust-foundation/tasks/00029-g48-mysql-schema-setup-impl.md`
- `Cargo.lock`
- `crates/framework/framework-api/README.md`
- `crates/framework/framework-api/src/models/actions.rs`
- `crates/framework/framework-api/tests/live_search.rs`
- `crates/framework/framework-session/docs/api/storage.md`
- `crates/framework/framework-session/docs/claims.json`
- `crates/framework/framework-session/docs/live-search.md`
- `crates/framework/framework-session/docs/source-behavior.md`
- `crates/framework/framework-session/docs/storage.md`
- `crates/framework/framework-session/src/storage/actions.rs`
- `crates/framework/framework-session/tests/live_search.rs`
- `crates/framework/framework-sql/docs/api/README.md`
- `crates/framework/framework-sql/docs/api/setup.md`
- `crates/framework/framework-sql/src/lib.rs`
- `crates/framework/framework-sql/src/setup.rs`
- `crates/framework/framework-sql/tests/setup.rs`
- `crates/framework/framework/Cargo.toml`
- `crates/framework/framework/consumers/api-disabled/Cargo.lock`
- `crates/framework/framework/consumers/direct/Cargo.lock`
- `crates/framework/framework/consumers/facade/Cargo.lock`
- `crates/framework/framework/consumers/full/Cargo.lock`
- `crates/framework/framework/consumers/headless/Cargo.lock`
- `crates/framework/framework/consumers/website/Cargo.lock`
- `crates/framework/framework/docs/api/sql.md`
- `crates/framework/framework/docs/claims.json`
- `crates/framework/framework/docs/source-behavior.md`
- `crates/framework/framework/proofs/coverage-map.json`
- `crates/framework/framework/src/native/commands/sql.rs`
- `crates/framework/framework/tests/cases/sql.rs`
- `crates/framework/framework/tests/live/cancellation.rs`
- `crates/framework/framework/tests/live/cascade.rs`
- `crates/framework/framework/tests/live_setup.rs`
- `crates/sql/sql/docs/api/engine.md`
- `crates/sql/sql/docs/api/errors.md`
- `crates/sql/sql/src/engine.rs`
- `crates/sql/sql/src/error.rs`
- `crates/sql/sql/src/lib.rs`
- `crates/sql/sql/tests/engine.rs`
- `crates/sql/sql/tests/engine/non_atomic.rs`

## Named limits

- **G27 / generated-workspace runner:** SQLite-only remains accepted. No MysqlMigrationRunner, target-lock/drift protocol or generated-workspace ledger is released or added; this is a named scope limit, not an implementation gap promised for follow-up. MySQL source upgrades use the existing aggregate adjacent command.
- **MySQL partial state:** acknowledged prefix may remain; failed in-flight outcome can be unknown. No automatic retry/repair/adoption, rollback or connection-affinity promise. Application-owned setup/reconciliation stays explicit.
- **Catalog purge:** supply dependency-safe create order and the complete intended catalog. Related MySQL DELETE preserves DELETE semantics; no discovery/teardown of outside tables or session constraint toggles. Standalone store purge is still a single-table action.
- **Native/provider scope:** finite direct local MySQL/PostgreSQL commands/searches prove these source paths. Live event-provider routing, arbitrary collations/JSON queries, remote delivery, installed targets and listeners are not claimed. UI source did not change; this task uses native/SSR consumer and WASM graph/compile evidence rather than a new browser matrix.

## Recorder evidence

Canonical store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks/runs`. These records are retained outside the worktree and never staged. Full nested argv, output and timings are in each matching .json/.log; disposable runners/reports are under /private/tmp/g48. Command wall time and reported Cargo-stage subtotal are separate; missing Cargo-stage data is not treated as measured zero.

Failed attempts remain visible: initial live compile callback type mismatch; initial admitted/raw TRUNCATE probe boundary (all owned DBs dropped); stale claim; inline error size Clippy finding; duplicate probe import; locked consumer resolution requirement; new purge identifier borrow. Each has a successful corrected proof below. The source 1064 result is superseded, not erased.

| Record | Status / intent | Command s | Reported Cargo s |
| --- | --- | ---: | ---: |
| `20261101T144236.670368Z-665c66191ac7` | passed — g48 baseline clean fresh target warm sccache | 2.04 | unavailable |
| `20261101T145010.560864Z-0d95f26ce189` | passed — g48 edit loop executor progress validation cancellation and exact lock edges | 28.29 | 25.00 |
| `20261101T145156.954043Z-787951406b3c` | failed — g48 edit loop real session API search setup aggregate upgrades and cancellation | 125.65 | unavailable |
| `20261101T145444.195406Z-5b8a0b772bcb` | failed — g48 edit loop live proof corrected owned Scripts callback | 16.69 | 10.98 |
| `20261101T151623.855521Z-4fb40c6fdd14` | passed — g48 edit loop live upgrades cancellation and preserved native CASCADE failure | 8.40 | 3.76 |
| `20261101T151920.220903Z-89c54d784f4b` | passed — g48 round 1 complexity and complete preflight error ordering | 4.28 | 0.74 |
| `20261101T151956.756977Z-31e31e66a360` | passed — g48 round 2 shared setup responsibility and SQLite migration regressions | 78.11 | 60.00 |
| `20261101T155126.216774Z-0917df595f51` | failed — g48 round 3 Rustdoc consistency and current live-evidence claims | 2.23 | unavailable |
| `20261102T220104.107232Z-7d47c49eb5d9` | passed — g48 round 3 corrected setup claim anchors and doctests | 19.09 | 11.24 |
| `20261102T220226.390750Z-08ba7a620dea` | failed — g48 gate default and compatible workspace including SQLite migrations | 66.80 | unavailable |
| `20261102T220443.444866Z-0feb6142a591` | passed — g48 targeted round compact optional progress error layout | 4.35 | 1.36 |
| `20261102T220515.410870Z-32163815c4de` | failed — g48 gate rerun compact errors default and compatible workspace | 635.02 | 183.21 |
| `20261102T221806.156888Z-05c1993d5e8f` | passed — g48 gate finish combined graph shared watchdog module | 671.78 | 243.32 |
| `20261102T223001.992313Z-ad215bbeb0c9` | failed — g48 aggregate facade graphs all eight consumers and owned output cleanup | 310.98 | 296.75 |
| `20261102T223941.247982Z-2165c8db9cd7` | failed — g48 owner purge ruling live MySQL foreign key and PostgreSQL cascade replay | 16.26 | unavailable |
| `20261102T224037.558392Z-53accb0b7bd7` | passed — g48 purge replay corrected identifier borrow and real FK tables | 13.78 | 4.66 |
| `20261102T224125.191264Z-5fcec8d1242d` | passed — g48 consumer lock offline resolution root tuple and byte assertions | 2.66 | unavailable |
| `20261102T224202.515312Z-5e0dd2d97637` | passed — g48 purge extension focused rounds setup SQLite migration and catalog tests | 26.45 | 12.89 |
| `20261102T224252.833400Z-d870f326d6a6` | passed — g48 final gate after MySQL purge ruling default and compatible workspace | 323.14 | 63.10 |
| `20261102T224608.985153Z-c575c3d81e32` | passed — g48 final documentation claims ledger local links and unchanged runnable fences | 2.70 | unavailable |
| `20261102T224848.637056Z-c81e823904f6` | passed — g48 final aggregate facade graphs all eight refreshed locked consumers | 1262.40 | 1214.05 |
| `20261102T225624.935961Z-11c611c83e2c` | passed — g48 external core cancellation watchdog and forced-kill negative control | 3.19 | 0.56 |
| `20261102T231020.693008Z-3f81e9c63ceb` | failed — g48 five changed crate coverage floors and explicit unset live gate proof | 89.82 | 63.31 |
| `20261102T231440.823084Z-9daf54ab4b1d` | failed — g48 coverage remaining crates with existing generated SQL app and generator consumers | 13.41 | unavailable |
| `20261102T231611.037548Z-390c65feb155` | passed — g48 coverage real SQL consumers with compatible default generated fixture | 318.79 | 244.08 |
| `20261102T232230.681800Z-3caa5358e6c5` | failed — g48 closing exact scope lock registry KB and implementation receipt | 2.69 | unavailable |

Through the completed records above: **26 attempts**, 10 failed; command wall total **4048.99s**, reported Cargo-stage subtotal **2439.01s** (incomplete, not end-to-end time). Observed Cargo activity: {'compiling': 4149, 'checking': 2909, 'documenting': 56}. Counters start with this effort's first recorded clean, without reconstructing earlier history. Peak owned target size **17.26 GiB**, below 23.4 GiB guard / 24 GiB cap.

Final scope/lock/KB/receipt verification: `20261102T232417.651291Z-b78c2b45fb05` (see its completed recorder result). Stopped for coordinator review; no successor release work started.
