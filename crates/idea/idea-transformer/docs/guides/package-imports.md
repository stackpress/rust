# Import compiled package sources

Wrap your application's loader in `PackageLoader` when a Cargo dependency supplies Idea
source as a static string. This makes a package import work from any application directory
without searching a source checkout, node_modules or the network.

## Compose a package import

This finite example uses MemoryLoader for the application. Replace it with
`ProjectLoader::new(absolute_project_root)?` to load your app's real files.

```rust
use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader, PackageLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = MemoryLoader::new("/application")?;
    app.insert("nested/schema.idea", "use \"package/schema.idea\"");
    let loader = PackageLoader::new(app)
        .with_source("package/schema.idea", "model Profile { id String }")?;
    let composed = Composer::new(loader, CompositionLimits::default())
        .compose("nested/schema.idea")?;
    assert!(composed.config.model.contains_key("Profile"));
    assert_eq!(composed.sources[1].name, "package/schema.idea");
    Ok(())
}
```

The package owns the source bytes, normally through `include_str!` and a public constant.
Registration accepts static text, validates the name/extension and rejects duplicate names.
Loading checks the byte bound before cloning. Ordinary Composer limits, reference tracking,
provenance, G19 merge rules and cycle checks continue to apply.

## Choose package or application resolution

An exact registered bare name wins over a same-spelled local path. Use
`./package/schema.idea` for an explicit app-relative file or `@/package/schema.idea` for
an app-root file. Absolute and unregistered application imports keep the fallback loader's
existing behavior. There is no change to an undecorated ProjectLoader or MemoryLoader.

A relative import inside an embedded file resolves against its catalog directory, using
the existing exact/extension/index candidates. Bare registered names can reach another
package. Missing embedded-relative files fail without probing a synthetic host path;
`@/` and absolute paths still select application files. The virtual package namespace is
reserved, so a custom fallback cannot return an identity that impersonates it. Each
loader owns its catalog; registration creates no process-global state or IO.

## Use the Stackpress catalog

The owning framework packages expose `SCHEMA_SOURCE`. With the aggregate's native
`codegen` feature, `stackpress_framework::commands::schema::rust::package_loader` adds:

- `stackpress-session/schema.idea` for Profile/Auth;
- `stackpress-api/schema.idea` for Application/Session and the session import;
- `stackpress/stackpress.idea` for the aggregate import chain.

There is no `stackpress/schema.idea` alias. The aggregate generate provider uses this
catalog. Direct consumers can register the same constants without selecting its runtime
plugins. Composition retains the complete declarations; it does not imply that every
framework source attribute is already supported by a selected downstream generator.
Generation of the built-in framework models is the separately gated G50 work.

See the [source-loader contract](../api/source-loader.md),
[composition guide](composing-sources.md) and
[package resolver tests](../../tests/package_loader.rs) for limits and executable evidence.
