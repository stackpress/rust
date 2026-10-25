# Idea generation wrapper

`stackpress-idea` combines source composition, reference-aware finalization, function-transform
planning, and owned output into `generate`. Use it in an app-owned codegen tool when you want
one finite generation call while keeping the loader, registry, and output workspace explicit.

These docs describe Rust 1.96, edition 2024, in the current checkout. The crate has
`publish = false`; use a local Cargo dependency. It adds no async executor, listener, global
cache, environment read, or second plugin system. Native writing needs the installed `rustfmt`.

## Start here

Follow [Getting started](getting-started.md) to load a native schema with ProjectLoader,
install into a temporary root, then check freshness. Continue with
[building an app-owned tool](guides/app-owned-codegen.md) when integrating generation before
application compilation.

The [parser docs](../../idea-parser/docs/README.md) own language, config, diagnostics, and
finalization. The [transformer docs](../../idea-transformer/docs/README.md) own loaders,
composition precedence, transform selection, plans, output ownership, and recovery. The wrapper
re-exports them as `parser` and `transformer` without redefining their contracts.

## Returning to the API

The [API page](api/README.md) covers `generate`, `GenerateRequest`, and `GenerationMode`.
Examples are plain fenced Markdown, compile-checked outside the repository; Cargo does not
test these pages automatically. [Source](../src/lib.rs) and
[integration tests](../tests/generation.rs) establish behavior.

Use [examples/generated-profile](../../../../examples/generated-profile/README.md) as the
canonical full tool/application integration. Its independent codegen package can build before
generated application source exists. An ordinary app build checks compilation and selection
compatibility; a separate generation check establishes schema freshness.
