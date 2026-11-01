# Rust generation bridge

Enable `codegen` in a native build host. `commands::schema` exists with schema-client; its `rust` module additionally exposes existing Idea/schema/SQL generation APIs. This is separate from selecting runtime stages or UI renderers.

## Source Options and event

`schema::options(req, &ClientOptions, &ServerOptions, &CliOptions, &Environment) -> Result<Options>` computes paths without IO. Options fields are idea: String, tsconfig: String, build: PathBuf, lang: String and prettier: Value. Defaults and precedence are in [configuration](configuration.md).

`schema::event` is the standard async event handler. It skips an earlier non-OK status or absent explicit client key, stages Missing tsconfig path for a missing/falsy path, otherwise resolves options, invokes the generate binding and stages 200. Pure options resolution and direct provider use do not add these wrapper checks.

## Rust build host

The module re-exports DefinitionRegistry, NormalizeOptions, GenerateRequest, GenerationMode, generate, parser and transformer from their existing crates.

| Surface | Parameters / result |
| --- | --- |
| rust::Host | registry: fn() -> transformer::Result<Registry>; aggregates: fn(&parser::SchemaConfig) -> transformer::Result<Vec<GeneratedFile>>; mode: GenerationMode. |
| rust::register | (&mut Registry, schema: ContextTransform, sql: Option<ContextTransform>, view: Option<ContextTransform>) -> transformer::Result<()>; schema first, then optional SQL/view with existing prerequisites/output partition ownership. |
| rust::provider | Standard async Handler; reads resolved Options/Host/Environment, constructs existing ProjectLoader and OutputWorkspace, invokes real generate; returns Continue or original loading/transform/format/filesystem error. |

Bind the provider under generate and publish Host explicitly. TS config/lang/Prettier remain source metadata; Rust emission uses the compiled toolchain rather than dynamic TypeScript modules. No generation runs at registration.

The [build-host guide](../guides/codegen.md) contains a compiled transform example; the [complete finite consumer](../../consumers/build-host/src/main.rs) exercises real emission and Check freshness. [Codegen tests](../../tests/cases/codegen.rs) cover provider errors and actual generation. API-BUILD-HOST remains the named limit for full API regeneration through this aggregate host.
