# Slice 3 grammar and compatibility findings

Design checkpoint companion to [parser evidence](slice-3-design-assessment.md). Load for exact probe inputs, source-versus-catalog discrepancies, diagnostics, and the proposed Rust language boundary; choices remain pending in [decisions](slice-3-design-decisions.md).

## Source anchors

Pinned Rust: `packages/idea-rust/crates/idea_parser_core/src/{parser,token,finalize,ast,error,types}.rs` at `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`. Current TypeScript: `packages/idea-parser/src/{definitions,Compiler}.ts` and `trees/{Type,Model,Enum,Prop,Plugin,Use,Schema}Tree.ts` at `cf8e922ba6e57746e15a05c26df0546dda2cc5c9`. See [language catalog](../../references/00022-upstream-idea-language-catalog.md) for intended syntax, and source for observed bugs/permissiveness.

## Probes and outcomes

These are 29 independently named small files, fed to both runners. `R` means pinned Rust; `TS` means the disposable current-source oracle. “Accepts” describes source parsing unless compilation/finalization is explicitly named. `DATABASE_URL=test`; `IDEA_ASSESS_MISSING` was absent from the Rust child environment.

| Probe | Exact source (escaped newlines where needed) | Observed result |
| --- | --- | --- |
| `use` | `use "./missing.idea"` | Both compile `use:["./missing.idea"]`; both final outputs are empty. Neither loads a file. |
| `env_plugin` | `plugin "p" { url env("DATABASE_URL") }` | Both produce `url:"test"`. |
| `env_enum` | `enum X { A env("DATABASE_URL") }` | R rejects `env` at 11..14; TS accepts. Rust scalar parser omits env. |
| `env_missing` | `prop X { url env("IDEA_ASSESS_MISSING") }` | R compiles empty string. Both source implementations use missing→empty fallback. |
| `prop_reference` | `prop A { x "v" } model X { id String @thing(A) }` | Both finalize attribute to `[{x:"v"}]`. |
| `forward_reference` | `model X { id String @thing(A) } prop A { x "v" }` | Both compile placeholder `${A}`, then finalization rejects unknown A. |
| `dotted_reference` | `enum Role { ADMIN "admin" } model X { role Role @default(Role.ADMIN) }` | Both reject dotted reference syntax; the catalog's `Role.ADMIN` example is not supported by current parser source. |
| `plugin_order` | `plugin "z" { } plugin "a" { }` | R AST preserves z,a but compiled plugin map iterates a,z; TS retains z,a. |
| `modifiers` | `model X! { a String b String? c String[] d String[]? }` | R produces immutable X with required scalar, optional scalar, required list, optional list. TS rejects trailing `?` on the list. |
| `optional_array` | `model X { a String?[] }` | R rejects. TS accepts but compiles type `String?`, required=true, multiple=true. |
| `repeated_array` | `model X { a String[][] }` | R accepts and erases all array suffixes; TS accepts and retains `String[]` as base type. Neither establishes useful nested-list semantics. |
| `repeated_optional` | `model X { a String?? }` | R accepts and erases repeated `?`; TS rejects. |
| `comment` | `// head\n/* note */ model X { id String // field\n }` | R accepts; TS rejects this mixed placement at `//`. Catalog intent supports comments, so this is not a desired Rust regression. |
| `unclosed_comment` | `/* bad` | R UnexpectedEof with **no span**; TS rejects. |
| `unclosed_model` | `model X {` | R error span 9..10 despite input length 9; TS silently compiles `{}`. |
| `unicode_error` | `// 😀\nmodel X { id é }` | R error span 21..22 splits the two-byte `é`; TS rejects. |
| `empty_method` | `model X @thing() { id String }` | Both compile flag `thing:true`; AST empty-array versus literal differs. |
| `bad_attribute` | `model X @9..A { id String }` | R accepts invalid lowercase-namespace grammar; TS rejects. |
| `lower_name` | `model lower { id String }` | R rejects; TS accepts due case-insensitive CapitalIdentifier regex. |
| `lower_enum` | `enum X { lower "v" }` | R rejects; TS accepts due case-insensitive UpperIdentifier regex. |
| `duplicate_decl` | `model X { id String } type X { id String }` | Both compilation paths reject duplicate X, including across declaration kinds. |
| `duplicate_column` | `model X { id String id Number }` | R retains two ordered columns named id; TS keeps last value (`Number`) at the original key position. |
| `duplicate_property` | `prop X { x 1 x 2 }` | Both compile x=2. |
| `duplicate_attribute` | `model X @a(1) @a(2) { id String }` | Both compile a=[2]. |
| `unterminated_string` | `prop X { x "bad` | Both reject; R reports UnexpectedEof 11..15. |
| `overflow_integer` | `prop X { x 9223372036854775808 }` | R rejects outside i64; TS accepts a JS Number with precision loss. |
| `nonfinite_float` | `prop X { x ` + 400 copies of `9` + `.0 }` | R accepts and serializes value as null; TS JSON output is also null for nonfinite numeric data. |
| `array_spacing` | `prop X { x [1 2] }` | Both compile x=[1,2], whitespace-delimited. |
| `object_identifier` | `prop X { x Elsewhere }` | R accepts an identifier in ordinary object data, then finalization rejects unknown reference; TS rejects this syntax. |

The separate 20,000-depth Rust process abort is recorded in [assessment](slice-3-design-assessment.md). String escapes are not decoded by either current implementation: the next double quote terminates the string. No new escaped-string syntax is assumed in this slice. Arrays/objects/method arguments are whitespace-delimited; commas, executable expressions, JS method calls and arbitrary evaluation are not Idea syntax.

## Diagnostics contract proposed for Rust

`SourceId` plus a source map retains each file's logical name, UTF-8 source text, and line starts. Every token/AST node/diagnostic carries a half-open UTF-8 **byte** span within that source. Missing closers use 0-width EOF spans. Every endpoint must be `<= source.len()` and a character boundary. Synthetic nodes must have a real enclosing/zero-width span, rather than invented key-length offsets. A span means a source range, not a JavaScript UTF-16 index.

Diagnostics have a typed stable code, concise message, primary span and optional related spans/import chain. Display resolves file and one-based line/Unicode-scalar column; tabs count as a scalar in coordinates, while snippet rendering can expand them. CRLF counts as one line break. Source excerpts are an explicit developer rendering option, not part of sanitized public errors. Environment variable **names**, not values, can identify missing inputs.

Pure parser APIs read no filesystem or process environment. Parse `env("NAME")` into an environment-reference node retaining its source span/raw syntax. Compilation receives an explicit immutable environment map and a missing-value policy. Host tooling snapshots only referenced/allowed keys before generation; tests pass maps or set child-process environments. No `set_var`, unsafe block, or implicit global mutation is needed.

Non-final compilation preserves actual reference nodes internally; a compatible JSON view may serialize `${Name}` placeholders. A literal string `${Name}` must remain distinguishable from a reference until finalization. Finalization resolves only valid earlier references and must not silently load imports. Transformer composition retains typed reference/provenance data until its deliberate resolution step; merging JSON placeholders alone cannot correctly resolve imported props.

## Intended initial language boundary (recommendations)

These are proposals for S3-D02, not accepted corrections to existing source:

- Support all six declaration forms, arbitrary capital type names, unknown well-formed metadata namespaces, required/optional scalar and list modifiers, `!`, scalar/array/object values, comments, prop references and env values in every scalar position including enums.
- Preserve source declaration/member order with ordered maps/arrays, including plugins. Duplicate top-level names remain errors. Recommend duplicate column, enum-key, prop/object-key and attribute names become explicit errors with both spans, instead of accidental last-wins or duplicate arrays. The owner may choose last-wins compatibility instead; implementation cannot silently decide.
- Recommend exactly `T`, `T?`, `T[]`, `T[]?`. Reject repeated modifiers and `T?[]` rather than emit a malformed base type. Retain strict catalog casing for declarations and enum keys, and lowercase dotted namespaces for attributes.
- Keep bare prop references in attribute arguments and finalization's prior-declaration rule. Do not invent dotted enum-member access or arbitrary ordinary-object identifiers. Carry the catalog example discrepancy as an explicit unsupported form with a useful error.
- Retain the current raw double-quoted string grammar in this slice; do not add an escape language without a separate decision. Generated Rust/JSON literals still require correct output escaping.
- Recommend integers use serde_json's exact supported integer range (signed/unsigned 64-bit); floats must be finite. Reject range overflow/nonfinite values instead of silently producing null or imprecise numbers.
- Recommend missing env defaults to a diagnostic, with an explicit `Empty` compatibility option. All environment reads are declared inputs; don't serialize their values into receipts/manifests.

Limits must precede allocation/recursion at tokenization, parsing, compilation, clone/serialization and import composition boundaries. Start with a hard recursive nesting ceiling of 64 plus lowerable byte/node/collection/file/dependency budgets; determine concrete defaults in M0 from bounded tests. Public preconstructed AST/config input needs validation too; limiting source parsing alone does not protect compilation, drop or serialization of an arbitrarily deep public tree.

## `use` source discrepancy and proposed composition

Current `idea-transformer/src/Transformer.ts::schema` uses `if (!parent || !parent.mutable) { schema.model[name] = model; continue; }`, with the same branch for types. Therefore an immutable local declaration is replaced by the child. The [catalog](../../references/00022-upstream-idea-language-catalog.md) instead says it is not extended. This checkpoint does not edit either source or KB claim.

Recommend: local props/enums win; missing types/models adopt the child; immutable locals remain untouched; mutable locals receive missing child attributes beneath local attributes and missing child columns prepended in child order; colliding columns remain wholly local. Apply direct `use`s in source order. With multiple children, earlier imported conflicting values win after entering the accumulated parent; later child-only columns prepend their group. Prove this nontrivial ordering explicitly. Never reverse a cached child's columns in place as upstream `_merge` does.

Imports resolve relative to the **importing file**, with explicit project-root `@/` aliases. Canonical identity detects cycles, including symlink aliases, and a bounded DFS reports the entire chain. Diamonds share an immutable loaded snapshot but merge independently; cycle detection uses the active stack, not a global “seen means cycle” flag. Child plugin declarations are not automatically activated (upstream composition also omits them); the root/tool selects generator plugins explicitly. JSON schema imports require validated normalized structure, order and limits, not unbounded `serde_json::Value` deserialization.
