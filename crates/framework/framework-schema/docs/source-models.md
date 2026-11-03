# Native source mechanics and normalization

Enable `source` on framework-schema for `source::{Value,Record,Date,Column,Kind}`,
Date conversion, source coercion, `SourceCodecs` and `default_value`. These native-only
mechanics have no server, Engine or codegen dependency. `source` selects the existing
`providers` algorithms and insertion-ordered `stackpress-lib::DataMap`.

`Value::Date` retains milliseconds and an optional host display string. JSON decoding never
guesses Dates from strings or object keys; JSON encoding emits UTC ISO text. A missing map
member differs from explicit `Value::Null`, including nested insertion order. The
[native record test](../tests/source_graph.rs) pins these distinctions. The session crate's
old public columns/codecs paths remain available and use these same types/mechanics.

`NativeField` converts exact field families without coercion. `SourceField<Name>` supplies
the actual declared Rust type; missing/wrongly shaped unprepared fields return None.
Nullable accessors distinguish a missing key from a present null (`Some(None)`). `Date` is
the typed native Datetime witness; lists and objects remain owned native values.
`ProfileFields`, `AuthFields`, `ApplicationFields`, `SessionFields` require every core field
and the shared Value/Record. `SourceDeclaration<Name>` additionally checks exact declaration
kinds, nullable/list wrappers and relation target identity; shared payloads do not erase those
differences. [Compiler cases](../../framework-sql/tests/source-ui) pin wrong
core declarations; [generated positive probes](../../framework-sql/tests/source-pass) pin
default and application-extended families. Field witnesses add no request validation policy.

`default_value(field, now_millis, providers)` resolves metadata rather than assuming a
framework field name: now() produces a native Date; other calls use explicit ValueProviders;
literal defaults retain false/null/empty. Existing `providers::{cuid,encrypt,decrypt,hash}`
retain their source formats. No provider runs at normalization or registration.

## Build-host selection

With `codegen`, `Schema::normalize_source(config, options, external_models)` normalizes the
composed source graph directly, returning `codegen::SourceGraph` rather than the Typed
emitter's Schema. It retains all private attribute names, arguments, order,
documentation, exact queries/wildcards, source defaults and forward/reverse declarations.
Source presentation arguments remain exact in `Field.attributes`; `Field.roles` now retains
source-compatible alias/argument bindings from the metadata-only `source_definitions()`
catalog. Fixed source URL/password types override supplied props. No renderer is selected
(`available = false`), and no browser descriptor is automatically published. Typed roles
continue to use their separate app-owned catalog.
Hash/encryption attributes likewise remain source storage metadata, not strict Typed rules.
SourceGraph cannot be passed to Typed emitters; a compiled Rustdoc negative case pins that
mode boundary. It is not a public DTO or an alternate runtime configuration switch.

The [complete graph proof](../tests/source_graph.rs) checks four models, 48 fields and every
one of 320 occurrences across all 51 attribute names against the unmodified composed AST.
Session-only callers explicitly declare `Application` and `Session` external; only Profile
and Auth become models, and no hidden tables are created. Unregistered targets fail.
No API-local wildcard/inverse/role adapter or source-line stripping is used on this path.

## Production source emission

`codegen::emit_source_contracts(&SourceGraph, &mut OutputPlan)` emits native schemas,
metadata-driven storage columns and core declaration/field witnesses into `source/`.
The caller registers a transform owning that partition. `SourceGraph::catalog()` derives
owning/reverse mappings and scalar query paths while retaining exact source attributes,
including wildcard selectors. This metadata does not execute queries or generate tables.
The [emission tests](../tests/source_emission.rs) cover every shipped construct, all 107
explicit role bindings, an added Profile field, external references and pre-output refusal.

`source::SourceColumn` combines a field's complete metadata with the existing codec family.
Its defaults derive from the declared value rule/literal rather than the legacy Column
getter's framework field-name switch. `SourceRecordSchema` implements common ordered record
operations; the generated schema wraps it and implements `SourceSchema`. Native seed/offset
state is private and is not inserted into descriptors. The generated metadata decoder uses
`source::__private::serde_json`; application crates need no direct serialization dependency.

The [default-model package](../../framework-models/README.md) is generated through this public
entry point, with an official finite `--check` command. [Its native oracle suite](../../framework-models/tests/native.rs)
proves original Node column/default/crypto behavior and all four core-field families. Extra
app fields use the same generator. Session-only output retains declared external links.

`Schema::normalize`, `emit_contracts` and Typed emitted bytes retain their existing behavior.
The compatibility selector `emit_contracts_with_semantics(&Schema, ..., Source)` refuses a
Typed Schema input: use `emit_source_contracts` with a SourceGraph. SQL stores/actions/events,
DDL and concrete UI renderers/client binding remain the separately released M2/M3 work.
Source codecs cover the shipped String/Boolean/Hash/Datetime/String[] inventory plus native
Date/Time scalars. Literal defaults pass through the source getter's unserialization step;
for example, a Boolean literal string `"false"` is truthy, and a Datetime literal zero becomes
a native Date. Provider defaults retain their provider result. The extended independent
consumer and [native tests](../../framework-models/tests/native.rs) pin these distinctions.

The emitter refuses numeric scalar codecs, lists other than String[], assertions beyond
required/cge, and explicit assertions on list elements. Parameterized `cuid(n)` and
`nanoid()`/`nanoid(n)` defaults normalize but are not emitted yet. Terminal relation query
selectors (rather than scalar paths or wildcards) and a model whose module is `catalog`
also fail before any output is added. None occurs in the shipped four-model graph. The
[refusal matrix](../tests/source_emission.rs) pins these boundaries; no construct is
silently filtered. Enum and fieldset definitions are refused earlier, during source
normalization. Output remains subject to the existing finite writer budget.
