# Schema data and checked JSON API

`SchemaConfig` is ordered non-final data. It retains imports, reusable props, and unresolved
placeholder values. Sources: [finalize.rs](../../src/finalize.rs), [types.rs](../../src/types.rs),
[json.rs](../../src/json.rs).

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, parse};

fn main() -> Result<(), IdeaError> {
    let schema = parse(
        "profile.idea",
        "model Profile { id String tags String[]? }",
        &CompileInputs::default(),
    )?;
    assert!(schema.model["Profile"].mutable);
    assert!(schema.model["Profile"].columns[1].multiple);
    Ok(())
}
```

Contents: [Data shapes](#1-data-shapes) · [Checked JSON](#2-checked-json-operations) ·
[Normalized format](#3-normalized-json-format)

## 1. Data shapes

Public collections are owned and preserve order. The parser does not interpret metadata or
require that a base type name has a matching declaration.

### 1.1. SchemaConfig

Debug, Clone, PartialEq, Serialize, and Default. Default has empty sections. Empty sections are
omitted from serialization. Rust raw identifiers serialize as ordinary `use`, `enum`, and `type`.

| Public field | Type | Meaning |
| --- | --- | --- |
| `r#use` | `Vec<String>` | Unresolved import specifiers in source order. |
| `plugin` | `IndexMap<String, PluginConfig>` | Plugin configuration, without code activation. |
| `prop` | `IndexMap<String, PropConfig>` | Reusable property objects. |
| `r#enum` | `IndexMap<String, EnumConfig>` | Named scalar choice maps. |
| `r#type` | `IndexMap<String, TypeConfig>` | Reusable fieldsets. |
| `model` | `IndexMap<String, ModelConfig>` | Models. |

Order is retained within sections and member collections. The original cross-kind statement
sequence belongs to the AST, not a flattened iterator over `SchemaConfig`.

### 1.2. Fieldsets and columns

`TypeConfig` and `ColumnConfig` are Debug, Clone, PartialEq, and Serialize. `ModelConfig` is a
type alias for `TypeConfig`; there is no distinct model constructor or method set.

| Type | Public field | Type of field | Meaning |
| --- | --- | --- | --- |
| `TypeConfig` | `name` | `String` | Declaration identity, matching its section map key. |
| `TypeConfig` | `mutable` | `bool` | `true` by default in source; `!` makes it false for composition. |
| `TypeConfig` | `attributes` | `IndexMap<String, AttributeValue>` | Ordered declaration metadata. |
| `TypeConfig` | `columns` | `Vec<ColumnConfig>` | Ordered unique columns. |
| `ColumnConfig` | `name` | `String` | Column identity. |
| `ColumnConfig` | `column_type` | `String` | Unmodified capitalized base type; JSON field is `type`. |
| `ColumnConfig` | `attributes` | `IndexMap<String, AttributeValue>` | Ordered column metadata. |
| `ColumnConfig` | `required` | `bool` | Whether the source omitted `?`. |
| `ColumnConfig` | `multiple` | `bool` | Whether the source included one `[]` suffix. |

The flags describe syntax, not validation of application values. Under transformer composition,
an immutable local fieldset can be replaced entirely by an imported child.

### 1.3. Metadata and data aliases

`AttributeValue` is Debug, Clone, PartialEq, and untagged Serialize:

| Variant | Meaning |
| --- | --- |
| `Boolean(bool)` | Flag state. Source `@flag` and empty `@flag()` both compile to true. |
| `Array(Vec<serde_json::Value>)` | Nonempty source method arguments, or checked JSON argument data. |

`EnumConfig`, `PluginConfig`, and `PropConfig` are all aliases for
`IndexMap<String, serde_json::Value>`. Enum syntax/checked JSON requires scalar values;
plugin and prop values may contain nested data. These types have no public inherent methods.

## 2. Checked JSON operations

Use these methods for a validated normalized boundary. Raw Serialize/Clone remain available
for compatibility, but do not enforce the checked policy on caller-built recursive values.

### 2.1. from_json

`SchemaConfig::from_json(source: &str, limits: ParseLimits) -> Result<Self, IdeaError>`

Import literal normalized schema data with the default diagnostic name `<json>`.

```rust
use stackpress_idea_parser::{IdeaError, ParseLimits, SchemaConfig};

fn main() -> Result<(), IdeaError> {
    let schema = SchemaConfig::from_json(
        r#"{"prop":{"Labels":{"title":"${Title}"}}}"#,
        ParseLimits::default(),
    )?;
    assert_eq!(schema.prop["Labels"]["title"], "${Title}");
    Ok(())
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `source` | `&str` | Complete normalized JSON object; strings are literal data. |
| `limits` | `ParseLimits` | Raw/decoded bytes, JSON containers, nodes, and collections. |

**Returns** New ordered config, or JSON syntax/shape, missing-field, duplicate, literal-range,
or limit errors named `<json>`. Imports are retained without loading; environment/reference-looking
strings are not evaluated. An empty object produces an empty config.

### 2.2. from_json_named

`SchemaConfig::from_json_named(file: &str, source: &str, limits: ParseLimits) -> Result<Self, IdeaError>`

Use an explicit identity when reporting imported JSON failures.

```rust
use stackpress_idea_parser::{ErrorCode, ParseLimits, SchemaConfig};

fn main() {
    let error = SchemaConfig::from_json_named(
        "profile.json",
        r#"{"model":{"Profile":{"name":"Profile"}}}"#,
        ParseLimits::default(),
    )
    .unwrap_err();
    assert_eq!(error.file, "profile.json");
    assert_eq!(error.code, ErrorCode::MissingProperty);
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `file` | `&str` | Logical diagnostic identity, not a file to read. |
| `source` | `&str` | Complete normalized JSON object. |
| `limits` | `ParseLimits` | Same JSON policy as `from_json`. |

**Returns** Config or the same checked failures as `from_json`, under `file`. Structural
failures without a lexical position are located at `0..0`; they do not identify an exact bad field.

### 2.3. validate

`validate(&self, limits: ParseLimits) -> Result<(), IdeaError>`

Check manually assembled or edited config before checked export/finalization.

```rust
use stackpress_idea_parser::{IdeaError, ParseLimits, SchemaConfig};

fn main() -> Result<(), IdeaError> {
    let mut schema = SchemaConfig::default();
    schema.r#use.push("shared.idea".into());
    schema.validate(ParseLimits::default())?;
    Ok(())
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `ParseLimits` | Borrowed-value traversal, normalized structure, and encoded-size policy. |

**Returns** `()` for valid config, or invalid names/flags/shapes, duplicate declarations/columns,
missing normalized properties, and budget failures. This does not resolve imports/references or
interpret metadata. It leaves the caller's data unchanged; source-free errors may omit coordinates.

### 2.4. to_json

`to_json(&self, limits: ParseLimits) -> Result<String, IdeaError>`

Validate and export compact ordered normalized JSON without external IO.

```rust
use stackpress_idea_parser::{IdeaError, ParseLimits, SchemaConfig};

fn main() -> Result<(), IdeaError> {
    let json = SchemaConfig::default().to_json(ParseLimits::default())?;
    assert_eq!(json, "{}");
    Ok(())
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `ParseLimits` | Validation and bounded encoded-output policy. |

**Returns** Owned JSON text, or validation/resource failures. Empty sections are omitted, and
string escaping or normalized field names can make output larger than the original Idea source.

## 3. Normalized JSON format

The root allows only `use`, `plugin`, `prop`, `enum`, `type`, and `model`; each may be omitted.
`use` is a string array. Other sections are objects keyed by globally unique declaration names.
Plugin identities are string keys; other declaration names follow the capital identifier rule.

Fieldsets require exactly `name`, `mutable`, `attributes`, and `columns`. Each column requires
exactly `name`, `type`, `required`, `multiple`, and `attributes`. Names must match their grammar;
the fieldset `name` must match the section key. Column `type` is already unmodified, flags are
booleans, and columns form a unique-name array. Attribute values are booleans or argument arrays
under valid lowercase dotted namespaces. No defaulting hides missing required fields.

Duplicate keys at any depth, cross-kind name collisions, unknown structural fields, malformed
flags/modifiers, non-scalar enum choices, integer overflow, and nonfinite numbers are errors.
JSON permits standard escapes/exponents independently of Idea's raw-string/decimal grammar.

Related: [JSON guide](../guides/json.md), [typed envelope](references.md),
[final output](finalization.md#1-finalschema).
