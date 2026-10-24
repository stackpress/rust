# Slice 3 risks, milestones and verification

Design checkpoint for [task 00006](tasks/00006-slice-3-idea.md). Load for the implementer's risk inventory, proving tests, gated milestones, four possible lanes, coverage treatment and rounds 1–4; release depends on [checkpoint decisions](slice-3-design-decisions.md).

## Risk inventory and proving tests

This is an open-ended safety mandate. The list is a starting set, not a waiver for newly discovered concerns. Existing event/lifecycle safety remains relevant at the generator-host boundary; pure parser/transformer work should not add locks/async/global registries merely to reproduce prior failure classes.

| ID / risk | Prevention and proving test |
| --- | --- |
| R01 Stack exhaustion on parse/compile/serialize/clone/drop | Bound recursion before descent, bound public constructed input, avoid unrestricted mutable recursive state. Child-process test reproduces deep source/JSON/AST, expects a diagnostic and normal exit within watchdog; test the maximum accepted depth and one above. |
| R02 Memory/CPU amplification from large tokens/references/import diamonds | Per-source and aggregate bytes/nodes/entries/depth/file counts plus bounded reference expansion/output bytes. Test wide/deep combined inputs and repeated refs/diamond graphs, including limits lowered by caller; no giant allocations before the check. |
| R03 UTF-8 slicing or bogus EOF/source ranges | Checked half-open byte spans and source IDs. Exhaustive representative Unicode/CRLF/EOF cases assert every endpoint lies within the correct source and on a character boundary; rendered file/line/column matches expectations. |
| R04 Tokenizer/parser grammar divergence | Shared cursor/literal scanning policy. Corpus tests compare accepted literals/token boundaries, env/comments/modifiers, unterminated values and errors across both entrypoints. |
| R05 Ambient environment nondeterminism/process mutation | Pure syntax plus explicit immutable env map. Test two isolated concurrent parses with different maps and no process changes; child-process CLI env tests cover absent/empty/non-Unicode allowed policy without set_var in test threads. |
| R06 Numeric overflow/nonfinite/null corruption | Exact integer range and finite float checks. Boundary tests i64/u64 ends, one outside, huge float, negative zero and decimals; no accepted number silently becomes null. |
| R07 Duplicate or malformed declarations silently erased | Approved strict/compatibility decision table, typed builders, related spans. Tests duplicate top-level/member keys, invalid namespaces/types/modifiers, malformed trailing input and missing braces; compare genuine compiled fixtures. |
| R08 Reference/literal confusion, wrong declaration order | Preserve typed reference/env nodes through compose/compile; separate literal `${Name}`. Tests prior/forward/unknown references, imported prop references, reference expansion budget and literal lookalikes. |
| R09 Import cycles, aliases or diamonds misclassified | Canonical identities, active recursion stack and immutable per-run snapshots. Test direct/indirect/symlink cycles and chain diagnostics; diamond imports must succeed without shared mutation. |
| R10 Wrong root/cwd or ambiguous resolution | Explicit project root/importer base and documented candidate order. Test nested imports outside process cwd, `@/`, absolute/relative/index paths, exact file precedence, missing vs directory vs permission-denied, and invalid UTF-8 source. |
| R11 Immutable-local/merge precedence mismatch | Accepted merge decision table with provenance. Tests local/child mutable/immutable/absent × attribute/column collision, multiple imports and order; cached child remains unchanged after both parents. |
| R12 Unvalidated JSON imports bypass limits/order | Bounded deserializer plus typed structural validation. Test wrong roots/kinds/fields/duplicate keys, depth, invalid spans/numbers and roundtrip member order. Normalized JSON cannot activate unknown executable plugins. |
| R13 Transform reordering or missing dependencies | Listed-order registry with full preflight. Test z,a order, equal declared order, missing/disabled/self/cyclic/wrong-order prerequisites, duplicate IDs and schema order mismatch; callbacks record that none ran after a failed preflight. |
| R14 Callback failure/panic mistaken for successful generation | Plan in memory before disk installation; preserve source/plugin error. Return error and unchanged output for Result failure; child-process panic test verifies prior tree intact and no successful receipt. Do not promise panic recovery. |
| R15 Generated-code bootstrap cycle | Codegen has no generated-app dependency or generated imports. Delete its output fixture, build/run tool alone, then compile app. Inspect cargo tree/metadata; adding a back-edge must fail review. |
| R16 Generator leaks to runtime/browser dependencies | Opt-in facade and separate tool package. Normal-edge trees for facade default, app and serde-only contracts consumer exclude parser/transformer/tempfile/codegen; compile contracts independently. No WASM support claim without a target run. |
| R17 Code injection/name/path collisions | Validate generated Rust identifiers/module paths; escape string literals for Rust rather than paste source; canonical relative output validation. Compile fields with keywords/Unicode strings/control characters/quotes; reject unsupported identifiers, snake/case-normalization collisions and traversal. |
| R18 Nonstable generated bytes or registration duplication | Stable ordered plan/aggregates, fixed formatter/header/newlines. Clean→repeat→repeat compares exact bytes and file list; run on differing cwd/temp roots; ensure one runtime operation registration. Toolchain version is part of the result. |
| R19 Stale model/plugin output remains active | Exact owned manifest and fresh aggregates. Remove model, generator selection or rename a supported fixture model; check files/exports/registration removed and remaining generated consumer compiles. Profile-only generator rejects unsupported rename before mutation. |
| R20 Accidental destruction/manual-file overwrite | Dedicated marked root and per-file ownership; no recursive purge of unmarked dirs. Sentinel/manual-edit/unknown-file/foreign-root/malicious-manifest tests preserve bytes and return conflict. CLI -f does not authorize deletion. |
| R21 Symlink/path race or case-insensitive collision | Reject output symlink components, validate canonical parent, require cooperative exclusive root ownership. Native symlink/case fixtures test rejection; document uncooperative writers outside guarantee instead of claiming stale checks solve them. |
| R22 Concurrent writers, stale locks or indefinite waiting | Create-new cooperative lock outside replaced tree, no auto-reclamation. Two-process test with explicit barrier proves second writer fails promptly; stale-lock test gives actionable recovery and does not hang. |
| R23 Partial filesystem install, formatter failure or power interruption | Stage before publish and explicit backup/journal/recovery state. Inject IO failure at each transition (permission/disk/write/rename/delete), kill child at install barriers, verify recoverable original or complete new tree and next-run refusal/recovery. No unproved atomicity/durability promise. |
| R24 Undeclared input or changing source during generation | Context records auxiliary reads; snapshots/inputs/check tool explicit. Tests change transitive source/env/config/selection, then --check detects changed output; source-change injection during acquisition fails deterministically where supported. Trusted plugin direct IO is reviewable, not sandboxed. |
| R25 Feature/selection/runtime contract mismatch | Shared selection source plus generated const flag/revision check, no inferred cfg equivalence. Build matching enabled/disabled configurations; deliberately mismatched generated selection and contract revision must fail app compilation. |
| R26 Blocking async executor or detached generation worker | Pure synchronous core; awaited bounded tool worker with explicit lifetime/lock ownership. Single-thread runtime responsiveness/cancellation process tests prove no lock released while a worker still writes; no detached success report. |
| R27 Error cause/provenance lost or source/env leaked | Typed phase/plugin/source/IO chain and safe public rendering. Induce child parse/IO/transform/formatter errors and compare logical file/line/column. Secret sentinels absent from stdout/manifests/public source snippets; verbose output still omits env values. |
| R28 Deadlock/poison/global state or Clone aliasing regression | Prefer owned/immutable state and no framework shared locks. Concurrent separate runs and clone-independence tests; if a lock appears, require deliberate poison policy and release-before-await/callback proof with child-process watchdog. Existing host nesting/Send tests remain green. |
| R29 Unsound imported unsafe/test shortcuts | Workspace unsafe forbid and explicit env maps; static scan and Clippy. No copied unsafe set_var, lifetime extension, unchecked UTF-8 or unsound Send/Sync. Any unavoidable unsafe triggers the evidence-backed veto/review before merge. |
| R30 Misleading coverage or mock-only proof | Measure every changed crate separately, include generated executable app code, real temporary files/CLI processes and actual generated compilation. Record skipped targets/features and critical branch gaps even above 70%. |
| R31 Public API/docs lock in nonexistent semantics | Doctests use real APIs after implementation, Rustdoc missing_docs deny, dense local explanations. Compile API examples and review inventory dispositions. Don't present design snippets as existing contracts. |
| R32 Temp/backup handles leak after normal error | RAII cleanup for completed rollback, explicit recovery-state retention on interrupted install. Fault-injection tests inspect leftover files/locks/open resources and ensure reruns are deterministic. Retaining recovery data is named, not silently deleted. |
| R33 Host/platform assumptions masquerade as portability | Prove native std filesystem behavior on installed target, including symlinks and rename transitions. Windows/Linux, editor WASM and network filesystems remain unverified until dedicated runs; no case-sensitive-only uniqueness policy. |

## M0: riskiest vertical proof, then stop

After accepted decisions only, one implementer owns scaffold/shared files during M0. Create actual parser/transformer/example code with the smallest real contracts needed to prove these connected risks:

1. Extract the pinned core with provenance; hard-limit nested parsing/compilation and valid source diagnostics; prove the existing deep abort becomes a bounded diagnostic. Preserve complete schema fixture outputs and explicit plugin z,a order.
2. Independently build codegen with all generated output absent, parse Profile, run one registered transform, stage/format/install its contracts/registration, and compile the real generated consumer using existing server signatures. No mock emitter or handwritten surrogate generated types.
3. Repeat generation byte-for-byte; remove Profile and verify file/registration cleanup, an empty compiling registry and operation 404. Compile contracts without server/parser dependencies.
4. Prove selection/revision mismatch is a compile failure. Exercise writer locking and one injected install interruption/recovery before promising the output contract. If portable safe recovery needs a broader library/OS API, stop with evidence and revise the design; do not improvise destructive cleanup.

M0 success means signatures, boundedness, bootstrap, ownership and generated compilation are proved, not full parser/transformer completion. Record exact commands and all limitations. **Stop for coordinator re-verification.**

## M1–M4 and gates

| Milestone | Deliverable and required signal |
| --- | --- |
| M1 parser completeness | Approved language table; full pinned/current compiled fixtures; typed JSON import/export; source spans/env/limits/references; tokenizer consistency; parser docs/comments; standalone parser tests. |
| M2 composition | Native/in-memory loader and order/provenance; imported props/JSON, cycles/diamonds/limits and IO failures; upstream composition behavior per accepted correction; no child schema mutation. |
| M3 transforms/output | Selected registration/preflight/context, output partitions/aggregates, full failure/lock/recovery matrix, clean/repeat/remove/check semantics; wrapper and facade wiring; effective dependency checks. |
| M4 generated application and DX | One-command app codegen, generated contracts and search registration; CLI/direct/buffered HTTP comparisons and malformed-input diagnostics; selected/disabled builds, no stale outputs, independent consumer compile; per-crate coverage. |

A milestone's tests pass before dependent work proceeds. Re-run earlier proofs with production types replacing M0 scaffolding. **Stop before refactor rounds**, reporting per-crate coverage, inventory completion/approved deferrals, remaining branch risks and any change to the API/dependency budget. No automatic publication, commit, deployment or KB promotion follows.

## Up to four implementation lanes after M0 interfaces are frozen

The coordinator creates and releases each lane; this checkpoint does not spawn agents. Shared-checkout ownership is exclusive. Use isolated worktrees when write sets overlap, with coordinator integration. The parallel docs implementer retains exclusive ownership of `crates/server/docs/` and `tools/cli/docs/` throughout.

| Lane | Exclusive write set | Dependency and boundary |
| --- | --- | --- |
| A parser | `crates/idea/idea-parser/**`, its tests and parser receipt | Starts from frozen M0 parser API; no transformer/output/app/shared manifest edits. |
| B composition | transformer `src/compose/**`, `src/source/**`, composition/source tests and receipt | Uses frozen parser contracts; propose API changes to integration owner before editing other modules. |
| C transforms/writer | transformer `src/registry.rs`, `src/plan/**`, `src/output/**`, corresponding tests and receipt | Owns output recovery implementation; no composition/app/shared exports edits. |
| D integration/example | wrapper `crates/idea/idea/**`, `examples/generated-profile/**`, integration tests and receipt | Sole implementer owner of root Cargo.toml/lock, facade src/lib.rs, transformer manifest/src/lib.rs and aggregate test module declarations. Integrates A–C exports by agreed patch. |

Lane D owns shared files only by coordinator assignment. No other lane runs commands that mutate the root lockfile; request dependency edits through D, then use `--locked`. No lane edits existing lib/server/CLI source or the prior example without a scoped finding/approval; these should not need changes. Spec index/task status/verification notes stay coordinator-owned after this checkpoint unless explicitly reassigned; lane receipts use unique filenames linked by the coordinator.

## Test execution and fail-fast policy

Resource exhaustion, deadlock, concurrent writer, panic/kill and worker-cancellation tests run child processes with finite input, explicit ready/go barriers and parent watchdogs. Use `try_wait` with a monotonic deadline; kill and reap on timeout, preserving useful stderr. Typical child deadline 5–10 seconds; compilation subprocesses receive a separately justified longer deadline. No sleeping forever, unbounded join, elapsed-time-only race assertion or ignored critical-path test.

Meaningful native IO tests use tempfile roots and inject errors through the actual source/output seam. Permission tests must assert the intended error is reproducible on this user/host; skip with a recorded limitation if host privileges bypass it, rather than claim proof. Crash recovery probes only disposable roots, never repository/user output. This is authorized proof/test work, not permission for destructive workspace commands.

## Coverage and final checks

Before a fresh implementation build, honor the workflow's `cargo clean` and `cargo llvm-cov clean --workspace`; coordinate this with other active lanes so shared build output is not deleted under another run. Build artifacts from other trial branches are never evidence. Assessment used a new empty extraction and did not run a repository build.

Development floor is **at least 70% executable lines per changed crate/valid feature scope**, including wrapper, facade changes, codegen tool and generated application. No workspace-average substitution. Use the generated app source as executed production code; cargo-llvm-cov's default example exclusions must be disabled/replaced explicitly, as in the [Slice 2 app receipt](slice-2-receipt-lane-d-app.md). If excluded code or 0-line reports appear, fix the measurement before claiming a pass. Publication floor remains 90%, with separate authorization.

Receipt commands, adapted to actual selected members/features after implementation:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo llvm-cov -p stackpress-idea-parser --fail-under-lines 70
cargo llvm-cov -p stackpress-idea-transformer --fail-under-lines 70
cargo llvm-cov -p stackpress-idea --fail-under-lines 70
cargo tree -p stackpress --edges normal
python3 .agents/scripts/validate-agent-workspace.py
```

Also run locked enabled/disabled facade and example feature combinations, scoped tool/app coverage with example production files explicitly included, the actual generation/check/consumer compile commands, and normal-edge trees for parser/tool/app/contracts. The final receipt supplies exact numerator/denominator, target/toolchain/tool/features/exclusions and all command variants; placeholders above are a plan, not passed evidence. Only the installed native target is assumed available; additional targets require a real toolchain and run.

## Rounds 1–4 after behavior is proven

1. **Logic/complexity:** review scanner decision paths, modifier/reference error precedence, merge order, registry validation, budget arithmetic, output state transitions and uncovered branches. Refactor only where justified; rerun affected tests and all changed critical paths.
2. **Responsibility/abstraction:** review parser versus metadata meaning, immutable schema/source provenance, composer versus registry/writer, tool versus app, owned outputs and dependency edges. Remove accidental coupling and unnecessary wrappers; rerun contracts, compile/import and tree checks.
3. **Consistency/Rustdoc/comments:** align error/API/config naming, named handlers, declarations/examples, module/public/private helper docs and per-block/branch explanations. Generated source follows the same human-readability contract. Run tests, doctests, fmt, warning-clean Clippy/Rustdoc.
4. **Safety/soundness:** independent open-ended inspection of recursion/drop/serialization, overflow/allocation, Unicode, input authority/secrets, FS races/interruption/recovery, worker ownership, cfg/additive dependencies, cloning/locks and test reliability. Add meaningful regressions for new findings, then rerun tests and per-crate coverage.

Each round records findings, changes or justified no-change result and verification. Repeat targeted rounds while material issues remain; no ceremonial rewrites. Coordinator independently reruns the final checks/proofs and reviews breadth/DX against the inventory. Full crate documentation is a follow-up gated task using chrisai-docing; design sketches are not its replacement.
