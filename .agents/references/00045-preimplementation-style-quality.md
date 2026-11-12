# Historical context: preimplementation style quality

Owner: [source catalog](../context/source-catalog.md). Load only to audit the pre-sync setup text, provenance, superseded plans or resolved questions.

This is a lossless demotion of the context at branch base `34be3146ffc3279595595b665bf371205744a3a2`, preserved by Task 00059. Statements about missing implementation, open decisions, versions and support describe that older text and have no current authority. Use the [current context index](../context/index.md) for accepted truth. Only prose links are rebased; fenced examples are unchanged.

Original owner: `.agents/context/rust-code-style.md`.

# Rust code style and human readability

Authority: project-owner instructions plus Rust community conventions. This is the Rust adaptation of installed chrisai-coding principles; its JS-specific syntax rules are not Rust rules.

## Formatting and naming

- Use conventional Rust formatting with rustfmt defaults, four-space indentation, snake_case modules/functions/variables, PascalCase types/traits/variants, and SCREAMING_SNAKE_CASE constants.
- Prefer clear names over abbreviations. Name lifecycle stages, domain rules, and ownership rather than generic helpers/managers.
- Keep imports explicit and organized by responsibility. Use deliberate public re-exports, not broad glob exports that conceal the public API.
- Follow the repository's eventual edition/MSRV/toolchain choice. No Cargo manifest or compiler baseline is selected by this KB setup; edition 2024 in the existing parser branch is a source fact, not an automatic workspace decision.
- Use `// ` with a space and normal readable prose; do not transfer the coding skill's JS-specific no-space first-comment convention.

## Responsibilities and abstraction

- Group behavior around the capability, model, feature, lifecycle, or domain a human would naturally inspect.
- Keep transport conversion, domain policy/mechanism, persistence, rendering, and generation ownership distinguishable.
- Small cohesive local helpers can remain local. Extract when naming clarifies a rule, reduces duplicate policy, improves testing, or establishes a real boundary.
- Avoid both giant multi-responsibility files and ceremonial one-line wrappers scattered across files.
- Prefer concrete structs/enums and ordinary functions until a trait genuinely enables an adapter, substitution, shared contract, or test boundary.
- Keep unrelated operations out of generic utils modules. Do not add generic parameters, trait objects, macros, or layers only to appear extensible.
- Prefer exhaustive enums for meaningful state; make invalid combinations difficult to construct.
- Keep build-time, server-only, shared, browser/WASM, and platform-native dependency paths explicit.
- Consistent patterns matter: analogous handlers/configuration/errors should follow the same visible sequence unless a documented constraint explains the difference.

## Logic and ownership

- Map meaningful decision paths before simplifying. Use named predicates, guards, exhaustive matches, and focused phases when they improve comprehension.
- Preserve rule ordering, side effects, and error precedence during refactoring. Reducing line count is not the goal.
- Prefer borrowing to unnecessary cloning, but choose a justified clone over lifetime machinery that obscures simple ownership.
- Use Result/Option, typed error variants, and cause chains. Avoid unwrap/expect in fallible library/runtime paths unless an invariant is proven and documented; tests may use them with clear failure intent.
- Keep unsafe exceptional, minimal, and locally justified with a `// SAFETY:` explanation and relevant tests.
- Avoid holding a lock or entered tracing guard across await. Propagate event/task context explicitly where required.
- Explain cancellation, partial mutation, transaction scope, retries, idempotency, ordering, and sent-response behavior next to the code that establishes them.

## Rustdoc requirements

- Every crate/module has `//!` documentation describing its responsibility, callers, ownership boundaries, and important feature/target constraints.
- Every function and method, including meaningful private helpers, gets a concise `///` purpose/contract explanation. Public structs/enums/traits/aliases/constants, public fields/variants, and important private state also receive documentation appropriate to their role.
- Public API docs explain inputs/outputs, mutation and side effects, errors, panic conditions when any, ordering/cancellation, platform/feature limits, and relevant ownership expectations.
- Use standard `# Examples`, `# Errors`, `# Panics`, and `# Safety` sections when applicable. Do not add empty boilerplate sections.
- Examples must use real eventual APIs, compile as doctests when practical, and explain non-obvious lines. Use `no_run` for meaningful external-resource examples; avoid ignored examples that silently rot.
- Prefer intra-doc links to related real items. Use `cargo test --doc` and warning-clean `cargo doc --no-deps` for the relevant crate/features once code exists.
- Do not publish speculative code examples as working Rust APIs.

## Explanatory comments are required

- Minimum density is one purposeful explanation per logical block, not one comment per file/function.
- Increase to line-by-line explanation for branching, mutation/state transitions, iterator chains with policy, async sequencing, serialization, unsafe work, and hidden assumptions.
- Explain the condition and consequence separately when a branch spans multiple lines; explain what is carried into the next phase.
- Explain what a state field contains, who changes it, and its invariants. Rustdoc covers declarations; inline comments explain execution.
- Comments may narrate a nontrivial algorithm's sequence. Keep them local, accurate, and concrete rather than replacing them with a vague overall remark.
- Do not merely repeat trivial syntax, leave commented-out code, or add filler to meet density. Rich explanation and clear code are both required.
- Update comments and docs in the same edit when logic changes. A misleading comment is a defect.

Load [Rustdoc and detailed comment examples](../references/00036-rustdoc-commenting-examples.md) when implementing a handler, stateful operation, or example; it gives concrete line/block density and review criteria.

## Complexity review

Use cyclomatic path count, cognitive load, nesting, boolean complexity, decision tables, mutation risk, fan-out, and duplicate policy as review signals. Do not invent a mandatory numeric complexity threshold or mistake a low lint score for understandable code.

Clippy's cognitive_complexity lint is not a reliable measure of comprehension; the current Clippy documentation explicitly cautions against that interpretation. A meaningful exhaustive match can be clearer than abstraction that hides rules. Human navigation and testable contracts decide whether extraction helps.

The installed [coding logic signals](../skills/chrisai-coding/references/logic-review-signals.md) and [maintainability signals](../skills/chrisai-coding/references/maintainability-audit-signals.md) provide the source principles; load them for refactoring. The project workflow adapts their scope and audit-only defaults under the user's standing instruction.

## Primary standards

- [Rust Style Guide](https://doc.rust-lang.org/style-guide/): default formatting conventions.
- [Rustdoc writing guide](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html): clear public documentation and meaningful examples.
- [Clippy cognitive complexity](https://clippy.rust-lang.org/master/index.html#cognitive_complexity): limitations of the lint as a comprehension metric.

Original owner: `.agents/context/testing-and-quality.md`.

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

