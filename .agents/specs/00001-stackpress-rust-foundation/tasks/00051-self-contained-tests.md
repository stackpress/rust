# Task 00051 — self-contained crate tests

Status: verified for the non-live lane; coordinator review pending. Branch `release/self-contained-tests`, base `7c014bb`. Acceptance criteria: none. Live replay is deferred below; this receipt does not authorize publication.

Tests now read their own package inputs. No assertions, expected output, golden/oracle content or `.stderr` changes. No Cargo.toml or lock changes, dependencies, listeners, installation, commit or push. The five coverage-owned crates are untouched.

## Changes

- Server/CLI/framework-view use local lib watchdog copies; SQLite maintenance uses a local generated-workspace watchdog; SQL uses a local discovery harness. Five new helper copies prepend a provenance comment and otherwise match canonical bytes.
- Four byte-identical inputs are local: Leptos corpus, framework-schema session/API schemas, aggregate build-host source-client schema. Source paths/SHA-256 pins are in [test-copies.json](../../../../tools/release/test-copies.json).
- CLI entry fixtures use `CARGO_TARGET_TMPDIR`, retaining unique names and exact-directory cleanup.
- G59 already localized API/session/aggregate live helpers and API management shared code. Eleven existing helper copies are now freshness-guarded without editing their Rust files. Coordinator explicitly directed keeping nested unpublished consumers where they are; packaged tests cannot import their inputs.
- [Boundary checker](../../../../tools/release/check_test_boundaries.py) scans all 23 candidates' src (a conservative superset of cfg(test)), tests/examples/benches and support scripts. Escaped includes/module paths, unresolved include expressions, fixed runtime sibling reads, Python repository anchors, symlinks and nested Cargo package inputs fail.
- [Copy checker](../../../../tools/release/check_test_copies.py) merges every `test-copies*.json`, rejects duplicate destinations and checks exact bytes/source hashes. Schema and intentional-divergence requirements are in [README](../../../../tools/release/README.md). Twenty copies and eleven checker regression tests pass.

## Test-verified review rounds

1. Logic/complexity: preserved helper bodies, loader keys, fixture bytes and cleanup order. Added negative controls; corrected facade discovery because the repository's example applications are not Cargo example targets. Boundary/regression proofs pass.
2. Responsibility/abstraction: crate-local test ownership; only repository freshness policy is shared. Separate lane manifests merge without duplicate destinations. Added drift protection to G59's unchanged local helpers. Copy/regression proofs pass.
3. Consistency/comments: local path/provenance conventions and documented limitations; raw/conditional attributes, string data, nested comments and line-number preservation. Follow-ups fixed quoted manifest paths and added Python anchor scanning after finding the coverage-owned repository SQL proof. Eleven regression tests pass. No Rust behavior refactor was justified.

## Verification

Environment: sccache; `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, two Cargo build jobs; `TRYBUILD` unset; retained `target/task00051`, separate consumer and instrumentation targets. All Cargo/proof/cleanup commands were recorded. Observed target peak 11,534,476 KiB (11.00 GiB); removed only completed task outputs at capacity gates.

Narrow tests: 336 passed / 40 gated skips (`20261111T022138.464184Z-c177043d0c4d`); source/schema 98 passed / 2 skips (`20261111T022501.599864Z-37eb5ce906ff`). Full default: 1,061 + isolated TCP watchdog passed; compatible: 1,226 + isolated watchdog passed. Main sweep skip counts 89/97 include the separately executed watchdog. No expectation diff occurred.

- gate fmt: exit 0.
- gate Clippy default: exit 0.
- gate nextest default: exit 0.
- gate TCP watchdog default: exit 0.
- gate doctests default: exit 0; 976 doctests passed.
- gate Rustdoc default: exit 0.
- gate Clippy compatible: exit 0.
- gate nextest compatible: exit 0.
- gate TCP watchdog compatible: exit 0.
- gate doctests compatible: exit 0; 981 doctests passed.
- gate Rustdoc compatible: exit 0.
- gate aggregate consumers and build-host check: exit 0.
- gate aggregate graphs: exit 0.
- gate session consumers: exit 0.
- gate session graphs: exit 0.
- gate generated workspace freshness: exit 0.
- gate generated admin freshness: exit 0.
- gate API checkpoint freshness: exit 0.
- gate checker regressions: exit 0.
- gate test boundaries: exit 0.
- gate copy freshness: exit 0.
- gate changed crate coverage: exit 0.

Boundaries pass for all 18 owned candidates. Pending coverage worker: `stackpress-sql-postgres` (3 references), `stackpress-sql-mysql` (10), `stackpress-framework-sql` (17, including repository-level `tests/proofs/lengths/prove.py`), `stackpress-framework-admin` (2), `stackpress-framework-email` (0 escaped reads found, still separately owned). Default strict checking remains available after integration; this lane uses `--pending-coverage`.

Coverage is LLVM-mapped native `src/**` for each changed Rust crate, including mapped in-source tests, without extra exclusion patterns. Separate tests/examples/consumers and dependencies are not attributed as implementation. Exact selected features, counts and benchmark records are in [verification evidence](../../../resources/self-contained-tests-00051/verification.json).

| Crate | Covered/total lines | Coverage |
| --- | --- | --- |
| stackpress-server | 2040/2173 | 93.88% |
| stackpress-cli | 1306/1402 | 93.15% |
| stackpress-sql | 3224/3394 | 94.99% |
| stackpress-sql-sqlite | 727/808 | 89.98% |
| stackpress-leptos | 7232/8683 | 83.29% |
| stackpress-framework-view | 1216/1520 | 80.00% |
| stackpress-framework-schema | 5811/6840 | 84.96% |
| stackpress-framework | 1176/1231 | 95.53% |

All eight meet the requested 70% lane floor; this is not a waiver of the final publication floor.

Live replay: **not run (servers unavailable)**. Both approved PostgreSQL/MySQL endpoints refused connections; no databases created. Coordinator directed finishing without waiting and will replay live suites in the combined gate.

Root lock bytes and all 699 registry tuples remain identical. Agent Workspace validation retains 146 inherited missing-link errors and 44 warnings; none concern this receipt. The initial false-positive checker failure was fixed; the generated-workspace freshness invocation was corrected to the documented `emit generate --check true` syntax. Both failed records are preserved. Initial sandbox writes to the shared benchmark store/receipt were denied before execution; approved escalation completed the requested writes.

Benchmark store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Snapshot through `20261111T035215.053608Z-cf3109250bfa`: 60 lane attempts, 17 emitting compilation; statuses {'passed': 56, 'failed': 4}. Accumulated command time 5400.712s, observed Cargo-stage time 3705.220s across 21 records. These are distinct, potentially overlapping accumulated durations, not elapsed time or compiler CPU time. Full IDs/commands are in the evidence. First narrow/fresh consumer/coverage targets began empty despite the wrapper's warm label; no cold-rebuild or performance comparison is claimed.

No context promotion: lane evidence awaits coordinator verification.
