# Slice 6 normalized schema and shared contracts

Proposed semantics behind [the owner choices](slice-6-design-decisions.md). Load before implementing `framework-schema` or consuming it from Slice 8. This layer interprets compiled Idea; it does not replace the existing parser, reference-aware composer or finalizer.

## Immutable normalization

`Schema::normalize(&SchemaConfig, &DefinitionRegistry, NormalizeOptions) -> Result<Schema, SchemaError>` consumes only already composed/finalized config. The build-host adapter rejects remaining imports, unresolved typed reference sites and props before normalization. Literal `${...}` strings remain literal. Direct caller-created config is revalidated for depth, bytes, declarations, attributes, duplicate names and type flags; parser validation alone is insufficient.

Normalize in passes: register declared enum/fieldset/model identities; resolve types and names; interpret recognized attributes; validate storage/key/relation semantics; derive validation and role descriptors; freeze ordered immutable values. Model relations may cycle; embedded fieldset value cycles are rejected with the complete path. Cloning descriptors copies owned data; relations use IDs/indices, not mutable parent pointers or Arc cycles.

`Schema`, `Model`, `Fieldset`, `Field`, `Attribute`, `TypeRef`, `Relation`, `StorageModel`, `ValidationRule`, `ValueRule`, `NameSet`, `Documentation` and `RoleBinding` replace class/facet inheritance. Accessors expose focused views without duplicating facts. No process-global dictionary, interior-mutability registry, import side effects or last-registration-wins semantics.

`DefinitionRegistry` is app-owned during tool construction, then frozen. Ordinary functions register types/assertions/roles and stable version IDs. Duplicate definitions and incompatible revisions error. Unknown foreign namespaces are preserved/inert; unknown names inside the selected owners' `is.*`, storage attributes and registered role families are errors when that consumer runs, so a misspelled validator never silently disappears. UI bindings that Slice 8 has not implemented can be retained as unavailable descriptors, with a generation error if selected for actual UI output.

## Value and storage matrix

| Idea declaration | Rust/wire value | Initial storage and limits |
| --- | --- | --- |
| String, Text | String | SQL text, unchanged UTF-8; size checked before storage/response. No truthiness/coercion. |
| Integer | i64 | SQL integer; reject fractional/out-of-range JSON and SQL values. Browser wire adapter must reject integers it cannot represent exactly. |
| Number, Float | finite-number wrapper serialized as a JSON number | SQL real; reject NaN/infinity and lossy implicit integer conversions. Decimal/money requires a later explicit codec. |
| Boolean | bool | SQL Boolean/strict adapter 0 or 1; no truthy strings. |
| Date, Time, Datetime | validated canonical string newtypes | Text: calendar date, local time without timezone, or UTC instant respectively. No machine timezone or epoch fallback. Do not silently use driver-native date codecs. |
| Json, Object, Hash | bounded JSON object wrapper | SQL JSON value/text according to renderer/adapter. Nested JSON can contain arrays/scalars/null; top-level remains object. SQL NULL differs from JSON null. |
| homogeneous string/integer/Boolean enum | generated enum with checked scalar Serde codec | Underlying scalar storage; duplicate scalar values rejected. Null/float/mixed-type enums require the later custom-type extension. |
| fieldset | generated owned nested struct | One bounded JSON column; validation recurses with field paths. Recursive embedded cycles rejected. |
| non-model T[] | Vec<T> | Bounded JSON array with validated elements; never CSV flattening. |
| T?, T[]? | Option<T>, Option<Vec<T>> | SQL NULL distinct from empty value/list. Required list accepts empty unless an assertion says otherwise. |
| model T / T? / T[] | relation descriptor and explicit relation result | No implicit JSON embedding or hidden column. Requires a valid owning FK/reverse mapping. |

Canonical date proposal: `YYYY-MM-DD`; time `HH:MM:SS` with optional normalized fractional seconds; datetime UTC RFC3339 `...Z`, normalized to a single precision policy by checked `time` parsing. M0 freezes precision/range and invalid-date examples. Date/time comparison rules compare typed values, never ambient local time. `future/past/present` receive an explicit clock and documented precision.

Create inputs represent missing, supplied and explicit null separately at the wire boundary. Patch inputs use `Patch<T>::Unchanged/Set(T)/Null`, with Null legal only for nullable fields. Generated input validation rejects unknown fields, write attempts to generated/relation fields and empty updates. Optional Serde fields alone must not collapse missing and explicit null. Typed direct inputs and transport decoding enforce the same business rules.

## Attributes and validation

| Metadata | Proposed meaning |
| --- | --- |
| `@id`, `@unique` | One or more ordered nonnullable scalar IDs; generated Key struct for compound identity. Unique is enforced in DB, not only by a pre-check. At least one ID required for generated model mutations. |
| `@generated`, `@default`, `@timestamp` | Explicit value rule. Literal defaults preserve false/zero/empty/null. Generated field requires a known provider or supported DB identity strategy. Timestamp is one Datetime field refreshed by writes, from the same injected clock. |
| `@active` | At most one required Boolean, default true; opt-in soft removal. Invalid String-active fixtures are deliberately rejected. |
| `@relation` | Owning singular relation declares local and foreign scalar fields plus optional inverse name. Referenced tuple must be primary/unique and type/nullability compatible. Initial attribute syntax supports one key pair; compound FK relation syntax is a later extension. |
| `@searchable`, `@sortable` | Explicit query participation. Supported scalar index hints become deterministic indexes; JSON/list indexing is not silently fabricated. |
| `@min/@max/@step` | Typed constraints/hints, validated for compatible types. They do not select an integer width or shrink data on their own. Storage length changes require explicit plan inspection. |
| `@description/@examples/@label`, model `@labels/@icon/@display/@query` | Owned descriptive/query metadata. Display templates remain data for Slice 8; no evaluation. Query paths resolve against known fields/relations and explicit projection policy. |

Built-in assertions now: type/presence, `required/ne`, scalar `eq/neq/option`, numeric `gt/ge/lt/le`, character `ceq/cgt/cge/clt/cle`, word `weq/wgt/wge/wlt/wle`, case/slug/hex, bounded Rust-regex, date/future/past/present and `url`. Normalize aliases `notempty/pattern/gte/lte/cgte/clte/wgte/wlte` once. Type-driven assertions compose with explicit rules. Character count is Unicode scalar count; word count is Unicode whitespace splitting, not locale segmentation. Empty-list presence rules are explicit.

`is.unique` becomes a storage uniqueness requirement plus a safe conflict result; preflight lookup can improve UX but cannot prove concurrency safety. `email`, credit-card (`cc`), `color`, and `price` need a separately registered validator in initial Slice 6; absent registration is a generation error. Their source spellings are retained in the inventory rather than approximated with an arbitrary regex. Password hashing/encryption are also deferred pending security design; `@hashed/@encrypted` cause generation refusal, never plaintext fallback. These baseline limits require the owner's scope choice.

Default strings are interpreted only by a finite recognized generator table: `now()` uses an injected clock; `cuid()`, `nano()`, `nanoid(length)` and `random()` require registered typed providers with declared output type/revision. No expression evaluation or silent synonym substitution. The proof uses supplied IDs and literal defaults, and a deterministic clock/provider fixture tests generation without random emitted bytes. DB identity is supported first on SQLite with authoritative RETURNING; network identity behavior depends on Slice 7's statement-local result contract.

Validation returns bounded `ValidationErrors { issues: Vec<FieldIssue> }`, each with logical field path, stable code and public message. No rejected value or SQL text is included. Cross-field/app-specific checks are ordinary explicitly called functions; remote uniqueness/auth cannot run in browser-only validation. Stored rows are decoded/type-validated without rerunning time-relative creation constraints; a row that became old is not corrupt merely because `is.future` would reject a new input today.

## Naming and serialization

Source names remain exact in descriptors/wire codecs. Rust uses conventional snake_case members and UpperCamelCase types, with explicit Serde renames. SQL uses deterministic snake_case table/column names and bounded named constraints. M0 checks keyword handling, ASCII-case collisions, `__` alias collisions, portable identifier lengths and long-name digest suffixes. Explicit typed naming overrides are tool config, not invented Idea attributes. No silent pluralization library is required.

Generated `validate_create`, `validate_patch`, `prepare_create`, `encode_storage`, `decode_storage` and `project` have distinct responsibilities. No automatic whole-row public serialization. Browser-safe contracts/validation carry only approved public descriptors. Full plugin config/env values, storage secrets, credentials and arbitrary props never become a public generated `config` dump. A schema fingerprint is a compatibility signal, not a secret or authorization token.

## Proposed interface supplied to Slice 8

Freeze a versioned descriptor before either lane generates real UI:

```rust
pub struct UiSchema {
    pub contract_revision: u32,
    pub schema_fingerprint: String,
    pub models: Vec<UiModel>,
}
pub struct UiField {
    pub id: FieldId,
    pub wire_name: String,
    pub label: String,
    pub value_type: PublicType,
    pub required: bool,
    pub input_required: bool,
    pub readable: bool,
    pub writable: bool,
    pub sensitive: bool,
    pub nullable: bool,
    pub default: PublicDefault,
    pub validation: Vec<PublicRule>,
    pub roles: Vec<RoleBinding>,
}
pub struct RoleBinding {
    pub role: Role, // Field, Filter, List, Span, View
    pub component: ComponentId,
    pub props: BoundedProps,
    pub definition_revision: String,
}
```

`UiModel` adds ordered fields, key shape, safe labels, relation target/cardinality/key mapping, and query/operation IDs. Only app-approved projected fields appear. Relations identify a fixed app-supplied lookup capability, never infer a URL or grant database access. `PublicRule` marks client-checkable versus host-only rules; messages/phrases are explicit stable data. No executable import path, Rust type object, SQL expression or server service crosses this interface.

Access flags come from an explicit app-owned field policy: readable/writable derive read-only/write-only states; sensitive input values are masked and never echoed/projected by default. A UI role such as password does not establish host access policy. `PublicDefault` is None, HostGenerated or an explicitly approved safe literal; no private provider/secret default is serialized. `input_required` accounts for default/generated rules, while `required/nullable` describes the value type. These are data contracts for Slice 8, not a new identity/auth implementation.

The build-host descriptor also takes an optional `DiagnosticMap` keyed by stable model/field/attribute paths. The app tool constructs it from current Composer sources/provenance and existing parser AST byte spans, then passes it as a declared auxiliary snapshot to both transform wrappers. No second language parser is created. Imported winning metadata must retain its original logical source/UTF-8 range; normalized JSON without original spans reports file/structural path and never invents coordinates. It stays tool-only and is excluded from public UiSchema. This integration is an M0 proof, not a currently available TransformContext capability.

Schema owns normalized roles, stable IDs, alias normalization, argument validation and ordered selection; Slice 8 owns Leptos component definitions/availability, Rust imports, actual generated wrappers, forms, accessibility and admin workflows. Its function registration supplies the complete catalog to the same tool before schema normalization. All field aliases and derived filter/span/list families in reference 00031 normalize to canonical bindings. Missing role is omitted. Multiple canonical bindings for one field/role are rejected rather than choosing arbitrarily. Fixed definition props override Idea props, matching upstream; unsupported props/erroring types are reported.

The same immutable Schema can feed both transforms; current TransformContext cannot carry typed inter-plugin objects. Initially the app supplies a named registry-construction function to its compiled transform wrappers; each wrapper normalizes deterministically from the same config plus declared catalog inputs and calls pure emitters. This preserves function plugins and custom compiled definitions without mutable shared context or a parser change. M0 proves both transforms' normalized fingerprints match. A future prepared-schema auxiliary snapshot is an optimization only after its checked codec is proved.

Slice 8 can build controls against a hand-authored descriptor fixture now. Real schema-to-UI output must wait for the descriptor, generated operation contracts, field projection, validation and relation tests from Slice 6; UI fixture success does not establish data-layer integration.
