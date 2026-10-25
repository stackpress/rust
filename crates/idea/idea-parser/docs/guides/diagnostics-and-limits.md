# Report diagnostics and lower limits

A schema tool needs actionable locations without automatically printing its inputs. Keep the
logical source identity, report the typed failure, and render excerpts only when your application
intends to disclose them.

## Inspect a duplicate

The later duplicate is the primary location and the earlier occurrence appears in `related`.
This example checks both locations using a source kept in memory:

```rust
use stackpress_idea_parser::{CompileInputs, ErrorCode, parse};

fn main() {
    let source = "model Profile { id String id Number }";
    let error = parse("profile.idea", source, &CompileInputs::default()).unwrap_err();
    assert_eq!(error.code, ErrorCode::DuplicateName);
    assert_eq!(error.file, "profile.idea");
    assert_eq!(error.related.len(), 1);
    let start = error.start.unwrap();
    let end = error.end.unwrap();
    assert_eq!(&source[start..end], "id");
    assert!(error.related[0].start < start);
    println!("{error}");
}
```

The printed diagnostic contains identity, line, column, and message. Slicing is valid here because
the error came from parsing this exact source and the offsets are present. General renderers must
handle absent coordinates and keep the correct source text for each identity.

Columns count Unicode scalars, including one position for a tab. Byte offsets are half-open UTF-8
ranges; EOF failures use equal start/end offsets at the end of the text. These coordinates are
different from display-cell widths or JavaScript UTF-16 indices.

## Apply a smaller policy

Lower limits when a tool only accepts small inputs. Compilation rechecks the policy even if
the AST was originally parsed with more generous limits:

```rust
use stackpress_idea_parser::{
    CompileInputs, ErrorCode, IdeaError, ParseLimits, SourceFile, compile_ast, parse_ast,
};

fn main() -> Result<(), IdeaError> {
    let source = SourceFile::new("settings.idea", "prop Settings { values [[[0]]] }");
    let ast = parse_ast(&source, ParseLimits::default())?;
    let inputs = CompileInputs {
        limits: ParseLimits {
            max_bytes: 4096,
            max_depth: 3,
        },
        ..Default::default()
    };
    assert_eq!(
        compile_ast(&ast, &inputs).unwrap_err().code,
        ErrorCode::Limit
    );
    Ok(())
}
```

The declaration's object plus three arrays exceed depth 3. Raising `max_depth` beyond 64 cannot
raise the hard ceiling. Lowering `max_bytes` also lowers the derived node and collection budgets.

Resource failures can occur after parsing: explicit environment values, repeated reference
expansion, and serialized output each add bounded work. A `Limit` diagnostic therefore needs the
stage and policy as context; it does not imply that the original text alone was too large.

You can now distinguish a syntax failure from a policy failure and identify related locations.
Use [IdeaError](../api/error.md) for code meanings and [ParseLimits](../api/inputs.md) for accounting.
