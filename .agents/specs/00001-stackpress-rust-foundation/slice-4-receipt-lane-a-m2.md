# Slice 4 lane A M2 receipt

Task: [00007 Slice 4 SQL](tasks/00007-slice-4-sql.md). Authority: G24/G26, the coordinator's M2 release and subsequent explicit plan-execution request. This stops at M2 for coordinator re-verification; it does not release M3, joint refactoring rounds or publication.

## First handoff: plan execution for lane B

Implemented in [Engine and Transaction](../../../crates/sql/sql/src/engine.rs):

- `Engine::execute_plan(&self, plan: &QueryPlan) -> Result<Vec<QueryResult>>` validates every query's dialect and input budget before acquisition, rejects nontransactional DDL (MySQL), begins one transaction, runs in plan order, commits only after every statement succeeds, and returns one owned outcome per statement. Failure at N stops before N+1 and calls `rollback_after`, preserving the original kind/source and any secondary rollback error. Failed/cancelled commit remains an unknown outcome and is never retried.
- `Transaction::execute_plan(&mut self, plan: &QueryPlan) -> Result<Vec<QueryResult>>` validates the complete plan before its first run, uses the existing exclusive session and returns the same ordered outcome shape. It never commits or consumes caller ownership. On a statement error, the caller chooses explicit rollback/rollback_after. Cancelling this borrowed method retains the caller's transaction; the adapter's active-statement cancellation policy applies. Dropping the owner initiates rollback.
- Cancelling the Engine-owned future drops its active transaction/session, invoking the existing adapter rollback-on-drop contract. The core fake models committed versus staged statements and active session ownership: cancellation after statement 1/before completion of statement 2 leaves no committed statements and zero active sessions.
- `Database` and `TransactionSession` declarations are byte-identical to the merged M1 versions. No driver trait or pre-existing execution signature changed.

Core evidence: `engine_plan_success_and_caller_owned_transaction`, `engine_plan_failure_preserves_original_and_cleanup_errors`, `engine_plan_validation_and_cancelled_owner`, and the private plan-validation tests. They cover success/order/one session, failure at statement N, original plus cleanup errors, input limits, dialect mismatch, mixed/empty plans, MySQL's transactional-DDL rejection and cancellation. Private malformed-plan tests use a NoIo adapter that panics if acquisition/execution is reached. Test-only code lives under `src/tests/` or `tests/` and is excluded from production line coverage. These are core ownership/policy proofs; lane B owns actual SQLite plan/transaction integration after merge.

## Baseline and boundaries

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s4-lane-a-core`; branch `trial/3-s4-lane-a-core`.
- Clean starting HEAD after coordinator fast-forward: `8b74328c704fea3666492b95750c17fd54b37be3` (M1 merged into trial/3 as `bb078d5`).
- Implementation/test/manifest ownership: `crates/sql/sql/**`. Root `Cargo.lock` was changed only after explicit coordinator approval to resolve the new G26 dev-dependencies. This receipt is the only requested `.agents` write.
- No root manifest/facade, SQLite adapter, lib/server/CLI/Idea/UI source, status or task record changes. No production network adapter, new normal dependency, unsafe code, commit, push, listener, deployment, publishing, KB promotion or messages to agents.

## Renderer and schema behavior

[Capabilities](../../../crates/sql/sql/src/dialect.rs) explicitly describe returning, transactional DDL, native constraint alteration, FULL JOIN, cascade, index idempotence, JSON numeric identity and text NUL support. They describe renderer behavior rather than the presence of a production adapter. SQLite behavior remains covered by the retained M1 suite and workspace adapter tests.

| Capability | SQLite | PostgreSQL | MySQL |
| --- | --- | --- | --- |
| INSERT/UPDATE/DELETE RETURNING | Yes | Yes | Unsupported |
| Transactional DDL plan execution | Yes | Yes | Unsupported before acquisition |
| Native type/default/nullability/primary/FK alteration | Reconstruction operations rejected | Supported typed operations | Supported typed operations |
| FULL JOIN | Yes (modern bundled SQLite) | Yes | Unsupported |
| DROP/TRUNCATE CASCADE | Unsupported | Supported | Unsupported |
| Idempotent index creation | IF NOT EXISTS | IF NOT EXISTS | Indexes inline within CREATE TABLE IF NOT EXISTS |
| Integer/real JSON storage identity | Distinct kinds | Native JSON numeric equality (1 = 1.0) | Native JSON numeric equality (1 = 1.0) |
| NUL in ordinary text binds | Supported | Rejected before rendering | Supported |

- Native DML uses one shared AST with renderer-specific identifier quotes and parameters. PostgreSQL `$n` numbering comes from bind-node traversal; no regex/global marker or quote replacement is used. Native offset-only syntax is explicit. MySQL RETURNING/FULL JOIN fail instead of disappearing. Explicit JSON writes use `WriteValue::Json`, bound canonical text and native JSON casts; SqlValue variants remain unchanged for lane B.
- PostgreSQL quoted identifiers are bounded to 63 bytes; MySQL's conservative common segment/alias bound is 64 characters. Literal dots remain one segment unless qualification was explicit. Empty/control/NUL names and native truncation hazards error.
- CREATE/ALTER render dialect-tagged ordered plans only after validating the complete operation set. PostgreSQL uses typed native types/identity, deterministic primary constraint names, separate comments and ordered idempotent indexes. MySQL uses its native field/auto-increment/unsigned/comment/default syntax and inline CREATE indexes so idempotence does not claim unsupported CREATE INDEX IF NOT EXISTS.
- Native field validation preserves no default versus explicit NULL/false/zero/empty. Positive lengths/precision, default types and native limits are checked. MySQL empty string defaults use an actual empty literal; other text defaults use UTF-8 hex conversion expressions, independent of quote/backslash mode. PostgreSQL text defaults use explicit E string escaping. Native defaults/types require an explicit TrustedSql marker for custom expressions.
- Unsupported native requests fail before a plan is exposed: SQLite reconstruction changes, PostgreSQL unsigned fields, MySQL SET DEFAULT FK actions and TEXT/BLOB/JSON indexes needing an unmodeled prefix/expression, unsupported identity-policy changes or adding identity fields without a coordinated key definition. Known PostgreSQL local key names are checked with quoted case sensitivity. Unsupported annotations are not silently discarded.
- JSON paths retain explicit key/index overloads on PostgreSQL, bound paths on MySQL, and checked per-query convenience options. Native extraction returns native JSON/JSONB on the network dialects; callers choose decoding. Equality retains JSON null versus missing/SQL NULL and bool/number/text distinctions. PostgreSQL's direct jsonb array iteration and MySQL JSON_OVERLAPS prevent recursive nested-array membership. Integer/real numeric equivalence on the network dialects is explicit in capabilities and is verified against both servers. PostgreSQL JSONB NUL strings/keys are rejected.
- Trusted native single statements retain dialect identity and a bounded lexical gate: native PG dollar parameters/casts/dollar-quoted strings, SQLite/MySQL anonymous binds, quoted tokens and comments. Batches/control commands, MySQL executable comments and raw network backslash-quoted text outside the supported subset error; callers bind data. Bound numbered PostgreSQL trusted expression fragments must precede other binds, rather than being silently renumbered.
- [TableDefinition and SchemaDiff](../../../crates/sql/sql/src/diff.rs) separate storage metadata from create-if-not-exists policy. Definitions normalize and validate explicit fields, ordered compound primary/index/unique/FK metadata and every field property by value. Same logical table is mandatory; no live introspection or inferred rename occurs. A typed NoChanges has no executable empty SQL. Reordering existing physical fields is explicitly Unsupported.
- Diff order removes dependencies first, changes/removes/adds fields, then adds keys/FKs. Explicit rename requires both pending halves and a known source. It replaces the selected remove/add pair, preserves a separate changed-definition operation and runs before dependent index/FK additions. Direct and diffed rename collision/missing-half rules remain enforced. `Engine::diff` is a detached convenience with no IO.

## M2 Now-row reconciliation

Rows refer to [the core test inventory](slice-4-design-tests-core.md). M1 CRUD/SQLite tests remain passing; the following closes the data/rendering rows assigned to M2 rather than sampling one builder.

| Inventory rows | Passing evidence |
| --- | --- |
| Engine 1-5/7-17, Helpers 3 | `tests/diff.rs`: no-op/same-table/invalid inputs; every field property; field add/remove; same/changed-definition rename; missing halves; primary/index/unique/FK additions/removals/changes; compound-key order; cloned value equality; FK table casing; dependency order/new foreign fields; explicit physical reorder rejection. |
| PostgreSQL 1-4/18-22 | `alter_full_native_plan`, `create_types_defaults_and_idempotent_indexes`, `fields_foreign_actions_and_invalid_lengths`, `native_key_and_changed_attribute_validation`; native `create_types_defaults`, `alter_fields_constraints`, `schema_diff_rename`. |
| PostgreSQL 5-8/14-16/23-24 | All DML native snapshots, commands/page/quoting/path tests; native CRUD/RETURNING, joins, predicates/page, JSON and table-command cases. |
| PostgreSQL 9-13/17/19 replacements | Both-dialect invalid/empty builder tests, explicit truthy-default replacement tests, trusted syntax tests and native empty/false/zero/default success. |
| MySQL 1-3/9-13/19 | Native CREATE/ALTER/rename snapshots and capability tests; nullable/unsigned/default/empty/precision tests; multiple indexes/unique/FK metadata; native CREATE/ALTER/diff cases. |
| MySQL 4-8/17/20-21 | All DML snapshots and native CRUD/joins/predicates/JSON cases, exact quoting/token tests, checked path options and parameter order. |
| MySQL 14-16/18 | Invalid mutation scope/insert/select/empty assignments tested for each dialect; no manufactured empty SQL. |
| SQLite 1-4/28-30 | Retained M1 typed plans/default/index/rename/JSON/parse-option tests plus M2 diff/reconciliation and whole-plan preflight tests. Actual new SQLite DDL/JSON execution remains lane B. |
| Create 7, Alter 13; Engine execution/transaction replacement rows | New core Engine/Transaction execute_plan tests prove ordered outcomes, failure/rollback/secondary source preservation and cancellation. Native SQLite atomic DDL is lane B's post-merge integration. No production network transaction adapter is claimed. |
| Dialect safety/replacement rows across all families | Identifier injection/quotes/dots/controls/size, null/missing/default/false/zero/empty, finite values, JSON key/index/type/membership, whole-plan invalidity and native capability errors are covered by retained/new core tests and the explicit network matrix. |

The production network adapters remain S4-N. Migration history, live introspection, SQLite reconstruction, automatic rename guesses, general framework stores, savepoints/middleware and platform expansion remain their named follow-ups. Native SQL validation is G26 renderer validation, not a production network adapter release.

A final failing regression (`foreign_set_null_cannot_target_primary_columns`) also showed that primary-key-derived NOT NULL semantics were not checked when a field retained its default nullable flag. CREATE/TableDefinition validation now rejects SET NULL on those primary columns before returning any plan, for both update/delete actions. The failing predecessor is preserved as `primary-set-null-regression-before.log`; all final suites and the final native run below passed after this fix.

## G26 opt-in native results

The custom `tests/network.rs` harness prints runtime-gated cases as ignored. With both environment variables unset: **0 native passes, 16 ignored**, and no connection is opened. The fresh core/workspace runs use that mode. Each configured backend runs all eight named families in one newly created database and owns its exact generated name through a drop guard; cleanup is attempted during errors/unwind as well as explicit close.

Final configured run (`network-final-source.log`): **16 passed, 0 failed, 0 ignored**:

- PostgreSQL **12.2**, `x86_64-apple-darwin17.7.0`, compiled by Apple LLVM 10.0.0; exact `SELECT version()` output is in the log. Eight cases passed; owned database `stackpress_test_13bb3_18e1fee9ee70a5b8` was removed.
- MySQL **8.0.27**, from `SELECT version()`. Eight cases passed; owned database `stackpress_test_13bb3_18e1feea05ddfe08` was removed.
- Both executed CREATE with typed defaults/identity/comments/compound indexes/FKs; one/bulk CRUD, DEFAULT/NULL, PG RETURNING and MySQL rejection; supported joins; groups/IN/escaped LIKE/limit/offset/zero; JSON extraction/typed paths/direct membership; native ALTER fields/primary/FK actions; same/changed-definition rename with retained rows; and table rename/truncate.
- JSON cases include isolated bool/number/text arrays and cross-type nonmatches, JSON null versus missing/SQL NULL, integer/real equality, direct nested-array nonmembership, nested index traversal, `01` keys and quotes/backslashes, and malformed JSON returning an actual native error while data/connection remain usable.
- Normal native tests never enumerate database catalogs, access another database's contents or modify roles/settings. The only top-level DROP statement is DROP DATABASE naming the guard's own generated database; standalone DROP TABLE/INDEX output is covered by pure snapshots rather than executed in these tests. Rendered ALTER DROP COLUMN/CONSTRAINT operates only inside the owned fixture database.

The initial live effort exposed two defects and is preserved: PostgreSQL 12 rejects DROP DATABASE WITH (FORCE), and MySQL 8.0.27 rejects the empty hex-conversion default expression. Both were corrected. One PostgreSQL database from that first failed cleanup was identified using only the failed test process's generated prefix and bounded nonce interval, then removed as exactly `stackpress_test_152e_18e1fcbdabfd0b50`; `s4-a-m2-own-db-recovery.log` records it. The temporary helper was deleted. No other database/role/setting was changed. Final cleanup uses ordinary DROP DATABASE after closing fixture connections, with a separate bounded runtime for Drop fallback. Subsequent complete runs removed both owned databases successfully.

Reproduction (local URLs have no passwords):

```sh
STACKPRESS_TEST_POSTGRES_URL=postgres://postgres@localhost:5432/postgres \
STACKPRESS_TEST_MYSQL_URL=mysql://root@localhost:3306 \
cargo test -p stackpress-sql --test network --locked --offline
```

Native IO required the sandbox's explicit socket approval. Initial denied socket attempts are failures in the retained log, not passes or skips. Connection/setup/query/cleanup budgets are finite. This is not a server-version/platform compatibility certificate beyond the versions actually exercised.

## Dependencies and lockfile audit

`sqlx = "=0.9.0"` (defaults off; `runtime-tokio,postgres,mysql`) and Tokio (`rt,time`) are dev-dependencies only. Normal dependency proof:

```sh
cargo tree -p stackpress-sql --edges normal --locked --offline
```

Output is only stackpress-sql itself. No runtime/driver/host dependency enters the normal SQL core tree.

The coordinator explicitly approved Cargo resolving these new dev-dependencies into the root lockfile. No cargo update was run. The diff is **+272/-10 lines**, adding **23 package identities**, removing **0**, and preserving every existing name/version/source identity. Existing dependency references acquire version disambiguation where parallel crypto versions now coexist; no existing package version changed. Added identities:

`block-buffer 0.12.1`, `chacha20 0.10.2`, `cmov 0.5.4`, `cpufeatures 0.3.1`, `crypto-common 0.2.2`, `ctutils 0.4.2`, `digest 0.11.3`, `etcetera 0.11.0`, `hkdf 0.13.0`, `hmac 0.13.0`, `hybrid-array 0.4.15`, `md-5 0.11.0`, `rand 0.10.3`, `rand_core 0.10.1`, `sha1 0.11.0`, `sha2 0.11.1`, `sqlx-mysql 0.9.0`, `sqlx-postgres 0.9.0`, `stringprep 0.1.5`, `unicode-bidi 0.3.18`, `unicode-normalization 0.1.25`, `unicode-properties 0.1.4`, `whoami 2.1.3`.

The lock identity-set comparison is in `lock-audit.json`. Shared lockfile reconciliation remains coordinator-owned at merge.

## Final verification and fresh coverage

Compiler/target: rustc/Cargo **1.96.0**, LLVM 22.1.2, native `aarch64-apple-darwin`; cargo-llvm-cov **0.8.6**. No toolchain, target, service or system package was installed. Host servers report their own architecture above.

Successful workspace checks use `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0` to bound disk use, and unset both network URL variables:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked --offline
cargo test --workspace --doc --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo +1.96.0 check --workspace --all-targets --locked --offline
RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-sql --no-deps --locked --offline
git diff --check
```

All passed: **352 workspace tests**, 0 failed, **44 ignored entries** (16 are the opt-in network cases; the others are existing harness entries), and **440 workspace doctests**, 0 failed/ignored. Core alone: **69 tests**, 0 failed, **16 network cases ignored**; **8 core doctests** passed (six examples/two intended compiler failures). Final stateful plan tests passed separately after strengthening committed/staged state assertions. Final core Clippy and tests were rerun after the last native JSON test additions.

The first broad --all-features attempt failed because the merged UI explicitly forbids combining ssr/hydrate/csr. No UI source changed. After reading its documented valid matrix, the workspace target was cleaned and default-mode checks above passed from fresh artifacts. The failed attempt remains in `tests.log`; it is not reported as a passing platform check.

Fresh coverage used only this crate's own tests with both network variables unset, after a clean. No SQLite adapter or other package's tests contribute:

```sh
cargo llvm-cov clean --workspace
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 \
cargo llvm-cov -p stackpress-sql --locked --offline --fail-under-lines 70 \
  --ignore-filename-regex '/tests/' --json \
  --output-path /private/tmp/stackpress-s4-lane-a-m2-evidence/core-coverage-final.json
```

**2,794/2,926 executable first-party library lines = 95.49%**, passing the 70% development floor. Only test-only source paths are excluded. Production source is not excluded; stable doctests and live network validation ran separately. Network-DDL library coverage from the self-contained test run is 303/384 lines; remaining gaps include some custom type/default/comment/native-limit/error variants that native validation exercises separately. Coverage is not branch or soundness certification.

At most two active Cargo target roots were used: the normal workspace target and its llvm-cov target. The stale M1 coverage target and invalid mixed-feature build were retired. Verification logs, exact raw coverage JSON, source SHA-256 manifest, lock audit, server results and initial failed efforts are retained at `/private/tmp/stackpress-s4-lane-a-m2-evidence/`. Build artifacts are disposable and removed at closeout as requested; evidence remains outside the target tree.

## Remaining boundaries and next gate

- Normal core stays driver-free. There is no production PostgreSQL/MySQL connection/session implementation, serverless/PGlite adapter, network fixture outside the explicitly gated tests, or native guarantee for untested server versions.
- This is the installed macOS ARM64 Rust target only; no Linux/Windows/WASM/mobile/network-filesystem verification is inferred. Network server versions differ from the host compiler architecture and are recorded separately.
- Raw SQL/type/default escape hatches remain audited trusted code, not sanitization. No arbitrary public QueryPlan batch constructor is exposed. PostgreSQL primary constraint removal uses the renderer's deterministic `<table>_pkey` name; arbitrary manually named native primary constraints or renamed-table constraint-name history require explicit migration metadata outside this API.
- Physical field reordering, new identity fields requiring coordinated keys, identity-policy conversion and prefix/expression indexes needing richer metadata return Unsupported. They are not silently approximated. SQLite table reconstruction and broader migrations remain S4-M.
- No dedicated milestone refactoring rounds were run: the coordinated milestone gate requires stopping before their separate release. Correctness fixes and test strengthening above do not claim the later joint rounds.
- Baseline Agent Workspace validation is **146 pre-existing errors and 20 warnings**, primarily absent sibling-repository links relative to the worktree. No existing KB document was modified; the post-write diagnostic comparison is reported in the final handoff.

Stop for coordinator review and lane B plan-execution integration. M3, shared merge, task/status changes, acceptance and release remain with the coordinator.
