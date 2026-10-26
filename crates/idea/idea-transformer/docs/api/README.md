# Transformer API reference

Look up the current public surface by responsibility. Start with
[Getting started](../getting-started.md) for a first run, or [Concepts](../concepts.md) for the
pipeline. The crate root re-exports these items; implementation modules are private.

## Surfaces

These pages cover public methods, fields, variants, aliases, defaults, and relevant derived
traits. They omit private registries and copy no dependency method catalog.

| Reference | Public items |
| --- | --- |
| [Source loader](source-loader.md) | SourceLoader, SourceId, SourceKind, LoadedSource |
| [Project loader](project-loader.md) | ProjectLoader |
| [Memory loader](memory-loader.md) | MemoryLoader |
| [Composition](composition.md) | Composer, CompositionLimits, ComposedSchema, ImportEdge, merge_type; Reference re-export |
| [Registry](registry.md) | Registry, TransformRegistry, Transform, ContextTransform |
| [Transform spec](transform-spec.md) | TransformSpec |
| [Transform context](transform-context.md) | TransformContext |
| [Transform selection](transform-selection.md) | TransformSelection, TransformInputs |
| [Output plan](output-plan.md) | OutputPlan, GeneratedFile, ModuleContribution |
| [Output workspace](output-workspace.md) | OutputWorkspace |
| [Reports](reports.md) | CheckReport, GenerationReport |
| [Errors and bounded reading](errors.md) | TransformError, TransformErrorKind, Result, LoadError, read_source |

## Example conventions

Each API page's first Rust block supplies its setup. Following Rust blocks are independent:
copy the setup before that block inside a function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named callback/helper functions remain
ordinary functions. Examples do not require an async executor. Temporary roots use an existing
canonical parent and never target this repository's checked-in generated source.

Signatures appear as inline code so they cannot be mistaken for additional runnable snippets.
`Self` means the enclosing type. **Returns** describes success and failures; parameter tables
include runtime arguments, not generic type parameters. Standard Debug/Clone/Default/Serde
implementations are summarized with their owners. The [parser reference](../../../idea-parser/docs/api/README.md)
owns parser types and finalization APIs; transformer `Reference` is the same parser type.
