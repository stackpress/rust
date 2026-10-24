# Slice 3 lane A: parser refactoring rounds

Receipt for [task 00006](tasks/00006-slice-3-idea.md). Load for the parser's four
behavior-preserving rounds, frozen-interface checks and final per-crate evidence.
The [Rust programming workflow](../../workflows/rust-programming.md) governs these rounds.

## Scope and baseline

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s3-lane-a-parser`.
- Branch: `trial/3-s3-lane-a-parser`; starting/current HEAD:
  `8bdca2c299a93f42405d809bdcbf063a33a3623b` (all four Slice 3 lanes integrated).
- Starting worktree was clean. Production/test/provenance changes are confined to
  `crates/idea/idea-parser/**`; this specifically requested receipt is the only KB addition.
- No commits, dependency/lockfile/shared-manifest changes, listeners, upstream changes,
  other trial-branch inspection, publication or deployment.
- Before the first fresh build, `cargo clean` and `cargo llvm-cov clean --workspace`
  both exited 0. Baseline `cargo test -p stackpress-idea-parser --all-targets --locked`
  passed: **19 tests**, 0 failures, three watchdog child helpers.
- All exported names, signatures, public fields, enum variants and serialized shapes remain
  frozen, including `Reference` and its declaration/column/argument-path representation.

## Round 1: logic and complexity

Findings:

- Reference discovery mixed declaration/column traversal, nested argument traversal,
  metadata-budget accounting and record construction. It had deep nesting and a
  nine-argument helper with a Clippy allowance.
- Consuming and lookahead word readers independently encoded the same lexical policy.
  Four identifier readers duplicated validation and error construction.
- Invalid structural reference-site branches and error precedence needed more direct tests.

Changes:

- Introduced private `ReferenceCollector` state and a borrowed `ReferenceOwner`, with
  focused declaration, column, argument and record-building steps. Removed the allowance.
  Original declaration order, nested-path order, forward flags, byte-accounting constants,
  source spans and budget/error precedence are preserved.
- Both word readers now use one `word_end` rule. Identifier readers share one checked
  private helper while retaining their existing casing predicates and diagnostic text/ranges.
- Added `tests/rounds.rs`: fixed reference/token/error JSON-shape assertions; source-ordered
  nested argument/column references; 19 invalid reference-site cases; flag destinations;
  comments/delimiters/non-ASCII word edges; invalid-site versus outstanding-import ordering;
  and caller-data preservation on failure.

Verification: `cargo test -p stackpress-idea-parser --all-targets --locked` exited 0:
**22 tests passed**, 0 failed, three child helpers. Log: `target/lane-a-round-1.log`.

## Round 2: responsibility and abstraction

Findings:

- `json.rs` combined raw lexical preflight, duplicate-aware Serde decoding, bounded output,
  normalized schema structure and the checked public import/export boundary.
- Reference discovery belonged beside reference validation/resolution, rather than inside
  the public parse/compile stage orchestrator.

Changes:

- `src/json/codec.rs` owns bounded raw decoding and serialization.
- `src/json/config.rs` owns normalized section, fieldset, column and metadata validation.
- `src/json.rs` retains checked public import/export and value-budget orchestration.
- Moved the cohesive discovery visitor to `src/references/collect.rs`; `stages.rs` now
  delegates collection after compilation. It uses the existing SourceFile accessor rather
  than widening field visibility across the module boundary.
- New modules remain private. No public type, signature, field, namespace or JSON tag moved
  or changed; there is no new generic abstraction or dependency.

Verification: the same locked all-targets crate command exited 0: **22 tests passed**,
0 failed, three child helpers. Log: `target/lane-a-round-2.log`.

## Round 3: consistency, Rustdoc and comments

Findings and changes:

- Corrected stale documentation describing composition as a future milestone and syntax
  parsing as environment evaluation. Clarified parser-owned Ast versus detached public
  nodes, source-free diagnostics, caller-owned SourceFile indexing and historical JSON spans.
- Clarified syntax-only optional/list flags and the data-only enum/plugin/prop aliases.
- Added compiling examples for SourceFile and Ast accessors.
- Documented parser/serializer state fields, argument-reference state transitions, depth
  release after successful closure, normalized-site traversal and copy-budget invariants.
- Updated `PROVENANCE.md` for the private repairs and unchanged API/grammar provenance.

Verification: locked all-targets crate tests and crate doctests both exited 0:
**22 tests and 22 doctests passed**, 0 failed. Logs:
`target/lane-a-round-3-{tests,doctests}.log`.

## Round 4: targeted follow-up and safety review

A material finding remained: legacy `compile_identifier` allocated an entire serialized
reference buffer solely to count its bytes before checking the remaining copy budget.

- Reused the existing bounded counting serializer, avoiding the temporary output allocation.
  Successful measurement proves the subtraction cannot underflow; cloning still happens
  only after charging the same escaped JSON byte count.
- Preserved the existing public Limit code, message and target span for exhausted budgets.
- Added a watchdog regression whose raw Idea tabs expand to more JSON bytes than the source
  budget; it proves the existing failure before the repair and after it. The child also
  covers zero-depth token boundaries and numeric-error byte/scalar coordinates after emoji.
- Final Clippy review collapsed a private nested metadata condition into a Rust 1.96
  compatible let-chain; tests reran after that adjustment.
- Reviewed recursion before descent, checked public-config clone/Serde boundaries, reference
  cycles/materialization, UTF-8 offsets and per-run ownership. **Nothing material remains**
  within this frozen-behavior scope. Existing explicit declaration/token dispatch matches
  remain together because their closed grammar cases are clearer than additional wrappers.

Verification: locked all-targets crate tests exited 0: **23 passed**, 0 failed, four child
helpers. Every ignored helper is explicitly invoked by an active watchdog parent; no critical
resource test is silently skipped. The new parent kills/reaps after a ten-second deadline.
Logs: `target/lane-a-round-4.log` and the final logs below.

## Final verification

All commands exited **0**, after the final code adjustment:

```sh
cargo fmt --all -- --check
cargo test -p stackpress-idea-parser --all-targets --locked
cargo test -p stackpress-idea-parser --doc --locked
cargo clippy -p stackpress-idea-parser --all-targets --locked -- -D warnings
cargo +1.96.0 check -p stackpress-idea-parser --locked
RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-idea-parser --no-deps --locked
git diff --check
```

Results: **23 tests and 22 doctests passed**, no failures; fmt, Clippy and Rustdoc clean.
The Rust 1.96.0 check passed. All ten full pinned/current compiled fixture contracts remain
passing, including the three expected standalone unknown-Text outcomes and typed JSON cases.

Final logs: `target/lane-a-rounds-final-{fmt,tests,doctests,clippy,msrv,rustdoc}.log`;
metadata: `target/lane-a-rounds-checks.json`.

A mechanical comparison against starting HEAD found **65 matching public declarations,
method signatures, struct/enum definitions and re-exports**. Its record is
`target/lane-a-rounds-public-surface.json`. No Serde representation annotation changed;
fixed-shape tests and the existing corpus/order/roundtrip tests cover the observable JSON.
Every Rust source/test function has Rustdoc; no Rust source line exceeds 100 columns.
These scans support, rather than replace, the code review and behavior tests.

## Clean coverage and limits

After the final checks, with no concurrent build in this isolated worktree:

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-idea-parser --all-targets --locked --no-default-ignore-filename-regex --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/tests/|/target/' --json --output-path target/lane-a-rounds-coverage.json
```

Both commands passed. Parser production source, including the new private submodules:
**2,258 / 2,391 executable lines = 94.44%**. The per-crate 70% development floor passes.
Summary: `target/lane-a-rounds-coverage-summary.json`. No production file/branch is excluded;
exclusions cover toolchain, dependencies, build output and integration-test harnesses.
Watchdog child execution is included; ordinary doctests ran separately.

Toolchain: rustc 1.96.0 (`ac68faa20`), Cargo 1.96.0 (`30a34c682`), cargo-llvm-cov 0.8.6,
LLVM 22.1.2, native `aarch64-apple-darwin`, parser default features. Other targets and a fresh
workspace-wide integration run were not part of this parser-only round gate.

Remaining uncovered lines chiefly include defensive malformed-private-AST compiler cases
and some JSON/reference validation failures. Coverage does not establish exhaustive paths.
Existing public raw Clone/Serde bypasses and absent-text historical span authentication remain
as documented M1 limitations; these rounds introduce no breaking validation or field changes.
No unresolved product/dependency decision remains. Coordinator review/integration is next;
this receipt does not claim acceptance, merge, publication or cross-platform verification.
