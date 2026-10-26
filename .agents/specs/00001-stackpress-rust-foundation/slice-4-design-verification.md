# Slice 4 risks, milestones and shared-file plan

Companion to [decisions](slice-4-design-decisions.md). Load for the implementer's open-ended risk inventory, proving tests, about two implementation lanes, G23 change requests and release gates. No production code is authorized by this design checkpoint.

## Risk register

Each prevention needs an observable/negative-control proof; coverage percentages alone do not prove these contracts. New material findings extend the register.

| ID / risk | Prevention | Required proving test |
| --- | --- | --- |
| R01 Value injection | Bound typed values; one audited dynamic-SQL conversion point. | Quotes, comments, semicolons, SQL-looking Unicode and malicious values survive exactly; table/data intact. |
| R02 Identifier/default/path injection | Typed identifier segments/operator/action/default enums; path encoding; explicit trusted escape. | Embedded dialect quotes/dots/controls, JSON path keys and default strings cannot append executable SQL; invalid forms error before effects. |
| R03 Bind arity/token confusion | Native prepared metadata plus typed bind nodes; no global replacement. | Missing/excess/zero binds; `?` in literals/comments; native named/sparse markers rejected; verify no NULL substitution or parameter drift. |
| R04 Blocking runtime | SQLx dedicated SQLite worker; no direct sync native work in handlers. | Current-thread runtime timer progresses while a real costly SQLite query runs; repeat with contention. |
| R05 Unsafe Send/lifetimes | Send futures, Database Send+Sync, exclusive transaction borrow, no unsafe adaptation. | Real named host handler compiles; non-Send service/future, escaping borrow, concurrent mutable queries and transaction clone fail compilation. |
| R06 Pool exhaustion/nested deadlock | Finite admission/acquire/busy bounds; release before dispatch; explicit transaction composition. | Real nested handler holds size-one lease, child acquisition errors promptly; barrier-controlled concurrent roots recover after release. Negative control without release hits watchdog. |
| R07 Connection leaks on cancellation | RAII lease ownership; driver recovery or discard, finite command buffers. | Cancel waiting/acquired/begin/active fetch at controlled boundaries; observe subsequent acquire and clean transaction state, no permit growth. |
| R08 Uncommitted write leakage | Guard owns one native transaction; rollback-on-drop, retire uncertain connection. | Drop/abort/panic after INSERT, then query from a new lease and reopen file: no uncommitted rows. |
| R09 Commit ambiguity and retry duplication | Unknown outcome documented; consuming commit, no automatic retry. | Cancel after commit enqueue and before acknowledgement with a controlled seam; record committed/unknown outcome and never replay. |
| R10 Rollback/begin failure hides cause | Preserve primary + secondary cleanup errors; retire connection. | Inject failed begin/failed rollback/closed worker; source chain retained, no blind unrelated rollback or clean reuse. |
| R11 Driver panic/poison | No first-party shared mutex over callbacks; native worker errors are failures. | Application unwind rollback; worker failure seam returns closed error; any added mutex gets explicit poison tests, never unwrap recovery. |
| R12 Clone aliases live state | Non-Clone Engine/transaction; explicit Arc handle; owned query/value clones. | Compile-fail cloning state; Arc sharing documented; cloned query mutation leaves original unchanged. |
| R13 Wrong transaction executor | Explicit &mut guard, no captured pool in actions. | Two-connection transaction sees its own writes while outside query doesn't; rollback affects only transaction. Store-like helper receives guard. |
| R14 Shared lastId/metadata race | Statement-local outcomes and RETURNING. | Concurrent inserts return their own IDs/counts; UPDATE/DELETE and later SELECT cannot reuse stale insert metadata. |
| R15 SQL textual row detection | Native prepared row metadata. | Leading comments, WITH, UPDATE/DELETE/INSERT RETURNING, PRAGMA and zero-row writes produce correct rows/affected fields. |
| R16 Dialect divergence | Three capability tables; separate snapshots and real SQLite integration. | Every builder/dialect family including unsupported returns/cascade/joins/type/DDL; never label network SQL execution passed from snapshots. |
| R17 Partial DDL batch | Validate entire plan before effects; one lease/transaction for SQLite. | Unsupported later operation leaves earlier schema unchanged; failing indexed CREATE/ALTER rolls whole batch back. |
| R18 Schema diff/rename destroys data | Typed equality, explicit rename reconciliation, no inferred meaning. | Ordered compound constraints, no-op diff, changed field after rename, collisions/missing halves, actual rename preserves rows. |
| R19 SQLite unsupported alterations | Reject before effects; reconstruction S4-M. | Change type/nullability/FK/primary fails with Unsupported and identical schema/data; supported ADD/DROP/RENAME/index runs. |
| R20 NULL/missing/default/false loss | Typed input/default states and fallible decode. | Missing vs NULL, false/0/empty default, nullable bool, i64 bounds, nonfinite f64, blob and JSON null roundtrip. |
| R21 JSON semantics drift | Typed key/index path, direct scalar membership, explicit decoding. | Nested objects/arrays/indices, key `01`, quotes, absent/null, bool/int/text distinction, membership nonmatches and invalid JSON handling. |
| R22 Result memory/CPU amplification | Finite retained-output/SQL/bind/in-flight budgets; stream collection internally. | Boundary + one row/bind/byte over; stop retaining and return Limit; record native VM/single-cell allocation limitation. |
| R23 Duplicate columns/alias collisions | Ordered metadata, ambiguous named reads error. | Join duplicate names, explicit aliases, long/quoted aliases and schema collision checks for future generator. |
| R24 Test cross-contamination | One tempfile-backed DB per test; owned handles closed before directory removal. | Parallel tests use same table names in distinct files; one failure cannot alter another; reopen proves persisted state. |
| R25 In-memory database disappears | Explicit special mode, no general persistence claim; file-backed tests. | Replacement/dropped connection recreates independent in-memory state; production helper rejects incompatible pooling options. |
| R26 Close/runtime teardown hangs | Explicit async close with caller deadline; no blocking Drop/join. | Close wakes waiters, waits on held lease, cancellation/re-await works; child runtime teardown exits under watchdog. |
| R27 Configuration/disclosure | Typed serde section, validation before open, app-controlled path, sanitized errors. | Unknown/invalid options fail startup; no request path override; sentinel secrets absent from public envelope/default logs. |
| R28 Accidental exposure/destruction | SQL service registration separate from capability/routes; no generic query/migrate endpoint. | Missing SQL capability remains no-listener/404; only explicit app operation reachable via buffered HTTP and CLI. |
| R29 Driver leaks into UI/lib/defaults | Optional facade edge; direct library dependencies; separate host example. | Normal-edge trees for lib/server/default facade/contracts exclude SQLx/SQLite/Tokio pool; opt-in tree contains only chosen backend. |
| R30 Unsupported platform claim | Native target receipt; compiled-out backend boundary. | Installed target only; report unrun Linux/Windows/WASM/mobile/network FS separately, no toolchain installation. |
| R31 Resource/open cancellation | Eager owned open; no cached lazy factory race. | Cancel during connect/init, failing path/read-only/create permissions, repeated open attempts; no partial host or leaked worker/permit. |
| R32 Recovery/durability overclaim | Explicit WAL/synchronous policy; no exactly-once/power-loss certificate. | File reopen/transaction recovery tests; process-kill only disposable roots; list filesystem/power-loss guarantees as unproved. |
| R33 Generated contract/semantics leakage | Keep S4-F explicit; public DTO independent of driver. | Later generated store compiles with actual adapter + pure contracts consumer; unsupported metadata errors rather than silently ignored. |
| R34 Stale/current source confusion | Current branch and fresh disposable target; locked production builds after coordinator integration. | Record HEAD, manifests/feature tree and exact commands; no other trial source/target or upstream build. |
| R35 False success/misleading docs | Explicit error/results and real doctests after M0. | Zero rows is not a swallowed driver error; host G12 error envelope/source chain remains correct; proposed snippets become compiled real examples. |

## M0: riskiest proof and mandatory stop

After owner choices, one implementer temporarily owns scaffold; coordinator sequences shared manifests/facade first. Create only real core/SQLite adapter code and one native proof app. Use actual production contracts, not the disposable fake Session trait as a substitute.

1. Prove the Engine/Database/Transaction signatures, named Send handler/service registration, exclusive borrowing across await, consuming completion and compile-fail alias/Clone/escape/non-Send cases. No lib/server changes unless compiler evidence requires a reviewed request.
2. Execute a bound query and a transactional INSERT on a fresh SQLite file; prove commit, explicit/drop/error/panic/abort rollback and outside-query isolation. Force cancellation around begin, active query and commit acknowledgement; retire uncertain sessions and state the unknown-commit contract.
3. Prove actual nested/concurrent host dispatch with a size-one pool fails finitely when reacquiring under a held transaction, while explicit &mut transaction composition succeeds. Test pool waiter cancellation, close/resume and subsequent use after cleanup. Parent watchdog must catch a deliberately stuck negative control.
4. Prepare dynamic generated SQL through the audited SQLx marker, validate native arity, show an unbound argument cannot silently become NULL, and demonstrate current-thread responsiveness. Check fresh file persistence and isolated tests; no external service/listener.

If a proposed cancellation/connection-retirement guarantee cannot be proved through the real driver contract, stop with the counterexample and revise the design. Do not add unsafe handles or hide uncertainty behind a successful query snapshot. Record exact compiler/target/features/results and the residual begin/commit fault-injection limits. **Stop for coordinator re-verification.**

## Milestones after M0

The coordinator releases each after the prior contract passes; independent lanes can work concurrently within a milestone.

| Gate | Lane A: SQL data/rendering | Lane B: SQLite execution/host proof | Completion signal |
| --- | --- | --- | --- |
| M1 | All six builders, Expr/JSON paths, SqlValue/row metadata, identifier/default validation, full core tests. | Prepared binding/arity, row collection/limits, typed errors/native outcomes, pool/options/close. | Real CRUD/RETURNING + hostile inputs, no swallowed errors or retained-output overrun. |
| M2 | SQLite/Postgres/MySQL renderers, explicit capability matrix, TableDefinition/SchemaDiff/rename reconciliation. | Actual supported SQLite DDL/JSON/query plans, transactions and complete cancellation/failure matrix. | All inventory Now rows mapped to passing tests; unsupported DDL has no effects; network execution remains deferred. |
| M3 | API/Rustdoc/doctests, standalone examples and inspection DX; inventory reconciliation. | New `examples/sql-profile` app-owned database/plugin/setup + direct/CLI/buffered HTTP proof, dependency/feature checks. | Clear first success, original prefix semantics if parity selected, independent DTOs, per-crate coverage. |
| Before rounds | Joint review with coordinator. | Joint review with coordinator. | Stop with tests, per-crate measured coverage, public API/budget deviations and remaining risks. |
| Rounds 1–4 | Complexity; responsibility/abstraction; consistency/Rustdoc/comments; dedicated safety/soundness. | Same scopes against pooling/state/async/cancellation code. | Each round test-verified; repeat targeted rounds while material findings remain. No ceremonial edits. |
| Closeout | Scoped final receipt. | Scoped final receipt. | Coordinator re-runs checks and reviews breadth/DX. Full crate docs follow as a separately released task. |

Recommended two lane ownership: A owns `crates/sql/sql/**`; B owns `crates/sql/sql-sqlite/**` and the new proof example. A owns core public traits after M0; B requests trait changes before editing them. Each has a unique spec receipt filename. The coordinator owns task status/verification and all cross-slice shared-file integration. Do not spawn agents at this checkpoint; this is a proposal for the coordinator's next release.

## G23 shared-crate requests

Every relevant shared component is listed, including explicit zero-change conclusions.

| Component | Needed change under recommendation | Justification / sequence |
| --- | --- | --- |
| `crates/lib` | **None** | Existing HandlerFuture/Send handler, errors, owned request/response and borrowed host context suffice. SQL core must not depend on lib merely for an error alias. |
| `crates/server` including HTTP | **None** | config_section/on_config/register Arc service/plugin_with/service/delegate and existing HTTP adapter already supply integration. Current lack of async global shutdown is a documented limit, not an implicit change request. |
| `tools/cli` | **None** | Existing main_with/run/typed config and app-owned binary execute capabilities. Awaited close can use app-owned entry composition; no new query argv parser. |
| `crates/idea/*` | **None** | Syntax and transform/output APIs exist; general schema interpretation deferred S4-F. No SQL-only metadata meanings inserted into parser/transformer. |
| Existing capability-host/generated-profile | **None** | Preserve previous proofs and parallel documentation; add a distinct SQL example after scope acceptance. If owner wants to replace them, coordinator must issue that scoped change. |
| Root Cargo.toml | Add actual SQL/core/SQLite member paths, optional facade deps/features and new example member. | Coordinator sole owner; sequence with Slice 5 before lanes use --locked. Core feature enables no backend. |
| Root Cargo.lock | Resolve only approved dependencies/features. | Coordinator integrates once and audits tree; no lane mutates shared lock concurrently. SQLx/rusqlite conflicting libsqlite3-sys links are avoided by choosing one production driver family. |
| Root src/lib.rs | Opt-in re-exports `sql` and `sql_sqlite`, documented feature/target boundary. | Coordinator integrates alongside feature declarations; default root still only foundation. |
| framework-schema/framework-sql | No files/manifests now, pending S4-D02. | S4-F design and owner release are required for real semantic integration; no placeholder crate. |

Any newly discovered shared change goes to the coordinator with compiler/test evidence and justification; lanes stop at that edit. Shared target cleanup is also coordinated. Do not delete another agent's build while cleaning a fresh effort; use independent target directories when appropriate and record them.

## Tests, fail-fast rules and quality gates

Concurrency/cancellation/poison/kill/runtime-shutdown proofs run in child processes with explicit ready/go barriers. Parent uses a monotonic 5–10 second deadline, kills/reaps on timeout and preserves stderr. Compile tests get a separate finite compilation timeout. A Tokio timeout inside a current-thread executor cannot prove freedom from synchronous blocking; the outer watchdog is mandatory. Avoid arbitrary sleeps as race synchronization. Use controlled connection/driver seams for transition fault injection, then rerun against the real native adapter.

Use independent tempfile files (plus WAL/SHM lifetime) and drop/close handles before root cleanup. Do not use a shared database filename, change process env/cwd, need a database server or install toolchains/system packages. Integration proof setup explicitly creates/installs/seeds only its disposable database, never a user's path. Negative controls prove the watchdog/test harness detects the failure being claimed.

Development coverage is at least **70% executable lines per changed first-party crate/valid feature scope**, with measured numerator/denominator, compiler/target/tool/features and justified exclusions. Generated executed code/proof app is not silently excluded by cargo-llvm-cov's example defaults. No workspace average substitutes for an under-covered adapter; important cancellation/DDL error branches are reviewed even over 70%. Publication later requires 90% plus separate authorization.

After implementation/rounds run fmt, meaningful unit/contract/integration/compile-fail/doctests, warning-clean Clippy/Rustdoc and per-crate llvm-cov. Coordinator re-verification uses the workflow's locked workspace checks plus enabled/disabled facade/example combinations and normal-edge dependency trees. Record actual commands rather than claim this design ran repository quality gates. This stage runs disposable probes and Agent Workspace validation only.

No commit, push, publishing, deployment, KB promotion or listener follows a passing test. Full docs remain the explicit follow-up in task 00007.
