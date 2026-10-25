# Import and export JSON

Tools may exchange schemas without exchanging Idea source. Choose normalized schema JSON for
literal data, or the compiled envelope when reference intent must survive the exchange.

## Round-trip normalized config

Use the checked methods instead of deserializing arbitrary JSON directly into recursive values.
This example exports a parsed model and imports it under a logical diagnostic name:

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, SchemaConfig, parse};

fn main() -> Result<(), IdeaError> {
    let inputs = CompileInputs::default();
    let original = parse("profile.idea", "model Profile { id String }", &inputs)?;
    let json = original.to_json(inputs.limits)?;
    let restored = SchemaConfig::from_json_named("profile.json", &json, inputs.limits)?;
    assert_eq!(restored, original);
    Ok(())
}
```

The text contains normalized column fields such as `type`, `required`, and `multiple`; it is
not source syntax or AST JSON. Imports preserve order and validate structural fields. Unknown
structural fields and duplicate keys are errors; metadata object payloads stay open data.

Plain config treats all strings as literals, including `${Name}`. `env("NAME")` is not evaluated
by JSON import, and a JSON `use` array is retained without loading its paths.

## Keep reference intent

Persist the complete `CompiledSchema` when a later step will finalize references. Its envelope
contains both `config` and `references`, and checked import validates their agreement:

```rust
use stackpress_idea_parser::{
    CompileInputs, CompiledSchema, IdeaError, SourceFile, compile_ast_with_references,
    finalize_with_references, parse_ast,
};

fn main() -> Result<(), IdeaError> {
    let inputs = CompileInputs::default();
    let source = SourceFile::new(
        "profile.idea",
        "prop Labels { title \"Profiles\" } model Profile @label(Labels) { id String }",
    );
    let ast = parse_ast(&source, inputs.limits)?;
    let compiled = compile_ast_with_references(&ast, &inputs)?;
    let json = compiled.to_json(inputs.limits)?;
    let restored = CompiledSchema::from_json(&json, inputs.limits)?;
    assert_eq!(restored, compiled);
    let final_schema = finalize_with_references(&restored.config, &restored.references, &inputs)?;
    assert!(final_schema.model.contains_key("Profile"));
    Ok(())
}
```

Only explicitly recorded sites are resolved. The historical source is not embedded in the
envelope, so valid stored coordinates do not establish that the original text had that content.

## Account for the JSON boundary

JSON uses its standard escapes and exponent notation. Idea source keeps raw strings and
decimal-only numbers. Checked JSON import rejects out-of-range integers and nonfinite numbers,
and both import and export enforce byte, depth, node, and collection limits.

Output can exceed a budget even when source text fits, because normalized field names and JSON
escaping add bytes. JSON container depth also includes schema/envelope structure. Validate
caller-built config before recursive clone or serialization; direct `Clone` and raw Serde are
outside this checked contract.

You can now choose between literal data and a reference-preserving handoff. See
[schema JSON methods](../api/schema.md#2-checked-json-operations) and
[compiled envelope methods](../api/references.md#2-compiledschema-json-operations) for exact returns.
