# Slice 4 driver assessment and disposable evidence

Companion to [decisions](slice-4-design-decisions.md). Load for exact candidate selection, async/Send/MSRV/license claims, probe reproduction and residual risk. Assessed 2026-10-25 using installed rustc 1.96.0 (`ac68faa20 2026-05-25`), cargo 1.96.0 (`30a34c682`), native `aarch64-apple-darwin`. Current branch is `trial/3`; initial reviewed HEAD was `d35066a5bded1b1eec74fc14472436c2455cd441`. No other trial branch/target was inspected.

## Candidate comparison

The exact selected artifacts and resolved dependencies, rather than unpinned latest documentation, are the basis of the compatibility result. All three candidates can execute SQLite tests without an external service. None proves mobile/browser support.

| Candidate | Async/ownership contract | MSRV 1.96 / license | Assessment |
| --- | --- | --- | --- |
| SQLx **0.9.0**, defaults off, `runtime-tokio,sqlite` | Native SQLite calls use per-connection worker threads; pool is Send+Sync, transactions have borrowing Send futures and own a lease. Async close and rollback-on-drop exist. Native queries still may finish after caller cancellation. | sqlx/sqlx-core/sqlx-sqlite declare Rust **1.94.0**, MIT OR Apache-2.0; compiled/tested with 1.96. Resolved libsqlite3-sys **0.37.0**, MIT; bundled SQLite is public-domain source. Existing C toolchain sufficed. | **Recommend**: existing pool/RAII/state mechanisms align with async host and later network driver direction. First-party builder/normalized-value/error/dialect/cancellation policy still required. |
| tokio-rusqlite **0.8.0**, explicit `bundled` | Send+Sync cloneable handle dispatches `'static + Send` synchronous closures onto one worker. `call` result/error must be Send+'static; no borrowing async transaction closure across awaits. Wrapper source uses an unbounded crossbeam queue and direct callback invocation. | No rust-version in resolved manifest; **verified builds/tests on 1.96**, not an inferred lower MSRV. MIT. Resolved rusqlite **0.40.2** and libsqlite3-sys **0.38.2**. | Viable for finite synchronous transaction closures. Async transaction guards/pooling/admission would need substantial custom ownership/protocol work. Clone shares the live worker. Cancelling caller does not cancel its queued closure, proved below. |
| rusqlite **0.40.2**, via that pinned wrapper's re-export | Direct synchronous calls; Connection is **Send, not Sync**. Borrowing transaction guard works synchronously. Using directly on an async executor blocks it; a dedicated worker or correctly owned blocking task is application work. | No rust-version in resolved manifest; build and actual transactional execution verified on **1.96**. MIT; bundled SQLite separate public-domain terms. Do not substitute the current master MSRV or latest 0.41 docs as this artifact's declaration. | Small synchronous core, but a custom worker/pool/async transaction protocol duplicates the riskiest layer. Not recommended for this first async-host adapter. |

Versions are intentionally fixed for this assessment. SQLx 0.8.6 documentation was consulted as background, but no compatibility/test claim here uses an 0.8.6 build; current chosen SQLx 0.9.0 has a materially different dynamic-SQL trust marker. rusqlite 0.41 was not tested; the comparison uses the version resolved by tokio-rusqlite 0.8.0. Do not combine both driver families in one production Cargo graph: their different `links=sqlite3` dependency versions can conflict.

## Primary sources and exact local evidence

- [SQLx 0.9 SQLite connection docs](https://docs.rs/sqlx/0.9.0/sqlx/struct.SqliteConnection.html): worker-thread API and explicit close. Corresponding downloaded `sqlx-sqlite-0.9.0/src/connection/{mod,worker}.rs` was read for Begin/Commit/Rollback cleanup.
- [SQLx transaction docs](https://docs.rs/sqlx/0.9.0/sqlx/struct.Transaction.html) and downloaded `sqlx-core-0.9.0/src/transaction.rs`: consuming completion and rollback-on-drop. Source `sqlx-core-0.9.0/src/pool/{mod,connection}.rs` documents acquisition cancellation/connection discard, close waiting and no async Drop guarantee.
- Downloaded `sqlx-core-0.9.0/src/sql_str.rs` and `executor.rs`: generated dynamic SQL requires `AssertSqlSafe`, a safe explicit audit marker; `describe` is gated by offline/macros, while ordinary prepared Statement exposes parameter metadata without those features.
- [tokio-rusqlite 0.8 call signature](https://docs.rs/tokio-rusqlite/0.8.0/tokio_rusqlite/struct.Connection.html) and downloaded `src/lib.rs`: worker/Send bounds, closure dispatch and unbounded channel.
- [rusqlite 0.40.2 Connection docs](https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html): Send/!Sync and synchronous ownership. [Rusqlite license/bundling description](https://github.com/rusqlite/rusqlite/blob/master/README.md) distinguishes MIT binding code and bundled SQLite terms; exact candidate metadata was also inspected.

Sources were accessed 2026-10-25. Cargo's downloaded normalized manifests and target-filtered metadata supply exact version/rust_version/license/features. This is an engineering dependency assessment, not a complete publication/license audit; publication is not authorized.

## Isolation and reproduction

Everything executable lives under **`/private/tmp/stackpress-s4-probe/`**:

- `sqlx/{Cargo.toml,Cargo.lock,src/lib.rs,examples/{aliasing,clone}.rs,target/}`;
- `rusqlite/{Cargo.toml,Cargo.lock,src/lib.rs,examples/non_sync.rs,target/}`;
- `evidence/`: captured test/expected-failure logs, native-filtered metadata and normal-edge trees;
- `results.json`: five verification command outcomes;
- `design/`: staged design Markdown before the separately approved write into this spec.

The two independent disposable Cargo workspaces avoid sqlite3 link conflicts and share no target output with the repository. Initially neither had a target; no stale artifact reuse, root cargo clean or shared coverage cleanup was necessary. The SQLx probe uses path dependencies on **current** lib/server only, compiled into the temporary target. No workspace root manifest/lockfile/production source was edited by the assessment. No upstream TypeScript source was changed or built, and no listener, toolchain/system-package/database-server install occurred.

Initial registry access failed under the network sandbox with DNS resolution errors. Explicit permission was requested and granted for cargo info/test fetching; crates downloaded into Cargo's normal cache. Subsequent reproduction runs used offline locked dependencies. An unrestricted metadata query initially needed un-fetched foreign-target packages; target-filtered metadata below completed offline. This is not evidence of a missing native dependency or failed native test.

Commands, from this checkout (each negative example must fail with the stated compiler diagnostic):

```sh
rustc --version
cargo --version
cargo info sqlx@0.9.0
cargo test --manifest-path /private/tmp/stackpress-s4-probe/sqlx/Cargo.toml --lib --offline --locked
cargo test --manifest-path /private/tmp/stackpress-s4-probe/rusqlite/Cargo.toml --lib --offline --locked
cargo check --manifest-path /private/tmp/stackpress-s4-probe/sqlx/Cargo.toml --example aliasing --offline --locked
cargo check --manifest-path /private/tmp/stackpress-s4-probe/sqlx/Cargo.toml --example clone --offline --locked
cargo check --manifest-path /private/tmp/stackpress-s4-probe/rusqlite/Cargo.toml --example non_sync --offline --locked
```

For each of the two manifests, also run `cargo metadata --format-version 1 --filter-platform aarch64-apple-darwin --offline --locked` and `cargo tree --edges normal --prefix none --offline --locked`. The actual commands include the appropriate `--manifest-path`. Test/check command re-verification used Python subprocess.run with a 120-second compile/test watchdog, captured stdout/stderr and checked exit/diagnostic; native tests complete in under a second after build. Production concurrency proofs need the stricter child-process controls in [verification](slice-4-design-verification.md).

Minimal candidate dependencies are `sqlx = { version = "=0.9.0", default-features = false, features = ["runtime-tokio", "sqlite"] }` versus `tokio-rusqlite = { version = "=0.8.0", features = ["bundled"] }`; both use existing Tokio, SQLx test uses tempfile/current lib/server. Resolved Tokio **1.54.1** declares Rust **1.85**, MIT. SQLx macros/Any/migrate/network drivers do not appear on the SQLx normal-edge tree, although Cargo.lock/metadata may mention optional macro packages.

## Probe outcomes

| Probe | Observed result |
| --- | --- |
| SQLx `static_bounds` | PASS: SqlitePool and SqliteConnection Send+Sync. This does not grant concurrent mutable execution. |
| `binding_types_returning_and_foreign_keys` | PASS: malicious text remains data, BLOB/boolean/ID/INSERT RETURNING and UPDATE RETURNING work, foreign_keys=1 and JSON extraction works. |
| `send_transaction_across_await_and_drop_rollback` | PASS: transaction query future is Send across await; dropping after INSERT leaves row count zero. |
| `aborted_task_transaction_rolls_back` | PASS: oneshot barrier after INSERT, abort/await task, next pooled query sees zero rows. |
| `unwind_transaction_rolls_back` | PASS: intentional task panic yields JoinError::is_panic and next pooled count is zero; no blanket panic recovery claim. |
| `held_connection_times_out_and_pool_recovers` | PASS: size-one acquired lease causes finite PoolTimedOut, releasing permits next acquire, closed pool yields PoolClosed. |
| `acquisition_cancellation_releases_waiter` | PASS: timeout while another lease is held does not strand next acquire. Cancellation after acquisition/begin is not covered by this test. |
| `sqlite_work_does_not_block_executor` | PASS: current-thread timer runs before a real 2,000,000-row recursive sum finishes; final sum is exact. Timed workload is a bounded responsiveness smoke, not a deterministic all-path blocking proof. |
| `isolated_files_and_reopen` | PASS: two tempfile databases share no schema; first file retains table after explicit close/reopen. |
| `existing_host_plugin_send_service` | PASS: current ServerBuilder plugin/on_config/register Arc service, named handler and raw emit execute SELECT 42 without shared-crate changes. |
| `dynamic_sql_and_parameter_description` | PASS: AssertSqlSafe generated String, prepared metadata count=1 despite quoted/comment `?`, bound O'Reilly survives. **Native missing bind becomes NULL**, proving first-party arity validation is required. |
| `close_waits_for_lease_and_can_be_resumed` | PASS: close times out while lease held, pool marked closed, releasing then re-awaiting finishes. |
| `erased_owned_transaction_is_send_and_borrowed_exclusive` | PASS: boxed Send session owns SQLx Transaction<'static>, query borrows &mut and consuming commit works; this is a signature fixture, not a production abstraction. |
| tokio-rusqlite `bounds` | PASS: native Connection Send; worker handle Send+Sync. |
| `worker_binding_and_synchronous_transaction` | PASS: real synchronous transaction in worker closure binds malicious-looking text and returns exact data. |
| `cancelled_caller_does_not_cancel_worker` | PASS: ready/go barriers stop native closure; abort async task then release worker; queued write completes and row count becomes one. |
| Compile failure `aliasing` | PASS expected exit 101 / **E0499**, concurrent mutable transaction query borrows rejected. |
| Compile failure `clone` | PASS expected exit 101 / **E0599**, transaction clone absent. |
| Compile failure `non_sync` | PASS expected exit 101 / **E0277**, native rusqlite Connection cannot satisfy Send+Sync. |

Totals: **13 SQLx + 3 tokio-rusqlite tests passed; 3 expected compile failures matched**. No ignored tests. The initial dynamic-SQL metadata probe failed to compile because `describe` is feature-gated; replacing it with public `prepare`/Statement metadata proved the intended contract without adding macros/offline features. The final logs contain the corrected probes. These counts do not represent workspace tests, coverage or production verification.

## What remains unproved

The actual Stackpress SQL types/renderers do not exist yet. M0 must replace signature fixtures with them; controlled begin/active-query/commit cancellation and rollback-failure retirement, actual nested/concurrent capability dispatch, admission/row budgets, raw-statement lexical validation and full DDL/JSON semantics are still required. Basic pool affinity/drop tests are not evidence for every driver transition. No network adapter, PGlite/serverless/native Tauri/browser/mobile, other OS, crash/power-loss durability, benchmark or publication compatibility was proved.

The source review and native probe combination justifies choosing SQLx for M0, subject to owner Q06. It does not justify widening safety claims to immediate native interruption or exactly-once commits. The coordinator should rerun surviving temporary probes and independently review every owner decision before releasing production work.
