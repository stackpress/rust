# Slice 6 upstream schema inventory

Companion to [decisions](slice-6-design-decisions.md). “Now” means proposed Slice 6 work after approval, never implemented here. “Later” names the responsible task/follow-up; “not ported” rejects a mechanism while preserving stated intent. Grouped symbols inherit the same disposition. TypeScript constructor ABI, overloads and class inheritance are not literal Rust API promises.

## Sources and evidence limits

Read the six requested references: [Modeling and generation](../../references/00013-upstream-modeling-and-generation.md), [schema 00021](../../references/00021-upstream-schema-api-contracts.md), [language 00022](../../references/00022-upstream-idea-language-catalog.md), [exports 00029](../../references/00029-upstream-exports-types-generated-contracts.md), [components 00031](../../references/00031-upstream-idea-component-catalog.md), [supplements 00041](../../references/00041-framework-package-supplements.md). Detailed SQL has its [own inventory](slice-6-design-inventory-sql.md).

Live read-only upstream roots: `../stackpress/packages/stackpress-schema/src` and `tests`, plus framework SQL source. The captured KB identifies Stackpress HEAD `a71d683`; the live working tree may contain later/dirty changes and is not certified by that historical hash. No upstream tests/builds were run and no source was copied into the repository. Current Rust behavior is established separately in [architecture](slice-6-design-architecture.md); stale “not implemented” context/precedent wording does not override source.

## Public schema exports and facets

| Export/type/member family | Disposition and Rust responsibility |
| --- | --- |
| `Schema.make`; ordered enums/fieldsets/models/plugins/props; `makeEnum/makeFieldset/makeModel/makePlugin/makeProp` | Now immutable checked normalization and ordered accessors. Build-time factory/registry replaces public mutation; props/imports resolved before this stage. Unknown plugin config retained privately, not emitted wholesale. |
| `Fieldset.make`, constructor, `Model.make`, constructor; schema/hasSchema, addAttribute/addColumn/attribute/column with name formats | Now owned fieldset/model descriptors and checked builder lookup; exact source identity plus explicit normalized names. Parent attachment mutations/throwing missing-parent getters are not ported. Model adds persistence semantics rather than inheritance. |
| `Column.make`, constructor; type/attributes/parent/hasParent, addAttribute/attribute | Now typed Field/TypeRef plus checked metadata; required/multiple/nullable validated. No late propagation of mutable parent/schema references. |
| `Attribute`, `args/enabled/isFlag/isMethod/value`, constructor | Now flag/method representation, immutable args and recognized-definition validation. Attribute case/grammar follows existing Idea output; no extra parser. |
| `Attributes.make/add/enabled/filter/value`; `args/disabled/flags/methods/props`; `Columns.make/add` and ordered maps | Now ordered immutable collections and focused queries; pre-freeze registration is explicit. No callable JS DataMap or arbitrary alias mutation after normalization. |
| `AttributeReference`: defined/definition/kind/isFlag/isMethod/forColumn/forSchema | Now resolved definition/applicability metadata with typed diagnostics. Unknown foreign metadata stays inert; selected-owner misspellings fail. |
| `AttributeAssertion`, `ColumnAssertion`, `FieldsetAssertion` | Now normalized rule registry, type-driven + explicit validation, bounded nested public field issues and host-only distinction. No Zod dependency or browser DB uniqueness query. |
| `AttributeComponent`, `ColumnComponent`, `FieldsetComponent` | Now all five role descriptors/aliases/participating lists and precedence contract; actual Leptos implementations/imports/wrappers are Slice 8. Missing roles omitted; unsupported selected controls are visible errors. |
| `ColumnDocument`, `FieldsetDocument` | Now descriptions/examples/labels as bounded data. Generated docs/translation catalog output is Slice 8 or docs follow-up; never execute templates. |
| `ColumnName`, `FieldsetName`: camel/dash/snake/title/underscore/string, singular/plural/display/icon/labels/event/table/type patterns | Now explicit NameSet conventions and typed overrides with collisions checked. Automatic linguistic pluralization is not ported; labels can be explicit. |
| `ColumnNumber`: chars/min/max/step | Now validated numeric/text hints and rule semantics; no hidden narrowing of SQL type from UI bounds. |
| `ColumnStore`: active/filterable/foreignRelationship/id/indexable/localRelationship/relation/searchable/sortable/spannable/timestamp/unique | Now explicit storage/query/relation metadata. UI roles do not grant access; relation mistakes fail rather than returning null/first match. |
| `ModelStore`: active/foreignRelationships/ids/indexables/localRelationships/query/restorable/searchables/sortables/timestamp/uniques | Now validated model storage view, deterministic PK/indexes, soft-delete/query policy. Invalid active/timestamp types rejected, even if a source fixture accepts them. |
| `ColumnType`: assertions/enum/fieldset/model/nullable/schema/hasSchema/name/required/multiple/get/has; `FieldsetType` | Now checked scalar/enum/embedded/model type resolution, exact optional/list mapping, rule derivation. JS capitalization/truthiness and multiple-nullability inconsistency not ported. |
| `ColumnValue`, `FieldsetValue`: defaults/generators/encrypted/hashed and value maps | Now literals, explicit provider defaults, typed storage/wire encoding. Encryption/hash transformations are S6-Security; no deterministic-IV CBC or unsalted password primitive port. Unsupported declarations fail. |
| `TypeDictionary`, `AttributeDictionary`, `AssertionDictionary`, `ComponentDictionary`; `dictionary`; define/defined/definition | Now app-owned frozen DefinitionRegistry and function registrations. No process-global import-order mutation or redefinition. Slice 8 registers concrete component definitions. |
| `defineAttributes/defineAssertions/defineComponents/defineBuiltIn`; `attributes` namespace | Now pure versioned built-in registration plus explicit extension registration. UI availability comes from Slice 8; unsupported runtime meaning remains explicit. |
| `ColumnInterface`, `DefinitionInterface`, `SchemaInterface`; assert/filter/populate/serialize/unserialize/defaults/shape | Now generated typed validators, create/patch preparation and separate storage/wire codecs/projection. No unchecked cast or generic constructor seed. Population is an explicit SQL tool operation. |
| `StackpressSchemaException` | Now standalone SchemaError with stable kinds/field paths/private causes, deliberate lib conversion at host edges. No source Display duplication per task 00010 #11. |
| `Revisions` constructor, insert/first/last/index/read/size (deep path) | Now explicit versioned immutable RevisionStore, content/parent checks and no-op detection; applied state belongs to a different DB ledger. Epoch naming/lazy mutable index is not ported. |
| `generators`, `camelize/capitalize/dasherize/lowerize/snakerize/removeUndefined`, deep `removeEmptyStrings/isJson` | Now checked naming/typed presence/canonical encoding internally where needed; no empty-string removal by default. Standalone JS helper ABI not ported. |
| `encrypt/decrypt/hash` | Not ported algorithms. S6-Security must choose audited authenticated encryption/password hashing/key rotation and dependency proofs; attributes currently refuse generation. |

## Every schema type family exported at root/deep types

| Source types | Disposition |
| --- | --- |
| `EnumConfig/ModelConfig/TypeConfig/PropConfig/PluginConfig/SchemaConfig` | Existing Idea crate owns syntax; build-host normalization consumes finalized values. Runtime/public schema does not re-export parser tooling unnecessarily. |
| `AttributeData/AttributeDataMap/AttributeDataComponent/AttributeDataAssertion/TypeAssertionMap`, `DefinitionBook` | Now typed definition, argument, public-rule and role specs; descriptions/examples preserved, executable imports remain tool-owned. |
| `AttributeDefinitionInput/Token`, `AttributeComponentInput/Token`, `AttributeAssertionInput/Token` | Now checked namespaced registration inputs and frozen definitions with owning kind/version. Component runtime provided by Slice 8. |
| `AttributeMapToken/AttributesEntriesToken/AttributesToken`, `ColumnTypeToken/ColumnToken/ColumnAssertionToken/ColumnRelationProps` | Now checked immutable schema conversion/value rules/relations; public direct inputs revalidated. No parent-pointer graph or unsafe tokens. |
| `IsArrayOrObject`, `DefinitionInterfaceMap`, `AssertInterfaceMap`, `SerializeInterfaceMap`, `UnserializeInterfaceMap`, deep `ErrorReport` | Now generated Rust input/value/patch/storage/public types plus bounded ValidationErrors; TS conditional/mapped types not ported. |
| `ClientProjectProps/ClientPluginProps`, `ClientFieldset`, `ClientConfig/ClientPlugin/Client` | Now independent app tool + static generated modules/contract revisions. Dynamic nullable ESM loader, ts-morph/Prettier/tsconfig/package fields not ported. History paths and output root are explicit tool config. |

## Language, metadata and generated output

All existing language forms and G18/G19 composition behavior are reused unchanged. `use/prop/env` finalization is the existing reference-aware path; no string-placeholder inference. Parser accepts custom capital types/open metadata, while selected semantic consumers reject unsupported meanings. Scalar/list/default/relation/role details are fully specified in [schema](slice-6-design-schema.md).

| Upstream feature | Disposition |
| --- | --- |
| String/Text/Integer/Number/Float/Boolean/Date/Datetime/Time/Json/Object/Hash, homogeneous enums, fieldsets, optional/list | Now explicit mapping and codecs. Mixed/null/float enums and custom decimal/native codecs are S6-Types follow-up, with generation refusal unless a registered implementation exists. |
| `display/icon/labels/query`; column `active/default/description/examples/generated/id/searchable/sortable/label/min/max/step/relation/timestamp/unique` | Now normalized as specified. `encrypted/hashed` are S6-Security, never inert accepted plaintext. |
| `cuid()/nano()/nanoid(length)/random()/now()` | Now recognized provider names/arguments/type contracts; now uses injected clock, ID/random requires registered implementation. A spelling is not evidence of a working generator. |
| `is.required/ne/eq/neq/option/unique/regex/date/future/past/present`, numeric/character/word comparisons, lowercase/uppercase/slug/hex/url/type rules and aliases | Now explicit checked validation/storage rules; unique remains host/DB authoritative. JS-regex parity not promised. |
| `is.email/cc/color/price` | S6-Validation extension: requires exact semantics/provider; initial unregistered use is a generation error. Do not substitute approximate validators. |
| Every field common input, collection, fieldset/relation name and alias from 00031; every view text/media/layout/link name and alias | Now role metadata/catalog interface only; Slice 8 owns actual support and selected-output diagnostics. No claim that metadata means an accessible implemented component. |
| Complete derived filter/span from field and list from view; fixed-definition prop precedence; missing role omission | Now normalize/validate/expose metadata contract; Slice 8 proves emitted Leptos behavior, SSR/hydration and accessibility. |
| Schema transforms config/enums/types/columns/schema/fieldset/model/tests; stale field/model pruning | Now inspectable owned Rust, byte-stable repeated generation/check, full rename/removal cleanup in promised partitions; generator input config remains private. |
| ts-morph shared project/index/manifest rewriting, CJS/ESM/JS/declaration exports | Not ported. Existing OutputPlan partitions + one aggregate and handwritten Cargo declarations replace those mechanisms. |
| Admin routes/screens, API/MCP tools, account/session/email/CSRF | Slice 8 for UI/admin composition; separately approved framework security/API/AI tasks for remaining runtime features. A generated data layer confers no exposure or auth implementation. |

## Source test behavior mapping

All 69 schema `it` declarations identified in the inspected five test files are accounted for below by source file/line ranges. These are disposition ranges, not executed Rust test counts. New tests must assert outcomes rather than merely that a helper class was constructed.

| Source test file / declarations | Target disposition and stronger proving behavior |
| --- | --- |
| Attribute.test.ts:12–45, 5 tests | Now args/flag/method/first value and focused metadata access. Class-instance assertions become typed invariant tests. |
| Attribute.test.ts:54–88, 3 tests | Now assertion definitions, all five component roles, applicability/kind. Rust registry has no global side effects. |
| Attribute.test.ts:107–122, 2 tests | Now ordered argument/disabled/flags/method/property views and checked additions; arbitrary post-freeze aliases not ported. |
| Column.test.ts:32–110, 10 tests | Now parent-free type/name/attribute resolution, enum/fieldset/model lookup and mapping; missing type/invalid ownership fails normalization. |
| Column.test.ts:137–148, 2 tests | Now explicit plus type-driven assertions; custom messages/aliases with actual validation results. |
| Column.test.ts:188–212, 6 tests | Now component-role metadata selection and missing-role behavior; actual components Slice 8. |
| Column.test.ts:219–320, 5 tests | Now descriptions/examples, char/min/max/step boundaries and incompatible argument rejection. |
| Column.test.ts:395–455, 5 tests | Now storage/value flags, serialized column inputs, ordered collections/name lookup; mutable parent machinery not ported. |
| Fieldset.test.ts:15–79, 7 tests | Now checked construction/schema references/attributes/columns/name views and assertion participation. |
| Fieldset.test.ts:103–179, 5 tests | Now ordered filter/field/span/list/view lists; Slice 8 proves rendered wrappers. |
| Fieldset.test.ts:200–277, 5 tests | Now docs, enum/fieldset columns, defaults. Encrypted-column discovery retained as metadata but actual generation refuses until S6-Security. |
| Model.test.ts:10–16, 2 tests | Now typed model/persistence view construction and column addition before freeze. |
| Model.test.ts:55–117, 11 tests | Now active, owning/inverse relations, IDs/indexables/restorable/query/search/sort/unique/timestamp. Reject source's String active/timestamp misuse; test valid Boolean/Datetime instead. |
| transform.test.ts:13, 1 test | Now exact renamed field/test-module + removed model cleanup with sentinel/manual-edit protection and consumer recompilation; generated Rust source is the proof. |

Generated schema column/schema test templates are production-output contracts, not additional executed source tests. Now compile and execute emitted validators/defaults/serialization cases across all supported types, optional/list/fieldset/enum forms and malformed input; unsupported crypto/validator/type cases must fail before output mutation. The current upstream schema source has no independent Revisions test file in the inspected directory; add new history/hash/branch/drift/failure tests rather than infer coverage.
