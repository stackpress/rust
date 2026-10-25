# Parse and inspect your first model

An application can inspect a schema before it chooses a generator or reads any files. This
walkthrough keeps the schema in memory and prints a checked model column.

## 1. Add the local dependency

Create an edition-2024 binary crate using Rust 1.96 or later. Add this dependency to its
`Cargo.toml`, replacing the path with your monorepo checkout:

```toml
[dependencies]
stackpress-idea-parser = { path = "/path/to/stackpress/rust/crates/idea/idea-parser" }
```

The package name contains hyphens; the Rust import uses underscores. No executor or optional
parser feature is needed.

## 2. Parse, compile, and inspect

Replace `src/main.rs` with the following program. The source declares one model with a required
ID and an optional display name:

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, SourceFile, compile_ast, parse_ast};

fn main() -> Result<(), IdeaError> {
    let inputs = CompileInputs::default();
    let source = SourceFile::new(
        "profile.idea",
        "model Profile { id String displayName String? }",
    );
    let ast = parse_ast(&source, inputs.limits)?;
    let schema = compile_ast(&ast, &inputs)?;

    let profile = &schema.model["Profile"];
    let id = &profile.columns[0];
    assert!(id.required);
    assert!(!profile.columns[1].required);
    println!("{}: {} {}", profile.name, id.name, id.column_type);
    Ok(())
}
```

`SourceFile` owns a logical name and text. `parse_ast` checks syntax; `compile_ast` produces
ordered schema data. Indexing `Profile` is safe in this example because the source is fixed;
use map lookup when model names come from a caller.

## 3. Verify the result

Run the binary from your consumer crate:

```sh
cargo run
```

The output is:

```text
Profile: id String
```

The assertions also prove that the optional marker became `required = false`. The parser only
used the text and explicit inputs supplied by the program; `profile.idea` is a diagnostic name,
not a file it opened.

## What happened

You moved from named text to checked syntax and then to normalized data. Compilation removed
the `?` from the base type name and retained optionality as a boolean. It did not check whether
`String` has application-specific meaning or validate a profile record.

You can now recognize the [stages](concepts.md#2-the-stages) and the resulting model shape.
Continue with [Writing a schema](guides/writing-a-schema.md), or look up the
[staged functions](api/stages.md).
