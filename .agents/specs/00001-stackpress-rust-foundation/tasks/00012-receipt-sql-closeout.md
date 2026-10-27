# Task 00012 SQL closeout receipt

Date: 2026-10-27. Implementer branch: `trial/3-s7-sql-m0`.
Baseline: authorized `git merge trial/3` fast-forwarded `32ae178` to `8e84ae7`.
Scope: both SQL adapters and their existing shared test harness, after the
[generated-store integration](00012-receipt-sql-store-integration.md).
Status: implementation and verification complete; awaiting coordinator review.
No commits, public API changes, core/framework edits, driver forks, installs or listeners.
The classifier change below is the coordinator-released safety exception to the
otherwise behavior-preserving closeout. The allowlist/domain rules remain coordinator decisions.

## Four whole-crate review rounds

Reviewed all 18 production modules plus test/probe/contract/store support in both
crates (57 Rust source files under their src directories at final inventory).
Evidence: `/private/tmp/s7-sql-closeout/`; each named command has a separate `.log`
and `.json` with exact argv, exit status and elapsed time.

1. **Complexity:** reviewed opening/admission, codecs/JSON, statement/control flow,
   lease disposal, session poison/completion and shared generated-store sequences.
   Split the compressed MySQL pool builder/session initialization into a readable
   builder and private `initialize_session`; preserved the exact SET SESSION command,
   whole-open timeout, categories and side-effect order. Added first-poll input-limit
   refusals while the pool is exhausted and after close; network dialect refusals
   also remain pre-acquisition. Existing generated-store zero-entry checks remain.
   `round1-offline-replay` passed in 17.556s; native M0/M1/M2 passed in 21.822s.
2. **Responsibility/abstraction:** extracted the four identical M1/M2 child watchdogs
   into test-only `sql-postgres/src/tests/support/process.rs`; extracted the two
   identical bounded M2 safe cause walkers into `support/diagnostics.rs`. Backend
   fixture creation, cleanup authority and admin deadlines stay backend-owned.
   Library-only success/timeout controls prove timeout is failure and kills/reaps.
   Production lease/session duplication stays explicit: typed native cleanup differs.
   JSON duplication alone does not justify a new shared crate or production sibling
   source dependency. No such dependency was added. `round2-offline` passed in 17.933s.
3. **Consistency/Rustdoc/comments:** checked every production error wrapper against
   #11: static safe context, typed category, retained cause via `SqlError::with_source`,
   no interpolated native Display. No error-context defect required a behavior change.
   Added ten compiled examples across the two crates for TLS selection, option
   budgets, construction, validation and Engine opening; existing compile-fail
   contracts remain. Used the repository's `chrisai-docing` skill. Block/line comments
   explain state transitions, timeout scope and source-sharing boundaries. Updated
   the support README. `round3-offline-final` passed in 111.678s and
   `round3-clippy-final` passed in 31.535s; strict Rustdoc and doctests passed.
4. **Safety/soundness:** reviewed ownership across cancellation and unpolled futures,
   cleanup registration/permits, retirement flags, transaction poison/error precedence,
   wire metadata, query/bind separation, native disclosure and lexical reuse. No unsafe
   code was found. The MySQL fragment defect below required the released safety fix.
   Final offline, lint/doc checks and all native milestone suites passed afterward.

Two intermediate test-authoring corrections are retained in logs: the first closeout
check incorrectly demanded pre-acquisition dialect refusal from SQLite, whose existing
check runs during execute; the corrected suite preserves that reference boundary.
The first extraction put test controls into harness=false targets, causing dead-code
Clippy failures; moving controls into library-only tests fixed them without lint suppression.
Neither correction changed production behavior. The native regression failure below was real.

## Released safety fix: MySQL whole-token reuse classification

The actual core gate accepts `éABS(...)`, `ABSé(...)` and `1ABS(...)`; the old scanner
misidentified an embedded ABS fragment as reusable. A pure characterization established
all three as accepted/reusable. More importantly, a newly created MySQL database with a
test-owned `éABS` stored function reproduced a user-variable leak to the next lease.
`classifier-offline-before` failed (101); `classifier-native-before` failed (1), and
its parent still confirmed the exact database DROP. This is a safety fix, not a refactor.

`reuse.rs` now retires unquoted non-ASCII tokens, consumes whole digit-prefixed
identifiers, and refuses adjacent quoted/unquoted identifier fragments. The sweep
covers longer identifiers, leading digits, quoted functions, schema-qualified calls,
backticks before embedded names, dots and comment/space trivia. Existing core rejection
of dollar/colon fragments is characterized without a core change. Unicode literals,
quoted column/table names, binds and ordinary numeric expressions keep their support.

Offline assertions cover 13 distinct retiring forms and four reusable controls. Native
M1 installs five routines only inside its parent's throwaway database and runs six
function-call forms across ordinary completion, acknowledged commit and rollback:
18 cases. Each verifies its result, a different next native connection identity and
no routine-written session variable. `classifier-native-after` passed all M1 groups;
`round4-native-coverage` repeated them after the final source edits. No global setting,
catalog lookup, production API, codec or SQL execution refusal was added. Unknown SQL
continues to execute under the existing contract; its connection is conservatively retired.
The lexical policy still assumes reusable DML has no hidden session-mutating triggers.

## Verification commands and results

All Cargo commands use `+1.96.0`, `--offline --locked`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, two build jobs and the configured worktree `target`.
Missing native gates produce reported ignored groups. Native runs set only the two
existing gates privately; credentials/URLs and native SQL diagnostics are not logged.

| Evidence label | Command scope | Result |
| --- | --- | --- |
| baseline-offline | cargo test -p stackpress-sql-postgres -p stackpress-sql-mysql | pass, 96.655s |
| fmt-final-safety | cargo fmt --all -- --check | pass, 2.396s |
| offline-final-safety | cargo test -p stackpress-sql-postgres -p stackpress-sql-mysql | pass, 43 checks: 24 unit, SQLite reference, 18 doctests; 16 ignored child/native cases |
| clippy-final-safety | cargo clippy -p both adapters --all-targets -- -D warnings | pass, 5.675s |
| rustdoc-final-safety | RUSTDOCFLAGS=-D warnings cargo doc -p both adapters --no-deps | pass, 2.244s |
| check-facade | cargo check -p stackpress --features sql-postgres,sql-mysql | pass |
| workspace-final | cargo test --workspace --all-targets -- --test-threads=1 | pass, 679 tests / 80 ignored, 626.319s |
| workspace-doc | cargo test --workspace --doc | pass, 905 doctests, 48.495s |
| round4-native-coverage | cargo llvm-cov --no-report -p stackpress-sql-postgres -p stackpress-sql-mysql, both gates | pass, 116.872s including compilation |
| native-doctests | cargo test --doc -p both adapters, both gates | pass, 18 doctests, 3.941s; no databases created |

The workspace all-target invocation compiled before the classifier fix (679 includes the
four watchdog controls); final affected tests, doctests, Clippy, Rustdoc and native coverage
were rerun after that fix. The workspace doctest invocation completed afterward. A green
pre-fix workspace run is not claimed as a post-fix workspace run. No shared consumer API changed.

Native final coverage executes both M0 parents, eight M1 groups, four network M2 groups
and SQLite reference, including generated stores, preflight, cancellation and close.
Final M2 stress: MySQL 4,254/6,062/7,469 requests at pool sizes 1/2/4; PostgreSQL
2,218/5,469/7,387. Each size runs about two seconds. Observed peak native/identity counts
stay within capacity, admission peaks at eight, clean runs retire zero leases, and all
permits return. Adversarial dispositions remain MySQL 78/24/54 and PostgreSQL 78/48/30
(acquired/reused/retired). Overflow returns in 3.375/4.291 microseconds; acquire timeout
is 200.342/202.093ms, distinct from Closed; 64 cancelled waiters leave no permits/leases.
Final close has native=idle=cleanup=0. Native stress child times: 6.357s MySQL, 6.777s PG.
Parent watchdogs kill/reap; cleanup has independent deadlines and timeout is failure.

`tree-default` and `tree-browser` have no PostgreSQL/MySQL driver edges. Opt-in trees
have only their selected driver (`sqlx-postgres` or `sqlx-mysql`). These are normal
dependency graphs, not a browser adapter execution proof. Root manifests, lockfile and
facade are unchanged; `public-api-audit.json` reports no exported declaration changes.

## Coverage and publication gaps

Compiler/target: Rust 1.96.0, aarch64-apple-darwin. Installed cargo-llvm-cov with matching
LLVM 22.1.2 tools; both selected adapters and native servers, default crate features.
Export all instrumented sources, normalize physical paths first (including path-included
test helpers), then aggregate every production src module in each adapter. Exclude only
test modules/runners/probes, dependency/build output and unrelated crates. No production
branch/file exclusion or threshold-only tests. Doctests run separately from coverage.

| Crate | Covered executable lines | Coverage |
| --- | --- | --- |
| stackpress-sql-postgres | 777 / 839 | 92.61% |
| stackpress-sql-mysql | 928 / 999 | 92.89% |

Both exceed 70%, and already exceed the numeric 90% publication floor in this measured
scope. No extra lines are needed merely to cross 90 here. Publication would still need:

- Repeat the declared publication feature/version/target and release integration scope;
  this combined native test scope does not certify every independent package graph.
- TLS verification/CA failure, nonempty-password authentication and open-handshake timeout
  evidence; local servers here are explicitly plaintext, passwordless PG12.2/MySQL8.0.27.
- Remaining native-driver category/missing-parameter-metadata branches, BEGIN/statement/control
  wall-timeout branches, some fallback wire types and PostgreSQL native JSON bind failure.
- JSON writer overflow/serialization defensive branches and invalid CA path branches;
  MySQL malformed/doubled quote/trivia classifier paths. Some helpers are defensive and
  cannot be forced by valid public input; do not invent unreachable production paths.
- Pending transport-close and already-consumed retirement branches, malicious transport
  stalls, supported runtime/target/pooler policies and the documented MySQL trigger assumption.

`coverage-all-sources.json`, `coverage-files.json`, `coverage-summary.json`, export argv,
and per-crate uncovered-line text/JSON retain exact evidence. Line coverage alone is not
branch/security proof, release acceptance, all-platform support or publication authorization.

## Exact owned-database ledger

Every name below was CREATE-acknowledged and DROP-confirmed exactly once in the named
log. No IF NOT EXISTS, adoption, prefix sweep, unrelated DROP/session termination or
role/global mutation. Owned connections close before DROP; PostgreSQL uses no FORCE.

| Backend / log | Exact database (DROP confirmed) |
| --- | --- |
| MySQL / round1-native | stackpress_s7_m0_e3c_18e258d5022c5990 |
| MySQL / round1-native | stackpress_s7_m1_e3f_18e258d5302d2108 |
| MySQL / round1-native | stackpress_s7_m2_e6a_18e258d564e417f8 |
| PostgreSQL / round1-native | stackpress_s7_m0_f25_18e258d6f4e37e10 |
| PostgreSQL / round1-native | stackpress_s7_m1_f4b_18e258d76f3df0f0 |
| PostgreSQL / round1-native | stackpress_s7_m2_fb9_18e258d81031c888 |
| MySQL / classifier-native-before (expected failure) | stackpress_s7_m1_9c55_18e259c84f714000 |
| MySQL / classifier-native-after | stackpress_s7_m1_c499_18e259d5028e8358 |
| MySQL / round4-native-coverage | stackpress_s7_m0_82fb_18e25a59e6b2bb88 |
| MySQL / round4-native-coverage | stackpress_s7_m1_8303_18e25a5a364a53e0 |
| MySQL / round4-native-coverage | stackpress_s7_m2_830f_18e25a5aa73fa3e8 |
| PostgreSQL / round4-native-coverage | stackpress_s7_m0_8700_18e25a5c98ff6848 |
| PostgreSQL / round4-native-coverage | stackpress_s7_m1_8736_18e25a5d00b55d58 |
| PostgreSQL / round4-native-coverage | stackpress_s7_m2_8787_18e25a5dafe76e88 |

`database-ledger.json` mechanically matches all 14 CREATE/DROP pairs. `sqlite-ledger.json`
records all 11 exact owned SQLite file paths (including failed/replayed controls), their
removal acknowledgements and direct absence checks for each file/-wal/-shm. No sweep.
Final coordinator-authorized read-only namespace listing (literal `stackpress_` prefix,
filtered on each server) returned `{"mysql":[],"postgres":[]}` in 2.302s after all
native work. Its parent watchdog is 30s; each backend has a 10s internal deadline.
`final-database-list.log` records this. Listing granted no adoption/drop authority.

## Task 00011 follow-up: MySQL string-key width (options only)

Current generated String keys are TEXT and correctly refuse MySQL DDL. Harness-owned
VARCHAR(191) is not a product default. Core already supports Varchar/Length::Size(n).
Options, none implemented/accepted here:

1. Explicit normalized optional field storage length, selecting VARCHAR(n); absence
   preserves TEXT/refusal. Recommended for review. Decide portable versus backend-specific
   metadata and align validation with declared character/byte semantics.
2. Derive storage intent from an existing static scalar-string validation upper bound.
   Less duplication, but validation edits become DDL/migration edits; list cardinality,
   dynamic bounds and overrides must remain distinct.
3. Named key/storage profiles supplying explicit key/indexed-string defaults. Convenient,
   but precedence/fingerprints/migrations become less local. Never silently infer 191.

Freeze syntax and normalized/serialized compatibility through task 00011 first. Refuse
invalid/unrepresentable lengths before IO; check full compound-index byte budgets under
the selected charset. FK source/target storage width, charset and collation must agree;
never silently widen or truncate. Preserve uniqueness/equality and strict text codecs.
Narrowing needs existing-value checks, explicit migration approval/reconstruction and
native generated-DDL plus DML/relation proofs. Prefix indexes/truncation are separate choices.

## Resources, deviations and handoff

All top-level builds use one worktree target. The largest sampled root size was about
3.21 GiB, below the 8 GiB budget; final commands enforce the bound. Resource deviation:
the existing `examples/generated-profile/codegen/tests/consumer.rs:113` overrides
CARGO_TARGET_DIR to a disposable tempfile target during workspace testing. Its source
and behavior are outside this write set and were not edited; strict one-target compliance
cannot be claimed for that invocation. The test owns that TempDir's cleanup. No stale
external target was inspected or reused. Other versions/TLS/targets remain unproved.
Final coverage/probe/namespace exports were retained before deleting the worktree
`target`; no profraw/profdata files remain outside that directory in either adapter.
Final worktree changes are only the two owned crates and this receipt; index is empty.
HEAD remains `8e84ae7`. No temporary verification package was added to the workspace.

KB baseline validation: 146 errors / 23 warnings from existing missing upstream source
links and existing file/link warnings. Final validation has the same 146 errors and 24 warnings: the sole new warning is
this coherent receipt exceeding the preferred 200 lines. No new hard/link error.
The receipt is kept coherent as one review/evidence document under the 500-line cap.
No context/status promotion or task-00011 implementation was made. Stop for coordinator
review; verification does not imply acceptance, merge, commit, push or publication.
