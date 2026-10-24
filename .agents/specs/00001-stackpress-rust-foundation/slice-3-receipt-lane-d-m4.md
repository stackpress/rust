# Slice 3 lane D: completed M4 and review rounds

Implementation receipt for [task 00006](tasks/00006-slice-3-idea.md). Load for the production
wrapper API, generated example, four rounds, boundary fix and exact verification.

Date: 2026-10-25. Started on `trial/3` at `a5c9a13`; the coordinator merged lane C's writer
rounds as `90b9dc4` during this effort. Final checks use that integrated checkout. Every pending
item in the [pre-integration receipt](slice-3-receipt-lane-d-integration.md) is resolved.
The old receipt had already moved from the example into this spec; only a pointer was added.
No commit, push, listener, agent message, publication or KB promotion occurred.

## Implemented behavior and API

The wrapper composes an entry through a caller-supplied `SourceLoader` and `CompositionLimits`,
resolves actual sites with `finalize_with_references`, moves finalized sections into the
registry's `SchemaConfig` view, calls `Registry::plan_with`, then checks or installs through
`OutputWorkspace`. It adds no second registry, shared cache or plugin system. Typed causes
survive each phase. Literal `${Name}` strings stay literal.

Final signature in [wrapper source](../../../crates/idea/idea/src/lib.rs):

```rust
pub fn generate(
    request: GenerateRequest<'_>,
    loader: impl SourceLoader,
    registry: &Registry,
    workspace: &OutputWorkspace,
) -> Result<CheckReport>
```

`GenerateRequest::new(entry, aggregates, mode)` supplies bounded defaults. Its public fields
expose the entry, explicit `CompileInputs`, composition limits, `TransformSelection`, app-owned
aggregate renderer and single `GenerationMode::{Check, Install}` choice. Both modes return the
writer's structured `CheckReport`. Checking acquires a transient cooperative lock but changes
no generated files/manifest. Pre-install failures leave managed output intact. Trusted
loaders/renderers own their external effects; callback panics propagate.

Codegen uses `ProjectLoader` rooted at the app. Explicit relative inputs resolve at caller cwd;
ordinary imports resolve at their importing file. Production context callbacks consume a
declared `templates/profile.rs` snapshot. Public `register` documents that requirement; the
tool's `generate` supplies the full run. Profile mapping/rendering lives in `codegen/src/profile.rs`;
the library entry owns CLI hosting, source acquisition and IO orchestration. `selection.rs` and
selection/revision assertions remain. `-c`, `--check true` and explicit false precedence work.

The native sibling-file proof moves Profile into disposable `types.idea`, comparing **every
generated file and manifest byte** with the inline schema. The checked-in schema stays inline
to keep the first author example small. Malformed root/child sources retain file/line/column,
return public 400 / exit 1, omit excerpts and preserve prior output. Native IO/internal errors
remain sanitized. CLI/direct/buffered HTTP envelopes still match the actual Slice 2 host.

README disabled commands ran against the real example: generation removed both Profile files,
the matching app returned 404 / exit 1, and restoring selection reproduced the original Rust
bytes. Both check spellings then passed. The owned manifest only gained byte counts from the
current writer; prior Rust SHA-256 values are unchanged. The Cargo-consumer proof passes absent
output bootstrap, repeat generation, Serde-only compilation/tree, selection/revision compile
failures, removed-model compilation and CLI/direct/HTTP 404, plus selected-to-disabled cleanup,
compilation and all three 404s.

Only the authorized transformer crate-level prose changed there. Its re-exports and all
registry/plan/output/compose/source code are untouched. No dependency, manifest, Cargo.lock,
facade implementation or reserved lib/server/CLI documentation changed in this effort.

## Baseline, rounds and follow-up

Before the first build, `cargo clean` and `cargo llvm-cov clean --workspace` passed; clean
removed 28,674 files / 2.7 GiB. Every later build/check/test/lint/doc used `--locked`.
Baseline: wrapper 3, native codegen 5, Cargo consumer 1, app 2 and disabled generator 1 passed.
One overlapping early enabled/disabled invocation reused Cargo's native binary path and caused
two transient CLI failures; serial reruns passed. Final feature-changing tests run sequentially.

| Round | Findings and behavior-preserving changes | Verification |
| --- | --- | --- |
| 1: logic/complexity | Extracted the exact Profile-shape predicate; reviewed field count/order/type/modifiers, metadata, removal and alias precedence. Added omitted/extra-field and second-model cases; preserved errors and output bytes. | Wrapper 3, generation 5, disabled 1 passed. |
| 2: responsibility/abstraction | Separated model rendering/assets from CLI/IO orchestration. Kept the wrapper cohesive, with independently owned loader, registry and writer. | Wrapper + full codegen 9 passed, including the real Cargo consumer. |
| 3: consistency/Rustdoc/comments | Documented ownership, roots, causes, modes, imports and template inputs; updated stale transformer prose and README; checked function Rustdoc and 100-column Rust. | Wrapper + codegen 9, wrapper doctest 1, scoped Clippy/Rustdoc passed. |
| 4: safety/soundness | Removed a premature environment-map clone: composition owns the snapshot; finalization retains value limits after env expansion. Added per-run environment, compatibility, budget and false-alias regressions. Nested consumers clear encoded coverage flags. Reviewed bounded data, cfg removal, interpolation, cause disclosure and awaited worker ownership. | Wrapper + codegen 10, disabled 1 passed before the follow-up below. |

A boundary follow-up found a confirmed defect: at default import depth 32, the leaf parser
diagnostic exceeded the adapter's fixed 32-cause walk and returned 500. A failing native
regression is retained in `deep-diagnostic-before.log`. The bounded walk now uses composition's
hard depth plus phase allowance, restoring the accepted 400 contract. All 6 native tests pass.

Disabled-only coverage initially failed at 42/108 (38.89%): the library proof missed the real
CLI host. Added substantive disabled host/native tests for stale check, install, both spellings,
absent output, uncompiled-plugin rejection, sanitized failures, malformed-source coordinates
and prior-output preservation. Both disabled tests pass. A shared finite CLI helper serves
both suites. No production exclusion or ignored regression hides the gap. Initial coverage
evidence remains in `coverage-codegen-disabled-before.*`.

## Final verification

Logs/reports: ignored `target/lane-d-m4-final/`. Toolchain: rustc/Cargo 1.96.0, rustfmt
1.9.0-stable, cargo-llvm-cov 0.8.6, native `aarch64-apple-darwin`. The installed named
`1.96.0` toolchain was used explicitly.

| Exact command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `cargo test --workspace --all-targets --locked` | 262 passed, 0 failed; 23 child/helper entries ignored in ordinary enumeration and exercised by parent tests. |
| `cargo test --workspace --doc --locked` | 430 passed. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Passed warning-free. |
| `cargo +1.96.0 check --workspace --locked` | Passed. |
| `cargo test -p stackpress-example-profile-codegen --no-default-features --all-targets --locked` | 2 passed; none ignored. |
| `cargo test -p stackpress-example-profile-codegen --test generation --locked` | 6 passed, including depth-boundary and byte-identical sibling imports. |
| `cargo test -p stackpress-idea --all-targets --locked` | 4 passed, also included in workspace tests/coverage. |
| Owned-function Rustdoc / 100-column scan; `git diff --check` | Passed. |
| `python3 .agents/scripts/validate-agent-workspace.py` | Passed with warnings only in other existing KB files; no errors. |

Facade matrix passed: `cargo test -p stackpress --no-default-features --test idea --locked`
alone and with `--features idea-parser`, `idea-transformer`, `idea`, and
`idea-parser,idea-transformer,idea` (0/1/1/1/3 tests). No facade wiring change was needed.

Trees saved from `cargo tree -p <package> --edges normal --locked` cover
`stackpress-example-generated-profile`, `stackpress-example-profile-codegen` and `stackpress`.
The app/default facade have no `stackpress-idea*`, tempfile or SHA dependency. Codegen has no
generated-app dependency. The disposable contracts consumer's tree is Serde/derive only.

## Clean per-crate coverage

Each collection was independently preceded by `cargo llvm-cov clean --workspace`. Default
example exclusions are disabled. Regexes exclude dependencies/toolchains, harnesses, build
output and other packages; **no measured crate's production file or branch is excluded**.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-idea --all-targets --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/tests/|/target/|/crates/(lib|server|idea/(idea-parser|idea-transformer))/|/tools/|/examples/' \
  --json --output-path target/lane-d-m4-final/coverage-wrapper.json --fail-under-lines 70

cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-profile-codegen --all-targets --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/(crates|tools|tests|target)/|/examples/(capability-host|generated-profile/app)/' \
  --json --output-path target/lane-d-m4-final/coverage-codegen.json --fail-under-lines 70

cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-profile-codegen --all-targets --no-default-features --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/(crates|tools|tests|target)/|/examples/(capability-host|generated-profile/app)/' \
  --json --output-path target/lane-d-m4-final/coverage-codegen-disabled.json --fail-under-lines 70
```

| Production scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| stackpress-idea | 41 / 41 | 100.00% |
| codegen, profiles enabled | 161 / 169 | 95.27% |
| codegen, profiles disabled | 96 / 108 | 88.89% |

All pass the 70% floor. Codegen main is 5/5 in both modes. Selected coverage includes the
Cargo-consumer proof; nested consumers use private uninstrumented targets and do not replace
measured tool source. Doctests run separately. App Rust is byte-unchanged: its earlier 85/90
(94.44%) measurement is historical, not a new collection here; its full proofs were rerun.

Remaining coverage gaps concern defensive app-root resolution, non-UTF-8 paths, worker/
registration failure and static-asset insertion. Disabled Profile mapping/template code is
absent through cfg. No in-scope finding remains. Native cooperative ownership, trusted
plugin/formatter and recovery limits remain; no new killed-worker, power-loss, network, WASM,
Leptos/Tauri or cross-platform proof is claimed. This receipt is implementation evidence for
coordinator review, not delegated acceptance or context promotion.
