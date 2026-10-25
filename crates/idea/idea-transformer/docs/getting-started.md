# Check your first generated file

You can test a generator before choosing an application layout or writing output. This
walkthrough imports a model from a second in-memory source, emits one text file, and reports
what an empty temporary destination would need.

## Prepare a consumer

Create a binary package outside this repository with Rust 1.96 and these dependencies. Replace
the checkout paths with your own absolute paths; keep the output directory disposable.

```toml
[dependencies]
stackpress-idea-parser = { path = "/path/to/rust/crates/idea/idea-parser" }
stackpress-idea-transformer = { path = "/path/to/rust/crates/idea/idea-transformer" }
tempfile = "3"
```

The parser supplies the schema type used by callbacks. `tempfile` gives the example a fresh
parent directory; canonicalizing it avoids temporary-directory symlink aliases on macOS.

## Compose, plan, and check

Put this program in `src/main.rs`. First notice the two `insert` calls: `schema.idea` imports
`models.idea`, which owns `Profile`. The named `emit_profiles` function then claims a file in
its registered `data` partition.

```rust
use stackpress_idea_transformer::{
    Composer, CompositionLimits, MemoryLoader, OutputPlan, OutputWorkspace, Registry, Result,
    TransformContext, TransformSpec,
};

fn emit_profiles(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    // Read the composed model and claim fresh contents in this transform's partition.
    let profile = &context.schema().model["Profile"];
    plan.write(
        "data/profile.txt",
        format!("{} columns\n", profile.columns.len()),
    )
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // These sources live in a virtual namespace; no source files are read from disk.
    let mut loader = MemoryLoader::new("/project")?;
    loader.insert("schema.idea", "use \"./models.idea\"");
    loader.insert("models.idea", "model Profile { id String name String }");
    let composed = Composer::new(loader, CompositionLimits::default()).compose("schema.idea")?;
    assert_eq!(composed.sources.len(), 2);

    // Registration order is execution order. With no root plugin blocks, it supplies defaults.
    let mut registry = Registry::new();
    registry.add(
        TransformSpec::new("profiles").owns(&["data"]),
        emit_profiles,
    )?;
    let plan = registry.plan(&composed.config, vec![])?;
    assert_eq!(plan.executed(), &["profiles"]);

    // Check briefly locks the parent, but leaves the generated root absent.
    let parent = std::env::temp_dir().canonicalize()?;
    let temp = tempfile::tempdir_in(parent)?;
    let root = temp.path().join("generated");
    let report = OutputWorkspace::new(&root)?.check(&plan)?;
    assert!(!report.is_current());
    assert!(!root.exists());
    println!("{}", report.differences.join(", "));
    Ok(())
}
```

`plan.write` creates an in-memory ownership claim. `check` compares the completed plan to disk;
it does not install the text. This schema has no references, so inspecting the non-final
composition directly is sufficient here. Use the [reference-aware flow](guides/composing-sources.md)
before rendering attributes that may contain prop references.

Run the consumer with `cargo run`. It prints:

```text
.stackpress-generated.json, data/profile.txt
```

Both paths are missing: the planned file and the writer's ownership marker. A successful
check can report stale output; `Ok(report)` alone does not mean `report.is_current()`.

## What just happened

The loader resolved the import relative to `schema.idea`. The composer consumed `use`, retained
the imported model, and recorded two source snapshots and one import edge. It did no output IO.

The registry granted `profiles` authority only under `data/`, executed it once, and returned
the plan. The workspace checked ownership and compared paths while holding a cooperative lock.
Because checking leaves output untouched, you can inspect the proposed result before installing.

## Next steps

You have composed an import, executed a compiled function, and observed a verifiable stale
report. Continue with [check and install](guides/checking-and-installing.md) to write and remove
owned output, or [generate modules](guides/generating-modules.md) to produce Rust module indexes.
The [wrapper quick start](../../idea/docs/getting-started.md) shows the entire finalized pipeline.
