# rust
The rust version...

Stackpress's Rust monorepo uses Leptos + Tauri, reusable event-driven capabilities,
configuration, and compile-time function-based plugins. Implementation has not yet
been scaffolded; the current workspace establishes the knowledge and coding contract.

Start with [AGENTS.md](AGENTS.md) and the [KB context index](.agents/context/index.md).
The [next-session handoff](.agents/references/00040-session-handoff.md) records the
decisions, complete source references, current state, and open design questions.

Programming follows the [implement/test/refactor workflow](.agents/workflows/rust-programming.md),
with [Rust style and explanatory documentation](.agents/context/rust-code-style.md)
and [70% development / 90% publication coverage](.agents/context/testing-and-quality.md).

Validate this knowledge workspace with:

```sh
python3 .agents/scripts/validate-agent-workspace.py
python3 .agents/scripts/verify-kb-provenance.py
```
