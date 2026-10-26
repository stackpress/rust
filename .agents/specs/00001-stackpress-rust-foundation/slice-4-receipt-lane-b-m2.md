# Slice 4 lane B M2 receipt

Task: [00007 SQL](tasks/00007-slice-4-sql.md), SQLite execution lane, milestone M2. Authority: the coordinator's M2 release and follow-up sequencing instruction, G24, and [the M2 milestone and risk register](slice-4-design-verification.md). This receipt stops for coordinator re-verification; it does not release M3, slice-wide rounds, full documentation or publication.

## Baseline and ownership

- Worktree `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s4-m0`, branch `trial/3-s4-m0`, starting/unchanged HEAD `8b74328c704fea3666492b95750c17fd54b37be3`.
- Started clean, with both lanes' accepted M1 code. Core has the six SQLite builders, Expr/JSON, SqlValue and ordered row metadata. Its M2 network renderers, TableDefinition/SchemaDiff and public execute_plan APIs are not present in this baseline.
- Writes: only `crates/sql/sql-sqlite/**` and this receipt. No edits to core SQL, shared crates, root/facade manifests, Cargo.lock, dependencies or features. No commit, push, listener, deployment, publishing or messages to other agents.
- Compiler/target: rustc 1.96.0 (`ac68faa20`, LLVM 22.1.2), Cargo 1.96.0, native `aarch64-apple-darwin`. SQLx stays exactly 0.9.0 with the approved features/dependency budget.

## Plan execution and integration boundary

**Pending on merge:** native integration tests through `Engine::execute_plan(&QueryPlan)` and `Transaction::execute_plan` remain the coordinator's explicitly sequenced short follow-up. The coordinator confirmed that lane A is implementing those methods on existing begin/run/commit traits, and instructed this lane not to wait or edit core.

Current native proofs render the real Create/Alter builders, call `QueryPlan::validate` over the entire batch before acquiring a transaction, execute each actual `plan.queries()` entry through the same exclusive public Transaction, then commit or use `rollback_after` on failure. This is the test-only `tests::m2::plan` helper, not a replacement public API, fake Database or claim that the absent public entry point was tested.

TableDefinition/SchemaDiff output execution is likewise not claimed before core integration. Existing explicit Create/Alter plans suffice for this lane's native tests; no core shape or trait change is requested. Network execution remains deferred.

## Production changes

- **Schema/metadata correctness after DDL:** a warmed pooled connection could retain its old schema view and SQLx result-column metadata after ALTER on another connection. Native regression first returned quoted expression names for newly renamed/added fields; a controlled cached `SELECT *` regression reproduced stale metadata with the fix disabled.
- Before preparing the requested statement, new private `execute::refresh_schema` steps the bounded static `SELECT 1 FROM main.sqlite_schema LIMIT 0`, then clears SQLx's statement cache. The regression holds the exact warmed connection while the other pool slot executes ALTER, then checks both reused `SELECT *` and newly quoted projections for complete columns, correct names and preserved values.
- The deliberate cost is native schema refresh plus statement-cache invalidation on each statement; no new global cache, shared mutex or schema-version state is introduced. This proves sequential explicit DDL followed by queries on a previously warmed connection. It is not a protocol for concurrent external/online schema mutation between refresh, preparation and execution; coordinated online migration remains outside this slice.
- SQLite documents automatic reprepare on schema changes in [the prepare contract](https://www.sqlite.org/c3ref/prepare.html), and result-name lifetime/naming in [the column-name contract](https://www.sqlite.org/c3ref/column_name.html). The SQLx-specific behavior and fix are established by the local native regression, not inferred from those documents alone.
- **Controlled eager-open proof:** public `open` delegates to private `open_pool`, which still applies the same finite policy and validates options before connecting. Tests can supply SQLx's safe `after_connect` hook to pause an initialized native connection before pool publication. There is no public callback/hook API or altered default policy.
- Existing lease retirement, rollback-only state, source-preserving errors, finite admission/pool/busy limits, no automatic retry and unknown commit-outcome semantics are preserved. No first-party unsafe code or lifetime adaptation was added.

## Native behavior and inventory mapping

The [integration inventory](slice-4-design-tests-integration.md) Now rows map to these passing file-backed tests. No upstream TypeScript suite was run or copied as production implementation.

| Inventory row | Passing native evidence |
| --- | --- |
| inquire-sqlite3 #1, create profile table | `m2::ddl::supported`: actual Create plan with typed fields, auto-increment key, false/zero/empty/NULL and SQL-looking text defaults; real compound FK definition/actions. |
| #2, safely replay creation with indexes | Same scenario renders a real three-statement table/unique-index/index plan, executes it twice under explicit idempotence and compares sqlite_master snapshots. |
| #3, alter profile table | Same scenario executes ADD with default, DROP, explicit column RENAME and index ADD/DROP; data survives. `cached_schema` proves correct results on an old connection. Rename/truncate/drop table commands also run; truncate does not reset the sequence. |
| #4, flatten data | M1 `values::binding` plus M2 typed Insert/default/JSON tests prove scalar encoding and stable bind order without mutating input. Generic schema-owned flattening remains S4-F. |
| #5, insert profile values | `m2::queries::builders`: bound bulk Insert/RETURNING and explicit whole-column DEFAULT. Hostile text survives and cloned builder data stays independent. |
| #6, update profile values | Same scenario: typed Update with grouped bounds/RETURNING and exact affected count. `queries::transactions` proves explicit mutable-guard composition. |
| #7, fetch all profiles | Select predicates, IN/NOT, distinct, explicit order, limit/offset/zero limit, LEFT JOIN, duplicate names/ordinals, NULL side and hostile explicit aliases execute natively. |
| #8, JSON object | `m2::json::paths`: exact typed object predicate, hostile bound value/path, literal numeric-looking/dotted/quoted/backslash/control keys and owned extracted values. |
| #9, JSON array | Same scenario: direct scalar membership and nonmatches for descendants/object members/scalar targets. |
| #10, JSON nested array | Same scenario: typed object keys plus explicit array index select an immediate nested array. |
| #11, JSON twice-nested array | Same scenario: further object nesting and array index select the intended direct array. |
| #12, delete profile | Typed Delete/RETURNING executes; inherited M1 WITH/DELETE RETURNING, zero-match and persisted remaining-data tests rerun. |
| stackpress-sql helpers #1/#2 Now encoding aspects | M1 native scalar/date-text cases and `m2::json::types`: explicit JSON encoding; SQL NULL vs missing/JSON null, false/true, integer/real/text stay distinct. Invalid JSON TEXT stays text but JSON-function use fails with a native source. No silent coercion or epoch fallback. |

Generated store/action/event templates, schema-specific flattening, revision/destructive policy and SQLite table reconstruction retain S4-F/S4-M boundaries. Other adapter/network/framework inventory rows remain in their named later scopes.

## Transactions, cancellation and failure matrix

All real async scenarios run in their own current-thread child under an independent **10-second process watchdog** and **8-second cooperative deadline**. Ready/go, oneshot, Notify or barrier state establishes boundaries; sleeps are used only to observe timer responsiveness, not to guess a cancellation boundary. Compiler proofs have a separate 180-second parent deadline. The inherited parked-child negative control is still killed/reaped after 150 ms.

| Transition/risk | Passing evidence and observed contract |
| --- | --- |
| Before open is polled | `m2::cancellation::open`: dropping the unpolled future creates no file. |
| Native connection initialized, before pool publication | Same scenario repeats three controlled cancellations in private after_connect. Each abandoned dirty native initialization is dropped, a fresh writer succeeds, and no uncommitted data remains. Directory/open and inherited missing/read-only/create failures retain sources without fallback. No claim of cancellation at every individual SQLite initialization instruction. |
| Waiting for pool acquisition | Inherited M0 `pool`: an actually queued future is dropped, admission recovers, held size-one lease still produces finite timeout; M1 close wakes a controlled queued waiter with Closed. |
| Acquired, before command enqueue | `m2::cancellation::prepare`: drop the actual production Lease immediately after native acquire; await all admission slots and use a fresh clean session. This directly tests RAII ownership, not a public async hook at that exact instant. |
| BEGIN queued, before acknowledgement | Inherited `transitions::cancellation`: native progress gate, repeated task abort, retire connection and successfully begin/roll back again. Failed native BEGIN is separately interrupted with a source; no blind unrelated rollback is issued. |
| Active ordinary write | Inherited `admission_cleanup`: pause inside native row mutation, abort, observe admission still occupied until worker-close acknowledgement. The already-applied autocommit write may persist; no automatic replay. |
| Active transaction write / idle active guard | Inherited native update gate plus drop/task-abort/application-unwind tests: rollback-only after cancelled statement, uncommitted data absent on fresh lease and reopen. |
| Active row streaming | `m2::cancellation::fetch`: repeat ordinary streaming cancellation at native progress, retain admission through cleanup, recover cleanly. Cancel streaming inside a dirty transaction, reject subsequent queries as RollbackOnly, explicitly roll back and observe no inserted row. |
| Partial DDL plan | `m2::cancellation::ddl`: abort after the first CREATE is acknowledged and before the next plan statement; no table/index survives and no COMMIT was attempted. |
| Before consuming completion future is polled | `m2::queries::transactions`: drop both unpolled commit and rollback futures; captured sessions still roll back their changes. |
| COMMIT queued, acknowledgement lost | Inherited native commit gate plus new DDL commit gate: task abort loses observation; this controlled gate commits the complete DDL batch, survives reopen and is never replayed. General outcome remains unknown. |
| ROLLBACK queued, acknowledgement lost | `m2::cancellation::rollback`: native rollback hook, task abort, admission held until cleanup, then reopen proves no uncommitted rows. |
| Prepare/statement/decode/limit error | Native syntax and interrupted pre-execution work retain sources; inherited M1 invalid UTF-8/limits and M0 arity/native failures rerun. Dirty transactions become rollback-only and clean up. |
| Mid-plan native failure | `m2::ddl::atomic`: existing index name makes later CREATE INDEX fail; unique-index failure follows a successful ADD; missing-index failure follows successful RENAME. Every batch restores identical schema/data, including after reopen. |
| Unsupported DDL before effects | `m2::ddl::unsupported`: valid earlier ADD followed by unsupported type/nullability/primary/FK operation rejects rendering of the whole batch. Changed semantics after explicit rename remain unsupported. Unsigned/comment/decimal create annotations reject rather than disappear. Snapshots/data stay identical. |
| Native commit/rollback failure | Inherited deferred-FK commit failure, failed BEGIN and interrupted rollback preserve sources and retire connections. New `native_failures` covers native commit veto and INSERT OR ROLLBACK ending a transaction before explicit cleanup; both primary and secondary rollback sources remain available. |
| Affinity / runtime responsiveness | Two-connection builder test sees its own changes while outside reads see committed data. Inherited costly-query timer proof and new real writer-contention timer proof run on current-thread Tokio. Busy is finite/typed, and a fresh transaction succeeds afterward. |
| Closed worker / panic | Source-preserving WorkerCrashed-to-Closed mapper seam is synthetic; real native interruptions and pool closure are exercised. No unsafe forced worker crash or power-loss fault was introduced. Application unwind, runtime teardown and nonblocking outside-runtime Drop remain inherited real-process proofs. No first-party callback mutex or poison-recovery path exists. |
| Close/runtime shutdown | Inherited M0/M1 close rejects new work, wakes waiters, waits on an owned transaction, supports cancellation/re-await and reaches zero pool size. Runtime teardown runs beneath its own process watchdog. |

## Bounded milestone reviews

These are the standing in-milestone reviews; the coordinator's later slice-wide rounds/docs gate remains unreleased.

1. **Logic and branch risk:** initial native tests exposed stale metadata after ALTER. Added a controlled old-connection regression and verified that removing refresh/invalidation makes it fail beneath the watchdog; restoring the fix passes. Expanded automatic rollback/commit-veto, unsupported rename semantics and cleanup admission assertions. Adapter all-target tests passed (`round-one.log`).
2. **Responsibility and ownership:** extracted `refresh_schema` from execution so native schema/cache policy has one named owner; retained the existing execute/collect/Lease/options boundaries. `open_pool` supplies the smallest private safe initialization seam and retains public defaults. Reused the existing native hook harness for rollback rather than adding a production hook API. All-target tests passed (`round-two.log`).
3. **Consistency/Rustdoc/comments and safety:** documented reprepare cost, native schema intent and private pool ownership; added meaningful hook/transition comments. Reviewed future-drop order, exact borrowed guard lifetimes, cleanup acknowledgements, static vs audited dynamic SQL, source chains, finite deadlines and feature/dependency boundaries. Added before-enqueue completion and current-thread contention evidence. All-target tests passed (`round-three.log`), followed by final checks below. No additional first-party unsafe/lock/panic recovery mechanism was needed.

## Final checks and measured coverage

All checks ran after final source changes on the native target above. The reused regular target is `target/`; instrumentation reused `target/llvm-cov-target/`.

```sh
cargo fmt --all -- --check
cargo test -p stackpress-sql-sqlite --all-targets --locked --offline
cargo test -p stackpress-sql-sqlite --doc --locked --offline
cargo clippy -p stackpress-sql-sqlite --all-targets --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-sql-sqlite --no-deps --locked --offline
cargo +1.96.0 check -p stackpress-sql-sqlite --all-targets --locked --offline
cargo check -p stackpress --no-default-features --features sql-sqlite --locked --offline
cargo build -p stackpress-sql-sqlite --example m0-host --locked --offline
```

**Passed:** 10 test/parent entries and two doctests, fmt, warning-clean Clippy/Rustdoc, explicit Rust 1.96 and facade-consumer checks. Six ignored child entrypoints are actually invoked by parents: 10 inherited M0 native scenarios, nine M1 scenarios, **15 M2 scenarios**, seven compiler-fail fixtures, runtime teardown and the watchdog negative control. The unchanged m0-host binary prints `profile-name: Ada` under a separate 10-second runtime deadline. No workspace-wide test pass is claimed for this isolated lane.

Adapter-only measurement used cargo-llvm-cov **0.8.6**, after cleaning, without dependency/core sources:

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-sql-sqlite --all-targets --locked --offline --no-report
cargo llvm-cov report -p stackpress-sql-sqlite --json --summary-only --fail-under-lines 70 \
  --ignore-filename-regex '(/tests/|/src/tests\.rs)' \
  --output-path target/s4-lane-b-m2-evidence/coverage.json
```

| Adapter production source | Covered/executable lines | Line coverage |
| --- | --- | --- |
| collect.rs | 62/66 | 93.94% |
| execute.rs | 100/113 | 88.50% |
| lease.rs | 37/38 | 97.37% |
| lib.rs | 115/115 | 100% |
| options.rs | 55/55 | 100% |
| **Library, package alone** | **369/387** | **95.35%, passes 70%** |

Only test-only source is excluded. No production source is excluded; every reported source is under the adapter. The primary report was saved before running the example, so no core, other lane or proof-app execution raises this figure. The unchanged 18 uncovered lines are defensive/current-driver metadata/type/count/empty-lease branches, not new untested M2 execution code. No core coverage figure is claimed.

Then `cargo llvm-cov run -p stackpress-sql-sqlite --example m0-host --no-report --locked --offline` ran beneath an outer process watchdog with a 10-second runtime deadline, excluding compilation. A subsequent report merged its profile before explicit LLVM export of the example binary/source (the ordinary report omits examples). Separate example coverage: **64/65 lines, 98.46%**. It is not used for the library gate.

Ignored local evidence in `target/s4-lane-b-m2-evidence/`: baseline and three review logs, final check logs/commands, stale-schema negative control, primary package JSON, separate example export/summary and KB baseline. `git diff --check` and the forbidden-path write-set audit passed. Stale M0/M1 lane-owned coverage build directories were removed while their evidence logs were retained; no other worktree target was touched. The disposable current instrumentation target is removed after report extraction; the regular target remains for the sequenced integration follow-up.

## Stop and residual limits

- Ready for coordinator M2 re-verification of this lane. **Public execute_plan native integration remains pending on merge by explicit coordinator instruction**, with no core edit or new decision requested here. TableDefinition/SchemaDiff integration, M3 host app and full docs remain later work.
- Native scalar/retained-output, optional rowid, cancellation/unknown-commit, best-effort Drop outside a live runtime and native VM/single-cell-allocation limits from M0/M1 remain unchanged. This is not online-migration serialization, immediate native interruption, exactly-once execution, power-loss/durability certification, or a safely forced worker-process-crash test.
- Native macOS/aarch64 only; no network execution, other OS/architecture, WASM/mobile, network filesystem or external database service was verified.
- KB baseline before receipt: **146 existing errors and 20 warnings**. No existing KB was repaired. Post-write comparison is reported in the handoff; passing code checks does not make that validator green.

Stop here without commit or further-stage implementation. Acceptance criteria: none; coordinator re-verification remains outstanding.

## Public execute_plan integration follow-up at 90aa05e

This coordinator-released follow-up starts from clean HEAD `90aa05e531c1ead9b73f44b288b338ee73c2744d` and **resolves the public execute_plan pending-on-merge item above**. Earlier evidence remains historical; the other scope/platform limits remain. Only adapter test files and this appended section change; production/core/shared code, manifests and dependencies are untouched. No commit or agent messages.

Removed the temporary `m2::plan` executor. All existing native DDL success/failure/cache tests and the commit-cancellation proof now call `Engine::execute_plan` directly. Three new watchdog scenarios exercise validation, active native cancellation and caller-owned transaction composition:

| Requested proof | Passing real-file evidence |
| --- | --- |
| Successful multi-statement plan | `m2::ddl::supported`: public three-statement Create/table/index plan, replay, constraints/defaults and subsequent CRUD. |
| Statement N failure / original error | `m2::ddl::atomic`: public failing CREATE/ALTER batches restore schema/data on rollback and reopen; downcast source retains native SQLx Database error code `1` and exact `index occupied already exists`, with no cleanup-error replacement. |
| Another dialect before effects | `m2::plans::validation`: real PostgreSQL and MySQL plans return Unsupported from both public methods while the sole SQLite lease is held. The caller guard remains usable; schema/data stay identical. |
| Unsupported SQLite DDL | Existing `m2::ddl::unsupported` still rejects the whole builder at rendering, including a valid earlier ADD; no executable plan escapes and schema/data stay identical. |
| Mid-plan cancellation | `m2::plans::cancellation`: public Engine batch pauses in a safe native text-collation comparison during statement two's index creation, after statement one's ADD. Aborting restores identical schema/data, including after reopen. The one-shot test-only mutex is released before callback signaling/waiting; the native wait has a three-second deadline. |
| Caller-owned transaction | `m2::plans::caller_transaction`: public Transaction batch preserves the caller's prior writes, permits later inserts/queries, hides uncommitted schema/data from outside reads, and leaves rollback/commit explicit. Both completion paths and persisted commit are checked. |

All scenarios retain the independent 10-second process / 8-second cooperative watchdog. M2 now has **18 native scenarios**. Scoped reviews covered (1) exact statement/error/cancellation boundaries, (2) removing the temporary executor and keeping only test-local synchronization, and (3) Rustdoc/comments/ownership safety. Each review was test-verified; no production refactor or further scope was warranted.

Final fmt, adapter all-target tests, two doctests, Clippy `-D warnings`, Rustdoc `-D warnings` and `cargo +1.96.0 check -p stackpress-sql-sqlite --all-targets --locked --offline` all passed. Test result: **10 passed parent/unit entries, six deliberately ignored child entrypoints launched by their parents**. Commands for tests/docs/Clippy/fmt match the earlier final-check block; no new workspace-wide or network execution claim is made.

Fresh adapter-only coverage reran the earlier clean/test/report commands with output under `target/s4-lane-b-m2-public-plans-evidence/`: **369/387 production lines, 95.35%**, package alone, no core/dependency/example execution credited. Only test-only source is excluded; all five production files and their per-file counts remain identical to the table above. The unchanged example was compile-checked by all-targets; its prior separately executed coverage is historical. Reused the existing targets, then removed completed instrumentation artifacts; retained check/review/coverage logs and JSON. `git diff --check` and the forbidden-path audit passed. KB baseline remains 146 existing errors / 20 warnings; post-write comparison is reported in the handoff.

Stop for coordinator re-verification; the public-plan integration follow-up has no remaining implementation item.
