# stackpress-idea

Combine the independent Idea parser and transformer in one synchronous generation API. Stackpress application tools supply their schema entrypoint, transforms and generated output owner.

```rust
use stackpress_idea::{GenerateRequest, GenerationMode};
use stackpress_idea::transformer::GeneratedFile;
let request = GenerateRequest::new("schema.idea", |_| Ok(Vec::<GeneratedFile>::new()), GenerationMode::Check);
assert_eq!(request.entry, "schema.idea");
```

Check mode compares managed output. Calling `generate` also requires the application's loader, registry and output workspace.

Licence: [Apache-2.0](LICENSE).
