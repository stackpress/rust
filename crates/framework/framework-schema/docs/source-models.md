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

## Extended source schemas

`Schema::normalize`, `emit_contracts` and Typed emitted bytes keep their existing behavior.
The compatibility selector refuses a Typed Schema with Source semantics; pass a
`SourceGraph` to `emit_source_contracts`. Source normalization and emission support
Integer/Number/Float, every scalar/list family, enums, embedded fieldsets, terminal relation
selectors and model names that collide with generator infrastructure. Generated catalog
and module names are escaped independently from SQL identifiers: a field `type` remains
SQL `type`. [Extension tests](../tests/source_extensions.rs),
[emission tests](../tests/source_emission.rs) and the
[compiled extended application](../../framework-models/tests/extensions.rs) pin these cases.

Every original assertion dictionary name/alias and authored message is retained. G51
explicitly makes broken or omitted rules work as named, including value eq/neq, regex,
word count, type checks, price/JCB card regexes and zero-valued numeric bounds. Character
cgt/clt are exclusive; exactly-N strings now fail. Character counts use UTF-16 units and
word counts retain literal-space splitting. Lists validate items in order. Database
uniqueness belongs to actions, so schema assertions alone cannot prove it.
The [dictionary and boundary matrix](../tests/source_extensions.rs) and captured
[original dictionary](../src/codegen/source-assertions.json) pin names/messages and the G51
exception. Unknown assertion names and invalid arguments fail normalization.

Defaults include `nanoid()`, `nanoid(n)`, `cuid()` and `cuid(n)` through the existing explicit
provider mechanism. Ring supplies randomness; deterministic original byte/hash vectors
pin both algorithms. Source CUID init accepts lengths zero/one (returning two/one characters)
and rejects lengths above 32; the public checked `CuidGenerator::with_length` remains 2–32.
No provider runs at normalization or registration. `number_step` shares the original numeric
step inference with native SQL DDL family selection. Source number spelling, non-finite
values and ECMAScript whitespace conversion are tested separately from Typed coercion.

Original conversion order remains observable: fieldset schema encoding wraps the column's
JSON again, fieldset lists assert the fieldset rather than each item, nullable scalars
bypass assertions, and source literal defaults pass through unserialization. Date offsets
and clocks stay explicit. These behaviors have original Node oracles in
[source extension tests](../tests/source_extensions/oracles.rs); they are not new DTO rules.

[SQL source stores/actions/events](../../framework-sql/docs/api/source-models.md) consume
these exact descriptors and the immutable catalog. The default package emits all four
models, while session-only source generation explicitly declares Application/Session links
external and emits no hidden API models. The portable SourceUiCatalog selects authored roles without codecs/provider arguments;
[admin source adapters](../../framework-admin/docs/source-models.md) render it.
Session/API generic registrars bind the application family at compilation. Output keeps the existing finite writer budget.
