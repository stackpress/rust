# Generated model fixtures

This crate owns its generated test models. `generation.rs` composes the shipped
`SCHEMA_SOURCE` exports through `PackageLoader`, normalizes the authored source graph,
and runs `emit_source_contracts` plus `emit_source_stores` through the managed writer.
Session-only graphs explicitly retain Application/Session as external declarations.
No library default model family or other crate's generated tests are imported.

Regenerate from the repository root:

```sh
cargo run -p stackpress-framework-api --features codegen --example regenerate_api_models --locked --offline
```

Pass `-- --check` for a read-only freshness check. Integration tests call the same
checker; source or generated-output drift fails. The example remains buildable when
output is missing, unlike an ignored integration test that imports that output.
Generator dependencies are dev-only and do not enter native/browser library graphs.
The fixture files are managed output: never edit them by hand.
