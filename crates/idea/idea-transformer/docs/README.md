# Idea transformer

`stackpress-idea-transformer` composes Idea or normalized JSON sources, runs selected Rust
generator functions, and manages a dedicated generated directory. Use it in a build tool when
you need inspectable output with explicit ownership and freshness checks.

These docs describe the current checkout on Rust 1.96, edition 2024. The package has
`publish = false`; examples use local Cargo dependencies. Composition and planning are
synchronous. Native output uses the installed `rustfmt` executable and cooperative filesystem
ownership; no listener or executor starts.

## Start here

Follow [Getting started](getting-started.md) to compose two in-memory sources, run one function
transform, and check a temporary output root. Read [Concepts](concepts.md) after that first
success for import precedence, selection, claims, and recovery boundaries.

The [parser docs](../../idea-parser/docs/README.md) own the language, AST, config, typed
references, and finalization. The [Idea wrapper](../../idea/docs/README.md) combines composition,
finalization, planning, and output into one call.

## Common tasks

Choose a guide by the result you need. Each guide keeps its working example separate from
the detailed API contracts.

| I want to… | Read |
| --- | --- |
| Load imports and resolve their surviving references | [Compose sources](guides/composing-sources.md) |
| Register a generator with contracts and declared input | [Register transforms](guides/registering-transforms.md) |
| Build module indexes without sharing file mutation | [Generate modules](guides/generating-modules.md) |
| Check freshness, install output, and remove stale files | [Check and install](guides/checking-and-installing.md) |
| Inspect and recover an interrupted installation | [Recover output](guides/recovering-output.md) |

## Returning to the API

Use the [API index](api/README.md) for signatures, defaults, fields, returns, and errors. Examples
are plain fenced Markdown, checked in a disposable consumer; Cargo does not test these pages
automatically. [Source exports](../src/lib.rs), [composition tests](../tests/), and the
[registry](../src/registry/tests.rs) and [writer tests](../src/output/tests.rs) establish the
documented behavior.

For a complete app-owned tool, follow the canonical
[generated-profile example](../../../../examples/generated-profile/README.md). Its Profile
mapping is example-specific; the transformer supplies no general template engine or framework
schema semantics.
