# Stackpress Rust contributor instructions

Read [.agents/context/index.md](.agents/context/index.md) first, then the smallest linked topic for the task. Follow [.agents/AGENTS.md](.agents/AGENTS.md) when maintaining the KB.

## Project contract

- This repository is the single Stackpress Rust monorepo. Preserve the event/config/plugin architecture while using idiomatic Rust; a 1:1 TypeScript conversion is not required.
- Leptos supplies the reactive UI; Tauri hosts installed desktop/mobile applications. Share UI, contracts, applicable validation, and operation source across targets; isolate platform and execution-host differences in adapters.
- The foundation is `stackpress-lib`. Expose convenient imports under `stackpress::`, using Rust identifiers such as `stackpress::inquire_mysql`.
- Plugins are selected before compilation; disabling one requires rebuild/redeploy. A registration function is the essential plugin contract; layout and additional abstractions remain optional.
- Preserve general `request`, selected operation, `response`, and `error` lifecycle concepts. Distinguish top-level handling from nested event dispatch.

## Implementation and review

Use [.agents/workflows/rust-programming.md](.agents/workflows/rust-programming.md) for programming tasks. Implement naturally, prove behavior with meaningful tests, then perform at least three distinct test-verified refactoring rounds covering complexity, responsibility/abstraction, and consistency/Rustdoc/comments. Repeat targeted rounds while material findings remain; avoid ceremonial edits when a round finds nothing.

For most crates the owner prefers [.agents/workflows/coordinated-crate-development.md](.agents/workflows/coordinated-crate-development.md). A coordinator agent directs and reviews, an implementer agent writes the code through Paseo, and the owner decides product questions. Work moves through gated stages (design checkpoint, proof, milestones, rounds, docs, scoped fixes) tracked in `.agents/specs/`.

The user's standing instruction authorizes those routine behavior-preserving rounds inside the requested task. Do not stop between rounds for the coding skill's default audit-only approval question. Request a decision only for genuinely new scope, changed public behavior, or an unresolved product choice.

Follow [.agents/context/rust-code-style.md](.agents/context/rust-code-style.md): conventional Rust formatting/names, Rustdoc, explanatory comments for every logical block, and line-level explanation for branching/stateful/non-obvious work. Use the installed chrisai-coding principles with the explicit Rust adaptation; do not apply JS-specific syntax rules to Rust.

For UI icons (choosing, creating, packaging or auditing), follow the installed [chrisai-iconography](.agents/skills/chrisai-iconography/SKILL.md) skill. The owner's icon family is Lucide.

Development coverage is at least 70%; publication preparation requires at least 90%. Follow [.agents/context/testing-and-quality.md](.agents/context/testing-and-quality.md) for scope, evidence, branch-risk review, and feature/platform limitations. Old TypeScript KB rules demanding >90% on every change do not override this policy.

## Boundaries

- Existing TypeScript KB contracts are reference material, not evidence of Rust implementation or automatic target requirements.
- Assess the existing Rust Idea parser before writing a new parser. Keep its Node/N-API distribution concerns separate from a Rust Cargo dependency.
- Preserve unrelated dirty files. Do not commit/push, publish, deploy, or enable KB MCP publishing unless requested.
- Never start a development listener directly. Use the devmetrics skill and its managed `devmetrics start` command if a preview/listener is needed. This setup does not install that skill locally; use an available session installation.
- No Cargo implementation has been scaffolded by the KB setup. Run build/test commands once an actual relevant Cargo manifest exists; never report documentation validation as code verification.
