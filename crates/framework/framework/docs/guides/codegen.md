# Use a separate build host

Keep code generation in a build application rather than a native runtime or browser graph. The aggregate's `codegen` feature exposes existing Rust generation APIs; `full` alone does not select them. The [build-host consumer](../../consumers/build-host/src/main.rs) emits Note contracts, stores and generated admin pages, then checks freshness.

## Register compiled transforms

This build-host fence registers the real schema emitter. Callbacks are statically selected; no configuration string imports executable code.

<!-- example: buildhost -->
```rust
use stackpress_framework::commands::schema::rust::{self, transformer};
use stackpress_framework_schema::{Schema, codegen};

fn contracts(
    context: &transformer::TransformContext<'_>,
    output: &mut transformer::OutputPlan,
) -> transformer::Result<()> {
    let definitions = codegen::DefinitionRegistry::standard();
    let schema = Schema::normalize(
        context.schema(),
        &definitions,
        codegen::NormalizeOptions::default(),
    )
    .map_err(codegen::transform_error)?;
    codegen::emit_contracts(&schema, output)
}

pub fn build_registry() -> transformer::Result<transformer::Registry> {
    let mut registry = transformer::Registry::new();
    rust::register(&mut registry, contracts, None, None)?;
    Ok(registry)
}
```

Add SQL and app-owned view transforms as needed; their existing prerequisites and output ownership rules remain active. The complete consumer supplies field policy, SQL/admin emitters, ProjectLoader, OutputWorkspace, GenerateRequest and Install/Check mode. No generated output is hand-patched by this guide.

## Bridge a generate event

The package ships the original `stackpress.idea` as `stackpress_framework::SCHEMA_SOURCE`.
Use `rust::package_loader(ProjectLoader::new(project_root)?)?` for direct source composition.
It registers `stackpress/stackpress.idea`, `stackpress-api/schema.idea` and
`stackpress-session/schema.idea`; there is no `stackpress/schema.idea` alias.
The generate provider uses the same catalog, so imports work from nested application files.
A mutable local model declaration adds fields under the existing G19 merge rules;
an immutable `!` local is replaced by the imported declaration.
[Catalog tests](../../tests/package_sources.rs) and the
[real provider test](../../tests/cases/codegen.rs) pin this behavior.
The source schema/store/client emitters consume this complete graph, including app-added
fields. The package catalog uses crate exports; applications need no source checkout or
relative include of a framework Idea file.

<!-- example: buildhost -->
```rust
use stackpress_framework::commands::schema::rust::{
    self,
    transformer::{Composer, CompositionLimits, MemoryLoader},
};
use stackpress_framework_schema::{Schema, codegen::NormalizeOptions};

pub fn compose_application() -> Result<usize, Box<dyn std::error::Error>> {
    let mut application = MemoryLoader::new("/application")?;
    application.insert(
        "schema.idea",
        "use \"stackpress/stackpress.idea\"\nmodel Profile { nickname String? }",
    );
    let composed = Composer::new(
        rust::package_loader(application)?,
        CompositionLimits::default(),
    )
    .compose("schema.idea")?;
    let graph = Schema::normalize_source(&composed.config, NormalizeOptions::default(), &[])?;
    assert!(
        graph
            .models()
            .iter()
            .find(|model| model.name == "Profile")
            .unwrap()
            .fields
            .iter()
            .any(|field| field.name == "nickname")
    );
    assert_eq!(graph.models().len(), 4);
    Ok(graph.models().len())
}
```

For files on disk, substitute `ProjectLoader::new(application_root)?` for MemoryLoader.
Callers assembling their own PackageLoader may register the public SCHEMA_SOURCE constants
from session, API and aggregate at the same canonical names. The default-model package's
internal regeneration tool retains repository asset includes: depending on those plugins
would cycle back into the generated models. External applications use the loader above.

To invoke generation through a native event, publish `rust::Host` (registry callback, aggregate-module callback and output mode), Environment and configuration, then bind `generate` to `rust::provider`. The event first applies source skip/input/tsconfig rules. The provider uses the real existing loader/registry/output workspace and preserves their errors. [Generation API](../api/codegen.md) separates those two layers.

The consumer also emits the complete extended source family and its native client/portable
UI partitions. That closes API-BUILD-HOST; the [API checkpoint tool](../../../../../examples/api-checkpoint/tool/src/main.rs)
has separate regeneration evidence. Runtime/browser codegen absence is checked in the
[consumer graph proof](../../consumers/graphs.py).
