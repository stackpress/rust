# Brief: Stackpress Rust foundation program

Spec file of [00001 Stackpress Rust foundation](index.md). Load first for the owner's goals, accepted scope, non-goals, and the sources this program inherits.

## Owner goals as written

Recorded on 2026-10-24 from the project owner's instructions in the trial/3 coordination session.

On coordination and the purpose of this trial:

> What I want is for you to coordinate with Codex (cli) Sol 6.1 xhigh. In trial/1 I used opus (you) and the results were better DX, more lib functionality implemented, but did not case for certain async issues (Deadlock, concurrent nested dispatch, poisoning, clone is an alias, etc.). In trial/2 I used Sol, and while it did a better job at safety, did not cover as much as the lib I wanted and while the DX was acceptable, it wasn't ideal.

On roles:

> I would want you more of a coordinator role than a coder btw. I need you to let sol implement with your general direction. then check its work considering your strengths and you have access to the project's KB.

On safety scope:

> these were only examples. There can be an X number of safety/best practices issues that you are not considering when you say stuff like this. Do you really need me to say all of it?

On standards:

> Additionally keep in mind of the coding standards I want in the KB as well as the rust programming workflow in .agents/workflows. Please review everything in .agents before starting.

On tracking:

> I also changed my mind I do want this in a spec so we can track this a little better (separate from the KB) until all is well.

## Scope

Build the Stackpress Rust monorepo in thin vertical slices following the [accepted source structure](../../context/source-structure.md), [project direction](../../context/project-direction.md), and [runtime and portability contracts](../../context/runtime-and-portability.md):

- Slice 0: root Cargo workspace and `stackpress` facade.
- Slice 1: `stackpress-lib` transport-neutral foundation, combining breadth and DX with safety.
- Slice 2: `stackpress-server` capability host plus the CLI adapter in `tools/`.
- Slice 3: Idea parser assessment, transformer, and generation.
- Slice 4: SQL and the first backend.
- Slice 5: shared Leptos UI and Tauri hosting.

Every slice follows the [Rust programming workflow](../../workflows/rust-programming.md), [Rust code style](../../context/rust-code-style.md), and [testing and quality policy](../../context/testing-and-quality.md): natural implementation, meaningful tests, at least three refactoring rounds plus a dedicated safety round, and at least 70% line coverage per changed crate during development.

## Non-goals

- Promoting these decisions into `.agents/context/` before the owner approves it.
- Reading or building on other `trial/*` branches.
- Publishing crates, deploying, or pushing without explicit authorization.
- Starting listeners outside the devmetrics-managed process.
- Recreating JavaScript syntax, callable objects, V8 reflection, or dynamic module loading.

## Sources

- [Context index](../../context/index.md) and its accepted topics.
- [Upstream Stackpress knowledge](../../context/upstream-stackpress.md) and the [foundation source map](../../context/source-map.md).
- [Open design questions](../../references/00039-open-design-questions.md), carried into [decisions](decisions.md).
- Upstream sibling checkouts, especially `../lib` for Slice 1, at the hashes recorded in the [upstream inventory](design-upstream-inventory.md).
