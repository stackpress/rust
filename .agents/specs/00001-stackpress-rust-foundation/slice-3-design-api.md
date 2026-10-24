# Slice 3 API sketch

Proposed API checkpoint for [task 00006](tasks/00006-slice-3-idea.md). Load for independently usable parser/transformer/wrapper surfaces, typed examples and errors; none of the new names below is an implemented or compile-verified API.

## Crate layout and dependency direction

Use the three [accepted Idea crates](../../context/source-structure.md), introducing only real implementations:

- `crates/idea/idea-parser` / `stackpress-idea-parser`: bounded syntax, typed AST, compiled schema and diagnostics. No dependency on lib/server/CLI, filesystem or process environment.
- `crates/idea/idea-transformer` / `stackpress-idea-transformer`: source loading/composition, registered transforms, input/output plan and host writer. Depends directly on parser, never on root `stackpress`.
- `crates/idea/idea` / `stackpress-idea`: convenience re-exports and a small `generate` orchestration entry if it removes actual caller boilerplate. No duplicate state or a second plugin system.

Facade features `idea-parser`, `idea-transformer`, `idea` are opt-in with no change to default runtime edges. Re-export as `stackpress::{idea_parser,idea_transformer,idea}`. Underlying crates never depend back on the facade. Parser is the only candidate for a later browser/editor build; that target remains unverified. Transformer and wrapper are build-host tools.

## Parser stages

Sketch signatures (parameter names/types are proposals):

```rust
parse(source: &SourceFile, limits: ParseLimits) -> Result<Ast, Diagnostic>
tokenize(source: &SourceFile, limits: ParseLimits) -> Result<Vec<Token>, Diagnostic>
compile(ast: &Ast, inputs: &CompileInputs) -> Result<SchemaConfig, Diagnostic>
finalize(schema: &SchemaConfig, inputs: &CompileInputs) -> Result<FinalSchema, Diagnostic>
```

`SourceFile::new(logical_name, text)` owns text and a source identity. `Ast` owns typed statements and spans. Schema/model/column config preserves order, open attributes and unresolved reference nodes; `FinalSchema` has resolved references and no `use`/`prop`. Finalizing a schema that still contains imports must error unless the caller explicitly opts into the legacy “discard uses” view; otherwise a successful return can conceal missing imported data. Pure syntax/compile stages never resolve files.

`CompileInputs` contains a bounded explicit environment map, missing-env policy and limits. It is immutable per run. Parser diagnostics retain their own types and source map, without importing lib Error or host correlation. JSON serialization/export is explicit; `SchemaConfig::from_json(source, limits)` validates normalized imports and order. Avoid exposing arbitrary mutable recursive internals that bypass depth/node/span limits.

Example: inspect a model without a host or generator:

```rust
use stackpress_idea_parser::{compile, parse, CompileInputs, ParseLimits, SourceFile};

let file = SourceFile::new("schema.idea", "model Profile { id String name String }");
let ast = parse(&file, ParseLimits::default())?;
let schema = compile(&ast, &CompileInputs::default())?;
let profile = schema.model("Profile").expect("the example declares Profile");
assert_eq!(profile.columns()[0].name(), "id");
```

**Returns:** an owned config with query methods and retained provenance. Errors name the diagnostic code and source range. No IO, plugins, registry locks, unsafe, or background work is hidden in this call. A source-free manually assembled AST must go through a checked builder before compilation.

## Composition and source adapter

```rust
pub trait SourceLoader {
    fn load(&self, importer: Option<&SourceId>, specifier: &str)
        -> Result<LoadedSource, LoadError>;
}

Composer::new(loader, limits) -> Composer
Composer::compose(entry) -> Result<ComposedSchema, TransformError>
```

This trait is justified by native/in-memory loaders, canonical import identity, limits and fault-injection tests. `LoadedSource` has a canonical identity, logical name and bounded Idea or validated JSON data. An actual implementation should limit reads before allocation, not simply check an already unbounded String. `ProjectLoader::new(explicit_root)` implements host resolution. No read changes the process cwd.

`ComposedSchema` returns config, source map, declaration provenance, ordered inputs and import graph. The per-run snapshot owns data; no `Clone` aliases mutable cached models. Composition is synchronous, finite and independent of event dispatch. Errors preserve source-chain/IO cause; malformed child input includes the loading chain. The detailed [merge table](slice-3-design-grammar.md) is subject to S3-D03.

## Function plugins and ordered transforms

```rust
pub type Transform = fn(&TransformContext<'_>, &mut OutputPlan) -> TransformResult<()>;

TransformRegistry::new() -> TransformRegistry
TransformRegistry::add(spec: TransformSpec, transform: Transform) -> TransformResult<()>
TransformRegistry::plan(schema, selection) -> TransformResult<OutputPlan>
```

Registration functions are the only essential plugin contract. A plugin need not implement a trait, create a directory tree, or use a macro. `TransformSpec` carries a unique ID, prerequisite IDs, runtime contract requirements and owned output partitions; registration preserves supplied order. Unknown/missing dependencies, duplicate IDs, wrong order and inconsistent schema/plugin selection fail before callbacks. No topological auto-reorder or priority second-order policy is introduced.

```rust
use stackpress_idea_transformer::{TransformRegistry, TransformResult, TransformSpec};

pub fn register(registry: &mut TransformRegistry) -> TransformResult<()> {
    registry.add(TransformSpec::new("profile-demo").owns(&["contracts", "server"]), emit_profile)
}
```

`emit_profile` is an ordinary named synchronous function. `TransformContext` borrows immutable composed schema, typed plugin config, declared input accessor and selected contract metadata. `OutputPlan::write(relative_path, contents)` claims one file; `contribute_module` declares a module/export to the coordinator-owned aggregate. It cannot overwrite another plugin's file or write to disk. Duplicate ownership/contributions error, even if bytes happen to match.

The registry is mutable during construction and immutable during planning; callbacks run sequentially. The same transformed config is not silently exposed as a shared mutable global. Later transforms observe explicit prior contributions through the plan's read-only query view if required; they do not arbitrarily edit prior files. Schema-rewrite plugins, async transforms or transform Abort are later public-contract proposals, not features inferred from server dispatch.

## Output and convenience call

```rust
OutputWorkspace::new(explicit_owned_root) -> TransformResult<OutputWorkspace>
OutputWorkspace::check(&plan) -> TransformResult<CheckReport>
OutputWorkspace::install(plan) -> TransformResult<GenerationReport>
stackpress_idea::generate(request, loader, registry, workspace)
    -> TransformResult<GenerationReport>
```

`CheckReport` lists added/changed/removed owned paths without writes. `GenerationReport` lists input names, selected IDs, generated/unchanged/removed relative paths and contract version; it excludes timestamps, machine-specific paths and env values from deterministic artifacts. Errors preserve phase/plugin/path/source metadata without embedding input secrets. The wrapper returns structured errors and starts no runtime/listener.

`install` consumes one completed plan, uses the owned-output contract in [generation design](slice-3-design-generation.md), and does not claim a cross-file transaction against uncooperative writers or power loss. Failed transforms produce no output changes. IO installation failure is a distinct state with recovery evidence, not a successful result or invisible rollback assumption.

## First author experience

Proposed example `examples/generated-profile/app/schema.idea`:

```idea
plugin "profile-demo" { }

model Profile {
  id String
  name String
}
```

From the workspace root, one command prepares inspectable source:

```sh
cargo run -p stackpress-example-profile-codegen -- emit generate
```

The app-owned codegen binary uses existing `stackpress_cli::main_with` and an independent `ServerBuilder`. It registers the `generate` event; `emit generate` therefore follows existing operator/data syntax, config/lifecycle/output/exit policy. A bare `stackpress generate` synonym would change G14's CLI grammar and is not included. Optional `--input`/`--check` are capability data after the event, with explicit safe defaults rooted at the example app.

Generated public type shape (illustrative):

```rust
/// Public profile fields declared in schema.idea.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Profile {
    /// Stable identifier used by the demo search operation.
    pub id: String,
    /// Public display name returned by the demo operation.
    pub name: String,
}
```

`generated::register(&mut ServerBuilder)` installs the generated operation plugin. It uses named startup/handler functions and the existing `Request`, `Response`, `Invocation`, `Flow`, typed config/service and error contracts. The generated host exposes `profile-search` and an in-process HTTP path for comparison to Slice 2. The generator owns the small demo's mapping, seed rows and operation semantics explicitly; Idea does not infer an operation from a model name.

When Profile is removed, `contracts/profile.rs` and `server/profile.rs` disappear, generated module declarations/registration disappear, and the remaining registry compiles as a no-op. The proof app bootstraps only the generated registry and has no handwritten hard import of Profile. `profile-search` then returns no-listener/404. An arbitrary external consumer still using a removed public type will correctly fail to compile; no compatibility shim is promised.

Parser failure example, to be verified rather than copied from speculative output:

```text
schema.idea:4:6: invalid type modifier (S3 code assigned during implementation)
```

Default CLI diagnostics include logical file/line/column and the safe message. Source excerpts and causes are developer diagnostics, with the existing verbose policy. Native IO failures preserve their cause without exposing whole source/env/config payloads. See [verification](slice-3-design-verification.md) for compilation, removed-model and disclosure proofs.
