# Refactoring round checklists

Owner: [testing and quality](../context/testing-and-quality.md); execution route: [Rust programming](../workflows/rust-programming.md). Load for the deliberate post-test rounds. The requested workflow already authorizes scoped behavior-preserving refactoring.

## Round one: logic, decisions, and tests

- What are the meaningful control-flow paths? Which inputs change results, side effects, status, authorization, or retry behavior?
- Are priority order, stable ties, matching, cancellation, error precedence, and state mutation visible?
- Do deeply nested conditions hide the active assumptions? Would guards or a named phase clarify them without changing order?
- Do boolean expressions combine unrelated rules? Name a domain predicate when it clarifies rather than merely moves complexity.
- Does a large match represent a clear exhaustive state machine? Keep it if extraction hides the rules.
- Do tests prove true/false, success/failure, empty/missing/unknown, boundary values, and meaningful combinations?
- Would changing `>` to `>=`, removing a negation, reordering guards, or swallowing an error fail a relevant test?
- Do concurrent/nested calls share mutable invocation state accidentally?
- Are fallible steps, partially completed mutations, and transaction scope explicit?

Record the concrete risk and smallest useful change. Complexity scores are private signals unless needed by the task; there is no invented mandatory cyclomatic number.

## Round two: responsibility and abstraction

- Where would a human look for the centered feature, entity, capability, or model?
- Does the entrypoint combine unrelated transport, data access, policy, presentation, generation, and state machinery?
- Is repeated policy duplicated across adapters in a way that can diverge?
- Does one small change require knowledge of several unrelated internals?
- Are tests reaching through implementation because a stable boundary is missing?
- Is a helper dumping ground growing instead of a feature-owned module?
- Would a trait allow a real adapter/substitution, or add ceremony to a single concrete flow?
- Can unnecessary wrappers, lifetime/generic machinery, or dynamic dispatch be removed?
- Does a split improve independent change/findability/testing, or merely scatter a cohesive implementation?
- Are public types, generated outputs, runtime consumers, and build-only dependencies in their correct ownership lanes?
- Can target-specific operations be isolated without duplicating shared application logic?

Keep modules cohesive and explain the ownership decision. Do not impose the TypeScript skill's class-per-file convention on Rust structs/impls/helpers.

## Round three: consistency, documentation, Rust idioms

- Do analogous handlers/config/errors use consistent naming and observable sequencing?
- Do public imports and facade features clearly express optional components?
- Do Rust types represent valid states and preserve useful errors/causes?
- Are borrowing/cloning and Send/Sync choices justified by actual execution needs?
- Are code blocks small enough to read methodically without unnecessary bouncing between files?
- Does every meaningful function/method have Rustdoc, and does every logical block have useful explanatory commentary?
- Are branch/state/async/serialization assumptions explained near their code?
- Do docs state inputs/output, mutation, errors/panics, ordering, cancellation, feature/target boundaries, and examples where applicable?
- Have stale comments, dead code, bogus guarantees, and duplicated declaration comments been removed?
- Do doctests, rustfmt, Clippy, and Rustdoc succeed in the relevant feature/target scope?

## Follow-up rounds and stopping

After an edit, rerun the narrow relevant tests. When a later round reveals a new logic/boundary issue, return to that concern and verify it. Continue until material in-scope findings are addressed and no new meaningful issues emerge.

A round may be no-change with a clear reason. Do not manufacture refactors or polish indefinitely. A known unresolved behavior decision can be recorded instead of silently changed, with independent work completed where possible.

## Receipt template

```text
Task / crate / feature / target:
Intended behavior:
Baseline tests and result:

Round 1 — logic/complexity:
  Findings and changes (or justified no-change result):
  Tests rerun / result:
Round 2 — responsibility/abstraction:
  Findings and changes (or justified no-change result):
  Tests rerun / result:
Round 3 — consistency/Rustdoc/comments:
  Findings and changes (or justified no-change result):
  Tests / doctests / lint / docs result:
Follow-up rounds:

Coverage tool / command / crate / features / target:
Coverage result and applicable floor (70 development / 90 publication):
Excluded or unmeasured code and rationale:
Remaining branch/adapter/platform risks:
Unresolved user decisions:
```

The receipt is implementation evidence, not delegated acceptance or permission to publish. Do not report a green coverage gate if measurement failed or covered a different target.
