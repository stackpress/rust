# G50 generated framework models — design checkpoint

Status: **Design accepted; M0 awaits a separate coordinator release.** Authority: [G50](../decisions.md), initially read after the authorized fast-forward to **2abc9f1** with uncommitted prerequisite work retained; the owner answers were read from **5c3df92**. G50 replaces the G49 API three-model projection, six descriptor corrections and 78-omission inventory; none is implemented or retained as a test fixture. [00030](00030-g49-schema-idea-files.md) records the shipped-source/import prerequisites and their verification.

The objective is one composed application schema driving the models actually used by session/API, including app-added columns. A generated lookalike unused by the runtime would not meet it. No generated Profile/Auth/Application/Session code, generator/runtime API change or default-model crate has been implemented for G50 at this stop.

## Original runtime and extension path

Upstream read-only commit: **a71d683051ba8350fdd12d6b5a33f268fdcc285f**. Asset hashes and byte equality are in 00030. Paths below are relative to its `packages/` directory.

| Source anchor | Observed behavior / Rust consequence |
| --- | --- |
| `stackpress-schema/src/plugin.ts:15–32` | Config priority 10 registers `client`; `client.module` defaults to stackpress-client. Required lookup propagates import failure; nullable lookup catches it and returns null. Rust keeps explicit compiled selection, not dynamic module import. |
| `stackpress-sql/src/plugin.ts:26–45` | Inside Listen, add SQL command handlers, await `ctx.plugin<ClientPlugin>('client')(true)`, and call every generated model's `listen(ctx)` in catalog order when present. No client means no generated CRUD listeners. |
| `stackpress-sql/src/transform/model.ts:34–113` | A generated model exports Schema, Store, Actions, columns, events and listen. Runtime uses the app-generated module, not merely its metadata. |
| `stackpress-sql/src/transform/events/index.ts:18–28,105–135`; `events/search.ts:76–105` | Generated CRUD listeners use named model events; source response/body and missing-Engine skips happen before actions. A generated search instantiates its generated Actions using database.seed, performs findAll/count and returns rows/total. G42 allows additional handlers on those names. |
| `stackpress-session/src/session/pages/account.ts:25–40`; `stackpress-api/src/plugin.ts:115,151`; `pages/oauth.ts:43–74`; `pages/token.ts:27` | Session account pages and API endpoints reach generated model actions through profile/auth/application/session events. G50 must bind these names to the application's generated catalog. |
| `stackpress-session/src/auth/events/{signup,signin}.ts:8–19`; `auth/AuthActions.ts:34–64` | Important qualification: custom signup/signin directly import the package AuthActions, whose make(ctx) reads the database plugin and whose constructor creates package AuthStore/ProfileActions. These custom methods do **not** themselves obtain the client's factories. G50 deliberately redirects their model collaborators to the app-generated ones while retaining handwritten account behavior. Do not misreport the original as exclusively client-backed. |
| `stackpress-session/src/auth/AuthActions.ts:428–453` | Stock signup forwards only name/type/roles into profile creation, then explicit authentication fields. Adding a column to a model does not automatically add an input to this handwritten form/process. See Q50-SIGNUP below. |
| [Existing G19 merge](../../../../crates/idea/idea-transformer/src/lib.rs), [full merge table](../../../../crates/idea/idea-transformer/tests/compose_rules.rs) | Normal mutable local declarations keep their attributes and whole colliding fields; missing imported fields prepend in child order. An immutable `!` local is replaced by the imported child. Import order and local declaration positions stay observable. No new merge semantics are proposed. |

An application can already compose this input with the released package catalog:

```idea
use "stackpress/stackpress.idea"
model Profile {
  nickname String?
}
```

`use "stackpress-api/schema.idea"` includes the session file; `use "stackpress-session/schema.idea"` composes Profile/Auth directly. The aggregate filename is `stackpress/stackpress.idea`, with no schema.idea alias. The [actual prerequisite tests](../../../../crates/framework/framework/tests/package_sources.rs) prove a nested app adds nickname while retaining original fields/metadata and that `Profile!` does not preserve the addition. A local colliding column replaces the whole column, including its attributes; generated plugin role compatibility must be proved against that resulting schema, never repaired by appending hidden fields.

## Replace generated-model responsibilities, retain plugin responsibilities

| Current Rust owner | G50 disposition |
| --- | --- |
| `framework-session/src/columns/schema.rs`, `columns/model.rs`; `profile/mod.rs`, model/schema parts of `auth/mod.rs` | Replace fixed field lists, literal/default dispatch, metadata and per-model wrappers with generated source-schema/column descriptors and methods. All composed metadata is generated; no known-omission allowlist. Legacy public names may re-export generated defaults where signatures can be preserved. |
| `columns/{value,dates}.rs`, generic coercion/assertion code in `columns/mod.rs`, schema provider helpers | Reuse generic native source mechanics at a shared lower owner, proposed `framework-schema::source` under a native source feature. Generate every model/field selection and codec call. Keep rich Date/missing/null and ordered Record semantics; moving reusable mechanics is not a handwritten model exception. Re-export old paths during migration where possible. |
| `framework-session/src/storage/{store,actions,batch,query,statement}.rs` | Generate model-specific stores, CRUD/actions, source-query catalogs, tables/indexes/relations, batch and setup methods. Extract reusable source SQL mechanics to `framework-sql::source`; emitter templates supply the complete composed model data. No fixed Profile/Auth field switches remain. |
| `storage/account.rs` | Split generic CRUD/model ownership from handwritten signup/signin orchestration, password policy and identifier/email helpers. The latter stay handwritten and take generated Auth/Profile collaborators. Password hashing calls remain explicit source business logic; generated @hashed storage calls use the existing source helper too, not a new password format. |
| `framework-session/src/auth/{events,pages,plugin,bridge,hooks,totp,config,templates}`, `account/`, `jwt.rs`, `session.rs`, permissions/views/presentation | Keep plugin registration, JWTs, permissions, request flags, redirects/errors, OTP/magic-link/2FA, password/signin policy, hooks/supervision and page/UI orchestration. Replace concrete default-model construction/field filtering at their model boundary. Do not generate auth policy from UI metadata. |
| `framework-api/src/models/{schema,store,actions,batch,query,membership}.rs`, `sql/{adapter,defaults,events}.rs` | Replace model-specific schemas/columns/store/actions/query and default event inventories with generated Application/Session and shared Profile implementations. Reuse lower source mechanics; preserve plaintext API secret behavior, Session's ignored q and source join defaults. Keep handwritten transport/endpoint semantics. |
| `framework-api/src/codegen.rs`, `codegen/source_compat.rs`, source role adapters | Remove synthetic external Profile rendering/import stripping. Generalize wildcard/inverse and source-component support into generators/shared UI definitions, then normalize the complete imported four-model graph. No three-model projection or fixture is carried forward. |
| Aggregate CompiledClient/CompiledModel/catalog and app-generated admin | Keep ordinary registration, source nine-stage order and app ownership. Extend the compiled runtime binding seam as below. Generated model callbacks still enter SQL Listen; admin stays last. No route ownership, global protection or runtime plugin loading. |

Existing [source model/actions](../../../../crates/framework/framework-session/src/storage/actions.rs), [custom account actions](../../../../crates/framework/framework-session/src/storage/account.rs), [API model schema](../../../../crates/framework/framework-api/src/models/schema.rs) and [generated SQL templates](../../../../crates/framework/framework-sql/src/codegen/templates/) are inventories to migrate, not interchangeable APIs today.

## Compile-time model binding proposal

Recommend **generated implementations of a shared source-model API**, selected by ordinary monomorphized registration functions. Keep the current strict typed generator semantics as the default for existing consumers. Add an explicit compiled source-semantics emission selection for the framework catalog; it generates the source CRUD/event contracts rather than changing all existing strict handlers. No runtime config selects executable models.

The native shared source API belongs in framework-schema/framework-sql, below session/API. It covers schema/default/filter/assert/serialize/unserialize, pure table/query builders and CRUD/batch/setup execution with an explicitly supplied Engine/DataSession. The required operations are new/seed/offset, defaults-at, filter/populate/assert, storage codecs, find/find-all/count, create/update/delete/remove/restore/upsert/batch, install/upgrade/purge/uninstall and named event registration. Generate source return shapes and error conversion, not conversions through strict public DTOs.

Illustrative proposed surface, **not compiled or frozen signatures**:

```rust
pub trait SourceModel: Send + Sync + 'static {
    type Schema: SourceSchema;
    type Store: SourceStore;
    type Actions: SourceActions;
    fn describe() -> Model;
    fn schema(seed: &str, offset_minutes: i32) -> Self::Schema;
    fn actions(engine: Arc<Engine>, seed: String) -> Self::Actions;
}
pub trait SessionModels: Send + Sync + 'static {
    type Profile: SourceModel;
    type Auth: SourceModel;
}
pub trait ApiModels: Send + Sync + 'static {
    type Profile: SourceModel;
    type Application: SourceModel;
    type Session: SourceModel;
}
// Existing ordinary default registrar remains; custom functions are compiled wrappers.
pub fn register_with<M: SessionModels>(app: &mut ServerBuilder) -> Result<()>;
```

`SourceActions` exposes its source methods with borrowed query/input/DataSession arguments and explicitly Send futures (associated future/RPITIT where practical; a named boxed lifetime-bound future only where needed). Native record bridging preserves owned extension fields, Date values and insertion order. A generated implementation supplies all field metadata; it does not delegate back to fixed default Profile/Auth descriptors. Plugins read only the required core role fields through the contract, retaining unknown extension fields in the generated record/schema. Wrong required core types or a non-Send implementation fail a compile proof, not a new request-time global gate.

Session's handwritten account wrapper becomes generic over generated Auth/Profile actions; its default alias preserves current direct imports/constructors where possible. API's binding uses the **same generated Profile type** as the selected session family. A combined app-generated family supplies both traits; prove associated Profile equality rather than allowing separate field catalogs. Routes/events are monomorphized for that family, so there is no mutable global factory, opaque runtime model import or reflection-based hardcoded fallback.

### Default generated models and dependency direction

Recommend one new **`stackpress-framework-models`** workspace package containing reproducible generated default output for the closed four-model aggregate schema. It depends on lower schema/SQL source runtime mechanics, never on session/API plugins. This avoids a cycle caused by Profile's Auth/Application/Session relations and the current API→session edge. Session/API define small default family bindings to those generated types; such type associations are plugin wiring, not copied schemas.

Generate checked-in output with a finite workspace build tool using the same public generator as applications. Keep source asset ownership in session/API/aggregate; no build-time network or source-tree discovery at application runtime. No heavy generator/build dependency leaks into default or browser runtime graphs. Use the established generated module root consistently, replacing today's fixed `crate::generated` assumptions through an explicit library-root emission option where needed. `--check` must reject stale output after source/definition/generator changes; never hand-edit the generated files.

An app with extensions generates its own family from composed source and selects its generated registrars. An app without extensions uses the generated defaults through the existing `register` path. Default facade remains stackpress-lib only; native model dependencies stay feature/target gated. Under Q50-SESSION-ONLY, session-only output generates Profile/Auth and retains Application/Session reverse links as declared, unresolved external references; no hidden API tables or invented stubs. API/aggregate output generates all four.

### Aggregate catalog reconciliation

Today [CompiledClient](../../../../crates/framework/framework/src/native/mod.rs) contains model listen/admin callbacks and describe, while [Client](../../../../crates/framework/framework/src/native/client.rs) publishes it at Config priority 10. That catalog does not yet select session/API's concrete model family.

Propose an explicit `FrameworkBindings` field on CompiledClient containing optional ordinary `Register` callbacks for the generated session/API family. `register_with` selects those callbacks at the existing session/API stages; SQL still invokes the **same** family's model.listen callbacks before those stages, and admin uses its matching generated catalog last. No change to priority/FIFO/Abort or phase policy. The binding function specializes `session::composite::register_with::<AppModels>` / `api::register_with::<AppModels>`; default register selects generated defaults. There is exactly one CRUD listener inventory, with no duplicate default-plus-custom registration.

Adding a public CompiledClient field breaks existing struct literals in this unpublished workspace. Treat it as a coordinator-owned coordinated API migration, not silently additive. Inventory/update every literal, consumer, source receipt example and generated aggregate emitter in one release. A builder/constructor can reduce future literal churn if the real call sites justify it. Absence/config/Engine modes must retain existing source skips and perform no startup IO.

Selecting a generated default collaborator does not manufacture a missing client or register default CRUD listeners behind the app's back. Keep standalone data registrars explicit, and preserve the no-client SQL Listen behavior. Generated model listeners and the plugin's custom auth processes have separate inventories as they do upstream.

## Generator work and full construct coverage

The [complete construct ledger](00031-g50-generator-inventory.md) enumerates **all 51 distinct attributes / 320 occurrences**, eight base type names, lists/nullability/default families and model sizes from the three shipped inputs. Read it when releasing generator work. It distinguishes already recognized metadata from concrete unsupported source semantics and gives an upstream/shipped anchor for every construct; no construct is deferred as a handwritten exception.

The implementation work is not limited to accepting @hashed/@encrypted: it includes source Column preparation/validation, default/clock/Date behavior; secret metadata; inverse/reverse relations and query wildcards; source query/return/batch behavior; MySQL writes without mandatory RETURNING; source setup/defaults/FK actions; generated source event registration; complete role/prop definitions and concrete UI adapters. Existing lower codec functions are reusable mechanisms. Source templates must use the existing bounded TemplateEngine and literal encoding; do not introduce generic source-wrapper templates or arbitrary evaluation.

Particular incompatibilities to prove before deleting handwritten stores:

- Existing strict generated handlers require OperationPolicy, strict public envelopes and an Engine. Source handlers skip missing Engine and enforce their own configured policy; G43 forbids adopting the former as an added framework gate.
- Existing generated writes require RETURNING. Source actions support MySQL's original generated-ID/lookup/partial-return behavior. Do not make live MySQL tests pass by changing adapters or requiring unavailable capabilities.
- Current generated table templates omit source DB defaults/comments and use delete Restrict for foreign keys, while native source stores use default values and delete Cascade. Preserve accepted Rust explicit-width/dialect contracts and generate the required source setup shape; do not silently migrate to a new table layout/index identity.
- Session alone names API reverse types. Full aggregate composition is closed; prove the owner-decided external-reference capability for session-only normalization without generating API tables.

## Behavior, tests, migration and locks

G42/G43 and resolved G44–G48 remain binding. No new default-deny, request-data rejection, account enumeration change, revocable session design, expiry/identifier normalization, guard, secret redaction policy or route arbitration. Source @hashed means existing SHAKE256; @encrypted means the existing seed-derived AES-CBC format. Native password checks and explicit signin secret removal remain handwritten source behavior.

The default generated implementation must pass the existing source oracles **before** replacing the old model implementation. Do not regenerate expected behavioral fixtures from the new implementation. G50 metadata now comes from source rather than the former 78 omissions; exact differences in generated metadata/fingerprints must be named, with source anchors. Existing native behavior differences from generation require coordinator/owner review, not automatic fixture updates.

| Area | Migration / required evidence |
| --- | --- |
| Source schemas/columns/codecs | Reuse original Node golden defaults, asserted errors, dates, string/list/object coercion, token encryption/hash/CUID vectors; test missing/null/empty/Boolean false and implicit required-ID behavior. Add app columns through source, not native RecordSchema mutation. |
| G46/G48 live stores | Port current opt-in session/API live search suites to actual generated Actions/DDL; run PostgreSQL/MySQL and finite SQLite q/like, Auth→Profile and API joins. Keep local identity gates, unique CREATE/DROP fixture ownership, deadlines and every create/drop receipt. Preserve MySQL non-atomic setup/progress, FK child-first DELETE purge, PostgreSQL CASCADE and SQLite DELETE. Ordinary transactions stay atomic and MySQL DDL refusal unchanged. |
| Auth/plugin behavior | Invocation traces, flags, missing auth/session/access/Engine/client modes, direct event reuse, source no-check cases, sequential effects and supervised hooks remain. Signup/signin wrappers call the generated model's full schema; no silent removal of extension fields at CRUD boundaries. |
| UI/admin/export | Regenerate default admin/client from the same full graph; compile roles/heads/forms/loaders/SSR/hydrate/CSR. Repeat session/API browser matrices, no-JS, keyboard/responsive/dark cases and public/SSR graph secret checks. CSV and nested JSON insertion order remain pinned. Handwritten account forms retain their source process inputs. |
| Aggregate and custom app | Stock default and extended app consumers must prove stored/read/updated extra fields through real registered handlers and custom auth collaborators, API joins, admin and export. Model/listener/admin catalogs all derive from the same schema. Verify SQL Listen order, nine source stages, any-handler Abort and no new guard. |
| Database transition | Generation itself opens no DB and runs no install/upgrade. Compare generated default TableDefinitions with the current accepted plans. Apply app extension DDL only through explicit reviewed migrations/setup; adding a Rust field must not silently recreate/purge a DB. Preserve MySQL partial-state reporting and SQLite runner scope (no new MySQL migration ledger/runner). |
| Public paths / graphs / locks | Prefer generated default aliases at existing session/API schema/store paths; list unavoidable type/signature changes. New source runtime owners/default-model package and CompiledClient migration need exact grants. Only workspace edges/new workspace entries are proposed; 699 external registry tuples remain identical. Consumer locks use one authorized offline resolution from root, immediate subset/root-byte assertions, then locked/offline. No dependency install or license change. |

## Staged release and ownership plan

This plan is proposed; **no stage is released by writing it**. Split around a generator lane and a runtime/consumer lane, with a coordinator owning shared manifests, facade/catalog contracts and integration.

| Stage | Generator owner / files | Runtime/consumer owner / files | Stop proof |
| --- | --- | --- | --- |
| M0 | `framework-schema`/`framework-sql` source contract and minimal structural emitter probe; explicit typed/source semantics selection | Disposable app/default-family compile probes; aggregate catalog proposal only | Real owned/borrowed Action signatures, SourceModel/SessionModels/ApiModels/Profile equality, Send futures, no lock across await; compile-fail wrong core type, duplicate family, non-Send, borrowed executor escaping; close full/source-only relation question. Stop before broad migration. |
| M1 | Normalize full source without adapters/stripping/omissions; source coercion/default/codec and relation/query metadata; all source UI definitions | Generated default package/tool skeleton using public APIs, no handwritten replacement models | Full four-model/default and extended inputs normalize; every ledger row has a test; checked-in regeneration/freshness and independent compilation; first source-native golden parity. |
| M2 | Source schema/store/action/event templates, DDL/setup and source query/return/batch compatibility; existing strict generation remains byte-stable | Default and custom runtime family tests via real SQLite and the opt-in native harness | All original column/actions/query/error oracles plus live G46/G48 paths pass from generated code; failures/partial state reported verbatim. Stop before replacing plugin paths. |
| M3 | General source UI catalog/admin/client emission and model-family/registrar outputs | Replace session/API fixed schemas/stores with generated defaults/custom families; retain generic auth/account logic; coordinator integrates CompiledClient callbacks/facade/consumers | Same appended column reaches CRUD, signup/signin collaborator, API joins, admin and export; default behavior traces unchanged; browser/wasm graphs and source order pass. |
| M4 | Generator docs/freshness/check tooling and final gap ledger | Remove superseded adapters/manual model lists; migrate external docs/examples/locks | Full compatible workspace + target + browser + consumer gates; ≥70% per changed implementation crate; three refactoring rounds plus async/ownership safety, then coordinator review. |

Every row with a shared-code need gets an exact-file/coordinator grant before edits. No broad root-lib/server changes are assumed. Existing tiny-template-engine byte-parity tests and app-owned generated-workspace migrations stay regression gates.

Deadlock-class proofs use the existing external parent process watchdog, with an independent readiness marker, deadline, exact owned-child kill/reap and forced-kill negative control. Cover nested events, borrowed transaction/session affinity, suspended/cancelled generated actions, hook shutdown and config reentry. An in-executor timeout is not sufficient. Finite fixtures only; no listener, watcher, network at registration or TTY. Live DB execution is opt-in and explicitly authorized per release.

## Owner answers under G50

1. **Q50-SESSION-ONLY — decided: keep the links external.** Session-only apps generate Profile/Auth and retain Profile's Application/Session reverse links as declared, unresolved references. Add and prove that generator capability; no hidden API tables, erased fields or invented model stubs. API/aggregate apps generate all four.
2. **Q50-SIGNUP — decided: keep the original signup inputs.** Stock signup passes only name/type/roles. App-added Profile columns appear everywhere else, including CRUD, admin, API, export and auth collaborators. Apps needing extra signup inputs provide their own form/event/hook; generation does not automatically forward them through stock signup.

No owner question remains at this checkpoint. The coordinator accepted stages M0–M4, the shared `stackpress-framework-models` default package and CompiledClient bindings field in principle; exact file/API grants come with the stage that needs them. No new question about the ten resolved behaviors, license metadata, SQLite dialect or MySQL migration runner is raised.

## Checkpoint evidence and stop

Prerequisite real types/tests already prove PackageLoader, shipped import order, G19 mutable/immutable extension and actual generate-provider wiring; 00030 contains the command records/closing results. The construct ledger was generated by a recorded finite proof from the existing Composer snapshot and rechecked source hashes. Its **51 attributes / 320 occurrences** match the shipped input vocabulary; model sizes are Application 13, Session 11, Profile 13 and Auth 11 (48 fields, 40 stored and eight relations).

The proposed G50 trait/default crate/emitter changes have not been compiled or implemented. G49 prerequisites must be verified and merged first. M0 is not released; stop after the G49 receipt, with no commit.
