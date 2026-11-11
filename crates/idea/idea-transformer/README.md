# stackpress-idea-transformer

Compose Idea sources and plan registered transforms before installing generated files in an application-owned output directory. Stackpress build tools use its loaders, registry and cooperative output ownership.

```rust
use stackpress_idea_transformer::{GeneratedFile, Registry};
let registry = Registry::new();
let _ = registry;
let _outputs: Vec<GeneratedFile> = Vec::new();
```

Applications register their selected transforms and choose the source loader and output workspace explicitly.

Licence: [Apache-2.0](LICENSE).
