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
Source presentation arguments remain exact in `Field.attributes`; `Field.roles` is the
separate Typed/public resolved-component surface, which Source does not populate or publish.
Hash/encryption attributes likewise remain source storage metadata, not strict Typed rules.
SourceGraph cannot be passed to Typed emitters; a compiled Rustdoc negative case pins that
mode boundary. It is not a public DTO or an alternate runtime configuration switch.

The [complete graph proof](../tests/source_graph.rs) checks four models, 48 fields and every
one of 320 occurrences across all 51 attribute names against the unmodified composed AST.
Session-only callers explicitly declare `Application` and `Session` external; only Profile
and Auth become models, and no hidden tables are created. Unregistered targets fail.
No API-local wildcard/inverse/role adapter or source-line stripping is used on this path.

`Schema::normalize` retains the existing Typed behavior. `EmissionSemantics::Source` still
refuses full contract/store emission before output; production source generation and plugin
rewiring are later gates. `emit_source_witnesses(model)` emits only typed core accessors for
`<Model>Schema`, using a [structural template](../src/codegen/templates/source-field.rs.tpl)
and each winning declared type. It does not emit schemas, tables, actions or defaults.
Source enum/fieldset normalization is explicitly unsupported at this stage; neither occurs
in the shipped graph. This is a named implementation limit, not a filtered source graph.
