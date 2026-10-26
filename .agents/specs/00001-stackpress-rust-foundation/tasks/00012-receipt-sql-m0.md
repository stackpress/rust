# Task 00012: SQL M0 implementation receipt

M0 only, 2026-10-27. Branch `trial/3-s7-sql-m0`, base/HEAD `7029d831f116de607f2a7bb2bfa34cb32a341c7e`.
Implementation evidence for [task 00012](00012-slice-7-production-servers.md); independent coordinator
review remains pending. No task status, acceptance, or Context promotion is claimed.

## Implemented scope

- New `stackpress-sql-postgres` and `stackpress-sql-mysql` owners implement the
  unchanged `Database` / `TransactionSession` contracts. Native pools and driver
  types remain private; owners are non-Clone. Root members, optional dependencies,
  opt-in `sql-postgres` / `sql-mysql` facade features/exports, and lockfile updated.
- Typed explicit connection fields, unknown-field rejection, redacted Debug,
  verified TLS defaults, caller-supplied password, and finite open/acquisition,
  statement/control, cleanup, connection-count and immediate admission limits.
  Omitted serialized budgets use accepted defaults. Serialization explicitly
  includes the secret; application code must treat serialized options as secret.
- Defaults: 4 native connections, 128 admitted operations, 5000 ms open/acquire,
  30000 ms statement/control, 5000 ms cleanup. Configured bounds match the design.
  TLS defaults to PostgreSQL VerifyFull / MySQL VerifyIdentity using bundled
  WebPKI roots; adding a CA file preserves identity verification. Plaintext is
  explicit. Statement logging is disabled; no driver URL is accepted by production.
- Same-lease native preparation, metadata arity checks, strict scalar binds,
  PostgreSQL checked widths/typed NULLs, ordered incremental result collection,
  retained name/value budgets, native completion counts, and source-preserving
  sanitized SQL errors. PostgreSQL classification uses fixed built-in OIDs,
  rejecting unresolved/custom/domain type identities rather than guessing by name.
- Exclusive transactions poison before a run can suspend; failed/cancelled runs
  reject later run/commit but allow rollback. Consuming completion owns the session
  before first poll. Primary and explicit cleanup failures remain distinct.
- Drop retirement tracks background work and retains admission through bounded
  local transport disposal. Close shuts admission, wakes waiters and awaits leases
  and registered cleanup; interrupted close can be re-awaited. First background
  cleanup failure is retained in bounded storage and reported once by close.
- MySQL configures only each owned connection's utf8mb4 charset/collation and
  session SQL mode: STRICT_TRANS_TABLES, NO_ENGINE_SUBSTITUTION,
  NO_BACKSLASH_ESCAPES, PIPES_AS_CONCAT. A token-aware transaction gate rejects
  CREATE/ALTER/DROP including behind WITH/EXPLAIN, skipping quoted/comment text.
  DML rollback requires transactional tables; the fixture explicitly uses InnoDB.

## M0 conservatism and source findings

Every M0 completed lease is retired, including acknowledged ordinary success.
Connection/prepared-cache reuse is deliberately deferred. No uncertain lease
returns to the pool. The existing SQLite/core crates were not edited for reuse.

SQLx 0.9's PostgreSQL constructor reads ambient TLS/startup fields without public
clearing methods. Production options reject PGSSLROOTCERT, PGSSLCERT, PGSSLKEY
and PGOPTIONS when present; all remaining connection fields are explicitly
replaced, and pgpass is never read. No implicit environment fallback is accepted.

SQLx MySQL treats Some("") as a password scramble. Explicit empty-password
configuration leaves its native password field absent; there is no ambient MySQL
password fallback. Nonempty secrets use the driver's password field. No RSA
feature was added; plaintext authentication requiring RSA is outside the budget.

SQLx PoolConnection's flagged Drop spawns even after detach. The adapter's own
Lease guard prevents uncertain recycle throughout application IO, then arms
close_on_drop immediately before in-runtime retirement. Outside a runtime it
uses unflagged detach followed immediately by raw socket RAII disposal; minimum
connections is zero, so no replacement is spawned. This path is disposal only,
never detach-and-reuse. It does not claim acknowledged remote rollback.

## M0 proofs and bounded fixture policy

Each backend's custom test harness keeps the existing environment gate. Unset
variables print ignored cases rather than early-return passes. URLs exist only
in the fixture layer and must select the authorized local host, port and user.
Each run generates one bounded ASCII name, CREATEs without IF NOT EXISTS, arms
cleanup only after success, and never adopts a URL database as a DROP target.

The parent owns the fixture; all pool/cancellation/close/runtime scenarios run
in a child with a 30 s parent watchdog and kill/reap. Administration uses a
separate runtime with 9 s native and 10 s wall-clock deadlines. The parent reaps
the scenario child before DROP. PostgreSQL 12 cleanup retries only DROP of its
exact armed name on structured SQLSTATE 55006, within that deadline; no FORCE,
catalog scans or unrelated-session termination. Failure/unwind cleanup remains
failing, and reports the exact name. SIGKILL/power loss cannot guarantee cleanup.

Both backends proved: one-row typed/typed-NULL native round trip; exclusive
transaction plus one-slot nested acquisition timeout; cancelled ordinary lease
followed by a different clean native connection; cancelled transaction run
remaining rollback-only with explicit rollback; unpolled commit disposal;
interrupted/re-awaited close and Closed rejection; and real transaction Drop
after runtime shutdown followed by a fresh-runtime/lease rollback observation.
Compile-fail doctests reject Clone, overlapping run borrows, and consumed reuse.

Measured servers were PostgreSQL 12.2 and MySQL 8.0.27. No listener, tool install,
commit, push, publish, role/global-setting mutation, or other database read,
listing, alteration or deletion was performed. Workspace verification's existing
independent-consumer tests create and remove their own transient Cargo targets;
all manually requested builds use this worktree's single target directory.

## Scoped reviews

- Logic/complexity: reviewed pre-effect validation, poison ordering, cancellation,
  guard/permit transfer, missed wakeups, watchdog error paths and error precedence.
  Fixed kill/reap on child-observation errors; verified native lifecycle and units.
- Responsibility/abstraction: options, execution, collection, lease/cleanup and
  transaction ownership are separate cohesive modules. Kept backend duplication
  local because SQL core/SQLite extraction was outside the write set. No further
  abstraction was justified; affected tests rerun.
- Consistency/Rustdoc/comments: aligned backend APIs, applied finite serde defaults,
  corrected WebPKI/TLS documentation, documented M0 conservatism and cancellation,
  and added actual ownership compile-fail doctests. Formatting/lints/docs checked.
- Safety/soundness: inspected SQLx 0.9 pool/raw socket Drop/close source; fixed the
  flagged-detach runtime hazard and verified outside-runtime disposal. Replaced
  PostgreSQL display-name classification with exact built-in OIDs and added an
  unresolved/lookalike type regression check. Post-OID native replay passed under coverage below.

## Verification and database ledger

All Cargo verification used Rust/Cargo 1.96.0 (`ac68faa20`), native
`aarch64-apple-darwin`, `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`,
and the single worktree `target`. All build/test/tree commands used
`--offline --locked`; no dependency or tool installation was needed.
Logs and machine-readable evidence are under `/private/tmp/s7-sql-m0/`.

| Command scope | Result / retained log |
| --- | --- |
| `cargo +1.96.0 fmt --all -- --check` | Passed; `fmt-final.log`. |
| `cargo +1.96.0 clippy -p stackpress-sql-postgres -p stackpress-sql-mysql --all-targets -- -D warnings` | Passed; `clippy-final.log`. |
| `cargo +1.96.0 test -p stackpress-sql-postgres -p stackpress-sql-mysql` with gates unset | 10 units + 8 doctests passed; 2 native harnesses explicitly ignored; `offline-final.log`. |
| Both gates set, same scoped test command | Native owners/lifecycle + units + doctests passed; `native-final.log`; final post-OID native replay is `coverage.log`. |
| Same scoped `test --doc`, both gates set after OID fix | 8 passed; `doctests-native-gates-final.log`. |
| `RUSTDOCFLAGS='-D warnings' cargo +1.96.0 doc -p stackpress-sql-postgres -p stackpress-sql-mysql --no-deps` | Passed; `rustdoc-final.log`. |
| `cargo +1.96.0 check -p stackpress --no-default-features --features sql-postgres` / `sql-mysql`, and combined features | Passed; `check-facade-postgres.log`, `check-facade-mysql.log`, `check-facade-final.log`. |
| `cargo +1.96.0 test --workspace --all-targets -- --test-threads=1` | 480 passed, 61 expected gated/child-only ignored, no failures; `workspace-final-sequential.log`. |
| `cargo +1.96.0 test --workspace --doc` | 875 passed; `workspace-doc.log`. |

The first full workspace run also passed (478 before two added regressions).
An intervening repeat failed an existing formatter watchdog: its Python child's
PID file was absent at a 200 ms deadline. No other crate was edited. The exact
formatter test then passed in isolation, followed by the full sequential pass.
Logs: `workspace-tests.log`, `workspace-final.log`, `formatter-isolated.log`.
Early MySQL runs found passwordless auth and test-SQL spelling/alias mistakes;
all their successfully created databases were dropped, including the failed run
whose child unwound. The initial sandbox-denied connection created no database.

Normal-edge trees (`cargo +1.96.0 tree -p stackpress -e normal`, selecting each
feature separately) prove no SQLx/driver edge in default or `framework-ui-csr`
graphs; PostgreSQL selects only sqlx-postgres, MySQL only sqlx-mysql. No macros,
migrate, Any, chrono or decimal feature was added. Logs: `tree-default.log`,
`tree-browser.log`, `tree-postgres.log`, `tree-mysql.log`. Existing lock versions
were retained; only new packages/edges were added (`summary.json`).

Coverage: cargo-llvm-cov 0.8.6, installed LLVM 22.1.2 tools from the stable
1.96.0 toolchain explicitly selected with LLVM_COV/LLVM_PROFDATA; compiler
remained `+1.96.0`. `cargo clean`, then `cargo llvm-cov clean --workspace`
preceded the fresh run. CARGO_LLVM_COV_TARGET_DIR was `target`, with both backend
gates set. Command: `cargo +1.96.0 llvm-cov --offline --locked -p
stackpress-sql-postgres -p stackpress-sql-mysql --json --summary-only
--ignore-filename-regex '/tests/|/src/tests.rs$' --output-path
/private/tmp/s7-sql-m0/coverage.json`. Test source is excluded; no production
source is excluded. Native scenario subprocess profiles are included. Doctests
are separately verified, not included in stable coverage.

| Production crate | Covered / executable lines | Coverage |
| --- | --- | --- |
| stackpress-sql-postgres | 469 / 597 | 78.56% |
| stackpress-sql-mysql | 510 / 615 | 82.93% |

Both exceed the 70% development floor, with the M1/M2 gaps below still explicit.
Coverage JSON, command log and per-crate summary are retained. These percentages
are development evidence, not publication (at least 90%) or complete branch coverage.

Every acknowledged CREATE below has a matching successful DROP in the log.

| Backend | Exact owned database | Log / outcome |
| --- | --- | --- |
| mysql | `stackpress_s7_m0_12bf3_18e221aa66e6f258` | `coverage.log` / dropped |
| postgres | `stackpress_s7_m0_12c52_18e221aacea6bef0` | `coverage.log` / dropped |
| mysql | `stackpress_s7_m0_406_18e221072e579c60` | `native-baseline.log` / dropped |
| postgres | `stackpress_s7_m0_5cf_18e221081ac1a500` | `native-baseline.log` / dropped |
| mysql | `stackpress_s7_m0_15907_18e220e4e3342360` | `native-diagnose.log` / dropped |
| mysql | `stackpress_s7_m0_86a9_18e2215426746738` | `native-final.log` / dropped |
| postgres | `stackpress_s7_m0_8865_18e221561b56dfc8` | `native-final.log` / dropped |
| mysql | `stackpress_s7_m0_1331b_18e220d76a9c2838` | `native-first-approved.log` / dropped |
| mysql | `stackpress_s7_m0_17547_18e220fb11b9c878` | `native-m0.log` / dropped |
| mysql | `stackpress_s7_m0_168da_18e220ef27e78e70` | `native-passwordless.log` / dropped |

Source identity: `source-sha256.json` covers all 21 changed Rust/manifests/lock
files, and `summary.json` includes coverage, lock deltas and this database ledger.
`git diff --check` passed; no SQL core, SQLite or other lane files changed.
The target directory is removed after verification; see `clean-final.log`.
The KB baseline has 146 pre-existing missing upstream-sibling link errors and
21 warnings; no unrelated KB repairs are authorized. Post-write validator output
is retained as `kb-final.log` and compared against `kb-baseline.log` at closeout.

## Remaining boundaries

M1/M2 are not started. JSON codecs (the approved JSON feature is enabled), full
scalar/typed-NULL/arity/outcome/error matrices, cached ordinary lease reuse,
all cancellation seams, adversarial cleanup deadline tests, workload/packet
memory observation, pool stress and the backend contract/generated-store harness
remain unproved. Unsupported M0 native output fails Decode, without coercion.
Real TLS certificate/identity handshakes and non-macOS native execution are not
claimed. Browser evidence is a normal dependency graph only. Native packet/row
allocation can precede retained-output charging. Closing a local transport does
not prove immediate remote session termination or exactly-once write semantics.
