# G50 generator construct inventory

Status: **Design evidence, not an implementation release.** Read with [the G50 checkpoint](00031-g50-generated-framework-models.md).

Inventory uses the original API/session source bytes pinned in [00030](00030-g49-schema-idea-files.md), parsed by the existing Composer/MemoryLoader probe. The aggregate file contains only its API import. No new parser or source edits were used.

Exhaustive vocabulary: **51 distinct attributes**, **320 occurrences**, eight base type names (String, Boolean, Datetime, Hash, Profile, Auth, Application, Session), String[] lists and nullable String?/Hash?. Defaults comprise cuid(), now(), strings, true/false, empty/nonempty arrays and an empty object; no enum, fieldset, numeric, custom provider or additional source syntax occurs in these three files.

Model sizes: Application: 13 fields; Session: 11 fields; Profile: 13 fields; Auth: 11 fields. Source field/attribute order is retained. An application may introduce further supported types; those are beyond this shipped-source inventory, not silently discarded.

Anchors below name a real occurrence in the shipped source (same bytes as upstream). Status is an assessment of `framework-schema/src/codegen/{metadata,assertions,registry}.rs`, `framework-sql/src/codegen/` and the concrete `framework-admin/src/codegen/{catalog,formatted}.rs` catalog. API-local role/wildcard/inverse adapters are existing work to generalize, not new per-model exceptions.

| Construct (occurrences) | Upstream/shipped occurrence | Existing capability / required generator work |
| --- | --- | --- |
| `@active` (4) | `framework-api/schema.idea:76` | G50-SOURCE: recognized soft-removal flag; retain active=-1 and source mutation results in the source operation emitter. |
| `@default` (27) | `framework-api/schema.idea:11` | G50-DEFAULT: recognized literals/cuid()/now(); source per-access defaults, DB defaults and exact omission semantics still need source-mode emission. |
| `@description` (40) | `framework-api/schema.idea:13` | G50-DDL: documentation retained; current table template omits source descriptions/comments. |
| `@display` (4) | `framework-api/schema.idea:5` | Metadata supported; keep literal templates and use existing bounded UI rendering, not executable expressions. |
| `@encrypted` (1) | `framework-session/schema.idea:154` | G50-CODEC: explicitly rejected; generate deterministic seed-based token encryption and encrypted lookup via existing AES helper. |
| `@examples` (39) | `framework-api/schema.idea:14` | Metadata supported; retain complete literals in generated private descriptors; browser publication remains explicit. |
| `@field.datetime` (2) | `framework-api/schema.idea:85` | G50-UI: shared formatted control exists when selected; normalize all source argument values without editing the asset. |
| `@field.input` (5) | `framework-api/schema.idea:37` | G50-UI: source type prop needs the existing API-compatible definition generalized; normalize all source argument values without editing the asset. |
| `@field.metadata` (1) | `framework-session/schema.idea:68` | G50-UI: source add prop is absent from the concrete shared definition; normalize all source argument values without editing the asset. |
| `@field.password` (1) | `framework-session/schema.idea:165` | G50-UI: metadata vocabulary exists; concrete source password adapter is missing from the shared generator catalog; normalize all source argument values without editing the asset. |
| `@field.relation` (4) | `framework-api/schema.idea:18` | G50-UI: source id/search/template adapter exists only in API-local definitions; normalize all source argument values without editing the asset. |
| `@field.stringlist` (1) | `framework-session/schema.idea:50` | G50-UI: source add prop is absent from the concrete shared definition; normalize all source argument values without editing the asset. |
| `@field.switch` (1) | `framework-session/schema.idea:173` | G50-UI: shared control exists; normalize all source argument values without editing the asset. |
| `@field.taglist` (3) | `framework-api/schema.idea:68` | G50-UI: alias to field.tags is supported; shared control exists; normalize all source argument values without editing the asset. |
| `@field.url` (3) | `framework-api/schema.idea:45` | G50-UI: shared URL control exists; preserve source prop shape and accepted nullable UI behavior; normalize all source argument values without editing the asset. |
| `@filter.input` (2) | `framework-session/schema.idea:41` | G50-UI: derives field.input; same source type prop gap; normalize all source argument values without editing the asset. |
| `@filter.relation` (4) | `framework-api/schema.idea:24` | G50-UI: derives field.relation; same API-local lookup adapter gap; normalize all source argument values without editing the asset. |
| `@filter.switch` (5) | `framework-api/schema.idea:77` | G50-UI: derives field.switch; shared control exists; normalize all source argument values without editing the asset. |
| `@hashed` (1) | `framework-session/schema.idea:164` | G50-CODEC: explicitly rejected; generate source SHAKE256 storage transformation via existing helper, not Argon2. |
| `@icon` (4) | `framework-api/schema.idea:6` | Metadata supported; existing Lucide mappings stay with the UI renderer. |
| `@id` (4) | `framework-api/schema.idea:11` | G50-VALIDATE: identity metadata supported; source implicit required refinement and errors differ from strict generated input preparation. |
| `@is.cge` (1) | `framework-session/schema.idea:157` | G50-VALIDATE: recognized rule; source counts JS UTF-16 units, not Rust Unicode scalar counts. |
| `@is.required` (13) | `framework-api/schema.idea:23` | G50-VALIDATE: explicit assertions parsed; preserve first source type error/refinement order, Boolean false and source messages. |
| `@label` (44) | `framework-api/schema.idea:10` | Metadata supported; no copied hand-written field labels. |
| `@labels` (4) | `framework-api/schema.idea:4` | Metadata supported; preserve exact source singular/plural pair. |
| `@list.clip` (2) | `framework-session/schema.idea:14` | G50-UI: alias to view.overflow; concrete source length/hellip adapter missing; normalize all source argument values without editing the asset. |
| `@list.date` (11) | `framework-api/schema.idea:87` | G50-UI: derives view.date; positional source format differs from shared locale-first props; normalize all source argument values without editing the asset. |
| `@list.image` (2) | `framework-api/schema.idea:46` | G50-UI: derives view.image; source positional/object width/height adapter exists only in API-local definitions; normalize all source argument values without editing the asset. |
| `@list.link` (1) | `framework-api/schema.idea:54` | G50-UI: derives view.link; source target adapter exists only in API-local definitions; normalize all source argument values without editing the asset. |
| `@list.lowercase` (1) | `framework-session/schema.idea:42` | G50-UI: derives view.lowercase; shared formatter exists; normalize all source argument values without editing the asset. |
| `@list.string` (8) | `framework-api/schema.idea:12` | G50-UI: alias to view.text; shared formatter exists; normalize all source argument values without editing the asset. |
| `@list.taglist` (4) | `framework-api/schema.idea:69` | G50-UI: alias to view.tags; source warning/pill/className adapter missing; normalize all source argument values without editing the asset. |
| `@list.template` (4) | `framework-api/schema.idea:29` | G50-UI: derives view.template; source template adapter exists only in API-local definitions; normalize all source argument values without editing the asset. |
| `@list.yesno` (1) | `framework-session/schema.idea:176` | G50-UI: derives view.yesno; shared formatter exists; normalize all source argument values without editing the asset. |
| `@query` (3) | `framework-api/schema.idea:7` | G50-QUERY: wildcard paths fail literal path validation; general expansion must replace the API-local adapter. |
| `@relation` (4) | `framework-api/schema.idea:112` | G50-RELATION: local/foreign mapping supported; source omits inverse names that current reverse checker requires. Infer sole inverse in the generator. |
| `@searchable` (3) | `framework-api/schema.idea:36` | G50-QUERY: flag/index supported; source q wildcard/unknown-field/where behavior needs source-query emission. |
| `@secret` (1) | `framework-session/schema.idea:164` | G50-SECRET: unknown selected attribute; model UI/native publication semantics need first-class support, without an added route guard. |
| `@sortable` (11) | `framework-api/schema.idea:84` | G50-QUERY: flag/index supported; source sort/skip/take behavior differs from strict public paging. |
| `@span.datetime` (2) | `framework-session/schema.idea:84` | G50-UI: derives field.datetime; shared formatted control exists; normalize all source argument values without editing the asset. |
| `@timestamp` (4) | `framework-api/schema.idea:102` | G50-DEFAULT: recognized now provider; source update/created return and native Date representation need source-mode emission. |
| `@unique` (1) | `framework-session/schema.idea:154` | G50-UNIQUE: index recognized; source Auth.token uniqueness uses stored codec and original lookup/error/return semantics. |
| `@view.date` (11) | `framework-api/schema.idea:88` | G50-UI: source positional format adapter exists only in API-local definitions; preserve G46 date behavior; normalize all source argument values without editing the asset. |
| `@view.image` (2) | `framework-api/schema.idea:46` | G50-UI: source positional/object width/height adapter exists only in API-local definitions; normalize all source argument values without editing the asset. |
| `@view.link` (1) | `framework-api/schema.idea:55` | G50-UI: source target adapter exists only in API-local definitions; normalize all source argument values without editing the asset. |
| `@view.lowercase` (1) | `framework-session/schema.idea:43` | G50-UI: shared formatter exists; normalize all source argument values without editing the asset. |
| `@view.metadata` (1) | `framework-session/schema.idea:69` | G50-UI: source className adapter is not in the concrete shared catalog; normalize all source argument values without editing the asset. |
| `@view.string` (10) | `framework-api/schema.idea:12` | G50-UI: alias to view.text; shared formatter exists; normalize all source argument values without editing the asset. |
| `@view.taglist` (4) | `framework-api/schema.idea:70` | G50-UI: alias to view.tags; source warning/pill/className adapter missing; normalize all source argument values without editing the asset. |
| `@view.template` (4) | `framework-api/schema.idea:30` | G50-UI: source template adapter exists only in API-local definitions; normalize all source argument values without editing the asset. |
| `@view.yesno` (5) | `framework-api/schema.idea:78` | G50-UI: shared formatter exists; normalize all source argument values without editing the asset. |

## Compound/runtime gaps beyond attribute recognition

| ID | Missing capability / source behavior and proof requirement |
| --- | --- |
| G50-SOURCE | Strict typed DTO preparation is not generated source Column preparation: String(object), arrays, truthiness, JSON text, missing/null/empty and native Date coercion; implicit required constraints and exact first-error messages. Source: `stackpress-schema/src/transform/column/{shape,assert,index}.ts`, shipped column classes. Move generic mechanics to an appropriate shared owner and emit all model-specific shapes. Reuse the existing original-Node column/default/error vectors. |
| G50-CODEC | Preserve preparation vs storage phases; token equality/encrypted uniqueness, native secret hash readback and seed defaults; emit codecs, never hardcode token/secret field names. Existing native schema helpers already implement SHAKE256/CBC/CUID. Source: `stackpress-schema/src/column/ColumnValue.ts:18–58`, `transform/column/index.ts` and session AuthStore/AuthActions. |
| G50-QUERY | Source selector wildcards, query defaults, joins, JSON paths/membership, positive hasnt, unknown paths, q/like, active=-1, skipped invalid sort and source take/skip differ from bounded generated public Query/OperationPolicy. Generalize the current source statement/query implementation, retaining G46 dialect spelling; do not weaken the separate strict mode. Source: generated Profile/Auth stores and SQL transform/store/select.ts. |
| G50-RELATION | Full graph has four forward mappings and four reverse collections. Derive inverse names, retain source list required flags, distinguish metadata from eager store joins, avoid recursive expansion. G50 decides that session-only output retains declared, unresolved external Application/Session links while generating only Profile/Auth, with no hidden API tables. Add and prove this external-reference capability; API/aggregate output generates all four. |
| G50-WRITE | Current generated writes require RETURNING; source supports MySQL via generated IDs/lookup and original partial fallback. Preserve mutation of caller input, empty-string/undefined removal, source duplicate preflight, create-return, update/delete result arrays, soft remove/restore and ordered batch rollback/results. Source: SQL transform/actions/{create,update,batch,upsert}.ts and current live-tested source Actions. |
| G50-EVENT | Existing generated strict handlers require a Policy, strict public input and an Engine; source handlers skip missing Engine/prior response and use source error/row envelopes without a new global policy. Generate source handlers as an explicit compiled semantics selection; G42 duplicates/priority/Abort stay. Source: SQL transform/events/{index,search,create}.ts. |
| G50-DDL | Emit source literal/date defaults, comments, index/constraint identity and FK actions. Current table template omits defaults/comments and uses Restrict instead of source FK delete CASCADE. Retain accepted Rust width/dialect contracts and G48 setup selector; installation is explicit and MySQL partial progress stays visible. Source: SQL transform/helpers.ts:62–160 and AuthStore.ts:230–246; current session/API store create plans. |
| G50-DEFAULT | Full descriptors/record/store/action/native registry generation must derive from composed metadata. Current emitters assume crate::generated paths and provide strict stores/DTOs, not the source Schema/Column/Actions/client API. Emit a reusable library root/exports and source role bindings, plus freshness/source-hash evidence. Preserve CUID/clock provider injection and no registration-time IO. |
| G50-UI | Reuse existing Leptos controls/renderers through source-compatible definitions. Preserve every positional/object prop, alias, default query and native/browser split; full metadata is generated, not copied from the former 78-omission inventory. Retain source G46 date spellings, escaped templates and Lucide catalog. |
| G50-SECRET | Recognize source @secret without adding authentication policy or suppressing source-native hash results. Keep native seeds/providers out of public metadata/SSR snapshots while API Application/Session secrets (without @secret) retain source output behavior. Source: Auth secret column, signin removes secret explicitly; API source secrets are plain CUIDs. |

Raw per-occurrence values remain in the disposable parsed-source evidence (`/private/tmp/g49/api-composed.json`). The shipped files themselves remain authoritative for every value and field order; no omission inventory or filtered source file is proposed.
