# Testing and quality policy

Authority: project-owner request on 2026-10-23. Normal development accepts 70% coverage; preparation to publish requires 90%. This supersedes legacy TypeScript contributor references requiring >90% on every new change.

## Meaning of the thresholds

Use executable line coverage for the initial operational definition, measured on first-party Rust implementation in the declared crate/feature scope. Record tool, compiler/target, selected features, numerator/denominator or report, and exclusions. The owner specified percentages; this line-coverage convention makes them usable without inventing a branch-score requirement.

- Development closeout: at least 70% in each changed crate's measured scope, plus the relevant workspace/integration result. Do not hide an under-tested changed crate behind unrelated well-covered code.
- Publication preparation: at least 90% for each crate being published and the applicable release integration scope. Publication itself requires explicit authorization.
- Preserve stronger existing regression coverage where practical; a lower allowed floor is not permission to delete valuable tests.
- An empty or unmeasured implementation is not 100% covered. Source/doc/KB presence is not coverage evidence.
- Dependencies, target build output, and tooling fixtures are not first-party production coverage. Generated code that is shipped/executed still needs a stated verification strategy; do not exclude it merely to raise the percentage.
- Do not add broad ignore patterns, unreachable branches, trivial assertions, or tautological mocks to game the gate. Document justified exclusions, especially target-specific code.

Line coverage does not prove branch, permission, lifecycle, concurrency, or error behavior. Even at 70%, tests must protect the meaningful critical paths affected by the change. At 90%, the remaining gaps must be visible and assessed.

## Meaningful tests

- Unit tests prove clear contracts, boundaries, invalid input, false/empty/unknown states, mutation, and error propagation.
- Event tests prove matching, priority/ties, abort, nested calls, independent concurrent invocation metadata, and lifecycle behavior.
- Plugin tests prove selected/disabled builds, simple function registration, composition, customization, and declared dependency behavior.
- Generator tests prove parsing/normalization, clean and repeat generation, removed/renamed output policy, generated compilation/import, and runtime registration.
- Database tests prove actual adapter/dialect behavior and transaction affinity; SQL string snapshots alone do not prove execution.
- Access tests cover allowed and denied callers across each exposed surface; UI hiding is not authorization.
- Leptos/Tauri tests distinguish website SSR/hydration, packaged rendering, operation adapters, target-specific integrations, and public snapshots.
- Tests assert observable behavior rather than reproduce implementation decisions. Use representative decision tables where combinations matter.

## Required implementation/refactoring sequence

Write the intended behavior naturally and obtain passing meaningful tests. Then run at least three distinct rounds:

1. Logic/complexity and branch-risk review; refactor while preserving behavior and rerun relevant tests.
2. Responsibility boundaries, abstraction, ownership, coupling, and duplication review; refactor and rerun relevant tests.
3. Consistency, idiomatic Rust, public API, Rustdoc, and detailed inline-comment review; refactor/document and rerun relevant tests/doctests.

Repeat targeted rounds while material findings remain. A round with no justified edits is valid if it records its review scope and conclusion. Do not perform pointless rewrites or an endless loop. After the last behavioral edit, run the meaningful affected checks again; broaden only for actual cross-crate/target concerns.

Use [the programming workflow](../workflows/rust-programming.md) for exact execution and [round checklists](../references/00037-refactoring-round-checklists.md) when reviewing. The standing user request authorizes routine behavior-preserving rounds inside the task, including Rust adaptation of installed chrisai-coding guidance.

## Commands once Cargo code exists

Typical relevant checks, adapted to the actual crate and valid feature set:

```sh
cargo fmt --all -- --check
cargo test -p actual-crate
cargo test -p actual-crate --doc
cargo clippy -p actual-crate --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc -p actual-crate --no-deps
cargo llvm-cov -p actual-crate --fail-under-lines 70
```

For publication preparation replace the coverage floor with 90 and verify each published crate plus release integration. Use workspace-wide commands when the selected members/features can actually compile together. Do not blindly use all-features when renderers, drivers, or target-specific options are incompatible.

`cargo-llvm-cov` is a proposed coverage tool, not installed by this setup. Its line-threshold flag is documented. Branch coverage and doctest inclusion in coverage currently have additional/unstable requirements; ordinary doctests should still run separately. Do not silently require nightly just to meet a stable line-coverage gate. [Tool documentation](https://github.com/taiki-e/cargo-llvm-cov).

No Cargo manifest or framework code exists at this setup stage. These commands are workflow templates, not commands claimed to have passed. Install extra toolchains/tools only as authorized by the implementation task and environment.

## Verification receipt

Record task/scope, implementation tests, each round's findings and justified edits/no-change result, tests rerun after edits, formatting/lint/docs, coverage scope/result, outstanding risk/target limitations, and any deferred behavior decision.

Passing a local proof is not full platform support, package acceptance, or release readiness. Spec acceptance is distinct from implementation, and publishing/commit/push/deployment remain separately authorized operations.
