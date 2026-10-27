# Task 00011 — Slice 6 M2 implementation receipt

M2 only, 2026-10-27. Worktree `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s6-m0`, branch `trial/3-s6-m0`, final integrated HEAD `fdcd2f9a76a31c51445bfff3bd041b5ec24e0786`. This is implementation evidence for coordinator re-verification, not acceptance or publication. Source remains uncommitted. M3 was not started.

## Ownership and frozen interface

M2 began at `cce3a314d64f95f962238b0984b636dd3581911c`; the requested integration of `trial/3` then reached `7b24106fbb01a5b9933a14d242e4bbd60310a945`, including task 00014's SQL test-stability fixes. A final integration reached the HEAD above. Only the granted framework/schema/example paths and the eight explicitly granted SQL-core paths below are dirty. No other slice's files, adapter traits, ordinary raw-query gates, Lib, Server or CLI implementation were edited. Root Cargo.lock adds only the app's already-locked `http` and `stackpress-cli` test dependency edges; no root manifest/facade/default-feature expansion.

G34 remains unchanged: `stackpress_framework_schema::CONTRACT_REVISION == 2`; the original app's `contracts` and `metadata` files are byte-unchanged. Descriptor/public DTO/query names and serialized shapes remain those frozen in the [M1 receipt](00011-receipt-m1.md). Partitions remain `contracts`, `metadata`, `stores`. SQL's existing runtime compatibility constant remains 1. New APIs and host envelopes are inside `stores`; no automatic route, lookup URL or descriptor-operation grant was added.

The new native surface is `<Model>Store`, plural `<Model>s`, native `Filter`/`SortField`/`Query`, `ConflictKey`, `BatchInput`/`BatchOutcome`, `View0`–`View3`, `PublicView0`–`PublicView3`, and opaque `Selected`. Selected host output is `stores::operations::<model>` with KeyRequest/CreateRequest/UpdateRequest/UpsertRequest/BatchItem/BatchRequest, named handlers and register. Schema/Operations use `_schema`/`_operations` native store and operation modules to avoid aggregate collisions; their pure contract modules keep normalized source names. BatchItem boxes payloads without changing its JSON shape.

## Implemented and proved

- Complete scalar/compound-key storage and keyed create/get/update/upsert/delete, deterministic find/find_all/count/search, prepared plans and strict codecs. The native fixture includes all scalar key families in one nine-member identity, integer/Boolean/string enum families, canonical temporal values, bounded JSON/lists/fieldsets and keyword/aggregate model names. Defaults/providers/business validators prepare once per instance. Timestamp refresh uses one explicit injected clock; immutable/generated fields cannot be patched.
- Typed scalar Eq/Ne/Ge/Le, escaped bound Like, literal case-sensitive q, direct scalar-array Has/Hasnt and explicit SQL NULL tests. Enums retain their declared choices. Eligible sorts have explicit NULL placement and complete-key ties; take/skip/count keep the frozen bounds. SQLite datetime ordering/comparison preserves exact nanoseconds while retaining established canonical UTC storage/FK spelling.
- Finite eager views and explicit include paths, with unloaded distinct from loaded empty/optional None. Breadth-first jobs batch across every parent at one path, deduplicate keys, derive chunks from the exact executor's bind/SQL/byte limits and reserve active-predicate binds. Lowerable path/depth/row/key/byte/statement budgets error without truncation. Native observation at Database/TransactionSession proves the 1-versus-20 parent N+1 bound, row/page/count correctness, two levels, reduced-budget chunks and an actual concurrent-writer SQLite snapshot.
- Active-only idempotent remove/restore, validated active-field rules and timestamp refresh; ordinary reads hide inactive rows and trusted maintenance visibility is explicit. Hard delete is a separate capability with RESTRICT FKs. Whole batches validate/prepare/render before their first write, retain input order, and use one transaction. Two-store failure rollback and success commit use the exact borrowed session across awaits. Mixed native/public batch modes and primary/nonnull unique upsert targets execute against SQLite.
- Atomic targeted upsert and create-if-absent, with no check-then-create. Unique-target RETURNING preserves the existing identity. DoNothing returns explicit `InsertOutcome::AlreadyExists`, containing no fabricated existing row. Concurrent writers prove one insert/one no-change. Last-writer-wins updates have no expected version or automatic retry.
- Submitted write/commit errors conservatively retain UnknownOutcome; cancellation can occur after a durable autocommit effect. Real backend effects followed by lost acknowledgements, cancellation, commit uncertainty and a secondary rollback-ack failure prove no replay/refetch-based inference and retained typed causes/cleanup errors. M0 ledger reconciliation and maintenance/watchdog negative controls still pass.
- Explicit named handler registration and required Engine/Policy services, app-owned duplicate marker and selected operations only. Prior completion skips work; nested delegation preserves abort. Authorization precedes decoding/IO; strict public envelopes/private host assembly and recursive projection prevent hidden-field/default/driver/path/credential disclosure. The original three-model CRUD/soft-delete/batch/search/detail sequences have direct/CLI/buffered-HTTP parity; no listener is involved.
- Generation from absent output has no bootstrap cycle and restores identical reviewed bytes. Repeat/check, SQL-store/handler rename/removal, sentinel preservation and edited-output refusal pass. Unsupported default includes retain exact winning Idea source spans; normalized JSON retains structural paths without invented positions. Fully private models generate native stores without requiring a public UI entry.

The full native type fixture and the small reserved-name/private-model fixture are generated separately to respect the unchanged one-MiB generator output ceiling. Each compiles/executes through an explicit `matrix` or `names` app test feature; select one fixture feature at a time. The pure consumer and native matrix now execute the same shared default/value/access tests from `examples/generated-workspace/support/matrix_contracts.rs`.

## Exact shared SQL-core changes

The coordinator accepted both proposals exactly and granted these additive edits exclusively through this receipt:

| Granted path under crates/sql/sql | Final change |
| --- | --- |
| src/dml.rs | `InsertConflict::{DoNothing,DoUpdate}`, private optional Insert clause, `on_conflict`/`conflict`, bounded validation and native rendering |
| src/lib.rs | Re-export InsertConflict only; existing export list wraps with rustfmt |
| tests/dml.rs | Three appended rendering/inspection/hostile-identifier/shape/bind-budget cases |
| docs/api/insert.md | Explicit target semantics, bind order, dialect refusal and no-row DoNothing RETURNING |
| src/engine.rs | Only `Engine::limits(&self)->QueryLimits` and `Transaction::limits(&self)->QueryLimits`, each with Rustdoc/doctest |
| tests/engine.rs | Appended default/custom/copied/frozen-policy/no-native-call getter test |
| docs/api/engine.md | Engine getter documentation |
| docs/api/transaction.md | Transaction getter documentation |

Conflict targets are nonempty checked identifiers; DoUpdate also requires nonempty checked assignments. One explicit VALUES row only; DEFAULT/multi-row shapes are refused. SQLite/PostgreSQL render before RETURNING with insert binds before patch binds. MySQL returns Unsupported naming its inability to restrict the conflict target before any effect. Existing limits/InsertData/WriteValue/adapter traits remain unchanged. Native SQLite tests additionally prove unrelated unique violations still fail and a failed update branch rolls back its statement. PostgreSQL rendering is proved; native PostgreSQL/MySQL generated CRUD was not attempted.

The only framework-schema implementation edit adds `#[cfg(feature="codegen")]` to the private codegen-only `RecordSchema::validate_defaults`, avoiding an inherited default-feature dead-code warning without changing any public/frozen behavior. The [accepted proposals and runtime docs](../../../../crates/framework/framework-sql/docs/README.md) record the contracts and usage.

## Review rounds

1. Logic/complexity: checked validation/effect order, complete identities, NULL/temporal comparisons, conflict scope, missing/no-change outcomes, batch preflight and executor-derived chunk boundaries. Replaced batch numeric modes/fallback with an exhaustive enum; added reduced path/depth, active-rule and key-only refusal cases. `round1.log` passes all native proof fixtures.
2. Responsibility/abstraction: separated immutable store catalogs, pure query compilation and bounded relation loading; generated read/write/conflict/selection/operation emitters have cohesive owners. The diagnostic regression exposed a fully private-model/public-UI assumption; fixed it with empty public eligibility while retaining the native store. `round2-complete.log` passes app/tool tests, compiler proofs and alternate fixtures. Intermediate extraction/import and initial diagnostic-fixture failures are retained in earlier round2 logs.
3. Consistency/Rustdoc/comments: documented runtime/emitted APIs, ownership, effect/acknowledgement order, bounds and disclosure; added local state/branch/async explanations. Boxed rich public batch payloads and escaped aggregate operation modules after Clippy exposed their matrix-only issues. `round3-complete.log`: 569 passed, 0 failed, 16 ignored including doctests; all three fixture Clippy variants clean.
4. Safety/soundness: reviewed exact executor borrowing, Send/static boundaries, finite graph/cache expansion, native effect/cancellation/cleanup precedence, identifier/bind construction, policy-before-IO and recursive disclosure. No unsafe code, ambient transaction, captured alternate pool, string-based backend classification or retry was introduced. Existing six alias/Clone/escape compiler failures and the native watchdog/fault proofs remain green.
5. Coverage-driven branch follow-up: initial app coverage was 49.66%; no generated files were excluded. Added actual rich handler lifecycles, mixed native batch modes, callable methods on each identity family, private-model host assembly/refusals, and the shared pure/native input matrix. An early helper violated the explicit-transaction detail contract and was corrected to use the caller snapshot; another helper borrow error was fixed without altering production code. `breadth-first`, `native-breadth-complete`, `breadth-second`, `callable-proof` and fresh final coverage pass. This coverage follow-up changed tests only; final verification includes the expanded tests.
6. Final name/shape follow-up: a source relation named `row` produced 39 consumer compiler errors before the fix (`row-counterexample.log`). Preserve the base `row` slot and use `_relation_row`/`__row` only for the colliding native member; portable source names cannot use those prefixes. Includes and Selected flattening retain logical source spelling. Centralized forward/inverse protocol mapping; actual SQLite loading, recursive public projection, whole-node selection and the named detail handler prove both base and related data survive. `row-fix-round1` and `row-fix-round2` pass; the final round reviews comments/docs, Clippy, all fixture checks and safe naming overrides. No frozen pure contract shape was changed.


## Verification and reproducible commands

Host: rustc/Cargo 1.96.0 (`ac68faa20`, 2026-05-25), aarch64-apple-darwin, LLVM 22.1.2; preinstalled cargo-llvm-cov 0.8.6. Logs/result JSON/scripts are under `/private/tmp/s6-m2/`. Each command uses the sole worktree `target/`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, TMPDIR under that target, and unset PostgreSQL/MySQL test URLs. The runner bounds each command group at 600 seconds with process-group kill/reap. Hang-class native proofs also retain their independent parent watchdogs; timeout is failure. Target was deleted after all checks, and no build/test process remains.

| Evidence | Result |
| --- | --- |
| compatibility.log / compatibility-commands.json | Absent-output tool build/generation and byte identity; all four generation checks; fmt; explicit Rust 1.96 checks; warning-clean Rustdoc; feature matrix/facade; WASM pure contracts; dependency graphs; named guide example compiled in a disposable crate |
| generation-followup.log | SQL store/handler rename/removal regression: six tool tests pass; scoped Clippy clean |
| coverage-final-row.log / coverage-commands.json | Fresh instrumented run: 146 passed, 0 failed, 16 ignored; all six measured crates clear 70% |
| final-checks.log / final-checks-commands.json | 669 all-target tests + 895 doctests passed, 78 ignored; fmt/Clippy/MSRV/generation clean |
| latest-workspace-row.log | Final integrated fdcd2f9: 669 passed, 0 failed, 78 ignored; 895 doctests passed; fmt and workspace Clippy clean |
| row-final-checks.log / row-final-checks-commands.json | 576 passed, 0 failed, 16 ignored including doctests; all fixture Clippy, explicit 1.96 checks, generation checks, Rustdoc and WASM clean |
| kb-baseline.log / kb-final.log | 146 inherited errors and 22 warnings before/after; identical findings, no new receipt finding; validator exit 1 is a baseline failure, not a green KB gate |

Core command families, from the worktree root with the environment above:

```sh
s6_features=stackpress-framework-sql/codegen,stackpress-framework-sql/server,stackpress-framework-sql/sqlite-maintenance
cargo fmt --all -- --check
cargo test -p stackpress-sql -p stackpress-framework-schema -p stackpress-framework-sql \
  -p generated-workspace-app -p generated-workspace-tool -p generated-workspace-contracts \
  --features "$s6_features" --locked
cargo clippy -p stackpress-sql -p stackpress-framework-schema -p stackpress-framework-sql \
  -p generated-workspace-app -p generated-workspace-tool --all-targets \
  --features "$s6_features" --locked -- -D warnings
cargo +1.96.0 check -p stackpress-sql -p stackpress-framework-schema -p stackpress-framework-sql \
  -p generated-workspace-app -p generated-workspace-tool -p generated-workspace-contracts \
  --features "$s6_features" --locked
RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-sql -p stackpress-framework-schema \
  -p stackpress-framework-sql -p generated-workspace-app -p generated-workspace-tool \
  -p generated-workspace-contracts --features "$s6_features" --no-deps --locked
cargo test -p generated-workspace-app --features matrix --test matrix --test matrix_contracts --locked
cargo test -p generated-workspace-app --features names --test names --test names_contracts --locked
cargo clippy -p generated-workspace-app --all-targets --features matrix --locked -- -D warnings
cargo clippy -p generated-workspace-app --all-targets --features names --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p generated-workspace-tool --locked -- emit generate --check true
cargo check -p generated-workspace-contracts --target wasm32-unknown-unknown --locked
cargo tree -p generated-workspace-contracts --edges normal --locked
python3 .agents/scripts/validate-agent-workspace.py
```

`compatibility-commands.json` records exact input/output paths for default/native/names/pure-matrix generation checks, standalone default/server/codegen/migrations/sqlite-maintenance builds, tool database-tools selection, opt-in facade and normal-edge graphs. The pure contracts graph excludes server/CLI/Idea/SQL/SQLx/Tauri; the tool has no dependency on the generated app. No listener, install, network database, commit, push or publication occurred.

## Coverage scope and limitations

Fresh command sequence: `cargo llvm-cov clean --workspace`; `cargo llvm-cov --no-report` on the six packages with the features above and `-- --skip exclusive_session_compile_failures`; then separate `--no-report` matrix/names app runs including their `_contracts` tests; `cargo llvm-cov report --json -vv`; `export-coverage.py`. Doctests and the compile-only borrowing parent pass separately; they are not claimed as instrumented runtime coverage. The tool rejects combining --no-report with --no-clean, so the corrected final fresh sequence uses --no-report alone.

The raw export repeats the exact llvm-cov export argv printed by the verbose report, removing only its default ignore-filename-regex option so generated example source remains visible. `coverage-all-sources.json`, `coverage-export-command.json`, `coverage-files.json` and `coverage-summary.json` retain per-file data. Aggregation selects production src paths, excludes test modules/dependencies/tooling fixtures, and includes all shipped generated app/default/matrix/names source and the pure consumer's original contracts plus its matrix. No generated files, uncovered branches or low-scoring model were excluded.

| Crate / complete measured selected production scope | Covered / executable lines | Percent |
| --- | ---: | ---: |
| stackpress-sql | 3015 / 3154 | 95.59% |
| stackpress-framework-schema, codegen | 3218 / 3546 | 90.75% |
| stackpress-framework-sql, codegen/server/migrations/SQLite maintenance | 2617 / 3022 | 86.60% |
| generated-workspace-app, all three generated fixture outputs | 10316 / 13519 | 76.31% |
| generated-workspace-contracts, original source + pure matrix | 1463 / 2083 | 70.24% |
| generated-workspace-tool, library + CLI | 161 / 212 | 75.94% |

This satisfies the 70% development floor; it is not publication's 90% claim or complete branch coverage. PostgreSQL/MySQL generated-CRUD execution, network repeatable snapshots, literal containment and datetime-order integration remain a later SQL-lane checkpoint; unsupported consistent read/order/q requests are refused. MySQL generated RETURNING writes/targeted conflicts remain gated. Without a portable typed known-unique category, handlers conservatively return sanitized 500/unknown outcomes rather than invent 409 classification.

Authorization is a synchronous app-owned capability boundary; direct stores remain trusted and policies requiring transactional row access belong in a custom domain workflow. Detail uses the public key/filter allowlist, so a wholly private key is refused while trusted get_key and explicitly selected public-projected get remain available; private-model tests prove this without widening exposure. Offset pages across separate calls can change; there is no optimistic version, keyset paging, implicit transaction inheritance, automatic route, nested relation mutation, retry, or arbitrary SQL input.

Default selection with no source @query falls back to the approved base row; an explicitly declared selection with no approved paths produces an empty object. Related rows are bounded at 10000, logical output at 16 MiB, cache retention at four times that ceiling, statements at 128, paths at 16 and depth at 3; limits can only be lowered. M0's explicit migration subset remains the migration boundary; M3 expansion has not begun. The historical framework-sql/docs/m2-progress.md is kept, prominently marked superseded by this receipt. No further shared edit is requested.
