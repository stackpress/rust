# Write a schema

A generator needs model structure and metadata without forcing those rules into the parser.
Start with a model, then add reusable data and namespaces owned by your tool.

## Describe a model

Use a capitalized declaration name and lowercase-leading field names. This complete example
checks optional and list modifiers after compilation:

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, parse};

fn main() -> Result<(), IdeaError> {
    let schema = parse(
        "profile.idea",
        "model Profile { id String displayName String? tags String[]? }",
        &CompileInputs::default(),
    )?;
    let fields = &schema.model["Profile"].columns;
    assert_eq!(fields[1].column_type, "String");
    assert!(!fields[1].required);
    assert!(fields[2].multiple);
    assert!(!fields[2].required);
    Ok(())
}
```

The column's base type stays `String`; flags carry the modifiers. `String??`, `String?[]`, and
`String[][]` are rejected. A custom capitalized base type is allowed without declaring it first;
the consuming tool owns type meaning and record validation.

## Add declarations and metadata

The statement forms serve different data needs. This small schema combines all six:

```text
use "./shared.idea"
plugin "profiles" { output "profiles" }
prop Labels { title "Profiles" settings { enabled true } }
enum Role { ADMIN "admin" MEMBER "member" }
type Address { city String }
model Profile! @label(Labels) { id String @primary role Role @default("member") }
```

`prop` holds reusable object data, `enum` holds scalar choices, and `type` holds reusable columns.
The plugin block is configuration data, not executable registration. `use` is a recorded specifier;
pass the source graph to the transformer to load and merge it.

Both `@primary` and `@primary()` compile to `AttributeValue::Boolean(true)`. Nonempty methods
become `AttributeValue::Array` in argument order. Metadata namespaces such as `@custom.label`
stay open as long as their spelling is valid. The immutable marker on `Profile!` records
`mutable = false`; during composition an imported child replaces that local declaration entirely.

## Follow the lexical rules

Names are ASCII identifiers with these accepted shapes. Underscore-leading names are accepted
where shown; later characters need not use the same case as the first character, except in enum
choices and attribute namespaces.

| Position | Rule |
| --- | --- |
| Prop, enum, type, model, reference target, base type | First character `A`–`Z` or `_`; remainder letters, digits, `_`. |
| Column | First character `a`–`z` or `_`; remainder letters, digits, `_`. |
| Enum choice | First character `A`–`Z` or `_`; remainder uppercase letters, digits, `_`. |
| Object/prop/plugin member | First character ASCII letter or `_`; remainder letters, digits, `_`. |
| Attribute namespace | Dot-separated segments, each starting `a`–`z`, then lowercase letters, digits, `_`. |
| Plugin identity, import specifier | Raw double-quoted string. |

Separate statements, members, values, and arguments with whitespace or comments. `//` and
`/* ... */` comments count as separators. Closing delimiters may directly follow a value.
Do not insert commas, colons, semicolons, or JavaScript expressions.

Strings end at the next double quote. Backslashes remain ordinary data, so `\n` is two characters,
and `\"` cannot embed a quote. Use checked normalized JSON input when standard JSON quoting is
needed. Integers retain the signed/unsigned 64-bit range; finite decimal floats use `f64`.
Idea source does not accept exponent notation.

Duplicates fail instead of overwriting earlier data, including cross-kind declaration collisions.
You can now describe ordered fields and metadata; continue with [environment inputs](environment-inputs.md)
or [reference resolution](resolving-references.md). Lookup: [schema data](../api/schema.md).
