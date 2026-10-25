# Resolve references without changing literal strings

Metadata often reuses a property object while also containing ordinary text. Keep typed
reference sites alongside config so the finalizer knows exactly which values to replace.

## Retain intent through compilation

Compile with `compile_ast_with_references`, then pass both parts to the typed finalizer. The
second argument below deliberately resembles the first argument's compiled placeholder:

```rust
use stackpress_idea_parser::{
    AttributeValue, CompileInputs, IdeaError, SourceFile, compile_ast_with_references,
    finalize_with_references, parse_ast,
};

fn main() -> Result<(), IdeaError> {
    let inputs = CompileInputs::default();
    let source = SourceFile::new(
        "profile.idea",
        r#"prop Labels { title "Profiles" } model Profile @label(Labels "${Labels}") { id String }"#,
    );
    let ast = parse_ast(&source, inputs.limits)?;
    let compiled = compile_ast_with_references(&ast, &inputs)?;
    assert_eq!(compiled.references.len(), 1);

    let schema = finalize_with_references(&compiled.config, &compiled.references, &inputs)?;
    let AttributeValue::Array(values) = &schema.model["Profile"].attributes["label"] else {
        panic!("label has arguments");
    };
    assert_eq!(values[0]["title"], "Profiles");
    assert_eq!(values[1], "${Labels}");
    Ok(())
}
```

The first argument resolves to the `Labels` object. The quoted second argument stays literal,
even though both values had the same spelling in the non-final config. Do not discover
references by scanning `${...}` strings in exported config.

## Choose the finalization entry

For a standalone source, `finalize_ast` resolves prior declarations and refuses outstanding
imports. Use typed finalization after a composer has loaded the source graph and selected the
surviving declarations, columns, attributes, and reference sites.

The compatibility `finalize(file, source, inputs)` discards imports without loading them.
A successful result from that function does not mean an imported source was composed.
Use the [transformer](../../../idea-transformer/docs/README.md) for actual `use` loading and merge rules.

## Keep the source rules intact

Bare references may appear in type/model metadata arguments, including nested arrays and
objects. Targets must be prior declarations in the same source; self and forward references
fail with `UnknownReference`. A target absent from that source may come from composition.

Each `Reference` carries a named-column path and original source span. Editing composed config
requires keeping these records in sync: stale, duplicate, or mismatched sites fail validation.
Missing targets and cycles fail resolution, and reference copies spend the expansion budget.

You have retained intent separately from display text. To persist it, use the
[typed JSON envelope](json.md#keep-reference-intent); lookup details are in
[references](../api/references.md) and [finalization](../api/finalization.md).
