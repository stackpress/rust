# Supply environment inputs

A tool may need different configuration for the same schema. Parse syntax once and supply an
explicit environment snapshot when compiling; this keeps process-global state outside the parser.

## Compile with one explicit value

Store an environment expression in source, then insert the allowed key into `CompileInputs`.
The AST can be constructed before any value exists:

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, SourceFile, compile_ast, parse_ast};

fn main() -> Result<(), IdeaError> {
    let source = SourceFile::new("settings.idea", "prop Settings { title env(\"TITLE\") }");
    let mut inputs = CompileInputs::default();
    let ast = parse_ast(&source, inputs.limits)?;
    inputs.environment.insert("TITLE".into(), "Profiles".into());
    let schema = compile_ast(&ast, &inputs)?;
    assert_eq!(schema.prop["Settings"]["title"], "Profiles");
    Ok(())
}
```

`parse_ast` retains the name `TITLE`, not its value. Compilation uses the supplied map and
produces a string; a value such as `"false"` stays a string rather than becoming a boolean.
The same checked AST can be compiled with another independent input snapshot.

## Choose missing-key behavior

By default an absent key returns `ErrorCode::MissingEnvironment` at the expression's source
range. Enable the compatibility policy only when an empty string is the intended result:

```rust
use stackpress_idea_parser::{CompileInputs, ErrorCode, IdeaError, parse};

fn main() -> Result<(), IdeaError> {
    let source = "enum Label { TITLE env(\"TITLE\") }";
    let mut inputs = CompileInputs::default();
    assert_eq!(
        parse("labels.idea", source, &inputs).unwrap_err().code,
        ErrorCode::MissingEnvironment,
    );
    inputs.missing_env_is_empty = true;
    let schema = parse("labels.idea", source, &inputs)?;
    assert_eq!(schema.r#enum["Label"]["TITLE"], "");
    Ok(())
}
```

The policy also applies in enum scalar positions, nested object/array values, and metadata
arguments. It never falls back to reading the process environment.

The map's aggregate key/value bytes and repeated materialized environment values are bounded.
Supplying an unused oversized map can therefore fail compilation too. You now control both the
values and the missing-input policy; see [inputs and limits](../api/inputs.md) for exact defaults
and [diagnostics](diagnostics-and-limits.md) for reporting failures.
