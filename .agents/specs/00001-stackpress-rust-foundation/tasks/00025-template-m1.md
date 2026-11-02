# 00025 M1: schema, module aggregates and Profile templates

Task summary: migrate this implementer's assigned emitters to real structural
templates after the [M0 proof](00025-template-m0.md). Current checkout `trial/3`,
base `62768feafeb40b13663751b288c62c83398dd843`, 2026-11-01. Acceptance criteria:
none; task tracking and integrated verification remain coordinator-owned.
Coordinator approved M1 after independent tests and coverage-file audit; rounds
and safety are released separately and are not claimed by this receipt.

## Implementation steps and notes

- Schema `emit`, `create`, `inputs`, and `query` prepare checked values for 16
  external assets in `src/codegen/templates/`. Assets own real declarations,
  functions, defaults, projection branches and query validation. Qualified types
  and generic wrappers also use scalar templates; lexical name derivation stays
  with normalization. The existing Rust literal encoder remains generation-owned.
- The local `source` module uses `[%` / `%]` so embedded Rust format braces remain
  exact. Rendered fragments are inert. Its independently configurable data budget
  is 16 MiB for descriptor/context encoding overhead; output remains 1 MiB, and
  OutputPlan's combined raw and writer's formatted caps remain unchanged.
- Concrete template limit failures retain `TransformErrorKind::Limit` and their
  `TemplateError` cause. Registry callbacks retain the existing outer `Callback`
  wrapper. Schema's unrelated SchemaError adapter is unchanged. Static asset
  paths identify failures without copying context/input values into diagnostics.
- Transformer module aggregation compiles one immutable `modules.rs.tpl` per
  nonempty plan and reuses it for partitions/root. Ownership validation precedes
  rendering; first-seen partitions, declaration order, partition insertion and
  root insertion order remain unchanged. Empty selections preserve app aggregates.
- Profile uses external contract/aggregate assets with finite checked field names
  and compile-time selection/revision flags. Its existing runtime implementation
  remains an immutable declared auxiliary snapshot; it is not recursively rendered.
- Added optional schema `codegen` lib edge and transformer normal lib edge only.
  The coordinator synchronized the lockfile, verified exactly three local edges
  across all lanes and released locked Cargo commands. This lane did not edit the
  lockfile or add any third-party dependency.
- Updated schema registration docs, transformer module guide and Profile guide
  using the already-applied chrisai-docing guidance. No generated-root, unrelated
  lane source, context, listener, install, commit or push was changed.

## Behavioral proofs

The pre-template schema baseline covers empty, small Profile, mixed scalar enums,
nested required/optional arrays, inert delimiter-bearing defaults and private/host
defaults. It records raw byte/hash inventory and exact final rustfmt bytes, paths
and owners. The post-migration test uses the real managed writer in exclusively
created temporary roots: final artifact fingerprints match. Raw whitespace can
differ for readable structural assets; no source compactor was introduced.

The real normalized JSON-default boundary probe locates the raw plan maximum,
checks raw aggregate overflow as direct `Limit`, and verifies rustfmt overflow
before installation. An accepted fixture has payload **200,257 bytes**, raw
aggregate **1,028,093 bytes**, formatted aggregate **1,031,183 bytes** (98.34% of
the unchanged 1,048,576-byte cap). A 512 KiB JSON default exercises a single
oversized public asset, preserving `Callback -> Limit -> TemplateError(Limit)`
and its authored asset identity. No common error policy or output cap changed.

The new module proof freezes exact one/multiple-module bytes, final newline,
interleaved contributions across three partitions, repeated compiled render use
and empty-selection manual aggregates. Existing Profile tests generate, repeat,
check, remove, and compile independent consumers with selection/revision mismatch
failures. Disabled Profile tests prove uncompiled callbacks stay unavailable.

## Verification process and results

All Cargo/proof commands use the benchmark recorder, absolute warm task target,
`RUSTC_WRAPPER=sccache`, 20G shared cache, incremental off, dev debug 0, jobs 2,
and `--locked`. Native macOS ARM64, Rust 1.96. Coordinator owns the integrated
workspace/golden-190 gate after all lanes stabilize; it was not duplicated here.

| Check | Result | Record suffix |
| --- | --- | --- |
| Old raw / managed formatted baseline captures | Passed before schema emitter edits | `7fdb4aceb84d`, `3c0027ca23dc` |
| Transformer/Profile initial focused suite | 98 passed, 7 skipped | `abb47d4d5b8f` |
| Schema final formatted parity, first pass | Passed | `94a4e8b5f9a8` |
| Final real raw/formatted boundary proof | Passed; exact metrics above | `75989ebe2439` |
| Final instrumented all-target suite, all three crates | 128 passed, 7 preexisting ignored | `816e83de3068` |
| Profile no-default-features all-target nextest | 3 passed, none skipped | `1c184e65ebbc` |
| Owned formatting / strict Clippy | Passed | `a70ace27eb9d`, `e1056d2cb380` |
| Owned doctests | 16 passed; Profile has none | `d7e5c7ddff30` |
| Warning-clean owned Rustdoc | Passed | `7a767ab0d016` |
| Profile report-only inclusion correction | Passed using retained profiles | `c057ea2fdcf6` |

The initial 128-test run passed 127 and failed only the test fixture's selection
of a raw maximum as an assumed formatted maximum. The corrected test explicitly
checks the separate caps. A second test correction distinguished aggregate
overflow outside callbacks from emitter overflow beneath Callback. Strict Clippy
found two needless test borrows; both were removed. Failed records are retained.
Final instrumented tests cover all corrected behaviors without rerunning a full
workspace gate. Independent coordinator tests passed the other 127 and the final
boundary proof (`989ff4a9131f`, `744f46cef05f`).

## Per-crate native coverage

Production file audit includes `cfg(test)` modules and `#[path]` declarations,
not merely paths under `src`. The explicit test-file filter is
`/tests/|/tests[.]rs$`; neither report contains test-only filename rows. No
production file is deliberately excluded from each crate's subtotal.

| Scope | Covered / executable lines | Production evidence |
| --- | --- | --- |
| Schema, codegen + providers | 3,189 / 3,829 = **83.29%** | Exact; separate provider test module excluded |
| Transformer | 2,020 / 2,162 = **93.43%** | Exact; test modules and `../tests/support/watchdog.rs` excluded |
| Profile codegen, raw | 174 / 194 = **89.69%** | Includes old inline diagnostic tests |
| Profile conservative production lower bound | 150 / 194 = **77.32%** | Subtract all 24 physical inline-test lines from covered numerator, keep full denominator |

Schema/transformer rows come from [combined report](../../../resources/template-refactor/m1-coverage.json).
The default cargo-llvm-cov regex silently excludes workspace paths under
`examples/`, including the selected Profile package. The verbose diagnostic
confirmed that exclusion. A report-only export with
`--no-default-ignore-filename-regex` retained the explicit test filter and used
the same profiles; [Profile report](../../../resources/template-refactor/m1-profile-coverage.json)
contains the package's three production source files. Its broader report has
dependency rows; only those three package files form 174/194. Shared selection
constants add no executable row. Existing inline `lib.rs:122-145` occupies 24
physical lines; subtracting all gives a conservative bound above 70%, with actual
production coverage at least that high. No test/prod source was changed to measure.

Source templates are data assets rather than executable Rust source coverage.
Their branch behavior is protected by formatted fingerprints, exact module bytes,
normalization/default matrices, managed boundary tests and real consumer builds.
These are native proofs, not browser/WASM/Tauri or live-database acceptance.

## Benchmark and target receipt

The shared store retains exact commands, logs and environments. This implementer
owns 18 M1 attempts: **15 passed, 3 failed**, **269.871795 s** command time and
**58.02 s** observed Cargo stages. Four additional `template M1`-labeled records
belong to coordinator lock synchronization/independent verification and are not
attributed here. One rejected recorder CLI invocation used unsupported kind
`fmt`; it created no run record and was rerun with `other`.

Shared counters at M0 end / this M1 checkpoint: attempts **2277 -> 2355**,
compilation-emitting attempts **1010 -> 1053**, declared successful cold workspace
builds **88 -> 88**. Deltas include all active lanes; this lane adds zero cold
full-workspace rebuilds. No repeated cleanup of the warm M0 target was performed.

Latest/peak aggregate retained task target total is **6,251,736 KiB = 5.9621 GiB**:
root task target 5,183,696 KiB; earlier relative-path rebases remain session
373,724, server 112,376, lib 107,668, sqlite 220,596, generated-workspace app
253,676 KiB. All task-owned paths are accounted below 12 GiB; unrelated targets
and the two originally dirty benchmark records remain untouched.

## Verification notes and next step

Coordinator independently accepted all three per-crate coverage scopes and
approved M1. Production source is stable for concurrent admin/SQL snapshots.
Three distinct test-verified rounds plus dedicated safety are released, including
engine and all three migrated crates; production edits wait for snapshot capture.
Task tracking and final integrated generated-artifact comparison remain with the
coordinator. No unresolved owner product decision is introduced by this milestone.
