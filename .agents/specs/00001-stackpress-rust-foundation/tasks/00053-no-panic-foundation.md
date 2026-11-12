# Task 00053 — no-panic foundation

Status: implementation complete; three review rounds and the local gate pass.
Coordinator verification, acceptance and merge remain separate.

## Implementation notes

Added all six G62 deny lints to the sixteen assigned library roots and root
`clippy.toml` with test-only unwrap/expect/panic allowances. No binary targets exist
in these assigned package manifests. Source examples and tests retain their own roles.
No dependency or manifest changed. The root lock is byte-identical, with 699 registry tuples.

Production occurrences of the six named forms (comments/docs/test-only code excluded):

| Crate | Before | After | Remaining |
| --- | ---: | ---: | --- |
| `stackpress-lib` | 7 | 1 | Named literal exception |
| `stackpress-server` | 8 | 0 | None |
| `stackpress-server-http` | 2 | 0 | None |
| `stackpress-cli` | 1 | 0 | None |
| `stackpress-leptos` | 10 | 0 | None |
| `stackpress-idea-parser` | 0 | 0 | None |
| `stackpress-idea-transformer` | 1 | 0 | None |
| `stackpress-idea` | 0 | 0 | None |
| `stackpress-sql-sqlite` | 0 | 0 | None |
| `stackpress-sql-postgres` | 0 | 0 | None |
| `stackpress-sql-mysql` | 0 | 0 | None |
| `stackpress-framework-view` | 0 | 0 | None |
| `stackpress-framework-csrf` | 0 | 0 | None |
| `stackpress-framework-email` | 3 | 2 | Named literal exception |
| `stackpress-framework-language` | 0 | 0 | None |
| `stackpress` | 0 | 0 | None |

The 32-site audit removed 29 explicit panic forms and retained three approved constant
parsers. Invocation owns a nonempty ancestry node; HTTP rejection carries a checked
StatusCode; paging reuses admitted size/offset; config, Serde, mutation and renderer
invariants use errors or structural matches. Existing application callback unwinds
and catch/resume behavior remain unchanged. G42/G43 route/security behavior is unchanged.

Public signature changes:

- `Nest::entries(&self) -> Result<&DataMap<String, Value>>`; internal callers, Rustdoc
  and API guide updated. The coordinator explicitly authorized caller-only
  edits: `.entries()?` in `framework-session/src/auth/bridge.rs`; `fixed.entries()?`
  and `target.len()` in `framework-api/src/helpers.rs`. Further authorized caller-only
  changes are inventoried below.
- `EditPolicy::display(&self, value: &Self::Value, locale: DisplayLocale)
  -> Result<String, EditError>`; decimal, phone, temporal and editor callers updated.
  New/commit/policy changes prepare display before publication. Void cancel/locale
  actions retain the draft on failure and expose the existing editor error state.

Additive infallible APIs requested by the coordinator: `Language::default()` is
English `en`, empty phrases and `%s`; `PublicLocation::session_asset() -> Self`
is exactly `/assets/session.js`. `SelectOptions::default()` constructs an empty list.
`Request::new()` and `Default` remain infallible by the coordinator's literal-only
exception ruling. The complete three-site registry is
[Named no-panic exceptions](../../../context/no-panic-exceptions.md): default request
URL, embedded Nodemailer services JSON and MIME JSON. Each has a unit test; each
expectation remains active when library code is compiled as a lib-test target.
No `allow` escape was introduced.

Generator output files and ownership manifest now call `sync_all` while still in
the temporary tree; sync failure cannot publish it. Journal/cleanup marker files
also sync before publication. Renames sync their parent directory best effort on
Unix. Existing ownership, rollback/recovery, error types and partial-write policy
remain. The real installation failure matrix now injects StageSync for both initial
and replacement installs, checking unchanged old bytes or an absent new root.

Regression evidence includes the formerly panicking unknown private editor icon
name, editor display-error publication/cancel/locale/policy behavior, fixed-language
and conventional-asset equality, and all three literal parsers. The other removed
sites were private structural invariants or compile-time constants; the audit found
no additional reachable hostile-input panic requiring a fabricated crash case.

## Three review rounds

1. Logic/complexity: explicit redirect termination, error precedence, editor mutation
   order, typed statuses and generator sync-before-rename. Final narrow run:
   **435 passed**, 32 skipped (including opt-in native cases).
2. Responsibility/abstraction: reviewed ancestry ownership, configuration decoding,
   formatting errors, staging/recovery and callback teardown. The cohesive existing
   boundaries needed no additional wrapper/extraction. **20 targeted tests passed**.
3. Consistency/Rustdoc/comments: updated entries signature/example, error-state and
   durability docs, production-only expectations and local invariant explanations.
   **8 targeted tests passed**, including the expanded sync-failure matrix.

## Coordinated cross-lane caller edits

The coordinator expressly authorized these signature-only adaptations and relayed
identical edits to the owning workers. No other changes were made in their crates.

| File | Lines | Adaptation |
| --- | --- | --- |
| `framework-session/src/auth/bridge.rs` | 49 | `.entries()?` |
| `framework-api/src/helpers.rs` | 79, 81 | `fixed.entries()?`; `target.len()` |
| `framework-api/tests/endpoints.rs` | 138 | `req.data.len()` |
| `framework-admin/src/components/formatted.rs` | 137 | `display -> Result`; delegate inner display with `?`, return `Ok(temporal_text(...))` |
| `framework-admin/src/components/formatted_tests.rs` | 150 | unwrap display in the assertion; same shown vectors |
| `framework-session/tests/email/integration/signup.rs` | 13 | `.entries()?.clone()` |
| `framework-session/tests/email/integration/mod.rs` | 183 | serialize `.entries()?` |
| `framework-session/tests/main/account_flows.rs` | 30 | `.entries()?` |
| `framework-session/tests/main/auth_flows/oracle.rs` | 48, 270 | `.entries()?` in fallible handler; `.entries().unwrap()` in unit test |
| `framework-session/tests/main/auth_flows/events.rs` | 334 | `.entries()?` |

Paths above are relative to `crates/framework/`. These edits preserve expected data,
including the Rust replay tests named oracle; no original/Node oracle is rewritten.

## Gate and coverage

The single resumable gate passed: fmt; workspace default/COMPATIBLE Clippy with
warnings denied; all **16/16** assigned crates alone with all-targets Clippy;
default and COMPATIBLE nextest, doctests and Rustdoc with warnings denied;
aggregate/session feature graphs and every consumer (including build-host check);
all six generator freshness checks; package and strict test-boundary checks.
All **23/23** release payload and test-boundary checks pass; package report writing
was included. Full commands, attempts and timings are in
[gate results](../../../resources/no-panic-foundation-00053/gate-results.json).

Default nextest: **1,092 passed, 56 ignored**. COMPATIBLE: **1,320 passed, 60 ignored**.
The finite example/TCP cases run serially and all remaining ordinary cases run
with two threads; their unions contain every selected test. Coverage's full
COMPATIBLE workspace/all-targets rerun passes **1,320 tests** serially.
The initial coverage attempt hit PostgreSQL PoolTimedOut on its first connection,
before assertions; its exact owned fixture was dropped. The serial rerun changes
no test, timeout or expectation and adds no coverage exclusions.

Coverage uses cargo-llvm-cov **0.8.6**, rustc **1.96.0**, native macOS ARM64 and the
release COMPATIBLE feature set. Figures are line coverage of each package's `src/`
tree in the ordinary workspace LLVM JSON; this is not branch or WASM-runtime
coverage. Browser/disabled-feature graphs and consumers are separate proofs.

| Crate | Covered / executable lines | Line coverage |
| --- | ---: | ---: |
| `stackpress-lib` | 4,214 / 4,432 | 95.08% |
| `stackpress-server` | 2,350 / 2,539 | 92.56% |
| `stackpress-server-http` | 1,732 / 1,839 | 94.18% |
| `stackpress-cli` | 1,306 / 1,404 | 93.02% |
| `stackpress-leptos` | 7,967 / 8,779 | 90.75% |
| `stackpress-idea-parser` | 2,261 / 2,391 | 94.56% |
| `stackpress-idea-transformer` | 2,132 / 2,282 | 93.43% |
| `stackpress-idea` | 41 / 41 | 100.00% |
| `stackpress-sql-sqlite` | 740 / 808 | 91.58% |
| `stackpress-sql-postgres` | 842 / 889 | 94.71% |
| `stackpress-sql-mysql` | 986 / 1,049 | 93.99% |
| `stackpress-framework-view` | 1,799 / 1,852 | 97.14% |
| `stackpress-framework-csrf` | 67 / 69 | 97.10% |
| `stackpress-framework-email` | 2,486 / 2,756 | 90.20% |
| `stackpress-framework-language` | 54 / 58 | 93.10% |
| `stackpress` | 0 / 0 | G61: no executable lines |

Every executable assigned crate meets 90%; minimum **90.20%**. The facade's
0/0 is exempt under G61 and is proven through feature/consumer checks. See
[coverage numerators and scope](../../../resources/no-panic-foundation-00053/coverage-summary.json).

JSON export explicitly selects all sixteen assigned packages from the completed
workspace measurement. The final export emitted no warnings.

Live PostgreSQL/MySQL use the coordinator-approved local servers. Exact create/drop
receipts include every attempt that created a database:

| Selection / attempt | Created | Confirmed drops | Paired |
| --- | ---: | ---: | --- |
| nextest-default / 2 | 34 | 34 | Yes |
| nextest-compatible / 1 | 40 | 40 | Yes |
| coverage / 1 | 33 | 33 | Yes |
| coverage / 2 | 40 | 40 | Yes |

All this lane's names pair exactly; none remain in final read-only catalogs. MySQL
is empty. PostgreSQL had three foreign fixtures, owned by the concurrent framework
lane, not this lane's (coordinator-authorized separate attribution):
`stackpress_g46_postgres_a9f2_18e732004ee40478`,
`stackpress_g46_postgres_b447_18e732020b1d9ef0`,
`stackpress_g46_postgres_d902_18e7320b289e2590`. See [owned database receipts](../../../resources/no-panic-foundation-00053/live-pairs.json).
No database server or listener was started.

## Recorder and invariants

All Cargo commands use the benchmark recorder, sccache, jobs=2, incremental=0,
dev debug=0 and the owned `target/00053-foundation`. Store:
`/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`.
Completed fixture executables, isolated consumer targets and noncoverage artifacts
were retired through recorded commands; instrumented profiles remained for export.
The sampled peak was **11.27 GiB**, below the 12 GiB cap.

The ledger contains **42 attempts** (27 passed, 15 failed). Summed end-to-end
`command_seconds`: **7568.030s**; separately observed Cargo
stage time: **4942.130s** across 12 records. These sums
include retries and may overlap other work; hidden nested Cargo output makes stage
time incomplete. Cache declarations are unknown; no cold rebuild or performance
claim is inferred. See [ledger summary](../../../resources/no-panic-foundation-00053/benchmark-summary.json)
and [individual records](../../../resources/no-panic-foundation-00053/benchmark-records.json).

All **64** frozen tracked lockfiles/`.stderr` files are byte-identical.
The root lock retains **699** registry tuples; no dependency/manifest change,
original/Node golden/oracle-data change or generated-output change occurred.
`git diff --check` passes. See [verified invariants](../../../resources/no-panic-foundation-00053/invariants.json).
KB validation retains **146 inherited errors and 44 inherited warnings**; no new
exception-registry/index/receipt error appeared. This inherited KB result is not a
clean KB validation claim. No commit, push, install, publication or deployment.
