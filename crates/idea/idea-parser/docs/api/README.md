# Idea Parser API

All public items are exported from `stackpress_idea_parser`; implementation modules are private.
Choose a surface below for parameters, fields, return values, and errors. The optional facade
path is `stackpress::idea_parser` with feature `idea-parser`.

## Surfaces

The references follow the data boundaries. Guides explain tasks; these pages support lookup.

| Surface | Public items |
| --- | --- |
| [Source and spans](source.md) | `SourceFile`, `SourceSpan`, `Ast` and their public accessors. |
| [Parsing and compilation](stages.md) | `parse`, `parse_ast`, `compile_ast`, `compile_ast_with_references`. |
| [Syntax inspection](syntax.md) | `SchemaToken`, `Statement`, `ImportToken`, `DeclarationToken`, `DeclaratorToken`, `IdentifierToken`, `ObjectToken`, `PropertyToken`, `ArrayToken`, `LiteralToken`, `DataToken`. |
| [Lexical tokens](tokens.md) | `tokenize`, `Token`, `TokenKind`, `TokenValue`. |
| [Inputs and limits](inputs.md) | `CompileInputs`, `ParseLimits`, `HARD_DEPTH`, `max_nodes`, `max_collection`. |
| [Schema data and JSON](schema.md) | `SchemaConfig`, `TypeConfig`, `ModelConfig`, `ColumnConfig`, `AttributeValue`, `EnumConfig`, `PluginConfig`, `PropConfig`; checked schema JSON methods. |
| [Compiled references](references.md) | `CompiledSchema` and its JSON methods; `Reference`, `DeclarationKind`, `ValuePath`. |
| [Finalization](finalization.md) | `finalize`, `finalize_ast`, `finalize_with_references`, `FinalSchema`. |
| [Diagnostics](error.md) | `IdeaError`, `ErrorCode`, Display and standard error integration. |

## Example conventions

Each Rust block is an independent program, including its imports. Examples use fixed schemas,
so indexing known names and asserting expected errors is intentional. Production callers with
dynamic names should use map lookup and propagate errors.

All parser operations are synchronous and perform no external IO. Logical file names identify
diagnostics; source loading, environment snapshots, composition, and generation belong to callers
or the transformer. The parser has no optional Cargo features.

Related: [Getting started](../getting-started.md), [Concepts](../concepts.md),
[source exports](../../src/lib.rs), [tests](../../tests/).
