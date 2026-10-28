# Rust programming: implement, test, repeatedly refactor

Use for implementing or fixing Rust code in this monorepo. This is the project-owner's standing coding workflow, layered over the installed chrisai-coding principles and the accepted Rust style/quality policies.

## Read first

1. [Context index](../context/index.md), relevant runtime/source contracts, and the current requested behavior or applicable Frozen spec.
2. [Rust code style](../context/rust-code-style.md) and [testing/quality](../context/testing-and-quality.md).
3. Installed [coding skill](../skills/chrisai-coding/SKILL.md), [logic signals](../skills/chrisai-coding/references/logic-review-signals.md), [maintainability signals](../skills/chrisai-coding/references/maintainability-audit-signals.md), [commenting principles](../skills/chrisai-coding/references/typescript-commenting-style.md), and [declaration documentation](../skills/chrisai-coding/references/typescript-jsdoc-and-declaration-comments.md) as needed for the owned round.
4. [Detailed round checklists](../references/00037-refactoring-round-checklists.md) when making a refactoring pass.
5. [Build benchmarks](build-benchmarks.md) for the required command recorder, persistent counters, and separate build/command timings throughout implementation and review.

When a crate or slice is built by a coordinator and an implementer agent, also follow [coordinated crate development](coordinated-crate-development.md). It assigns these steps to roles and adds stage gates, review, crate documentation, and scoped fixes.

## Scope and authorization

The distributed coding skill targets existing JS/TS/React/HTML/CSS. This user explicitly requests its complexity, responsibility, abstraction, consistency, and high-comment-density principles for Rust. Apply those principles through this Rust adaptation; retain the installed skill package unchanged.

Its default review-only stop/approval rules do not require repeated permission within this workflow: the user's request already authorizes routine behavior-preserving refactoring rounds. Stay within the task. Ask only for a new product/public-contract decision, destructive action, or materially expanded scope that is not already authorized.

Do not require the user to freeze an unrelated spec before a small authorized change. If the task is driven by a Frozen spec, follow its contract and implementation workflow. If a needed design answer is unresolved, continue independent work and isolate the decision rather than invent an answer.

## 1. Understand the narrow contract

- Inspect current code, direct collaborators, tests, manifests/features, and worktree status.
- Identify the capability owner, callers, mutation/error/side-effect order, generated artifacts, and platform boundary.
- State what observable behavior must change and what must remain true. Distinguish upstream TypeScript behavior from an accepted Rust target decision.
- Use source and current tests over resemblance to a neighboring file. Preserve unrelated changes.

## 2. Implement naturally

- Reach the intended behavior with the simplest understandable implementation.
- Use normal Rust functions, types, and local composition. Do not front-load speculative architecture or a large abstraction hierarchy.
- Add useful intent comments as you code; the later documentation round completes them rather than authorizing misleading intermediate code.
- Keep external side effects, secrets, server-only dependencies, generation, and target-native work at the right boundaries.
- Refactor opportunistically if it removes an immediate obstacle, but save deliberate multi-pass review for after behavior is proven.

## 3. Establish working evidence

- Add/update meaningful unit tests for expected behavior, invalid/boundary cases, errors, state changes, and important decision combinations.
- Add contract/integration tests when a unit test cannot prove adapter, generation, persistence, SSR/hydration, or authorization behavior.
- Run the narrow relevant suite. Fix real failures before presenting the implementation as working.
- Record a baseline. A mock-only green suite is not proof of a live external boundary.
- Record all build/test/check/lint/doc/coverage and clean commands through the [build benchmark workflow](build-benchmarks.md), including later rounds and final verification. Preserve its records under `.agents/resources/build-benchmarks/`; label scope and cache state accurately and avoid wrapping both an outer proof and its nested Cargo commands.

## 4. Round one: logic and complexity

- Review meaningful control-flow paths, nesting, boolean decisions, state mutation, side-effect order, retries/fallbacks, and uncovered branches.
- Use named predicates, guards, focused functions, or decision tables where they make rules easier to see.
- Preserve behavior, especially matching/priority, false abort, nested dispatch, response state, and error precedence.
- Rerun tests after the refactor. Record findings, changes or justified no-change result, and verification.

## 5. Round two: responsibility and abstraction

- Review whether a human can locate the centered feature/model/capability and change it independently.
- Separate unrelated routing, transport, policy, persistence, rendering, state, and generation responsibilities when currently mixed.
- Keep cohesive code together. Remove unnecessary wrappers and abstractions; extract real repeated policy and stable substitution boundaries.
- Check ownership/borrowing, feature dependencies, public imports, and shared versus server/platform-only code.
- Rerun tests and affected compile/import/generation checks. Record the round's result.

## 6. Round three: consistency, Rustdoc, and explanatory comments

- Align analogous APIs, config, handlers, error types, module names, and registration patterns.
- Apply idiomatic Rust formatting, explicit public boundaries, intentional traits/enums, and readable ownership.
- Document modules, declarations, functions/methods, public contracts, examples, error/mutation/order semantics, and feature/target constraints.
- Provide an explanation per logical block and line-level commentary where branching/state/async/serialization hides intent. Comments must explain the actual sequence and assumptions, not generic filler.
- Update stale comments and examples. Run tests, doctests, rustfmt, Clippy, and Rustdoc appropriate to the changed crate/features.
- Record findings/changes and verification.

## 7. Repeat targeted rounds until stable

- Revisit earlier rounds if a later extraction or documentation pass reveals a real logic or boundary issue.
- Complete at least three distinct rounds. A no-change round is valid when it states what was examined and why the result is clear.
- Continue while material correctness/maintainability findings remain within scope. Do not create edits solely to satisfy a round count, and do not endlessly restyle already clear code.
- Add tests when new failure paths or gaps are discovered; rerun the affected tests after each meaningful edit.

## 8. Final verification and coverage

- Run the meaningful affected test/contract suite after the final behavioral change. Broaden to affected consumers/targets for shared changes.
- Check formatting, lints, Rustdoc/doctests, relevant generation/exports, and compatible selected features.
- Development coverage floor: 70% executable lines in each changed crate's declared scope. Publication-preparation floor: 90% for each published crate and applicable release scope.
- Report tool, feature/target scope, actual result, exclusions, and remaining critical gaps. Do not manipulate exclusions or add shallow tests to reach a number.
- Use the actual Cargo manifest and valid feature matrix. Do not blindly enable mutually exclusive renderers or platform dependencies.
- If a tool/target is unavailable, state the limitation; do not claim a passed coverage/target gate from documentation or another platform's result.

## 9. Closeout receipt

Report:

- Implemented behavior and semantic owner.
- Baseline verification.
- Each of the three rounds and any follow-up rounds: findings, changes/no-change outcome, tests rerun.
- Final tests, formatting, lints, docs, coverage, and feature/platform scope.
- Benchmark store path, relevant record IDs, attempt/rebuild counter changes, and separate command/build durations (or unavailable timings).
- Material remaining uncertainty and any unresolved decision.

Code verification does not imply acceptance, commit/push, publication, deployment, or all-platform support. Perform those only when separately authorized. Update accepted KB contracts when the change actually establishes new reusable truth.

When a dev listener is needed, use devmetrics. A server embedded in Tauri or a frontend tool's automatic development listener still falls under the project's managed-listener rule.
