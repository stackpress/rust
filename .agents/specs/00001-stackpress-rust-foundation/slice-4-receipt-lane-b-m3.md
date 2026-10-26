# Slice 4 lane B M3 receipt

Task: [00007 SQL](tasks/00007-slice-4-sql.md), M3 SQLite application/host proof, per the coordinator's release and exact-parity contract. Read the [example quick start](../../../examples/sql-profile/README.md) for first use; this receipt records verification and boundaries. Stop for coordinator re-verification, without release of slice-wide rounds, full crate docs or publication.

## Baseline, ownership and manifests

- Worktree `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s4-m0`, branch `trial/3-s4-m0`, clean starting/unchanged HEAD `ea760dae06b3dfb29080b55a8084b00ecf0ac64b`. The coordinator completed the fast-forward from ce9e90f before implementation; only the merge commit changed that baseline.
- New owned crate: `examples/sql-profile/**`, package `stackpress-example-sql-profile`, with library and `sql-profile` / `sql-profile-setup` executables.
- **Added `examples/sql-profile` to root workspace members.** Cargo.lock adds only this package's 16-line local package entry. No new third-party package/version, facade feature or facade dependency was introduced.
- Adapter source remains unchanged. No edits to `crates/sql/sql`, lib, server, CLI, Idea, existing examples, root facade source or shared feature definitions. This receipt is the sole `.agents` addition.
- No commit, push, deployment, publishing, listener or messages to other agents. HTTP proof is buffered/in-process. Acceptance criteria: none; coordinator verification remains outstanding.
- Toolchain: rustc/Cargo 1.96.0 (`ac68faa20`, LLVM 22.1.2), cargo-llvm-cov 0.8.6, native `aarch64-apple-darwin`. No toolchain, service or system-package installation.

## Application behavior

- `src/lib.rs` composes a plain database plugin and a profiles plugin requiring it, through the existing ServerBuilder. Named Config/Listen/Route callbacks register one shared Engine, `profile-search`, and explicit `GET /profiles/:id` delegation. No general SQL/setup/migrate capability or route is registered.
- `database::DatabaseConfig` owns the `sqlite.path` startup section. App code fixes read-only/no-create policy; configuration cannot enable creation/writes. The callback opens an existing file eagerly and checks the three qualified columns with LIMIT 0 before publishing its Engine. Qualification prevents SQLite's quoted-string fallback from accepting a missing column. Startup errors preserve sources and await close; no partial service is published.
- `profiles::ProfilesConfig` preserves default limit 3, range 1..=100, unknown-field rejection and exact original validation message. Request data cannot override the limit, file or SQL. Typed ID lookup and the explicit `profile search requires a nonempty id` guard preserve the original 400 versus typed-input 500 behavior.
- The SQL predicate is the fixed audited expression `substr(CAST("id" AS BLOB), 1, length(CAST(? AS BLOB))) = CAST(? AS BLOB)`, with two separate bound copies of the ID. No user text is concatenated into SQL. Byte-prefix semantics preserve case, Unicode and embedded NUL exactly, without LIKE's wildcard/case-folding behavior.
- Queries filter first, order by stable stored ordinal and then limit. The owned Profile DTO has only String `id` / `name`; native rows, driver types, ordinals, config and SQL metadata do not enter the envelope. Response total is the returned-row count, matching the original service.
- `setup::install_and_seed` is an explicit separate operator/library action. One caller-owned transaction creates profile/unique index and seeds ordinal 0/1/2 with `abc123 / Ada Lovelace`, `abc456 / Grace Hopper`, `abc789 / Margaret Hamilton`. It uses real Create/Insert builders and Transaction::execute_plan, commits only after all writes succeed, and preserves primary plus secondary cleanup errors.
- Setup refuses an existing profile installation, without clearing/reseeding/migrating it. Schema/data from a failed transaction roll back; an empty file can remain after failed initial creation. Ordinary startup/search never calls setup. The separate setup executable requires exactly one explicit native file argument.
- `sql-profile` uses `stackpress_cli::main_with(build_application)`. Embedding callers can await Engine::close; CLI retains the existing RAII shutdown path. No G23 shutdown-hook change was requested.
- README gives one explicit setup → CLI success path and explains stored ordering/policy. Its config example uses an ignored `target/` file. Rustdoc's real app/setup/close snippet compiles as a no_run doctest; tests execute both real binaries with disposable paths, rather than installing into the documented operator path.

## Proofs and exact parity

| Proof | Passing evidence |
| --- | --- |
| Same canonical seed / order / configured limit | `tests/cases/parity.rs` compares the actual capability-host application with SQL hosts at limits 1, 3 and 100, plus ordered code override 2→1. Setup storage tests inspect all ordinals, IDs and names. Prefix `abc4` demonstrates filtering before limit. |
| Complete wire bytes | Direct responses are serialized in full and compared with the actual reference; both real CLI runners' stdout bytes agree; native buffered HTTP status/body bytes agree for the same URI. CLI's sole trailing newline is preserved and explicitly checked against the direct/HTTP envelope payload. No handwritten envelope surrogate or row-only comparison substitutes for reference parity. |
| Hostile prefixes | Canonical parity tests cover `%`, `_`, backslash, quotes, SQL-looking text, Unicode and case differences. Positive fixtures add such IDs (plus embedded NUL) and compare native query results with Rust starts_with; this catches bugs hidden by empty matches. Positive literal matches also pass through the CLI runner. `abcZ` before `abcA` proves ordinal rather than ID sorting. |
| Input/error/exposure parity | Missing, empty, numeric, null and object IDs match reference envelopes byte-for-byte. Client limit/path/SQL fields are ignored. Unknown SQL capability and unexposed paths remain ordinary 404/no-listener responses. HTTP percent-escaped paths match the reference's existing path-spelling/capture behavior; the SQL plugin introduces no decoder or shared HTTP change. |
| Actual main_with executable | `tests/executables.rs` invokes the Cargo-built binary with native argv, including spaces in file/config paths and hostile IDs. Its stdout bytes match the real reference CLI runner. Missing/empty IDs, unknown event, usage and missing-install errors have the expected exits/output. |
| Explicit setup and no startup setup effects | Missing file stays absent; empty/wrong schema fails startup without table installation. Opening an installed DB preserves operator edits. Invalid/unknown config fails; missing plugin prerequisite fails before a host exists. Database-only composition exposes no search capability. |
| Setup atomicity/refusal | Repeated install fails without duplicating/changing rows. An unrelated existing `profile_id` index makes the later CREATE INDEX fail after table creation; schema snapshot is identical afterward and the native source remains available. Missing parent paths fail without fallback. |
| No swallowed native failures | The registered read-only Engine rejects an actual write. Closed Engine and an externally introduced invalid BLOB name produce 500/source-bearing failures, never empty successful rows. |

All native library scenarios (`parity`, `literals`, `startup`, `setup`, `failures`) run in separate current-thread child processes with independent **10-second parent** and **8-second cooperative** deadlines. Actual executable children each have their own 10-second kill/reap watchdog and independent 64 KiB stdout/stderr ceilings. A synchronously parked negative-control child is killed/reaped after 150 ms. Every database fixture has an independent tempfile directory; handles are closed before cleanup. No process cwd/environment is mutated; only child command configuration carries the scenario marker.

## Bounded milestone review passes

These satisfy the standing in-milestone Rust reviews, without releasing the coordinator's later slice-wide rounds/docs gate.

1. **Logic/error/branch risk:** reviewed reference validation precedence, filter-before-limit, ordinal order, exact byte-prefix behavior and source preservation. Corrected the test URI encoder to preserve ordinary unreserved characters rather than change the reference input, and pinned existing percent-path behavior. Added positive literal CLI matches and actual empty-ID rejection, plus setup native-failure rollback/refusal. All-target app tests passed (`round-one.log`).
2. **Responsibility/ownership:** reviewed app composition, fixed database policy, DTO/persistence conversion, explicit transactional setup and separate executable ownership. Native/decode logic stays in the adapter/core; no new generic service abstraction, shared crate edit or global state was needed. Test-owned watchdog/fixtures remain outside production. Targeted native suite passed (`round-two.log`); no further production refactor was justified.
3. **Consistency/Rustdoc/comments/safety:** reviewed public/private contracts, cancellation/drop/close limits, audit-marker ownership, serde/typed-input behavior, comments and source/dependency boundaries. No first-party unsafe, mutex across await, hidden retry, implicit migration or listener was introduced. README uses the chrisai-docing guided-learning principles for a short first-success path. Final fmt/lint/docs/tests/MSRV checks below passed after final edits.

## Final checks

Commands ran against the two affected packages, with locked/offline resolution:

```sh
cargo fmt --all -- --check
cargo test -p stackpress-sql-sqlite -p stackpress-example-sql-profile --all-targets --locked --offline
cargo test -p stackpress-sql-sqlite -p stackpress-example-sql-profile --doc --locked --offline
cargo clippy -p stackpress-sql-sqlite -p stackpress-example-sql-profile --all-targets --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-sql-sqlite -p stackpress-example-sql-profile --no-deps --locked --offline
cargo +1.96.0 check -p stackpress-sql-sqlite -p stackpress-example-sql-profile --all-targets --locked --offline
cargo check -p stackpress --no-default-features --locked --offline
cargo check -p stackpress --no-default-features --features sql --locked --offline
cargo check -p stackpress --no-default-features --features sql-sqlite --locked --offline
```

All passed. App: **four passed test/parent entries**, two ignored child entrypoints actually launched by parents, five native scenarios, both real executables and one compiled doctest. Adapter: **10 passed entries**, six deliberately ignored child entrypoints launched by parents, inherited M0/M1/M2 matrix/compile-fail/runtime/watchdog proofs and two doctests. No whole-workspace or all-platform pass is claimed from these package-scoped checks.

## Dependency and feature evidence

These exact normal-edge trees passed and are retained in `target/s4-lane-b-m3-evidence/tree-*.log`:

```sh
cargo tree -p stackpress --no-default-features --edges normal --locked --offline
cargo tree -p stackpress-lib --edges normal --locked --offline
cargo tree -p stackpress-server --edges normal --locked --offline
cargo tree -p stackpress --no-default-features --features sql --edges normal --locked --offline
cargo tree -p stackpress --no-default-features --features sql-sqlite --edges normal --locked --offline
cargo tree -p stackpress-example-sql-profile --edges normal --locked --offline
```

Default facade, lib, server and core-only SQL contain no SQLx/SQLite driver. The opt-in SQLite facade and example contain SQLx 0.9.0/sqlx-sqlite/libsqlite3-sys, with no normal-edge sqlx-postgres/mysql. Applications depend downward on core/adapter/foundation; only dev-dependencies reference capability-host for comparison. SQLx is not added directly to the app or leaked into DTOs. G26 network renderer execution belongs to its separate lane; no network service was used here.

## Independent per-crate coverage

Both measurements use cargo-llvm-cov 0.8.6 and the existing `target/llvm-cov-target` beneath the reused target root. Each package measurement begins with `cargo llvm-cov clean --workspace`. They do not share a combined numerator or use another crate's tests for their gate.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-sql-sqlite --all-targets --locked --offline --no-report
cargo llvm-cov report -p stackpress-sql-sqlite --json --summary-only --fail-under-lines 70 \
  --ignore-filename-regex '(/tests/|/src/tests\.rs)' \
  --output-path target/s4-lane-b-m3-evidence/adapter-coverage.json

cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-sql-profile --all-targets --locked --offline \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/(crates|tools|tests|target)/|/examples/(capability-host|generated-profile|profile-ui)/' \
  --json --summary-only --fail-under-lines 70 \
  --output-path target/s4-lane-b-m3-evidence/app-coverage.json
```

| Production scope | Covered / executable lines | Line coverage |
| --- | --- | --- |
| Adapter collect / execute / lease / lib / options | 62/66, 100/113, 37/38, 115/115, 55/55 | **369/387, 95.35%** |
| App database.rs | 36/37 | 97.30% |
| App lib.rs | 10/11 | 90.91% |
| App profiles.rs | 71/71 | 100% |
| App setup.rs | 43/44 | 97.73% |
| App main.rs | 5/5 | 100% |
| App bin/setup.rs | 20/20 | 100% |
| **App total, package alone** | **185/188** | **98.40%** |

Both independently pass 70%. The example uses the established **--no-default-ignore-filename-regex** method; all six production source files, including both executed binary entrypoints, were asserted present. Only dependencies/other packages, tests, standard library/toolchains and generated build output are excluded; no app production file/branch is excluded. App's three uncovered lines are two defensive secondary-close-error paths and the static profiles-registration propagation path. All 31 app production functions execute. Adapter's existing 18 defensive lines remain as in M2; no core coverage is claimed.

After saving the adapter-only report and before cleaning for the app, the unchanged adapter m0-host example was separately instrumented/run under a 10-second runtime watchdog, printing `profile-name: Ada`. A subsequent profile merge plus explicit LLVM export gives **64/65 lines, 98.46%** for that example; it does not raise either package's independent gate. The new app measurement includes actual executable child profiles directly, without this separate export workaround.

## Evidence, resource use and stop

Ignored local evidence: `target/s4-lane-b-m3-evidence/` holds implementation/review/final check logs, exact command JSON, trees, independent coverage JSON/summaries, legacy example export, missing-line review and KB baseline. `git diff --check` and forbidden-path audit passed. Root Cargo changes are limited to the member and local lock entry described above.

Reused one target root. Removed this worktree's old generated incremental cache, then removed completed instrumentation artifacts/incremental cache after extracting reports; regular dependency artifacts and all evidence logs remain. No other worktree target was touched. Host-wide free space was about 4.4 GiB at final cleanup; concurrent disk use remains outside this lane.

Residual boundaries: native aarch64 macOS only; no listener/network execution, Linux/Windows/WASM/mobile or power-loss/durability certificate. Setup is one-shot installation, not migration/destructive policy tooling. Native cancellation can lose acknowledgement; normal CLI shutdown is RAII. Existing host path-capture behavior is preserved; URI decoding is not changed. Full adapter/core docs, slice-wide refactor/safety rounds and S4-F/M/N/H remain separately gated.

KB baseline before receipt: **146 existing errors / 20 warnings**, principally relocated links. No existing KB was repaired or promoted. Post-write comparison is recorded in the handoff. Stop at M3 for coordinator re-verification, without commit or further-stage work.
