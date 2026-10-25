# Idea concepts

A schema often serves several tools: an editor needs syntax, a generator needs normalized models,
and a composer needs unresolved references. Idea keeps those needs in separate stages, so callers
can stop at the representation they need.

Contents: [Language](#1-the-language) · [Stages](#2-the-stages) ·
[References](#3-references-and-literal-strings) · [Inputs and limits](#4-inputs-and-limits) ·
[Diagnostics](#5-diagnostics) · [JSON](#6-json-is-a-separate-boundary)

## 1. The language

Idea describes ordered declarations and metadata. For example, a property object can supply
metadata to a model without embedding executable code:

```text
prop Labels { title "Profiles" }
model Profile @label(Labels) { id String displayName String? }
```

`Labels` is reusable data; `@label(Labels)` is a reference in a metadata argument. The parser
retains the namespace `label` without deciding what a generator should do with it.

The six statement forms are `use`, `plugin`, `prop`, `enum`, `type`, and `model`. Values include
raw double-quoted strings, exact 64-bit integers, finite decimal floats, booleans, null, arrays,
objects, and explicit `env("NAME")` expressions. Arrays, members, and arguments use whitespace
or comments as separators. Commas and executable expressions are outside the grammar.

Columns accept exactly `T`, `T?`, `T[]`, and `T[]?`. These become a base type plus `required`
and `multiple` flags; unknown capitalized type names are accepted. `type` and `model` declarations
share the same compiled shape. A trailing `!` on the declaration name sets `mutable = false`.
During transformer composition an imported child replaces an immutable local declaration;
`!` does not protect the local from replacement.

Declarations, members, attributes, and columns retain their order within their corresponding
collections. Repeated names are errors, including declaration names reused across kinds.
See [Writing a schema](guides/writing-a-schema.md) for casing and literal details.

## 2. The stages

Think of the AST as a checked description of the source and the config as data ready for a tool.
Each public stage has a specific boundary:

| Stage | Result | Responsibility |
| --- | --- | --- |
| `SourceFile::new` | `SourceFile` | Own logical identity and exact UTF-8 text; construct coordinate indexes. |
| `tokenize` | `Vec<Token>` | Inspect lexical tokens, raw spelling, and spans; no declaration validation. |
| `parse_ast` | `Ast` | Validate syntax and retain typed environment/reference nodes. |
| `compile_ast` | `SchemaConfig` | Evaluate explicit environment inputs; retain imports, props, and placeholder JSON. |
| `compile_ast_with_references` | `CompiledSchema` | Produce config plus actual, structurally addressed reference sites. |
| `finalize_ast` | `FinalSchema` | Resolve prior references in one checked source; reject imports. |
| `finalize_with_references` | `FinalSchema` | Resolve surviving typed sites after composition; reject imports. |

`parse(file, source, inputs)` is the direct text-to-config convenience. `finalize(file, source,
inputs)` is a compatibility text-to-final entry: it resolves prior references and discards `use`
directives without loading them. Prefer `finalize_ast` for a standalone source that must reject
imports, and the transformer for a source graph. There is no public `compile` function; checked
compilation is exposed through `compile_ast` and `compile_ast_with_references`.

Only `parse_ast` can construct the immutable `Ast`. Public syntax nodes support inspection and
serialization, but editing a detached node cannot feed unchecked syntax into compilation.
Compiling an AST rechecks the requested limits against its retained text, so a later lower limit
still applies.

## 3. References and literal strings

An identifier argument and a quoted string can have identical compiled JSON text. Their intent
is different:

```text
prop Labels { title "Profiles" }
model Profile @label(Labels "${Labels}") { id String }
```

Both arguments appear as `"${Labels}"` in plain non-final config. Typed compilation records only
the first argument as a `Reference`. Typed finalization replaces that site with
`{"title":"Profiles"}` and leaves the quoted string unchanged.

A reference identifies its owning type/model, optional column name, metadata namespace, and path
through argument arrays/objects. It also retains the target name and original `SourceSpan`.
Using column names rather than positions lets a composer prepend columns without shifting sites.
The composer must retain only sites belonging to surviving metadata and declarations.

References can target prior props, enums, plugins, types, or models. Bare identifiers are accepted
only inside metadata argument data, including nested arrays/objects. `Role.ADMIN` is unsupported.
A same-source forward or self reference fails finalization. An imported target absent from that
source can be supplied by composition; missing targets and cycles still fail.

## 4. Inputs and limits

Compilation sees only `CompileInputs.environment`, an ordered map supplied by the caller.
`env("NAME")` becomes a string, without scalar coercion. Missing keys fail by default;
`missing_env_is_empty = true` explicitly enables empty-string compatibility.

Default `ParseLimits` allows 1 MiB of source bytes and depth 64. Effective depth cannot exceed
`HARD_DEPTH = 64`; byte limits also derive a maximum of 100,000 nodes and 10,000 entries per
collection. Environment snapshots, repeated expansion, reference metadata/copies, and checked
JSON operations have bounded work and allocation too. A source that parses can still exceed a
later expansion or serialized-output budget.

`SourceFile::new` constructs indexes before a parse policy is supplied. The checked boundaries
do not make arbitrary caller-created `serde_json::Value` trees safe to recursively clone,
serialize, or drop. Use `SchemaConfig::validate` and checked export before passing assembled
config onward; callers still own the construction and destruction of their own data.

## 5. Diagnostics

Failures return `IdeaError`, with a typed `code`, a human-readable `message`, source identity,
optional `start`/`end` byte offsets, and one-based line/column. Byte ranges are half-open;
parser-produced endpoints land on UTF-8 boundaries. Columns count Unicode scalars, not bytes
or display cells; tabs count as one scalar, and CRLF is one line break.

For duplicate names, `related` points to the earlier declaration/member. Missing closing
delimiters use a zero-width EOF span. Source-free validation failures can omit byte coordinates.
Display prints `file:line:column: message` without including source excerpts or environment
values. Applications choose whether to render excerpts from their retained source text.

## 6. JSON is a separate boundary

`SchemaConfig::from_json` and `from_json_named` import normalized config, rather than Idea source
or syntax-node JSON. `to_json` validates and exports compact ordered JSON. These methods reject
duplicate keys, unknown structural fields, malformed normalized fields, and exceeded limits;
metadata payloads remain open data.

Plain schema JSON has no reference intent: every string remains literal. Use
`CompiledSchema::{to_json, from_json}` to transport the `{ "config": ..., "references": ... }`
envelope and retain actual sites. Checked import validates site structure and destinations;
without the historical source text it cannot authenticate stored coordinates against that text.

JSON supports standard JSON escapes and exponent syntax; Idea source keeps its existing raw
string and decimal grammar. Both reject integer range overflow and nonfinite numbers.
After this distinction is clear, use the [JSON guide](guides/json.md) to choose the correct format
or the [API index](api/README.md) to inspect the exact boundary you need.
