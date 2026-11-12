# Stackpress Rust

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

0.2.0 is a normal Cargo release; “beta” describes its maturity. Use Rust 1.96 or newer.
Read the [changelog](CHANGELOG.md) and [0.1 to 0.2 migration guide](docs/migrating-0.1-to-0.2.md).

```toml
[dependencies]
stackpress = "0.2"
```

Stackpress's Rust monorepo uses Leptos + Tauri, reusable event-driven capabilities,
configuration, and compile-time function-based plugins. The workspace contains independent libraries, optional framework capabilities,
and application examples. The 23 packages are being prepared for the coordinated 0.2.0 beta release.

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

## Crates and licences

The 23 release candidates use lockstep version `0.2.0`. Examples and proof consumers
remain unpublished. Third-party notices accompany the affected crates.

| Crates | Licence |
| --- | --- |
| `stackpress`; `stackpress-framework`, `-view`, `-language`, `-schema`, `-sql`, `-admin`, `-api`, `-csrf`, `-email`, `-session` | [Stackpress License](LICENSE) |
| `stackpress-lib`, `-server`, `-server-http`, `-cli`, `-idea-parser`, `-idea-transformer`, `-idea`, `-sql`, `-sql-sqlite`, `-sql-postgres`, `-sql-mysql`, `-leptos` | Apache-2.0; each crate ships its own `LICENSE` |
