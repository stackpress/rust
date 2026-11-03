# 00025 M0: reusable template engine proof

Task summary: implement the [approved engine design](00025-template-design.md),
library/facade exports, focused tests, and API/guide documentation. Current
checkout `trial/3`, base `62768feafeb40b13663751b288c62c83398dd843`, 2026-11-01.
Coordinator approved M0 and released the separately scoped M1 migration after
reviewing these proofs. Acceptance criteria: none; task tracking remains coordinator-owned.

## Implementation steps and notes

- Added `crates/lib/src/template/{mod,error,data,helpers,parse,render}.rs`.
  A bounded structural parser handles nested same-name blocks; each render owns
  its counters/buffer and borrows lexical scopes/root data. Source/data/helper
  insertions are inert; only visited branches perform lookup or helper calls.
- Implemented the frozen public API, default/custom literal delimiters, strict
  typed lookup, array/object loops and metadata, compiled helper registration,
  uppercase/lowercase/clipping/fallback helpers, static-context errors and spans.
- Borrowed validation counts exact compact JSON bytes without a root clone or
  serialized buffer. Scalar-pair input is checked before string copies. Data
  budgets can be raised independently from output budgets; a >1 MiB context
  proof retains a two-byte output cap. Existing generator/writer caps are untouched.
- Added 19 focused unit tests and an actual external structural template fixture.
  The 128-level valid render and 10,000-opener malformed input run under an
  external 10-second watchdog, using the existing library test watchdog.
- Added named exports from lib/facade and `stackpress::template`; facade doctests
  exercise named, module, and `stackpress::lib::template` paths.
- Added `crates/lib/docs/api/template.md` and
  `crates/lib/docs/guides/render-build-templates.md`, linked from existing indexes.
  The chrisai-docing pass added a TOC, method examples/Returns prose, helper/error
  usage, and compiled-pair reuse. All five Rust examples run in a disposable
  consumer; its adjacent external template also renders the asserted exact bytes.
- No third-party dependency, manifest, lockfile, emitter, generated-root, context,
  listener, commit, push, installation, or unrelated benchmark edits in M0.

Initial focused run: 17/18 passed; one nested-loop expected string was manually
mistyped (`112122122` instead of `11212122`). Corrected only that expectation.
The expanded 19-test proof then passed. Independent coordinator review identified
`manual_is_multiple_of`; even/odd now use `index.is_multiple_of(2)` and its inverse.
Final Clippy and coverage tests verify this correction. The full workspace test
gate preceded this behavior-equivalent lint edit; final workspace doctests,
Clippy, Rustdoc and native lib coverage use the final source.

## Verification process and results

All Cargo commands were benchmark-recorded. Gates use `RUSTC_WRAPPER=sccache`,
`SCCACHE_CACHE_SIZE=20G`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`.
Jobs were 3 during M0; M1 uses 2 while migration lanes are active.

| Check | Result | Record suffix |
| --- | --- | --- |
| Required scoped `cargo clean` / `cargo llvm-cov clean --workspace` | Passed before first build; no owner-target cleanup | `bdf7ab40762e`, `49ca7c996558` |
| Focused nextest engine proof | 19 passed; two unrelated lib unit tests filtered | `0dbe1356f70f` |
| Workspace nextest, all targets, locked, no-fail-fast | 953 passed, 85 skipped, zero failed | `da75c127d2cd` |
| Workspace doctests, locked | 965 passed, zero failed/ignored | `ca359fa2cc24` |
| Workspace Clippy all targets, locked, `-D warnings` | Passed after even/odd fix | `d6a3b6429741` |
| Workspace Rustdoc, no dependencies, `RUSTDOCFLAGS=-D warnings` | Passed | `78034e839b42` |
| Final workspace rustfmt check | Passed | `5af3479db849` |
| Final native lib coverage tests, all targets, locked | 73 passed, 5 preexisting ignored; all 19 engine tests included | `9da586d61f83` |
| Coverage report from those profiles, floor 70% | Passed; numerical results below | `f16f2b9ebcb2` |
| Complete five docs examples, offline/locked consumer | Passed without warnings | `36319fb872f6` |
| Normal-edge lib dependency tree | Passed; unchanged standalone dependency surface | `16a8f0f44882` |

Coverage tests succeeded, but their first report write failed because its parent
directory did not exist. Created only the scoped report directory and generated
the report from retained profiles; did not repeat tests solely for that path fix.
The failed attempt is retained as failed, rather than relabeled successful.

Native executable line coverage, `cargo-llvm-cov 0.8.6`, default lib features,
macOS ARM64, Rust 1.96:

| Implementation scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| Complete `stackpress-lib`, raw report including old inline tests | 4,160 / 4,426 | 93.99% |
| Conservative production-only lib lower bound | 4,107 / 4,426 | 92.79% |
| New template module subtotal | 703 / 714 | 98.46% |
| Template `data.rs`, `helpers.rs`, `mod.rs` | 95/95, 68/68, 108/108 | 100% each |
| Template `error.rs`, `parse.rs`, `render.rs` | 28/29, 220/224, 184/190 | 96.55%, 98.21%, 96.84% |

Raw [coverage report](../../../resources/template-refactor/m0-coverage.json)
contains **no** `/tests/` or `/tests.rs` filename rows, independently checked.
Only test source is excluded; no production exclusions. Two existing inline
`cfg(test)` modules remain in production filenames: `error.rs:598-632` (35
physical lines) and `queue.rs:17-34` (18). The raw lib ratio therefore is not an
exact production-only metric. Subtracting all 53 physical test lines from the
covered numerator while retaining the full denominator gives the conservative
production lower bound 4,107/4,426 = 92.7926%; the actual production ratio can
only be higher. The template subtotal is exact: its separate `tests.rs` is
excluded and it contains no inline test module. Coordinator flagged
ambiguous shell escaping of the original regex; the recorded report and filenames
were re-audited and accepted. Prefer the unambiguous equivalent
`--ignore-filename-regex '/tests/|/tests[.]rs$'` on subsequent reports. An optional
report-only repeat with that spelling was manually deferred by the coordinator
until lock synchronization: SQL/admin manifests had changed while `Cargo.lock`
remained under exclusive coordinator ownership. No repeat test run or record was
created. This was not an automatic safety rejection; the existing report is accepted.

The facade edit adds re-exports/docs, without a new executable implementation
denominator. No browser/WASM/installed-target claim is made by native coverage.
Trusted compiled helpers remain outside allocation/effect/panic sandboxing.
Remaining engine coverage gaps include unreachable defensive compiled-state
guards and error-display/span branches; the branch-risk matrix is exercised by
explicit behavior tests, rather than inferred from the percentage.

## Targets and benchmark receipt

Store: `.agents/resources/build-benchmarks/`. Exact recorded argv, environments,
logs and IDs are retained there; suffixes above identify unique run records.

This implementer's `template M0 ` labels identify 16 attempts: 14 passed,
2 failed (incorrect test expectation; missing report directory). Total command
time **635.427230 s**; observed Cargo stages **328.87 s**. These are distinct
measurements; compilation/discovery/test time cannot be inferred from each other.
Workspace nextest alone: **376.368466 s** command, **142.0 s** observed Cargo stage.
An initial docs consumer passed with one unused-import warning; the final five
examples removed that warning. Sandbox-denied commands before recorder startup
have no run records and are not included in attempt totals.

Shared counters at M0 start/end: attempts **2255 -> 2277**, compilation-emitting
attempts **998 -> 1010**, successful declared cold workspace builds **88 -> 88**.
The shared deltas include coordinator activity; only the 16 labeled attempts and
their timings above are attributed to this implementer. This effort adds zero
declared cold full-workspace rebuilds: its cold focused build selected lib only,
and later workspace gates retained the warm target.

Early commands used relative `target/template-refactor`. Existing compiler-proof
subprocesses rebase that path after changing cwd. Coordinator found this inherited
test-harness behavior; no harness was edited or active child killed. All later
commands use absolute `/Users/cblanquera/server/projects/stackpress/rust/target/template-refactor`;
the disposable docs consumer has its own `doc-consumer` child target.

Latest/peak measured task-owned total: **5,720,096 KiB = 5.4551 GiB**, below 12 GiB:

| Retained target | KiB |
| --- | ---: |
| Root `target/template-refactor` including coverage/docs consumer | 4,652,056 |
| `crates/framework/framework-session/target/template-refactor` | 373,724 |
| `crates/server/server/target/template-refactor` | 112,376 |
| `crates/lib/target/template-refactor` | 107,668 |
| `crates/sql/sql-sqlite/target/template-refactor` | 220,596 |
| `examples/generated-workspace/app/target/template-refactor` | 253,676 |

All are retained for warm reuse/accounting; no unrelated targets were removed.
The disposable example source is
`/var/folders/w2/g12vs0ts64g_3k6kjf_dd_bc0000gn/T/stackpress-template-m0-docs-dtgepzwk`.

## Verification notes

Coordinator reported independent lib/facade 74 tests and 273 doctests passing,
then 19 focused post-fix tests, Clippy and Rustdoc passing; source/API/docs review
and numerical coverage audit approved M0. Final task-status edits belong to the
coordinator. Required three maintainability rounds plus dedicated safety round
remain later work covering engine and all migrated emitters; M0 is not their receipt.

Agent Workspace validation initially had 11 missing historical screenshot-link
errors. The final checkpoint reports 12 errors and 32 warnings: those 11 plus
a concurrent admin asset rename leaving task `00018-issue14-m2.md` linked to
`pages_host.rs.in`. No task 00025 link error was reported; its design length
produces a coherence warning. These inherited/concurrent errors are not a clean
KB result and were handed to the coordinator without editing unrelated scope.
The coordinator subsequently repaired that historical path with an explicit
path-only maintenance note; validation returned to the inherited 11 errors and
32 warnings. The 12-error result above remains the historical checkpoint.

## Acceptance notes and next step

Acceptance criteria: none. M0 approved by the coordinator; M1 release covers only
the assigned schema/transformer/Profile lane. Freeze the shared manifest/lockfile
boundary before any further Cargo command. No context promotion or publication.
