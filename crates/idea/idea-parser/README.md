# stackpress-idea-parser

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

Parse Idea schemas into checked syntax and serializable configuration. Stackpress generation tools use this independent parser without filesystem IO or ambient environment reads.

```rust
use stackpress_idea_parser::{parse, CompileInputs};
let schema = parse("schema.idea", "model Profile { id String }", &CompileInputs::default())?;
assert_eq!(schema.model["Profile"].columns[0].name, "id");
# Ok::<(), stackpress_idea_parser::IdeaError>(())
```

Inputs and parser limits belong to the caller. See [PROVENANCE.md](PROVENANCE.md) for the extracted parser source.

Licence: [Apache-2.0](LICENSE).
