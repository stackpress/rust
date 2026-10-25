# Compose imports and resolve references

Split a large schema into files when its models and reusable props need separate owners.
Compose the complete import graph before finalization so real references see the declarations
that survived precedence. The parser owns the [reference and finalization contract](../../../idea-parser/docs/README.md).

## Resolve the effective schema

Run this in a consumer with the parser and transformer dependencies from
[Getting started](../getting-started.md). `Name` is a real reference in the imported model;
the adjacent quoted placeholder is literal data.

```rust
use stackpress_idea_parser::{CompileInputs, finalize_with_references};
use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut loader = MemoryLoader::new("/project")?;
    loader.insert("schema.idea", "use \"./models\"");
    loader.insert(
        "models.idea",
        "prop Name { label \"Ada\" } model Profile @value(Name \"${Name}\") { id String }",
    );
    let composed = Composer::new(loader, CompositionLimits::default()).compose("schema")?;
    assert_eq!(composed.imports[0].specifier, "./models");
    assert_eq!(composed.references.len(), 1);

    // Keep the actual sites alongside config; never rediscover references from JSON strings.
    let finalized = finalize_with_references(
        &composed.config,
        &composed.references,
        &CompileInputs::default(),
    )?;
    let json = serde_json::to_value(finalized)?;
    assert_eq!(
        json["model"]["Profile"]["attributes"]["value"][0]["label"],
        "Ada"
    );
    assert_eq!(
        json["model"]["Profile"]["attributes"]["value"][1],
        "${Name}"
    );
    Ok(())
}
```

Add `serde_json = "1"` to inspect the finalized value this way. The assertions prove that
only the unquoted site expanded. Source spans and forward-reference restrictions remain attached
to imported sites; importing a later declaration does not erase a source-local forward error.

## Move to native files

Replace `MemoryLoader` with `ProjectLoader::new(absolute_project_root)?`. Place the two source
texts in `schema.idea` and `models.idea`; keep `compose("schema")` unchanged. Nested ordinary
imports resolve at each importing file, while `@/shared` always uses the project root. The root
does not confine access: absolute and parent-relative imports are allowed.

Supply explicit environment data through `Composer::with_inputs` before composing. Parser
environment expansion occurs during compilation; finalization receives the applicable value
limits. For an integrated finalized generator, use the
[wrapper](../../../idea/docs/getting-started.md) rather than rebuilding its orchestration.

## Diagnose a winning declaration

Inspect `sources` for original snapshots, `imports` for acquisition edges, and `provenance`
for the origin of a winning declaration, attribute, or whole column. Keep source text behind
deliberate developer access because it can contain secrets. A typed JSON site's span may name
the original Idea file; provenance names the JSON artifact that supplied it.

You can now split inputs without losing the distinction between syntax references and data.
See [composition](../api/composition.md) for limits and [the G19 table](../concepts.md#21-g19-merge-and-replacement)
before relying on collisions across files.
