# Stackpress Idea Parser

`stackpress-idea-parser` turns Idea text into ordered schema data. Use it to inspect syntax,
compile explicit inputs, or prepare schemas for composition. Parsing performs no filesystem,
network, or process-environment reads.

These docs cover the public API in this checkout. The workspace uses Rust 1.96 and edition 2024;
the crates currently have `publish = false`, so examples use local Cargo dependencies.

## Start here

Begin with [Getting started](getting-started.md): parse a schema, compile it, and print the first
column of a model. Read [Concepts](concepts.md) next to understand the stages and the distinction
between a reference and a string that looks like one.

The parser validates language syntax and normalized data shapes. Type names and well-formed
metadata remain open data; parsing does not validate records, activate plugins, or generate code.

## Common tasks

Choose the guide matching the result you need. Each guide includes an executable example and
links to the relevant API details.

| I want to… | Read |
| --- | --- |
| Describe models, reusable data, and metadata | [Writing a schema](guides/writing-a-schema.md) |
| Compile the same syntax with explicit values | [Supplying environment inputs](guides/environment-inputs.md) |
| Resolve references without changing literal strings | [Resolving references](guides/resolving-references.md) |
| Exchange checked normalized data | [Importing and exporting JSON](guides/json.md) |
| Report useful failures and lower budgets | [Diagnostics and limits](guides/diagnostics-and-limits.md) |

## Returning to the API

Use the [API index](api/README.md) for signatures, fields, defaults, and failure behavior. The
workspace facade re-exports this crate as `stackpress::idea_parser` with feature `idea-parser`.
Direct consumers use `stackpress_idea_parser`.

For imports spanning several files, use the [Idea Transformer](../../idea-transformer/docs/README.md).
The parser records `use` directives but never loads them. The [Idea wrapper](../../idea/docs/README.md)
coordinates generation; the canonical application is [generated-profile](../../../../examples/generated-profile/README.md).

Examples are plain fenced Markdown, checked separately from doctests. Source authority is the
[crate source](../src/lib.rs), [tests and fixtures](../tests/), and [provenance record](../PROVENANCE.md).
The extracted core is Apache-2.0; its provenance describes the repairs and the distribution boundary.
