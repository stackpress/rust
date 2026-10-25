# Build an app-owned codegen tool

The first generator must build before its output exists. Put generation in a separate package
that depends on `stackpress-idea` and selected generator functions; the final application
depends on generated contracts/runtime code. Keep the tool independent of that final app so
missing generated source cannot create a bootstrap cycle.

## Follow the canonical integration

Use [examples/generated-profile](../../../../../examples/generated-profile/README.md) as the
working reference, including its generation/search commands, selection switches, freshness
check, explicit path handling, and consumer proofs. Its
[codegen entry](../../../../../examples/generated-profile/codegen/src/lib.rs) hosts generation
through the existing CLI/server boundary. Its
[Profile generator](../../../../../examples/generated-profile/codegen/src/profile.rs) owns
the deliberately limited model mapping; this is not a general framework schema generator.

The tool registers named functions in explicit order, snapshots the template under a logical
input name, and supplies an app-owned root renderer. The app and tool compile the same small
[selection.rs](../../../../../examples/generated-profile/selection.rs) independently. Generated
const assertions catch mismatched selection/revision at app compilation; a separate check
catches changed schemas. These checks answer different questions.

## Define the synchronous boundary

Keep path resolution, explicit inputs, and the registry visible in a helper like this one.
Callers supply their aggregate function and already selected registry.

```rust
use stackpress_idea::{GenerateRequest, GenerationMode, generate, parser, transformer};
use std::path::Path;
use transformer::{CheckReport, GeneratedFile, OutputWorkspace, ProjectLoader, Registry};

fn generate_app(
    project: &Path,
    output: &Path,
    registry: &Registry,
    aggregate: fn(&parser::SchemaConfig) -> transformer::Result<Vec<GeneratedFile>>,
    mode: GenerationMode,
) -> transformer::Result<CheckReport> {
    let loader = ProjectLoader::new(project).map_err(|cause| {
        // Preserve the loader cause through the wrapper's TransformError result boundary.
        transformer::TransformError::from(std::io::Error::other(cause))
    })?;
    let workspace = OutputWorkspace::new(output)?;
    let request = GenerateRequest::new("schema.idea", aggregate, mode);
    generate(request, loader, registry, &workspace)
}
```

`project` must be an absolute existing directory; `output` needs an existing nonsymlink parent.
For registered functions that require templates or runtime contracts, populate `request.selection`
before calling generate. The canonical tool supplies that policy rather than relying on empty
defaults. Supply explicit `request.inputs` for allowed environment values; no wrapper global
environment read occurs.

## Integrate the tool boundary

In an async CLI host, run this blocking filesystem/formatter work on an owned blocking worker
and await its completion. Dropping the waiting future does not cancel a worker or release its
real writer lock. The canonical entry demonstrates this with the existing server event and
`stackpress_cli::main_with`; follow the [CLI docs](../../../../../tools/cli/docs/README.md) for
operator versus capability arguments and output/exit policy.

Handle `GenerationMode::Check` by examining `is_current()`, returning your chosen nonzero
status for stale output. Run that check before building the final application. Plain Cargo
builds validate Rust compilation and selected contract assertions, not schema freshness.
`build.rs`/`OUT_DIR` integration and a general template engine remain outside the current route.

## Keep output recoverable

Use a dedicated marked root and never mix handwritten files into it. Both modes reject
unknown/manual files and edited owned bytes. Review them explicitly rather than adding a force
switch. Follow the transformer's [recovery guide](../../../idea-transformer/docs/guides/recovering-output.md)
after interruptions; there is no crash-atomic replacement or power-loss guarantee.

You now have the package boundary, synchronous generation seam, and distinct build/freshness
checks. Adapt the canonical example's application-owned parts while preserving the
[request API](../api/README.md) and transformer ownership contract.
