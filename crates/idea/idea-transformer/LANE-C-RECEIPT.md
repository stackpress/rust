# Slice 3 lane C: M3 transforms and output writer

Implementation stop before refactor rounds. Coordinator re-verification and integration are
pending. No refactor round, commit, push, listener, dependency change or KB edit was performed.

## Baseline and owned files

- Branch: `trial/3-s3-lane-c-writer`.
- Base and unchanged HEAD: `54b188e3a5811f8d05ec6017fcb7382466660135`.
- Initially clean isolated worktree; sources were this worktree and its KB. Other lanes/trial
  branches were not read or used as build evidence.
- Implementation: `src/registry.rs`, `src/registry/context.rs`, `src/plan/mod.rs`, `src/output.rs`.
- Tests: `src/registry/tests.rs`, `src/plan/tests.rs`, `src/output/tests.rs`.
- `src/lib.rs` contains only added lane C module/re-export lines. Existing crate-level M0
  overview prose remains integration-owner work; it now understates implemented capabilities.
- Root manifests/lockfile, parser, composition/source, wrapper, examples, lib/server/CLI and
  `.agents/` are unchanged. The pre-existing `merge_type` test remains in the output test file.

## Implemented API and behavior

All M0 public names and function signatures remain available. The additions are:

- `TransformRegistry`, an alias for `Registry`; `TransformSpec`; `ContextTransform`.
- `Registry::add(spec, callback)` and
  `Registry::plan_with(schema, &TransformSelection, aggregates)`.
- `TransformContext`, `TransformInputs`, `TransformSelection`: immutable schema/plugin config,
  exact runtime-contract revisions, bounded explicit auxiliary snapshots and consumed-name logs.
- `OutputPlan::write`, `contribute_module`, `contributions`, `files`, `inputs` and existing
  `executed`. Registration supplies callback ownership; callers cannot fabricate active scope.
- `ModuleContribution`: ordered public module declarations. After callbacks succeed, one
  aggregate owner emits root/partition `mod.rs` files. Missing targets, duplicate contributions,
  case aliases, duplicate file owners and file/directory prefix collisions error.
- Legacy transforms can continue writing their own partition `mod.rs` when no contribution
  aggregate claims that path; they cannot overwrite a root aggregate or another owner's file.
- `GenerationReport` and `OutputWorkspace::install_report`. The M0 `install` still returns
  `CheckReport`. Reports contain logical consumed names, execution order, sorted relative
  generated/unchanged/removed paths and writer contract revision 1. They contain no timestamps,
  native roots, input contents or environment values.

Registration validates bounded metadata, unique/nonempty IDs, reserved aggregate identity and
case-insensitive partition exclusivity before mutation. Planning validates the whole selection,
required and optional-present prerequisites, listed order and runtime contracts before *any*
callback. Missing/disabled/self/cyclic/later requirements error; nothing is auto-reordered.
Callback Result failures preserve the original cause; contextual callback Display adds the
plugin ID without echoing callback input values. Panics propagate and cannot install a plan.

The coordinator-approved selection table is documented in `Registry::plan` Rustdoc and tested:

| Root plugin blocks | Registered selection | Result |
| --- | --- | --- |
| None | Nonempty | Run the explicitly registered defaults in listed order, unchanged from M0. |
| None | Empty | Succeed with aggregate-only output; installation removes old owned partitions. |
| Present | Any | Require exact IDs and order. Unknown/unregistered, extra selected or mismatched IDs reject. |

Output and auxiliary inputs each retain a 1 MiB/1024-file budget; paths are portable relative
ASCII, at most 240 bytes and 16 components. Metadata/contract collections are bounded as well.
The planner never performs managed filesystem writes. Rust formatting precedes installation.
Generated module text uses LF/final newlines; legacy non-Rust file contents remain verbatim.

## Output state machine and ownership protections

The real runner stages a complete marked tree, publishes intent, retains the prior owned root,
installs the stage and clears validated retained trees/intent. Installation errors identify the
failed phase and retain their underlying error chain. A normal repeat does not rewrite output.

- SHA-256 and optional installed byte lengths detect edited owned files before replacement or
  removal. Newly written manifests include lengths. M0 v1 manifests without lengths remain
  readable and current when their exact ownership/content match; no forced migration occurs.
- Marker deserialization rejects unknown fields and duplicate ownership keys. File paths,
  owners, digests, total bytes, exact file/directory set and case/prefix collisions are checked.
- Nonempty unmarked roots, missing/edited promised files, manual files/directories and special
  or symlink paths block changes. Root ancestors, data, marker, stage, backup, journal and lock
  surfaces are covered. A dangling recovery sibling is not mistaken for absence.
- Every retained tree is validated before any recovery rename or cleanup. Recovery prefers a
  valid installed root, otherwise restores the complete backup, otherwise the first complete
  stage. An initially empty adopted root has no owned predecessor and recovers to the stage.
- Cleanup intent is published through a same-parent temporary-file rename before deletion.
  Only this recognized state tolerates already removed promised backup files or an empty
  leftover cleanup directory, and it requires a valid installed root. Unknown/edited remaining
  files still block. A partial backup is never restored as the destination.
- Create-new locks fail immediately for competitors. PID text is informational only; there is
  no waiting or automatic stale-PID reclamation. Normal returns release the acquired lock.

## Failure, lock and recovery matrix proved

`complete_install_failure_matrix` injects IO errors into the actual runner at ten boundaries,
for absent and owned roots (19 cases; first install has no backup cleanup). Stage-write uses
`StorageFull`, intent-write uses `WriteZero`, and other injected operations use
`PermissionDenied`. These are injected errors, not claims of exhausting the physical disk.

| Failure boundary | Retained state and observed outcome |
| --- | --- |
| Stage write / stage publication | Temporary stage is dropped; prior/absent destination unchanged; no journal; retry succeeds. |
| Intent write, before publication | Complete fixed stage, no recognized intent; check/install refuse; recover refuses; explicit inspection/removal permits retry. |
| Intent published / before backup rename | Old owned root remains; recovery keeps it and clears the stage. First install recovers the complete new stage. |
| Backup retained / before install rename | Destination absent; complete backup and stage remain. Recovery restores old output, or installs stage on first install. |
| Stage installed | Complete new root wins; recovery validates/discards the prior backup and clears intent. |
| Backup deletion | Complete new root plus cleanup intent/backup remain; recovery resumes validated cleanup. |
| Intent clearing | New root is complete; recovery clears the final recognized intent. |

Additional native proofs:

- Nine killed-process cases at explicit ready/go barriers: intent published, backup retained,
  stage installed, backup cleanup and intent clearing, with absent/owned roots as applicable.
  Parents use monotonic five-second deadlines, kill and reap. A stale lock blocks recovery
  until the parent has reaped the writer and explicitly removes its lock as the operator.
- A separate-process competitor observes a lock acquired before spawn and fails promptly.
- A callback-panic child exits unsuccessfully under a kill/reap watchdog; prior bytes/marker
  remain intact, without a writer lock, stage or successful installation.
- Real native permission denial during backup deletion returns failure with the complete new
  root and cleanup intent retained. Permissions are restored, and recovery succeeds. This
  host did not bypass the permissions; the test would fail rather than claim proof if it did.
- Interrupted cleanup is also exercised with missing owned files and an empty markerless
  leftover directory. A manual sentinel in partial cleanup blocks and survives recovery.
- Invalid/unrecognized/partial journals, missing recovery trees, pre-intent orphan siblings,
  unmarked retained trees, malformed/duplicate/unknown-version markers, length/hash drift,
  missing owned files and manual sentinels refuse recovery/writes without silent deletion.
- Empty plan/root marker installation, clean/repeat/check, changed bytes, removed files and
  empty-selection partition cleanup pass. Check leaves generated bytes/marker untouched;
  independent temporary roots serialize identical reports.
- Initially empty roots recover at intent, backup, install and final-intent boundaries.

## Exact verification commands and results

Before the first build, both exited 0:

```sh
cargo clean
cargo llvm-cov clean --workspace
```

Final build/test checks all exit 0; logs and machine-readable command/results are in
`target/lane-c-checks/` (`summary.json`, individual `<check>.log` files):

```sh
cargo fmt --all -- --check
cargo test -p stackpress-idea-transformer --all-targets --locked
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo check -p stackpress --features idea,idea-parser,idea-transformer --locked
```

- Transformer: **33 passed, 0 failed**, plus 3 child-only helper entries ignored in ordinary
  enumeration and exercised by their watchdog parents (lock, killed install, callback panic).
- Workspace: **189 passed, 0 failed**, 19 enumerated helper entries ignored.
- Workspace doctests: **403 passed, 0 failed**.
- Clippy/Rustdoc warning-free. `git diff --check` passes; no Rust lines exceed 100 columns.
- Normal-edge trees were recorded with the following commands (all exit 0):

```sh
cargo tree -p stackpress --edges normal --locked
cargo tree -p stackpress-lib --edges normal --locked
cargo tree -p stackpress-server --edges normal --locked
cargo tree -p stackpress-idea-transformer --edges normal --locked
cargo tree -p stackpress-example-generated-profile --edges normal --locked
cargo tree -p stackpress-example-profile-codegen --edges normal --locked
```

The default facade, lib, server and generated application trees contain no Idea generator,
`tempfile` or `sha2` dependency. No manifest/lockfile was changed. G22 is unchanged.

## Clean production coverage

Final measurement followed another clean:

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-idea-transformer --all-targets --locked --no-default-ignore-filename-regex --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/tests/|/src/(output|registry|plan)/tests.rs|/target/' --json --output-path target/lane-c-coverage.json
```

Production result: **975 / 1,013 executable lines (96.25%)**, above the 70% development
floor. `target/lane-c-checks/coverage-summary.json` records each production source file's
numerator/denominator. The count sums only `crates/idea/idea-transformer/src/` files in the JSON
and fails if empty or below 70%; parser execution is not used to inflate the changed crate.

Exclusions are toolchain/dependencies, build output and separate test harness files, including
`src/{output,registry,plan}/tests.rs`. The M0-only interruption adapter was placed in the test
file so no test-only helper counts as production. No production branch or generated shipped
code is excluded. Only this lane's transformer scope is claimed, not other lanes' coverage.
Native rustc/cargo **1.96.0**, cargo-llvm-cov **0.8.6**, **aarch64-apple-darwin**, default crate
features. The remaining unmeasured branches include rare native IO/process/formatter plumbing
failures; coverage does not prove all branch combinations or all filesystem failure modes.

## Limitations and integration handoff

- This is the M3 implementation gate, **before rounds 1–4** and independent coordinator review.
  It does not authorize merge, publication or claim whole Slice 3 completion.
- Cooperative exclusive ownership is required. SHA-256 is drift detection, not marker
  authentication or protection from an outside writer racing inspection/replacement.
- No crash-atomic visibility, power-loss durability or network-filesystem guarantee. Kills
  cover explicit state barriers; partial purge is separately simulated and permission-tested.
- A killed writer leaves its lock for operator inspection/removal. A stage before recognized
  intent, malformed journal or random temporary directory left by process death requires
  operator inspection. The library does not guess authorization or purge those paths.
- Legacy callback errors retain their M0 display behavior; plugins are trusted Rust and can
  deliberately do direct IO or emit sensitive text. New contextual public errors omit callback
  values while preserving the original developer cause. No plugin sandbox is claimed.
- Auxiliary snapshots/logs here cover explicit auxiliary inputs only. Composition/root/import
  input provenance and wrapper report integration belong to lanes B/D, not a second loader.
- Rustfmt is an explicit installed toolchain dependency; stability assumes the same formatter.
  Native permission/symlink/rename proofs ran on macOS only; Linux/Windows/WASM are unverified.
- Shared `src/lib.rs` overview documentation and wrapper/export integration remain lane D's
  shared-file responsibility; lane C added only its module/re-export lines.
