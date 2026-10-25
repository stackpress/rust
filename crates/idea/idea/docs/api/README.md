# Idea wrapper API

The crate exposes `generate`, `GenerateRequest`, `GenerationMode`, and direct crate re-exports
`parser` and `transformer`. Source: [lib.rs](../../src/lib.rs). Lower-level types keep their
[parser](../../../idea-parser/docs/api/README.md) and
[transformer](../../../idea-transformer/docs/api/README.md) owners.

```rust
use stackpress_idea::{GenerateRequest, GenerationMode, generate, parser, transformer};
use transformer::{GeneratedFile, OutputWorkspace, ProjectLoader, Registry};
fn aggregate(_: &parser::SchemaConfig) -> transformer::Result<Vec<GeneratedFile>> {
    Ok(vec![GeneratedFile::new("index.txt", "Profiles\n")])
}
let parent = std::env::temp_dir().canonicalize()?;
let temp = tempfile::tempdir_in(parent)?;
std::fs::write(
    temp.path().join("schema.idea"),
    "model Profile { id String }",
)?;
let loader = ProjectLoader::new(temp.path())?;
let workspace = OutputWorkspace::new(temp.path().join("generated"))?;
let registry = Registry::new();
```

Each following Rust snippet is independent, using this setup inside a function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Helpers remain ordinary functions.
The consumer needs `stackpress-idea` and `tempfile`. Contents:
[1. Request](#1-generaterequest) · [2. Mode](#2-generationmode) · [3. generate](#3-generate)

## 1. GenerateRequest

`GenerateRequest<'a>` owns per-run options except the borrowed entry string. The selected
Registry and OutputWorkspace remain separate borrowed arguments to generate.

### 1.1. GenerateRequest::new

`GenerateRequest::new(entry: &'a str, aggregates: fn(&parser::SchemaConfig) -> transformer::Result<Vec<GeneratedFile>>, mode: GenerationMode) -> Self`

Construct a request with finite defaults and explicit output intent.

```rust
let request = GenerateRequest::new("schema.idea", aggregate, GenerationMode::Check);
assert_eq!(request.entry, "schema.idea");
assert_eq!(request.mode, GenerationMode::Check);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `entry` | `&'a str` | Loader specifier; resolved independently of the output root. |
| `aggregates` | `fn(&SchemaConfig) -> transformer::Result<Vec<GeneratedFile>>` | App-owned pure root renderer using finalized config. |
| `mode` | `GenerationMode` | Single comparison or installation decision. |

**Returns** A request with default CompileInputs/CompositionLimits and empty TransformSelection,
without IO. The aggregate is a function pointer, not a capturing closure. It runs after
finalization and before registry planning, so its own callback can run even when later registry
preflight will fail. Treat it as pure rendering.

### 1.2. Public fields

All fields can be set directly before generate consumes the request. Options belong to one
run rather than a shared mutable registry cache.

| Field | Type | Default / role |
| --- | --- | --- |
| `entry` | `&'a str` | Borrowed constructor argument. |
| `inputs` | `parser::CompileInputs` | Default explicit environment, missing-env policy, parser/value limits. |
| `limits` | `transformer::CompositionLimits` | Default acquisition, import, expansion, JSON bounds. |
| `selection` | `transformer::TransformSelection` | No runtime contracts or auxiliary snapshots. |
| `aggregates` | `fn(&SchemaConfig) -> transformer::Result<Vec<GeneratedFile>>` | Constructor's app root renderer. |
| `mode` | `GenerationMode` | Constructor's Check/Install choice. |

## 2. GenerationMode

GenerationMode is Debug, Clone, Copy, PartialEq, and Eq with two variants:

| Variant | Behavior |
| --- | --- |
| `Check` | Format/compare under a transient writer lock, without changing generated files or manifest. |
| `Install` | Stage/install through the cooperative ownership/recovery contract. |

There is no implicit default mode. Check and Install share composition/finalization/planning;
Check cannot silently repair stale or manually edited output.

## 3. generate

`generate(request: GenerateRequest<'_>, loader: impl SourceLoader, registry: &Registry, workspace: &OutputWorkspace) -> transformer::Result<CheckReport>`

Compose imports, finalize actual reference sites, render app aggregates, plan selected function
transforms, then compare or install the complete output.

```rust
let request = GenerateRequest::new("schema.idea", aggregate, GenerationMode::Install);
let before = generate(request, &loader, &registry, &workspace)?;
assert!(!before.is_current());
let request = GenerateRequest::new("schema.idea", aggregate, GenerationMode::Check);
assert!(generate(request, &loader, &registry, &workspace)?.is_current());
```

The first report describes pre-install differences. Only the subsequent Check asserts current
disk state; `Ok` alone means the operation completed without error.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `request` | `GenerateRequest<'_>` | Consumed per-run inputs, limits, contracts, aggregate renderer, and mode. |
| `loader` | `impl SourceLoader` | Owned or borrowed adapter; root is a resolution policy, not a sandbox. |
| `registry` | `&Registry` | Compiled function selection in explicit order. |
| `workspace` | `&OutputWorkspace` | Dedicated owned root and recovery/locking policy. |

**Returns** The transformer's CheckReport in both modes. In Install mode it is the pre-install
comparison. For a classified GenerationReport, use lower-level planning plus
[install_report](../../../idea-transformer/docs/api/output-workspace.md#4-install_report);
this wrapper does not expose a second report mode.

Source, composition, finalization, aggregate, preflight/callback, ownership, formatting, lock,
IO, and recovery errors propagate with causes intact. Pre-install failures leave managed output
unchanged. Callback panics propagate. Trusted loaders/renderers/functions own direct side effects
outside the managed writer contract.

The final SchemaConfig view contains plugin/enum/type/model sections, with imports/props consumed
and actual references resolved. Literal `${Name}` strings remain data. Environment expansion
uses request.inputs during composition; finalization retains the same value limits without
re-reading environment. The call creates no executor/listener or process-wide cache.

Related: [getting started](../getting-started.md), [app-owned tool](../guides/app-owned-codegen.md),
[transformer concepts](../../../idea-transformer/docs/concepts.md).
