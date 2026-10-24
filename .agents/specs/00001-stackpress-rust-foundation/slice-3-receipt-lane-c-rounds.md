# Slice 3 lane C refactoring rounds

Registry, in-memory plan and owned-output implementation receipt. Stopped after the requested
rounds; independent coordinator review is pending. No commit, push, listener, dependency change
or public API change was made.

## Scope and baseline

Branch `trial/3-s3-lane-c-writer`, baseline/unchanged HEAD
`8bdca2c299a93f42405d809bdcbf063a33a3623b`; initially clean worktree. Both `cargo clean` and
`cargo llvm-cov clean --workspace` passed before the first build.

Write set: transformer `src/registry.rs`, `src/plan/**`, `src/output.rs`, `src/output/**`, their
tests, and this receipt. Compose/source/lib.rs, manifests and other lanes' files are unchanged.
A source-level comparison preserves all **49 public function/type declarations** in the owned
modules, including signatures and multiplicity; root re-exports were not edited. Workspace
consumer tests exercise the existing exported paths.

Baseline `cargo test -p stackpress-idea-transformer --all-targets --locked`:
**65 passed, 0 failed, 3 child-only helper entries ignored in ordinary enumeration**.
Evidence is under `target/lane-c-rounds/`.

## Round 1: complexity, behavior preserved

Registration insertion mixed ID, partition and prerequisite/contract validation. Extracted
partition checks and `TransformSpec::validate_requirements`, retaining original validation
order, error precedence and all-or-nothing registration mutation.

Recovery mixed candidate validation with nested destination restoration. Extracted
`restore_root`: every retained candidate is validated before the first mutation; the existing
root, backup and first stage retain the same preference and filesystem effects.

Verification: **65 crate tests passed, 0 failed**, 3 helper entries ignored;
`round-1.log`.

## Round 2: responsibility and abstraction, behavior preserved

- `output/ownership.rs` owns marker serialization, hashing, native-tree inspection and cleanup
  authority; `output/format.rs` owns rustfmt process handling; `output/report.rs` owns stable
  differences and receipts. OutputWorkspace retains lock/install/recovery orchestration.
- Check and install share `compare_plan`, under the same existing writer lock: readiness,
  inspection, exact owner/content comparison and missing-marker reporting.
- Portable-path rules moved to `plan/path.rs`, beside collision policy. Registry retains its
  internal re-export for existing callers. Registry owns selection/execution; plans own claims
  and ordered contributions. No general filesystem trait or runtime was introduced.

Verification: **65 crate tests passed, 0 failed**, 3 helper entries ignored;
`round-2.log`.

## Round 3: consistency, Rustdoc and comments, behavior preserved

Clarified empty partition-list behavior; added a compiling `plan_with` example with an explicit
auxiliary snapshot; explained run-local logs and callback authority. Added local explanations
for formatting/lock sequencing, repeat installs, ownership checks, byte accounting, module
ordering and native names. Named existing journal payloads as constants without changing bytes.
New modules document their actual responsibility; final comments also explain the new safety
boundaries and formatter child ownership.

Verification: **65 crate tests passed**, 3 helper entries ignored; **12 crate doctests passed**;
no failures. Logs: `round-3.log`, `round-3-doc.log`.

## Round 4: targeted safety fixes, observable behavior changes

Three material findings justified the final targeted round. These are separate from the
behavior-preserving rounds. Exported names, signatures and public struct fields remain intact.

| Defect | Minimal fix and resulting behavior | Proof |
| --- | --- | --- |
| Full-path folding missed shared-directory aliases with different leaves, such as `data/Models/a.rs` and `data/models/b.rs`. | Compare validated components. A shared folded directory needs one exact spelling; unrelated branches remain valid. Plans and installed markers share the stricter rule. | The regression failed before the fix with the alias accepted, then passed. Ordinary same-spelling siblings remain valid. |
| Bounded reads could block opening a FIFO marker, promised file or journal before rejecting special files. | Check symlinks and regular-file metadata before opening managed text; special paths return an error promptly. | All three old cases blocked for five seconds and were killed/reaped by the watchdog parent. All three pass after the fix. |
| Whole-tree recursive deletion could remove the ownership marker before a later deletion failed. | Purge validated non-marker entries first; preserve the root marker through partial errors; remove it only when no other entries remain, then remove the empty root. | Four actual deletion-boundary faults recover. Real permission-denied deletion retains the marker. A killed child at the new cleanup-entry barrier recovers after explicit stale-lock inspection/removal. |

The per-entry regression initially failed because the old all-at-once purge offered no such
interruption boundary. The marker-order risk itself follows from `remove_dir_all`'s unspecified
traversal order; the fix makes recovery independent of that order. Existing unmarked/nonempty
retained trees are still refused, never retroactively claimed or purged. Deletion IO errors may
now name `CleanupEntry` rather than the coarser `CleanupTree` phase.

Additional tests cover unchanged protections: forged traversal/absolute/control-character
marker paths preserve an external sentinel; dangling stage/backup/journal/lock links survive
refusal; unrelated temporary siblings survive installs; non-UTF-8 owned/marker contents reject;
non-UTF-8 root arguments reject before IO; check and install return identical differences for
add/repeat/change/remove plans. Existing failure/lock/recovery, digest, manual-file, formatting
and panic proofs remain green.

The host refused creation of non-UTF-8 file/directory names with **EPERM (OS error 1)**.
Native APFS refuses these names with **EILSEQ (92)**; the sandbox reports **EPERM (1)**.
That native-name inspection branch runs only on Unix hosts that permit such names; it is not
claimed proved here. `non-utf8-host.log` records this limitation. Content and root-argument
rejection are proved here; no lossy filename conversions or unsafe code were introduced.

Verification: **72 crate tests passed, 0 failed**, 4 child-only helper entries ignored in normal
enumeration and exercised by watchdog parents; **12 doctests passed**. Final review found no
additional material issue within the cooperative-writer contract requiring another round.

## Exact final verification

All commands below exited 0. Logs and argv/results are under `target/lane-c-rounds/`, including
`summary.json` and individual `<check>.log` files.

```sh
cargo fmt --all -- --check
cargo test -p stackpress-idea-transformer --all-targets --locked
cargo test -p stackpress-idea-transformer --doc --locked
cargo clippy -p stackpress-idea-transformer --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-idea-transformer --no-deps --locked
cargo test --workspace --all-targets --locked
```

Crate all-targets: **72 passed**, 4 helper entries ignored, no failures. Crate doctests:
**12 passed**, no failures. Workspace consumers: **243 passed**, 22 helper entries ignored,
no failures. Formatting, warning-clean Clippy and warning-clean Rustdoc passed. `git diff
--check` passed, and no owned Rust line exceeds 100 columns.

Before-fix logs: `regression-case-before.log`, `regression-fifo-before.log`,
`regression-cleanup-before.log`; all three targeted commands exited 101 as expected.
Host encoding command:

```sh
cargo test -p stackpress-idea-transformer --lib --locked non_utf8_names_and_owned_text_preserve_the_tree -- --nocapture
```

## Clean coverage

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-idea-transformer --all-targets --locked --no-default-ignore-filename-regex --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/tests/|/tests\.rs$|/target/' --json --output-path target/lane-c-rounds-coverage.json
```

Whole transformer production: **1,804 / 1,869 executable lines (96.52%)**, above the 70%
development floor. Lane C registry/plan/output subtotal: **963 / 992 (97.08%)**. Unchanged
composition/source coverage does not hide an untested owned subsystem. `coverage-summary.json`
records per-file counts; its summation filters JSON entries to this crate's production `src/`
and rejects an empty measurement or less than 70%.

Exclusions: toolchain/dependencies, build output, integration test directories and separate
`tests.rs` harness files, including unchanged composition unit tests. No production branch is
excluded. Native rustc/cargo **1.96.0**, cargo-llvm-cov **0.8.6**, **aarch64-apple-darwin**,
default features. Rare native-IO/formatter failure plumbing remains partly uncovered; line
coverage is not exhaustive branch or crash-durability proof.

## Boundaries and stop

Cooperative exclusive ownership remains required. Regular-file/symlink checks do not defeat
an outside writer racing inspection and open/rename/delete. SHA-256 detects drift, not a
maliciously forged internally consistent marker. No crash-atomic visibility, power-loss,
network-filesystem, Linux/Windows or WASM claim is added.

Stale process-death locks and pre-intent/malformed/unrecognized temporary state still require
operator inspection. The writer does not auto-reclaim a PID or guess ownership of unknown
siblings. Kills cover explicit barriers; permission faults and injected errors cover partial
transitions, not arbitrary power failure. No new product decision or public API is proposed.
Stop for coordinator review; no commit was made.

The attempted Paseo coordinator status-message call was rejected by approval with no stated
reason. It was not retried; findings are preserved in this receipt. Writing this requested
receipt required escalation because the environment protects `.agents/` as read-only.

## Agent Workspace validation

`python3 .agents/scripts/validate-agent-workspace.py` exited **1** with **146 missing-link
errors and 19 warnings**. The errors concern unchanged documents whose upstream-relative
links resolve to absent sibling checkouts in this isolated worktree. None names this new
receipt. Those files were not changed: repairing the repository-wide link layout is outside
this lane. Full output is `target/lane-c-rounds/kb-validator.log`.

The final tracked/untracked path audit passes; `scope-audit.json` records the write set.
Public source comparison preserves **49 function/type declarations and 12 public fields**.
All required code checks and the clean coverage gate pass independently of KB link validation.
