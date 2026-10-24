# Slice 3 upstream inventory and ownership

Design checkpoint for [task 00006](tasks/00006-slice-3-idea.md). Load for every relevant parser/transformer export, language/generation family, test behavior, and Slice 1 deferral; implementation scope follows pending [decisions](slice-3-design-decisions.md).

## Sources and disposition vocabulary

Read on 2026-10-24: Idea HEAD `cf8e922ba6e57746e15a05c26df0546dda2cc5c9`, pinned Rust core `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`, Lib HEAD `e2cc6b8cad77b5ad18be8a94e0d40e72e2f6183a`, Stackpress HEAD `a71d683051ba8350fdd12d6b5a33f268fdcc285f`. The Stackpress checkout has pre-existing dirty work; cited files were read from its current working tree, not assumed identical to HEAD. No upstream files were edited.

“Now” means proposed Slice 3 implementation after its gates, not work already delivered. “Later” names a later slice or a new task requiring approval. “Not ported” rejects a source mechanism, while preserving the useful capability where stated. Read [accepted structure](../../context/source-structure.md) for crate ownership and [Slice 1 inventory](design-upstream-inventory.md) for the original deferrals.

## Parser exports, types and mechanisms

Source: [parser index](../../../../idea/packages/idea-parser/src/index.ts), `types.ts`, `definitions.ts`, `Compiler.ts`, trees, plus the pinned core's seven source modules.

| Upstream surface | Disposition and reason |
| --- | --- |
| `parse`, `final`; `Compiler.schema`, `Compiler.final` | Now: explicit syntax parse, non-final compile and finalization stages; no implicit filesystem composition. Preserve typed unresolved references until finalization. |
| `Exception`; Rust `IdeaError`, `ErrorCode` | Now: parser-owned diagnostic/code/source spans and cause where applicable; host integration converts deliberately to lib Error, without parser→server dependency. |
| `Lexer`; Rust `tokenize`, `Token`, `TokenKind`, `TokenValue` | Now: bounded tokenizer and useful source-positioned debug output. Parser and lexer should share cursor/literal rules; don't retain a second divergent grammar. |
| `AbstractTree`, `EnumTree`, `PropTree`, `TypeTree`, `ModelTree`, `PluginTree`, `SchemaTree`; internal `UseTree` | Now: six declaration kinds and complete schema parsing. JS class inheritance/static-overload API is not ported; ordinary Rust functions/enums supply the capability. |
| `Compiler.array/data/object/literal/identifier/enum/prop/type/model/plugin/use` | Now: typed internal compilation helpers with validated input; public stage-level APIs suffice until a real subcompiler caller exists. No unchecked arbitrary object tokens. |
| `Reader`, `Definition`, `Parser`; mutable `Lexer.define/get/clone/load/match/next/optional/read/substring/nextSpace` | Not ported as an open runtime grammar-mutation API: no evidence this is needed by the proof. Internal cursor/lookahead replaces it. Later editor/language-extension task can propose an actual contract. |
| `UnknownToken`, `Token`, `DataToken`, `ImportToken`, `SchemaToken`, `DeclarationToken`, `DeclaratorToken`, `IdentifierToken`, `ObjectToken`, `PropertyToken`, `ArrayToken`, `LiteralToken` | Now: equivalent typed syntax/spans; ESTree field names and byte-for-byte AST JSON are not promised. Compatible compiled schema is the interoperability target. |
| `UseReferences`, `Scalar`, `Data`, `AttributeValue` | Now: typed literal/reference/env nodes and JSON-compatible compiled values; no JS `any`, object aliases, or implicit process globals. |
| `EnumConfig`, `PluginConfig`, `PropConfig`, `ColumnConfig`, `TypeConfig`, `ModelConfig`, `FinalSchemaConfig`, `SchemaConfig` | Now: ordered declarations/members, ordered columns, required/multiple/mutable flags, open attributes, optional schema sections and JSON import/export. Avoid runtime framework meaning. |
| `definitions`, `scalar`, `data`, `scan`, `reader`, `identifier` deep imports | Now internally as scanner rules; not separate compatibility exports. Extensibility remains metadata/transform based, not arbitrary regex readers. |
| Rust `compile_schema`, `finalize`, `parse_ast` | Now: extraction starting point, with bounded input validation, typed AST, ordered structures, explicit environment, and source diagnostics before reuse. |
| Rust N-API crate, JS loader, per-platform npm packages | Not ported: separate distribution concern; no generator needs them. |

## Language catalog coverage

Sources: [language catalog](../../references/00022-upstream-idea-language-catalog.md), [component catalog](../../references/00031-upstream-idea-component-catalog.md), and the [source/probe findings](slice-3-design-grammar.md).

| Family | Disposition and boundary |
| --- | --- |
| `use`, `enum`, `prop`, `type`, `model`, `plugin` | Now: syntax/compiler; transformer owns recursive `use`, compiled registry IDs replace executable module paths. |
| Whitespace, line/block comments, quoted strings, integers/floats, booleans/null, nested arrays/objects | Now: bounded grammar and valid spans, including malformed EOF. String escape extension remains later language work. |
| `env("NAME")`, prior prop references, unresolved placeholder JSON view | Now: pure syntax plus explicit compile inputs. Dotted enum reference syntax is not ported now; both current parsers reject it. |
| Required/optional/multiple, custom type names, immutable `!` | Now: typed syntax flags; exact modifier/casing/duplicate policy needs S3-D02 approval. Type existence/storage compatibility is a later semantic concern. |
| Arbitrary well-formed `@flag` / `@method(...)` / namespaces | Now: syntax and arguments, unknown metadata retained. No built-in whitelist. |
| `display`, `icon`, `labels`, `query` | Syntax now; meanings later `framework-schema` generation task. |
| `active`, `default`, `description`, `examples`, `encrypted`, `generated`, `hashed`, `id`, `searchable`, `sortable`, `label`, `min/max/step`, `relation`, `timestamp`, `unique` | Syntax now; schema/type/name/document/value/assertion semantics later framework-schema; persistence and migration behavior Slice 4 framework-SQL. |
| Built-in String/Text, Number/Integer/Float, Boolean, Date/Datetime/Time, Json/Object/Hash; custom enums/fieldsets/models | Parser accepts names now. Full Rust value mapping later framework-schema; proof supports only its declared Profile shape and rejects unsupported shapes. |
| `cuid()`, `nano`/`nanoid`, `random()`, `now()`, timestamp/default expressions | Preserved strings now; generator-expression semantics later framework-schema/framework-SQL. No expression execution in parser. |
| All `is.*` presence/equality, dates, numeric comparisons, character/word counts, formats/casing and type families; aliases | Preserved syntax now; dictionaries, messages and generated validation later framework-schema task. No claim of validation from parsing an attribute. |
| All field controls/collections/fieldset/relation; view text/media/layout/link families and aliases | Preserved syntax now; component dictionaries and Leptos generation Slice 5. Existing Frui/React implementations are not ported. |
| Derived filter/span from field and list from view; fixed-definition prop precedence and omission of absent roles | Later Slice 5 schema/UI integration, with runtime tests; never implemented implicitly by Idea. |

## Transformer exports, composition and tests

Source: [Transformer](../../../../idea/packages/idea-transformer/src/Transformer.ts), `Terminal.ts`, `types.ts`, `tests/Transformer.test.ts`, `tests/Terminal.test.ts`, fixtures and `tests/in/make-enums.ts`.

| Surface/behavior | Disposition and reason |
| --- | --- |
| `Transformer.load`, constructor, `loader`, `input`, cached `_schema`, `schema()` | Now: explicit loader/project root, immutable bounded per-run snapshots and composed schema/source map. No process cwd mutation or stale cache across runs. |
| Recursive Idea imports; `.json` normalized schema imports | Now: importing-file-relative and project alias resolution, canonical IDs, cycle/depth/file/byte limits, typed validated JSON and import-chain errors. |
| Prop/enum soft merge; `_merge` of attributes and ordered columns | Now: explicit decision table and provenance. Avoid reversing shared child columns in place. Immutable-local discrepancy requires S3-D03 decision. |
| `transform(extras)`; declaration-order plugin callbacks with `{transformer,config,schema,cwd,...extras}` | Now: selected compiled registration functions, ordered specs/dependencies, typed transform context and in-memory output plan. No unstructured arbitrary extras. |
| `.js/.cjs/.mjs/.ts/.mts` dynamic resolution/default import; silently skip non-callables | Not ported: Rust function registration replaces executable loading; unknown/unselected plugin IDs fail before effects. |
| Missing input and absent plugin errors | Now: descriptive source/IO errors; composition without plugins is valid, generation with no selected transform errors. |
| `CLIProps`, `FileLoaderOptions`, `TerminalOptions`, `PluginProps`, `PluginWithCLIProps` | Now by capability: explicit loader/request/context/options types. CLI is an adapter dependency, not imported by transformer. JS type intersections are not ported. |
| Transformer `Terminal`, `control`, command/brand/config wiring | Existing Slice 2 `stackpress-cli` supplies argv/config/output/exit behavior. Add app-owned generation host, not a duplicate terminal or an empty stock binary. |
| Fixture composition (`Profile/Auth/Connection/File/Address/Contact/Roles/Config`), missing input, generated enum file, local attribute precedence | Now: behavioral equivalents and stronger exact-order/removal tests. Sample enum generation becomes a test transform, not an automatic production plugin. |
| Upstream “JSON import equals Idea” test | Now: replace with an actual awaited result comparison. Current test compares uncalled `.schema` methods, so its success is not evidence of equivalent outputs. |

## Parser test inventory

The fixture matrix and own-test executions are in [assessment](slice-3-design-assessment.md). The TypeScript Mocha files were inspected, not executed as a suite.

| Test family | Proposed proving behavior now |
| --- | --- |
| `EnumTree`, `PropTree`, `PluginTree`, `UseTree` | Full valid declaration fixtures, structured empty/invalid declaration errors, enum scalar env; standalone helpers replaced by typed stage tests. |
| `TypeTree`, `ModelTree` | Ordered columns, attributes/props, negative numbers, immutable marker, optional/list decision table; no mock-lexer type assertion tricks. |
| `Schema` | Complete AST and compiled/final golden contracts, trailing garbage/commas, duplicate declarations, unresolved references. Rust AST goldens describe Rust's chosen typed shape separately. |
| `Compiler` | Invalid preconstructed input, missing columns, incorrect kinds/references/duplicates/import type. Exhaustive enums make wrong-kind paths unrepresentable where possible. |
| `Lexer` | Scalar/array/object token data, whitespace, identifiers and position behavior, invalid/boundary input. Add UTF-8, CRLF, bounded recursion, number overflow, fuzz-like deterministic corpus and tokenizer/parser agreement. |
| Existing Rust tests | Retain useful schema coverage, replace environment mutation, upgrade nonempty assertions to exact contracts. |

## Stackpress schema and generated contracts

Sources: [Modeling and generation ownership](../../references/00013-upstream-modeling-and-generation.md), [schema API](../../references/00021-upstream-schema-api-contracts.md), [generated contracts](../../references/00029-upstream-exports-types-generated-contracts.md), and current `stackpress-schema/src/{Schema,plugin,types}.ts`, `scripts/generate.ts`, `transform/index.ts`, `transform/helpers.ts` and related transforms.

| Capability | Disposition and reason |
| --- | --- |
| `idea` event collects package transforms; `generate` orchestrates | Now in a separate codegen host using Slice 2 CLI `emit generate`; registration/order stays explicit. A generated app cannot bootstrap its own first generator if it needs missing generated modules. |
| Inspectable source, declared inputs, sequential transforms, repeat stability and compilation/import | Now: owned output plan, deterministic writer, generated consumer compile and runtime registration proof. |
| Shared ts-morph project; cooperating index/package rewrites; CJS/ESM/JS/declaration emit and Prettier | Mechanisms not ported. Now single-owner Rust module registries aggregate declared contributions, followed by deterministic Rust formatting. Cargo dependencies are handwritten and declared in advance. |
| Schema/Fieldset/Model/Column/Attribute; attributes/columns collections, schema-parent propagation | Later framework-schema task. A narrow example generator reads typed Idea structure directly; do not scaffold a partial public semantic lens library for two String fields. |
| All assertion/component/document/name/number/store/type/value lenses, global dictionaries/definitions, interfaces and crypto/naming helpers | Later framework-schema, Slice 4 persistence or Slice 5 UI according to semantic owner. Process-global mutable registries and deterministic-IV crypto are not target recommendations. |
| Generated config/enums/types/columns/schema/assert/filter/populate/serialize/unserialize/tests | Later framework-schema beyond the explicit proof's public Profile types. Parser config is not a replacement for these interpreted interfaces. |
| Model stores/actions/events/listeners; SQL tests/migrations | Later Slice 4. Proof operation registration now is an explicitly example-owned in-memory capability. |
| Field/filter/list/span/view wrappers, admin/routes, AI tools and runtime client loader | Later Slice 5 UI/admin or separately approved framework/API/AI tasks. No fabricated implementation claims. |
| Revisions/history | Later framework-schema/SQL task; schema history is distinct from applied migrations. |
| Promised schema pruning | Now only for the proof generator's dedicated output root and manifest; full schema-generator pruning later. Do not claim universal SQL/view/admin/AI cleanup. |

## Slice 1 host-filesystem and template deferrals

Actual Lib sources are [system/FileLoader.ts](../../../../lib/src/system/FileLoader.ts), [system/NodeFS.ts](../../../../lib/src/system/NodeFS.ts), [Template.ts](../../../../lib/src/Template.ts), and relevant tests. The earlier top-level source labels identify capabilities, not literal current paths.

| Export/type/behavior | Disposition and reason |
| --- | --- |
| `FileLoader` constructor, cwd/fs, `absolute`, `basepath`, `relative`, `resolve`, `resolveFile`, `_fileExists`; absolute/relative/base/root alias, required/optional absence, extension/index candidates | Now only the schema-source resolver in transformer: fixed project root, importer-relative base, `@/`, exact path then documented `.idea`/`.json` and directory-index candidates, canonical file identity and IO distinction. No speculative general loader in lib. |
| `FileLoader.lib/modules`, node_modules ascent, executable `import`, `esmImport`, `include`, default-export unwrapping | Not ported: declared Cargo modules/function plugins replace package discovery and code execution. JSON import is data-only and validated. |
| `NodeFS` exists/readFile/realpath/stat/writeFile/mkdir/unlink, injected filesystem; `FileSystem`, `FileStat`, `FileRecursiveOption` | Now sufficient native read/metadata/canonicalization and staged generation output create/read/write/rename/remove via std::fs behind real test seams. Preserve IO causes; permission denied is not “missing.” No Node-shaped universal public FS trait. |
| `createReadStream`, `FileStream`, general recursive traversal and runtime file serving | Later native-file/streaming adapter task (Slice 2 follow-up); generation only needs bounded text/JSON and owned output files. Native target support requires its own proof. |
| `TemplateEngine`, `render`, delimiters/container/resolver; `TemplateOptions`, `TemplateResolver`, `TemplateHelper`, `TemplateHelperOptions` | Later explicitly scoped template task after Slice 3, pending S3-D06. Current proof needs small typed Rust emitters, not a public general template language. This is a disclosed scope deferral, not completed template parity. |
| Helpers `?`, `!`, `\|`, `@` (array/object loops and iteration metadata), `=` arithmetic, `.` clipping, `A/a` case | Inventory retained; later template task if requested by an actual generator. Use typed Rust loops/conditions/string operations now. No eval/Function/arbitrary method expression compatibility. |
| Same-name nested helper limitation; truthiness/fallback, trim/join, missing/null behavior and custom helper registration | Later template task must choose explicit semantics and bounded nesting/errors, not inherit regex limitations. No template-specific upstream test file exists. |

Dependency placement matters: host filesystem and generation stay in transformer/tool paths. `stackpress-lib`, server default builds, generated contracts and browser paths acquire no generation or filesystem dependency. See [generation design](slice-3-design-generation.md) and its explicit dependency budget.
