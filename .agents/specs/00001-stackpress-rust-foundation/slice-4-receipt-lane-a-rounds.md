# Slice 4 lane A refactoring and safety rounds receipt

Task 00007, `stackpress-sql`, rounds 1–4 and targeted follow-ups, 2026-10-26. Base `104d96d`, branch `trial/3-s4-lane-a-core`. Coordinator implementation evidence; no commit, merge, acceptance or publication is claimed. This follows the [refactoring round checklist and receipt template](../../references/00037-refactoring-round-checklists.md) and [M3 receipt](slice-4-receipt-lane-a-m3.md).

## Intended behavior and baseline

Preserve the accepted six builders, typed values/JSON, three renderers, explicit schema differences, owned results and exclusive transactions. Public declarations/signatures are frozen. Core keeps zero normal dependencies. Unsupported/invalid operations reject before effects; native adapters still own IO, preparation/arity and result collection.

Clean worktree/base confirmed first. Baseline core: **69 passed, 0 failed, 16 visibly skipped opt-in network cases**, using only this crate's tests. Current host/compiler: Rust/Cargo **1.96.0**, LLVM **22.1.2**, `aarch64-apple-darwin`; cargo-llvm-cov **0.8.6**. No toolchain, service or dependency was installed.

## Round 1 — logic and complexity

Reviewed render/validation sequencing, mutation scope, rename reconciliation, schema comparison, native case matching, bind order, transaction error precedence and cancellation. Kept exhaustive expression/DDL matches and the two-phase reconciliation because they expose the actual decisions.

- **Defect fix: target column ordinals.** Native ADD appends; a diff previously claimed to fulfill a target inserting a new field in the middle. `SchemaDiff::render` now simulates append/remove/rename effects after reconciliation and rejects an unrealizable order before exposing a plan. Explicit rename still retains an earlier ordinal. Regressions: `diff_rejects_unrealizable_new_field_ordinals`, `reconciled_rename_preserves_middle_field_ordinal`; both failed before the fix.
- **Defect fix: PostgreSQL quoted source case.** Rename/change source metadata matched case-insensitively while the SQL quoted exact case. Native PostgreSQL lookup now requires exact case; SQLite/MySQL column matching retains its own semantics. Regression: `postgres_alter_source_names_preserve_quoted_case`, failed before the fix.
- Named the two pending rename predicates and expanded the removal-position scan so missing-half decisions and second-pass skips can be inspected directly. No change to reconciliation/error ordering for valid inputs.

Verification after this round: **72 passed, 0 failed, 16 network cases skipped** (`round1.log`). Original failing regressions are retained separately.

## Round 2 — responsibility and abstraction

- Moved the pure lexical admission gate into private `statement.rs`; query ownership/budgets remain in `query.rs`. No public parser/export was introduced. Raw input validates identity/budgets/lexical shape before cloning, preserving mismatch precedence while avoiding a copy of invalid oversized input.
- Centralized PostgreSQL/MySQL native name matching. JSON native-content validation now returns the already validated encoding for reuse by JSON expressions and writes; MySQL path encoding is reused too. PostgreSQL's bounded NUL walk follows encoding rather than doing a second encode.
- `Alter::build` copies retained metadata once and returns normalized operations directly, avoiding a clone of operations that it immediately discarded.
- Retained cohesive DML, schema and plan responsibilities; no generic framework, trait hierarchy or normal dependency was added.

Verification: **72 passed, 0 failed, 16 network cases skipped** (`round2.log`). Native snapshots, lexical adversarial tests, JSON type/path tests and pure data inspection protect the extraction.

## Round 3 — consistency, Rustdoc and comments

Corrected DateTime reference text: SQLite TEXT, PostgreSQL/MySQL VARCHAR(30), with explicit UTC validation rather than an implicit codec. Its doctest now checks VARCHAR(30). Documented ordinal reconciliation, stable adapter dialect identity, raw bind representation checks and the distinction between per-statement budgets and retention of a whole plan's outcomes. Explained UTF-8-safe lexical slice boundaries, pre-copy validation and incremental rendering locally. Public signatures, re-exports and the M3 first-success path remain intact.

Documented where recursive limits apply: rendering/encoding validates bounds; arbitrary user construction and derived recursive operations do not invoke those validators. The safety follow-up adds an intended compile-fail proof that exclusive `Transaction` is not Sync, while positive tests prove it is Send.

Round verification: **72 tests passed**, **416 doctests passed**, warning-clean core Rustdoc (`round3-*`). Following later safety fixes, the final checks rerun every affected test, all doctests and warning-clean docs. Final core doctests: **417 = 414 runnable + 3 intended compile-fail**, 0 ignored/no_run.

## Round 4 — safety and soundness

Reviewed the following boundaries in production source and their tests; this is a scoped code review, not a formal soundness proof:

| Boundary | Evidence / conclusion |
| --- | --- |
| Identifier integrity | All dialect quoting retains literal punctuation and explicit qualification; empty/control/NUL and dialect length limits reject. Existing `hostile_identifiers_and_values_stay_data`, `ddl_hostile_input_dialect_and_clone_independence`, `network_raw_lexical_gate_and_identifier_bounds`; new UTF-8 and casing regressions. |
| Ordinary binds | Values remain separate from SQL; checked byte/count budgets and finite floats apply before IO. Native arity remains adapter-owned. `raw_dialect_error_precedes_nonfinite_bind_error`, `raw_postgres_text_nul_is_rejected_during_rendering`, `plan_and_bind_cardinality_boundaries`. |
| TrustedSql | An explicit authorship assertion, not sanitization. Raw code retains one dialect and lexical admission; custom type/default escape hatches retain their bounded audited surface. Existing lexical/trusted-expression/trusted-DDL tests cover batch, quote/comment, NUL and wrong-dialect failures. No interception/rewrite hook was added. |
| Recursion/cardinality/size | `expression_and_json_exact_depth_and_node_limits` proves depth 64 vs 65 and total nodes 10,000 vs 10,001; `exact_sql_and_json_byte_budgets` checks delimiter-inclusive byte boundaries; plan/field/bind limits and existing JSON path/index/DDL bounds remain tested. |
| Integer/float edges | `numeric_edges_preserve_exact_owned_values` checks signed zero, subnormal/min/max finite floats, nonfinite rejection, i64/u64 boundary and narrowing failures. Existing strict conversion/date tests cover calendar, timezone, type mismatch and metadata decoding. |
| Public panic paths | No explicit production unwrap/expect/panic/unreachable/unsafe sites. Byte/index slices have guards; `hostile_utf8_boundaries_do_not_panic` exercises malformed UTF-8-containing SQL/date/path text, controls and usize::MAX ordinal through safe public APIs. Allocator exhaustion or panics in caller-supplied trait implementations are not caught or certified. |
| Send/Sync and ownership | `public_auto_traits_match_ownership_contracts` checks every concrete owned public data type/result and Engine as Send+Sync; Transaction, session objects and SqlFuture as Send. The Transaction compile-fail doctest proves no shared concurrent guard access. Database remains Send+Sync, session Send; no trait bound changed. |
| Plan atomicity/errors/cancellation | Existing success/Nth-statement failure/rollback-failure/mixed-dialect/upfront-budget/cancelled-owner tests remain. `caller_owned_plan_retains_guard_after_failure_or_cancellation` additionally proves a borrowed plan future retains caller ownership until explicit rollback and keeps the original source. SQLite native public-plan proofs run in the workspace suite. No implicit retry was introduced. |
| Schema/constraint effects | Ordered remove/rename/add and native capability checks remain explicit. Known foreign nullability is carried through declared operations; native catalog introspection and arbitrary external schema history remain outside core. |
| Dependency/platform boundary | Source declaration audit confirms the frozen public shape; no manifest/lock diff; normal-edge tree contains only stackpress-sql. Host/driver/OS concerns stay outside the production core. |

### Material findings and targeted follow-ups

- **Defect fix: inconsistent ALTER metadata matching.** The common reconciliation pass used exact case to update its field state after accepting a case-insensitive rename/remove. It now uses its established matching rule; native rendering retains PostgreSQL's exact-case rule. `alter_case_insensitive_metadata_follows_rename_and_removal` failed before the fix and proves both accepted rename/FK state and rejected removed-field references.
- **Resource hardening:** INSERT row/VALUES bodies, UPDATE assignments and network CREATE bodies now use the existing incremental SQL join budget. Network DDL freezes each CREATE statement or ALTER operation before generating the next, rather than collecting all unchecked strings first. Valid SQL/bind order is preserved. Oversized input stops earlier with Limit, before later invalid clauses expand; `sql_budget_stops_later_clause_expansion` failed before this change. Error precedence can therefore differ for an input that is already over budget and also has a later invalid clause.
- **Defect fix: ALTER SET NULL versus primary/NOT NULL effects.** CREATE's primary rule was missing in ALTER. Validation now tracks known field nullability, primary additions, explicit nullable changes, renames and newly added/removed foreign actions. Dropping a primary alone retains native NOT NULL; an explicit nullable change can restore a valid target. `alter_primary_rejects_set_null_relationships` failed before the fix; `primary_removal_requires_explicit_nullable_change_for_set_null` protects the valid restoration path. This validates declared knowledge, not unprovided live schema metadata.
- **Defect fix: raw PostgreSQL NUL binds.** Raw SQL bypassed the representation check used by typed builders. One private value validator now serves both paths. The raw NUL regression failed before the fix; PostgreSQL rejects it during rendering while SQLite/MySQL retain exact bound text.

Targeted logic/responsibility reviews were repeated after these findings. Final round-4 core result: **85 passed, 0 failed, 16 skipped**, plus **417 doctests**. No material finding remains in the reviewed scope. The NULL restoration sequence was additionally checked on both real servers in the final run below. Later edits were verified; no ceremonial restyling or unrelated crate edits were made.

## Final checks and coverage

All normal checks use one target root and `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`, with both network URL variables unset:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked --offline
cargo test --workspace --doc --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --offline
cargo +1.96.0 check --workspace --all-targets --locked --offline
cargo tree -p stackpress-sql --edges normal --locked --offline
git diff --check
```

Passed: **411 workspace tests, 0 failed, 49 ignored** harness entries; **854 workspace doctests, 0 failed/ignored**. Final core-only tests remain **85 passed, 16 network cases visibly skipped**; core doctests **417 passed**. The additional live-fixture assertion was compiled, Clippy-checked and reverified with the core suite after the broad run. All four standalone examples were also actually run successfully. Public declaration and generated-HTML audits remain clean. No normal dependency, manifest or Cargo.lock change occurred.

Fresh core-only coverage, after cleaning that same target root, excludes only test paths:

```sh
cargo clean --target-dir target
CARGO_TARGET_DIR=target CARGO_LLVM_COV_TARGET_DIR=target \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 \
cargo llvm-cov -p stackpress-sql --locked --offline --fail-under-lines 70 \
  --ignore-filename-regex '/tests/' --json \
  --output-path /private/tmp/stackpress-s4-lane-a-rounds-evidence/core-coverage.json
```

**2,941/3,082 executable library lines = 95.43%**, above the 70% development floor. Only this crate's tests contribute; no adapter tests, live-network run or stable doctest coverage is folded into it. The new private lexical module is included in production coverage. This is executable line coverage, not exhaustive branch/platform certification. Remaining gaps are concentrated in optional native DDL type/default/comment/error variants (318/403 native-DDL lines covered); other notable gaps include a few JSON validation errors and adapter forwarding failures. All new production modules remain in the denominator; no coverage threshold/exclusion was weakened.

## G26 final live renderer validation

After all normal/instrumented checks, the final corrected run uses the existing guarded dev-only harness with both variables set:

```sh
STACKPRESS_TEST_POSTGRES_URL=postgres://postgres@localhost:5432/postgres \
STACKPRESS_TEST_MYSQL_URL=mysql://root@localhost:3306 \
cargo test -p stackpress-sql --test network --locked --offline
```

Final result: **16 passed (8 PostgreSQL + 8 MySQL), 0 failed, 0 skipped**. The existing ALTER case also verifies on both servers that an explicit nullable change after primary removal permits a NULL default row under a SET NULL foreign key.

- PostgreSQL: **12.2 on x86_64-apple-darwin17.7.0, Apple LLVM 10.0.0 (clang-1000.11.45.5), 64-bit** (`SELECT version()`).
- MySQL: **8.0.27** (`SELECT version()`).
- Final owned PostgreSQL database: `stackpress_test_afe5_18e20563ccf76810` — DROP acknowledged.
- Final owned MySQL database: `stackpress_test_afe5_18e205693b4bcc98` — DROP acknowledged.
- Initial attempt: **15 passed, 1 failed, 0 skipped**. The new nullable-row assertion hit the dev-only PostgreSQL fixture's generic NULL-as-text binding (`42804`, text versus BIGINT). It now uses server DEFAULT VALUES and checks the resulting NULL row; production renderer code did not change for that fixture correction. Both initial owned names, `stackpress_test_83e8_18e20541f4b290b8` and `stackpress_test_83e8_18e205420ded9488`, were also dropped. Initial and final logs remain separate.

**Leftovers from both attempts: none.** All four CREATE acknowledgements have matching successful owned-name DROP acknowledgements. A second attempt was necessary to verify the corrected new fixture; the initial failure is not counted as a passing run.

Only generated throwaway databases are created, selected and dropped; the existing Drop guard remains armed on failure. No other database, role or setting is read/altered/dropped; no catalog scan is used for cleanup. Network driver dependencies remain dev-only. The self-contained run reports **0 network cases passed, 16 skipped** visibly; the live result above is counted separately.

## Evidence and remaining limits

Evidence is retained at `/private/tmp/stackpress-s4-lane-a-rounds-evidence/`: exact commands/results, round tests, expected pre-fix failures, source hashes/declaration audit, coverage JSON, native versions/owned-name cleanup and KB validation. One target directory was reused sequentially and cleaned between normal/instrumented/native phases, then deleted after final checks. Eleven stale own profiles from the prior milestone's instrumented target were removed before the fresh final build; their filenames are recorded, not included as source or coverage evidence.

No unsafe or listener was introduced. Limits remain per statement; a complete plan/outcome vector retains multiple statements/results, and caller-constructed recursive trees/derived operations do not provide an independent process heap/stack quota. Core does not parse all native SQL grammar, introspect arbitrary existing constraints, supply production network adapters, or promise SQLite reconstruction/field reordering. Trusted escape hatches require trusted authorship. Installed-target evidence does not establish Linux/Windows/WASM/mobile support. Commit cancellation remains unknown outcome; applications must not replay blindly.

Agent Workspace baseline: **146 existing errors, 21 existing warnings**, primarily missing sibling-repository paths. This receipt is under 200 lines and adds no expected diagnostics. Post-write validation must retain the exact baseline error/warning sets; no unrelated KB repair is part of this lane.

Stop for coordinator re-verification of rounds 1–4 and scoped defect fixes. No commit or agent message was performed.
