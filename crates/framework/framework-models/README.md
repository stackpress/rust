# Default generated framework schemas

`stackpress-framework-models` contains the native source schemas generated from the shipped
`stackpress/stackpress.idea` → `stackpress-api/schema.idea` →
`stackpress-session/schema.idea` graph. It currently exports `ProfileSchema`, `AuthSchema`,
`ApplicationSchema` and `SessionSchema`. Store/action/event generation and plugin binding
belong to the later G50 stages; importing this package registers nothing.

```rust
# #[cfg(not(target_arch = "wasm32"))] {
use stackpress_framework_models::ProfileSchema;
use stackpress_framework_schema::source::SourceSchema;

let schema = ProfileSchema::new("app-owned-seed".into(), 0);
assert_eq!(ProfileSchema::describe().name, "Profile");
let defaults = schema.defaults_at(42)?;
assert_eq!(defaults["type"], "person");
assert_eq!(defaults["created"], stackpress_framework_schema::source::date(42));
# }
# Ok::<(), stackpress_framework_schema::SchemaError>(())
```

The seed and local Date offset are explicit native constructor arguments. Construction
runs no defaults or application IO. `defaults_at` supplies a clock; `with_providers` supplies
an app-owned `ValueProviders`. Missing members, explicit null, native Dates and insertion
order remain distinct. The complete private metadata includes all source fields, attributes,
relations, selectors and presentation bindings. It is not a browser publication or a route
policy. Source schemas are absent on wasm32; browser UI integration is a later stage.

[Native tests](tests/native.rs) check the original Node column and crypto vectors, every
core accessor, defaults, partial assertions, filter/populate and codec phases. [Generator
controls](tests/generation.rs) prove check-only drift rejection and app extension. Defaults
and schemas depend on the lower schema source runtime, without session/API/server/Idea
runtime dependencies. `serde_json` is a test dependency; emitted code uses the stable
`stackpress_framework_schema::source::__private::serde_json` support path.

## Regenerate and check

From the workspace root:

```sh
cargo run -p stackpress-framework-models --features generate --bin framework-models-generate --locked --offline
cargo run -p stackpress-framework-models --features generate --bin framework-models-generate --locked --offline -- --check
```

The finite tool uses the public `Schema::normalize_source` and
`codegen::emit_source_contracts` APIs with the existing `PackageLoader`, `Registry` and
`OutputWorkspace`. Checked-in files beneath `src/generated/` and their managed manifest
are generated output: never edit them by hand. `--check` reports stale output and writes
nothing. `--output <path>` selects another managed root; `--input <file>` replaces the
single app entry source while retaining the package catalog. Relative auxiliary app files
need the application's own loader rather than this default-package tool.

An application extends a model by importing its package Idea file and redeclaring it with
additional fields under the existing G19 composition rules. The public emitter accepts the
composed graph; it does not repair overridden core declarations. The independent M1 probe
compiles a Profile `nickname String?` default with only the schema runtime dependency.
Session-only generation explicitly declares Application/Session external, retaining their
links without generating hidden API models or tables.

The M1 source codec inventory covers the shipped scalar/list types: String, Boolean, Hash,
Datetime and String[]. The [source runtime guide](../framework-schema/docs/source-models.md)
lists the refused constructs outside that inventory and the distinction between normalization
and emission. Typed emitters and their existing generated applications are
unchanged. Source `@hashed` uses SHAKE256 and `@encrypted` uses the existing deterministic
AES-CBC format; these are the original storage codecs, separate from password hashing in
authentication flows. Concrete renderers, SQL execution and model-family plugin wiring are
not supplied by this M1 package.
