# Task 00012: SQL M1 implementation receipt

M1 only, 2026-10-27. Branch `trial/3-s7-sql-m0`, HEAD/base `aac17e22ef9173a15e20f2c261d7d0479a1aba09` (coordinator's accepted M0).
Implementation evidence for [task 00012](00012-slice-7-production-servers.md). Independent coordinator review remains pending; no acceptance or Context promotion
is claimed. M2 pool stress, backend contract harness and generated-store integration
were not started.

## Implemented behavior

Only `crates/sql/sql-postgres/**` and `crates/sql/sql-mysql/**` changed. SQL core, SQLite, other lanes, root Cargo/facade and Cargo.lock are unchanged. SQLx stays
unmodified at `=0.9.0`; no vendoring, patch, dependency or tool install. JSON adds only serde_json's raw_value feature, preserving wire numeric tokens locally.

- Strict native scalar/typed-NULL codecs, finite real checks, PostgreSQL width
  checks and MySQL unsigned overflow checks. JSON/JSONB decode to compact,
  recursively key-sorted Text with normalized strings, exact numeric tokens,
  duplicate-key rejection and a 128-level traversal bound. SQL NULL differs from
  JSON null. Raw JSON input and retained output are budgeted before ownership;
  native packet and transient parser allocations are not a hard heap guarantee.
- Native prepared metadata independently checks arity on the same lease. Outcomes
  come only from that statement's native completion: PostgreSQL insert_id None
  and RETURNING rows; MySQL checked ID, zero -> None, no sticky last-ID query.
- Successful PostgreSQL completions reuse only after separate DISCARD ALL outside
  a transaction, SQLx statement-bookkeeping clear, and acknowledged restoration
  of application_name/standard_conforming_strings. Native identity remains the
  same while temp tables, session settings, advisory locks and rows are cleared.
- MySQL uses the owner-approved conservative lexical allowlist because Query has
  no provenance and SQLx has no public general reset. Only plain SELECT/INSERT/
  UPDATE/DELETE with listed safe calls qualify. Variables, session constructs,
  unknown/qualified/quoted calls, CTEs, DDL and ambiguity retire. UPDATE SET is
  ordinary DML. The acknowledged ROLLBACK safety command is not a general reset.
  Hidden session-mutating triggers are outside this lexical rule; no catalogs
  are inspected. Rules and exact allowed calls are public in the codec docs.
- Transactions reuse only after acknowledged completion and exclusively successful
  permitted runs. Any failed/cancelled run poisons before suspension, remains
  rollback-only and retires even after an explicit successful rollback. A
  disallowed MySQL run disqualifies the entire transaction after either completion.
- Errors/cancellation/reset failure retire uncertain state. Reset and retirement
  each have the configured finite cleanup deadline; cancellation during reset
  retains Lease ownership. Consuming native close is polled once before private
  test pausing, ensuring timeout drops its owned raw socket/capacity guard.
  Background retirement holds admission and is observed by re-awaitable close.
  Primary failures retain their typed cause and separate cleanup_error.
- Cancellation loses observation. A write or COMMIT can succeed without a result;
  reset/cleanup failure can also follow a successful write. No automatic retry,
  duplicate write, certain-rollback claim or immediate remote-termination promise.

Owner clarification: results are decoded by their wire type; domains over supported base types decode as that base type. PostgreSQL 12 RowDescription flattens the domain to base OID 23 in the native characterization. Advertised
custom/domain/unknown OIDs still fail, even for NULL. No catalog lookup was added.
The precise rule is in each applicable codec matrix/Rustdoc; the coordinator owns any subsequent change to the protected accepted-design matrix.

Crate documentation: [PostgreSQL codec and lease policy](../../../../crates/sql/sql-postgres/CODECS.md)
and [MySQL codec and lease policy](../../../../crates/sql/sql-mysql/CODECS.md).
These same files are included in public crate Rustdoc. Existing ownership
compile-fail examples remain valid.

## Native proofs and review rounds

Each backend's four M1 child groups cover:

- Values: supported widths/families and typed NULLs; i64/u64 boundaries; BOOL
  0/1/2; UTF-8, empty text/binary and arbitrary blobs; JSON SQL/null, nested order,
  large numeric digits, invalid/duplicate JSON and caps; canonical UTC text;
  unsupported numeric/date/UUID/array/enum types; duplicate columns; missing/extra
  binds, explicit casts and literal question marks. MySQL strict-mode overflow
  rejects native nonfinite arithmetic; its exact decoder gate separately rejects
  NaN/Inf. PostgreSQL actually returns native NaN/Inf for decoder rejection.
- Outcomes: insert -> update -> select/DDL/delete sequences, concurrent exclusive
  lease IDs/results, PostgreSQL RETURNING, MySQL ID overflow after exactly one
  successful write, transactional DDL rollback on PostgreSQL and pre-effect DDL
  rejection after MySQL DML; prior DML rolls back and no DDL table appears.
- Reuse: same native identity on acknowledged clean completion; full row draining;
  no leftover temp/session/advisory-lock/open-TX state; fresh ordinary autocommit
  observed by an independent owner. Raw MySQL SET is rejected by core; private
  fixture injection proves dirty retirement and absent @variable on the next lease.
  Temp DDL, stateful functions, both dirty-TX completions and errors retire.
- Lifecycle: before/after prepare, execute/row, BEGIN, COMMIT and ROLLBACK gates;
  real slow pending execution; cancellation poisons TX; unpolled completion;
  acknowledged-but-unobserved and real polled native COMMIT permit outcomes 0/1
  without duplication; cleanup failure/deadline, reset failure/deadline after a
  write, primary Driver/Decode/Limit vs cleanup precedence, retained admission,
  cancelled cleanup/close and re-await. M0 additionally replays pool exhaustion,
  outside-runtime Drop and the riskiest owner/lifecycle proofs.

Private gates are compiled only into test targets, which use the same production
modules through a separate test root; no driver or probe API escapes. Adversarial
close tests withhold first-party acknowledgement after one real close poll. They
prove deadline/ownership/error handling, not a malicious TCP peer stalling close.
All hang-class groups run under a parent watchdog (30 s child, 120 s total), with
kill/reap. A separately bounded admin worker closes its own socket before exact
DROP (9 s IO/10 s wall deadline); PostgreSQL retries only its owned DROP on 55006.
Failure/unwind remains failing. No database adoption, catalog listing, global
setting/role mutation, unrelated session termination, or prefix sweeping occurs.

1. Logic/complexity: reviewed allowlist token boundaries, qualified calls across
   comments, transaction poison order, reset/retirement guards and error precedence.
   Removed an unreachable MySQL JSONB arm; expanded native family, clean rollback,
   dirty completion and reset-deadline cases. Offline/native replay passed.
2. Responsibility: confirmed separate options, owner/admission, execution, codecs,
   lease cleanup, reuse policy and session ownership. Extracted owner code into
   database.rs so the private native root compiles the actual implementation
   without duplicate Cargo-target warnings. Further cross-backend abstraction
   would cross the core write boundary; no additional extraction was justified.
3. Consistency/docs: aligned timeout documentation and public codec/reuse/cancel
   contracts, preserved root imports and compile-fail ownership examples, and
   corrected the domain wording. Tests, doctests, fmt, Clippy and Rustdoc pass.
4. Safety/follow-up: reviewed SQLx pool Drop/close ownership and no-retry outcomes;
   preserved M0's outside-runtime disposal. Full workspace testing found global
   serde arbitrary_precision changed Idea fixture number representation. Removed
   that feature and implemented bounded raw-subtree canonicalization instead;
   added huge/exponent-token, escaped-key-duplicate and depth-boundary regressions.
   The exact Idea fixture regression then passed, followed by full workspace/native
   replay. No SQL core or parser edit was needed.

Early native failures found a fixture's signed connection-ID decoding mismatch,
PostgreSQL DISCARD combined with SET causing an implicit transaction, and the
incorrect domain-rejection expectation. Fixed the fixture, separated reset
commands, and applied the owner's wire-type decision. Every failed fixture also
has an exact DROP confirmation below. Logs are retained, including the failed
workspace run, rather than replaced by only successful evidence.

## Verification

Rust/Cargo 1.96.0 (`ac68faa20`), `aarch64-apple-darwin`; native servers PostgreSQL
12.2 and MySQL 8.0.27. Cargo commands use --offline --locked, one worktree target, CARGO_INCREMENTAL=0 and CARGO_PROFILE_DEV_DEBUG=0. Logs/evidence reside at
`/private/tmp/s7-sql-m1/`; `checks.py` contains the exact final matrix commands.

| Check | Result / log |
| --- | --- |
| `cargo +1.96.0 fmt --all -- --check` | Pass, fmt-final.log |
| scoped `clippy --all-targets -- -D warnings` | Pass, clippy-final.log |
| scoped `test`, gates unset | 17 units + 8 doctests pass; 10 native cases explicitly ignored; offline-final.log |
| scoped `test`, both exact native gates set | M0 + 8 M1 groups + units/doctests pass; native-final.log; exact final code replays in coverage.log |
| `RUSTDOCFLAGS='-D warnings' cargo doc ... --no-deps` | Pass, rustdoc-final.log |
| `cargo +1.96.0 check -p stackpress --no-default-features --features ...` | Each backend and both pass; check-facade-{postgres,mysql,both}.log |
| `cargo test --workspace --all-targets -- --test-threads=1` | 487 pass, 69 expected gated/child-only ignored; workspace-replayed.log |
| `cargo test --workspace --doc` | 875 pass; workspace-doc.log |
| normal/feature dependency trees | Default/browser no SQLx/driver edge; each adapter only its backend; tree-*.log, features-*.log |

No SQLx macros, migrate, Any, chrono or decimal features; no arbitrary_precision feature remains. Root manifests, facade and lockfile have no diff. The initial
workspace failure is preserved in workspace-final.log; json-feature-regression.log
proves its exact Idea fixture passes with adapters selected after the fix.
`/usr/bin/time -l` around the native test/build command recorded 314,654,720 bytes
maximum RSS; this is whole-command observation, not native packet isolation or a
hard memory-cap proof. Large-cell tests reject 10,000-byte cells at a 20-byte cap.

Coverage: cargo-llvm-cov 0.8.6; explicitly selected installed LLVM 22.1.2 tools
from stable 1.96.0, compiler +1.96.0. Fresh `cargo clean` and `llvm-cov clean
--workspace` preceded scoped `llvm-cov --json --summary-only` with both gates set,
CARGO_LLVM_COV_TARGET_DIR=target. Exclusion regex is
`/tests/|/src/(tests|probe|m1)\.rs$` (test fixtures/root/probes only). No production
file is excluded; subprocess profiles are included. Doctests passed separately.

| Crate | Covered executable lines | Coverage |
| --- | --- | --- |
| stackpress-sql-postgres | 737 / 811 | 90.88% |
| stackpress-sql-mysql | 887 / 963 | 92.11% |

Both exceed the 70% development floor. This is scoped line evidence, not publication or full branch/platform support. Reports: coverage.json and coverage-summary.json.
Uncovered paths include defensive impossible native metadata/pairs, allocation/
encoding failures, some invalid configuration and driver error classifications.
TLS/RSA authentication modes, external poolers, other backend versions/targets,
malicious-peer transport stalls and M2 integration/stress remain unverified here.

## Exact owned database ledger

Each entry was CREATEd without IF NOT EXISTS and has an acknowledged exact-name DROP in the listed log, including failure/unwind cases. Gates were opt-in only; credentials/URLs are absent from this receipt. No other database was touched.

| Backend | Database | Log / confirmation |
| --- | --- | --- |
| mysql | `stackpress_s7_m0_15341_18e22424a3ab5d10` | coverage.log / dropped |
| mysql | `stackpress_s7_m1_15343_18e22424e90719d0` | coverage.log / dropped |
| postgres | `stackpress_s7_m0_15349_18e224254ebff2d8` | coverage.log / dropped |
| postgres | `stackpress_s7_m1_1535a_18e22425b0c247d8` | coverage.log / dropped |
| mysql | `stackpress_s7_m1_1650e_18e222e90609a350` | native-baseline.log / dropped |
| postgres | `stackpress_s7_m1_16515_18e222e95fa14aa8` | native-baseline.log / dropped |
| mysql | `stackpress_s7_m1_7045_18e2236399e4dcf0` | native-domain-accepted.log / dropped |
| postgres | `stackpress_s7_m1_704a_18e22363ff16aba8` | native-domain-accepted.log / dropped |
| mysql | `stackpress_s7_m0_dc65_18e223df13e6e950` | native-final.log / dropped |
| mysql | `stackpress_s7_m1_dc67_18e223df5c89bea8` | native-final.log / dropped |
| postgres | `stackpress_s7_m0_dc6d_18e223dfa30c30b8` | native-final.log / dropped |
| postgres | `stackpress_s7_m1_dc98_18e223dffcfe09e8` | native-final.log / dropped |
| mysql | `stackpress_s7_m1_150d3_18e222dbd84d3358` | native-first.log / dropped |
| mysql | `stackpress_s7_m1_1f40_18e223115c6abd10` | native-reset-fixed.log / dropped |
| postgres | `stackpress_s7_m1_1f48_18e22311bc06fb08` | native-reset-fixed.log / dropped |
| mysql | `stackpress_s7_m0_7c19_18e2238a2aae78d8` | rounds-native.log / dropped |
| mysql | `stackpress_s7_m1_7c1b_18e2238a75d80c98` | rounds-native.log / dropped |
| postgres | `stackpress_s7_m0_7c21_18e2238a9dee7460` | rounds-native.log / dropped |
| postgres | `stackpress_s7_m1_7c4a_18e2238af5cf76c0` | rounds-native.log / dropped |

## Closeout boundaries

The single target directory and stray adapter coverage profiles were removed; clean-before-coverage.log, cov-clean.log and clean-final.log retain the evidence.
source-sha256.json records all adapter source/docs/manifests; database-ledger.json records all 19 created names and drops. No commit, push, publish, listener or tool
install occurred. Only this separately approved M1 receipt is added under .agents.

The KB validator baseline reports 146 existing missing sibling-source link errors
and 21 warnings. No unrelated KB repair is authorized. Post-write validation must
preserve that issue set. Coordinator independent review is the next gate; stop at
M1 and do not infer M2 release or product acceptance from this receipt.
