# Slice 3 lane D: M4 pre-integration receipt

Spec file of [00001 Stackpress Rust foundation](index.md), recording Slice 3 lane D (integration and example, M4 baseline) of [task 00006](tasks/00006-slice-3-idea.md). Load for the generated-profile example, codegen diagnostics and `-c` alias, the consumer proofs, facade `idea*` features, coverage, and the items pending lane B/C integration. Moved losslessly from `examples/generated-profile/LANE-D-RECEIPT.md`.

Date: 2026-10-25. Checkout: `trial/3`, base/HEAD `54b188e` (accepted M0).
Stopped before refactor rounds. **M4 is pending integration**, not fully passed:
the approved empty-registry change belongs to lane C, and the production composition
exports belong to lane B. Neither has been integrated into this checkout at this stop.
The coordinator explicitly approved recording the disabled-build proof as pending rather
than working around M0's rejection.

## Implementation

- Codegen discloses retained parser `IdeaError` coordinates through a deliberate public
  400 envelope, retaining the typed cause. Native malformed Idea exits 1 and reports
  `broken.idea:2:16:`. Source excerpts stay absent; native IO failures remain sanitized 500s.
  The filename comes from the selected input rather than always claiming `schema.idea`.
- Added `-c` as capability data, equivalent to `--check true`; an explicit long-form value
  takes precedence. The G14 operator/event grammar is unchanged.
- Compiled generator registration and the Profile emitter are now behind `profiles`.
  The disabled tool has an empty selection. `app/schema-disabled.idea` provides its empty
  plugin-block input. A disabled tool cannot activate the callback from an Idea plugin block.
- Added explicit rejection of ignored plugin settings and expanded unsupported-shape tests
  for field order, optional/list types and model/column metadata. The generator still owns
  exactly the advertised Profile shape and demo operation.
- Added [README.md](README.md) using the installed chrisai-docing guided-learning guidance:
  first generation/search, generated layout, freshness, explicit paths, matching disabled
  builds and conservative output recovery. The disabled instructions await lane C integration.
- App parity now compares complete generated envelopes against the **actual** Slice 2 host,
  including missing/empty/typed input failures, as well as CLI/direct/buffered HTTP success.
  The prior example source remains unchanged. Its new dependency is dev-only.
- Replaced `verify-m0.py` with [Rust consumer tests](codegen/tests/consumer.rs): independently
  copied tool/app sources, no initial generated tree, real Cargo builds, repeated generation,
  freshness, selection/revision compile failures, a Serde-only contracts consumer, model
  removal, and selected-to-disabled cleanup. Disposable app tests check removed registration
  through direct, CLI and buffered HTTP adapters. Every child has a finite deadline; Cargo
  output is file-backed to prevent pipe saturation. Temporary consumers get fresh offline
  lockfiles once, then every build/check/test/tree uses `--locked`.
- Removed `check-coverage.py`; reproducible package-specific coverage commands and counts
  are below. It was M0 infrastructure, not example runtime or a reusable project tool.
- Retained `selection.rs`: G20 explicitly requires the shared compile-time selection and
  revision source. Both app and codegen compile it independently; no app dependency enters
  codegen. Generated const assertions continue rejecting mismatches.
- Added wrapper failure-preservation integration tests and isolated/combined facade feature
  tests. Existing wrapper orchestration and facade wiring already work on the frozen M0 API.
  Wrapper production composition integration remains for the coordinator's B/C merge notice.

## Ownership and dependency changes

All edits are within assigned example/wrapper/facade/shared-test paths. `Cargo.lock` changes
only by adding the existing `stackpress-example-capability-host` path package to the generated
app's dev dependency list. No external package/version or budget increase was introduced.
Root `Cargo.toml` and transformer shared files needed no pre-integration edits.
No other lane's source/worktree or other trial branch was read or edited.

The deleted Python files were linked by historical M0 spec receipts. Those receipts stay
coordinator-owned; their historical evidence is retained in the M0 commit. Current equivalents
are the Rust tests and the exact coverage commands in this receipt.

## Verification

Initial fresh-effort commands both passed before the first build:

```sh
cargo clean
cargo llvm-cov clean --workspace
```

`cargo clean` removed 34,271 files / 3.2 GiB. The one lockfile refresh was
`cargo check -p stackpress-example-profile-codegen -p stackpress-example-generated-profile --offline`;
every later repository build/test/check/lint/doc command used `--locked`.

Logs are under ignored `target/lane-d-m4/`.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `cargo test -p stackpress-idea --all-targets --locked` | 2 passed. |
| `cargo test -p stackpress-example-profile-codegen --test generation --locked` | 4 passed, including actual native CLI diagnostics, exit policy, `-c`, no-write stale check and sanitized IO. |
| `cargo test -p stackpress-example-generated-profile --all-targets --locked` | 2 passed, also exercised by the workspace run and package coverage. |
| `cargo test --workspace --all-targets --locked --no-fail-fast` | **167 passed, 1 failed, 17 helper entries ignored**. Sole failure: final disabled-generation step of the consumer proof, the known M0 empty-registry restriction. |
| `cargo test --workspace --doc --locked` | 395 passed. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Passed warning-free. |
| `cargo test -p stackpress-example-profile-codegen --no-default-features --all-targets --locked` | Pending-integration regression fails at `Registry::plan`: `Idea plugins must match selected generator order`. This is the expected lane C dependency. |
| `cargo test -p stackpress-example-profile-codegen --test consumer --locked` | Reaches the same expected final disabled-generation failure; earlier assertions pass. See consumer-final.log. |
| `git diff --check`; owned Rust 100-column scan | Passed. |

Facade command matrix (each exited 0):

```sh
cargo test -p stackpress --no-default-features --test idea --locked
cargo test -p stackpress --no-default-features --test idea --locked --features idea-parser
cargo test -p stackpress --no-default-features --test idea --locked --features idea-transformer
cargo test -p stackpress --no-default-features --test idea --locked --features idea
cargo test -p stackpress --no-default-features --test idea --locked --features idea-parser,idea-transformer,idea
```

Results respectively: 0, 1, 1, 1 and 3 passed. The facade changes contain documentation and
re-exports only, with no executable production lines to measure. Feature import compilation
and normal dependency trees are its applicable proof rather than an empty 100% coverage claim.

## Clean per-package coverage

Toolchain: rustc 1.96.0 (`ac68faa20`), Cargo 1.96.0, rustfmt 1.9.0-stable,
cargo-llvm-cov 0.8.6; native `aarch64-apple-darwin` only.
Each measurement was independently preceded by `cargo llvm-cov clean --workspace`:

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-idea --all-targets --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/tests/|/target/|/crates/(lib|server|idea/(idea-parser|idea-transformer))/|/tools/|/examples/' \
  --json --output-path target/lane-d-m4/coverage-wrapper.json --fail-under-lines 70

cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-generated-profile --all-targets --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/(crates|tools|tests|target)/|/examples/(capability-host|generated-profile/codegen)/' \
  --json --output-path target/lane-d-m4/coverage-app.json --fail-under-lines 70

cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-profile-codegen --test generation --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/(crates|tools|tests|target)/|/examples/(capability-host|generated-profile/app)/' \
  --json --output-path target/lane-d-m4/coverage-codegen.json --fail-under-lines 70
```

| Package production scope | Covered / executable | Coverage |
| --- | --- | --- |
| stackpress-idea | 15 / 15 | 100.00% |
| stackpress-example-generated-profile | 85 / 90 | 94.44% |
| stackpress-example-profile-codegen | 151 / 153 | 98.69% |

Every command exited 0 and independently cleared 70%. Default example exclusions are disabled;
only dependencies, other packages, test harnesses and build output are excluded. No production
file/branch in the measured package is excluded. Generated server source is **74/74**, aggregate
registry **4/4**, handwritten app builder **7/7**; data-only contract/selection constants have no
separate executable line count. App main is **0/5** in this instrumented suite and is exercised
by the independent native consumer proof. Codegen native main is **5/5** in its measurement.

Codegen coverage deliberately runs the four-test native `generation` target; the separate
Cargo-consumer proof is pending integration and is **not claimed passed** by this measurement.
Its nested Cargo consumers use private uninstrumented targets. Disabled-feature coverage and
the production wrapper/composer path require collection after B/C integration.

## Dependency evidence

All of these commands exited 0; complete trees are saved as `tree-<package>.txt`:

```sh
cargo tree -p stackpress --edges normal --locked
cargo tree -p stackpress-lib --edges normal --locked
cargo tree -p stackpress-server --edges normal --locked
cargo tree -p stackpress-idea-parser --edges normal --locked
cargo tree -p stackpress-example-generated-profile --edges normal --locked
cargo tree -p stackpress-example-profile-codegen --edges normal --locked
```

Default facade/lib/server/app contain no `stackpress-idea-parser`, `stackpress-idea-transformer`,
`tempfile` or `sha2`. Parser remains independent of the runtime host. Codegen contains the
approved generation dependencies and has no generated-app dependency or crossterm edge.
The separately compiled contracts consumer's normal tree contains Serde/derive only, with
no Stackpress, tempfile or SHA dependency; the Rust consumer test asserts this before the
known final disabled-generation failure.

## Pending integration and remaining limits

1. Lane C's approved empty-selection planner change is required by `disabled.rs` and the final
   consumer step. The tests are active regressions; no no-op callback or test ignore hides it.
   Re-run selected/disabled suites, consumer proof, workspace checks and affected coverage after
   integration. The coordinator has the exact evidence and approved this behavior.
2. Lane B's source/composition exports are not present. The wrapper/tool still use M0's
   single-source parsing path. Integrate production composition and declared-input/planning
   exports only after the coordinator's merge notice; do not claim transitive imports here.
3. The latest disposable fixture adds explicit removed-model direct/HTTP/CLI 404 checks; its
   final test run is recorded separately from the earlier workspace count above.
4. Typed malformed source diagnostics are public; generic transform/formatting/ownership
   failures remain sanitized by the CLI. Verbose operator output exposes developer causes.
   There is no automatic force-recovery switch. Output still requires cooperative ownership.
5. The tool awaits its blocking generation worker and the native runtime waits during shutdown.
   No new killed-worker/cancellation or power-loss/platform proof is claimed by this lane.
6. No refactor rounds, commit/push, listener, publication, deployment, KB promotion or indexing
   occurred. Round release and independent acceptance remain coordinator-owned.
