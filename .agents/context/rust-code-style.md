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
