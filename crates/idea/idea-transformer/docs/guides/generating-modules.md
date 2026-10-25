# Generate Rust module indexes

Two functions should not patch the same `mod.rs`. Give each transform an exclusive partition,
then contribute module declarations for the registry's aggregate owner to render once all
callbacks succeed.

## Write a module and declare it

This complete planning example needs only the transformer dependency. `profile` writes a
real module file before contributing its name.

```rust
use stackpress_idea_transformer::{OutputPlan, Registry, Result, TransformContext, TransformSpec};

fn profile(_: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    plan.write("contracts/profile.rs", "pub struct Profile;\n")?;
    plan.contribute_module("contracts", "profile")
}

fn main() -> Result<()> {
    let mut registry = Registry::new();
    registry.add(TransformSpec::new("profile").owns(&["contracts"]), profile)?;
    let plan = registry.plan(&Default::default(), vec![])?;
    let files: Vec<_> = plan.files().collect();
    assert_eq!(
        files[0],
        ("contracts/mod.rs", "aggregate", "pub mod profile;\n")
    );
    assert_eq!(files[2], ("mod.rs", "aggregate", "pub mod contracts;\n"));
    assert_eq!(plan.contributions()[0].owner, "profile");
    Ok(())
}
```

The plan now has three claims: the plugin module, its partition index, and the root index.
Claims iterate in path order; index declarations retain callback/contribution order. A
contribution without its matching owned file fails planning rather than emitting a dangling module.

## Choose the root renderer

Pass root-level `GeneratedFile` values to `plan` when the app needs an index or registry of its
own. They use owner `aggregate`, cannot claim nested paths or partition names, and are inserted
before callbacks. If you use module contributions, omit an app `mod.rs`: the automatic aggregate
will claim that path and duplicate ownership is an error.

For a richer registry function, render the root yourself and use ordinary partition files
without `contribute_module`. The canonical
[generated-profile example](../../../../../examples/generated-profile/README.md) demonstrates
that app-owned choice and selection assertions. The writer separately creates the ownership
manifest; no callback should claim it.

You can now coordinate module declarations without shared file mutation. See
[OutputPlan](../api/output-plan.md) for identifier rules, file budgets, and read-only inspection.
