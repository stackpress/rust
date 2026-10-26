# Slice 6 architecture, evidence and shared requests

Companion to [decisions](slice-6-design-decisions.md). Load for actual-code findings, crate/dependency placement, disposable evidence and coordinator sequencing. New APIs below are proposals; only the explicitly listed source/probe facts were verified at this checkpoint.

## Crates and feature boundaries

| Package/module | Responsibility and edges |
| --- | --- |
| `crates/framework/framework-schema`, `stackpress-framework-schema` | Default: owned descriptors, value/validation/role contracts and errors, with Serde and narrowly justified pure codecs. Optional `codegen`: Idea parser/transformer integration, normalization adapter and Rust emission. No server, SQL drivers, CLI or filesystem in browser/default graph. |
| `crates/framework/framework-sql`, `stackpress-framework-sql` | Default native library: schema + independent SQL, generated-store runtime support, explicit DataSession, strict codecs and typed errors. Optional `server`: current lib/server named handler helpers; optional `codegen`: Idea transformer and pure SQL emitters/planner. No backend selected by default. |
| framework-sql `migrations` / `sqlite-maintenance` | `migrations` adds checked history IO/hash/planner/operator runtime; SQLite feature adds the reviewed adapter-maintenance API. Keep filesystem/driver dependencies outside pure contracts and default schema. No universal DB migration claim. |
| Independent `examples/generated-workspace/tool` | Compiled schema/SQL function transforms, generation/revision/planning handlers. Optional `database-tools` selects SQLite maintenance/pool and app-owned config/seed steps. Depends on source contract/selection configuration, never final generated app. |
| `examples/generated-workspace/app` | Generated contracts/stores/selected operations plus explicit database/default-provider/policy plugin. Current native CLI/buffered HTTP test entry; no listener required. Installation/seed fixtures explicit and separate from startup. |
| Browser-safe contracts consumer | Imports only generated contracts/approved public descriptor with schema's pure validation; no framework-sql/server/driver/tool edges. Actual Slice 8 wasm build proves this graph after interface freeze. |
| Root facade | Coordinator adds opt-in `framework-schema`, `framework-sql` features and explicit `stackpress::{framework_schema,framework_sql}` exports. No default feature/dependency expansion. |

No extra generator/runtime crate is mandated now: feature-gated modules and separate tool/client packages give a real acyclic graph. Cargo features are additive; actual target dependency trees, not feature names, prove isolation. New packages contain real implementations only; no framework umbrella placeholder is needed.

The schema and SQL transforms register ordinary functions using current Registry/TransformSpec/OutputPlan. Schema owns contracts/metadata; SQL requires schema and owns stores/operations. The aggregate owns module roots. A `register.rs` module inside operations owns selected registration helpers; the transformer-generated root `mod.rs` remains module declarations only. Slice 8 chooses different partitions for UI; it cannot edit schema/SQL files.

Keep G20 selection/runtime contract checks and `emit generate --check` before build. Each generated partition records compatible schema/generator/runtime contract constants. CI compares normalized fingerprint and selected feature descriptor; compiling a required removed type correctly fails. No compatibility fallback to stale generated files is supplied. OutputPlan currently caps 1 MiB/1,024 files; benchmark the realistic generated example first, then request a configurable bounded budget only if evidence requires it. Do not inflate/split roots to conceal the limit.

## Proposed dependency budget

No production dependencies are added at this checkpoint. Coordinator reviews exact feature edges/MSRV/licenses before M0's serialized lock update. Reuse existing compatible locked versions; do not fetch “latest” or install tools implicitly.

| Scope | Proposed direct dependencies/features and reason |
| --- | --- |
| Pure schema/contracts | Existing serde derive + serde_json; ordered Vec/explicit IDs, no global maps required. Existing `regex` for bounded Rust regex rules, `url` for URL rule, and `time` parsing/formatting for canonical date/time; enable only needed pure features, no system clock in public validation. Current lock has regex 1.13.1, url 2.5.8, time 0.3.55. |
| Schema/SQL codegen | Existing Idea path crates and schema runtime; ordinary Rust emitters/string literal escaping, no template/eval/syn/proc-macro dependency required initially. Same formatter/toolchain policy as existing output workspace. |
| SQL runtime/server | Existing stackpress-sql + schema; optional lib/server only for selected host integration. No direct SQLx/native handles in generic store APIs. |
| Migration IO | Existing sha2 0.10.9 for deterministic fingerprints and tempfile for staging plus std filesystem. Framework-owned snapshot codec avoids SQL-core Serde dependency. Trusted history root/exclusive cooperative writer contract is explicit. |
| SQLite maintenance | Existing stackpress-sql-sqlite and its SQLx `=0.9.0`/Tokio feature budget. New direct SQLx use belongs inside that adapter, not framework generator/store/client. |
| Tool/app/tests | Existing CLI/server/Tokio/Serde; tempfile, trybuild and existing coverage tooling as used. Named default providers avoid pulling random/crypto libraries into browser validation. |

Native `time`/URL/regex support and wasm compatibility must be proved in the actual consumer graph; being present transitively in Cargo.lock is not approval of a new normal edge. No encryption/password/CUID/Nano-ID library, ORM, reflection, network backend, toolchain or install is approved by this proposal. A default provider's algorithm is an explicit app choice, not a hidden new framework dependency.

## Current-code findings

Read current Idea parser config/finalization and transformer registry/context/output/plan APIs, independent SQL builders/dialects/schema/diff/Engine/Transaction and SQLite session/lease/options, server service/delegation/registration, and generated-profile/sql-profile/profile-ui source. These facts determine the shared request scope:

- `SchemaConfig` contains open ordered attributes and type flags; it is not normalized framework semantics. Reference-aware wrapper generation resolves true reference sites and preserves literal lookalikes. Reuse it rather than reparsing or guessing strings.
- `ContextTransform` is a function pointer. Immutable inputs/contracts and disjoint ownership are available; arbitrary typed inter-transform payload storage is not. Shared normalization can remain a pure function. TransformError keeps causes privately but has no public arbitrary-source constructor, so framework callbacks need an additive conversion seam to preserve SchemaError/DataError correctly.
- `TableDefinition` includes ordered fields/primary/indexes/FKs; `SchemaDiff` is pure and rejects a different table name/physical reorder. It does not inspect live state, reconstruct SQLite tables or provide an applied ledger. No Serde snapshot representation is required in core.
- `QueryPlan` is builder-created, with no public arbitrary-batch constructor. `queries()` plus `Transaction::execute` can execute validated component statements after full framework-plan preflight; do not invent a raw batch escape. Current execute_plan deliberately refuses MySQL nontransactional DDL.
- Engine is non-Clone/Send+Sync; Transaction is non-Clone/Send, not Sync, with exclusive methods and consuming completion. It does not expose a public dialect getter. No general executor trait is required to add DataSession inside framework-sql.
- Ordinary raw SQL admits only checked statement families and rejects PRAGMA/BEGIN/COMMIT. SQLite leases retire uncertain transactions. Reconstruction must preserve those gates and use adapter-owned connection policy.
- Current Insert has values/returning but no native conflict/upsert clause; broad generated upsert needs an additive core design/proof. A check-then-create shortcut is unsound under concurrent writers.
- Existing ServerBuilder services/config, plugin dependencies, named handlers and Invocation::delegate are sufficient. `Response::rows` accepts rows and total. Current examples' profile-search total is returned rows; the new generated search's optional matched total is a distinct documented operation contract.
- profile-ui already distinguishes in-memory/SQLite backend source from client contracts and uses explicit setup. Preserve those proofs; new generated-workspace is separate. No existing generated file is hand-edited.

## Disposable reconstruction probe

Created only `/private/tmp/stackpress-s6-probe/`, with an independent new Cargo workspace/target/lock. Dependencies: paths to current stackpress-sql and Idea parser, SQLx `=0.9.0` with defaults off + `runtime-tokio,sqlite`, and Tokio macros/rt. No repository Cargo build/clean, production source/manifest edit, network DB, listener, install or other trial branch was used.

Commands run:

```sh
cargo run --manifest-path /private/tmp/stackpress-s6-probe/Cargo.toml --offline
cargo run --manifest-path /private/tmp/stackpress-s6-probe/Cargo.toml --offline --locked
rustc -Vv
cargo -V
```

First SQLx-only fresh compile completed successfully in 15.32 seconds; adding the current parser for the exact walkthrough took 7.36 seconds. Locked re-runs used Python subprocess.run with a 120-second watchdog and saved stdout/stderr to `/private/tmp/stackpress-s6-probe/probe.log`. Host: rustc 1.96.0 (`ac68faa20`), Cargo 1.96.0, `aarch64-apple-darwin`; SQLx 0.9.0, libsqlite3-sys 0.37.0, SQLite **3.51.3**. This is a small in-memory driver mechanism probe, not the file-backed production proof.

| Check | Observed result |
| --- | --- |
| Exact walkthrough Idea through current parser finalization | Three models, one fieldset, one enum, two plugin IDs in the intended order. This checks syntax/finalization only, not proposed framework semantics. |
| Current ordinary SQL renders `PRAGMA foreign_keys = OFF` | Correctly rejected before execution. |
| Disable foreign keys after BEGIN | Readback remained 1; confirms maintenance must change policy before BEGIN. |
| Create/copy/drop/rename parent with inbound ON DELETE CASCADE child, enforcement disabled before EXCLUSIVE transaction | Committed renamed field `Ada`, child count remained 1, foreign_key_check empty. |
| Copy into new required column with no backfill | Copy failed; explicit rollback retained original parent value and child count. |

All five checks passed on final locked re-runs; no ignored cases. No coverage was measured. Not proved: proposed maintenance/session API, file/WAL durability, indexes/triggers/views/generated columns/sequence preservation, concurrency, cancellation/commit acknowledgement loss, migration ledger, generated code or cross-platform support. These are M0/milestone requirements, not conclusions borrowed from the probe.

## Cross-slice contracts for coordinator handoff

**Slice 7:** generated stores target current Database/TransactionSession, SqlValue/Row/QueryResult and the three dialects. Require strict existing scalar/JSON codecs, authoritative statement-local insert IDs/RETURNING, Send/exclusive affinity, finite acquire/busy limits, typed private driver error categories and cancellation/unknown-commit semantics. Supply it the generated three-model CRUD/transaction contract test harness once M0 types freeze. It supplies production PostgreSQL/MySQL adapters; this slice does not duplicate them. Any isolation options or core conflict-clause/dialect accessor changes are jointly sequenced before lanes compile. Native server hosting/streaming is unnecessary for Slice 6's proof.

**Slice 8:** supply the versioned UiSchema/UiModel/UiField/RoleBinding contract in [schema](slice-6-design-schema.md), safe validation/key/patch/query/public DTOs, operation IDs and finite relation lookups. Schema/SQL runtime and generated operations remain Slice 6-owned; Slice 8 owns concrete component catalog/import resolution/Leptos wrappers/admin screens and their visual acceptance. Agree descriptor/version/partition names through the coordinator, not cross-agent edits. UI controls can start with fixture data; real schema-to-UI integration waits for M1/M2. No browser acquires framework-sql or native DB access.

## G23 and other shared-file requests

All requests are pending coordinator sequencing. A lane must stop at these files rather than edit them independently.

| Shared owner | Requested change and justification | Proof before acceptance |
| --- | --- | --- |
| lib | **None.** Existing errors/Send/request/response/Invocation suffice. | Preserve current lifecycle/error/disclosure regression suites. |
| server including HTTP | **None.** Existing registration, typed services and delegation suffice. No implicit transaction propagation or automatic routes. | Generated handlers and startup prerequisites compile; direct/CLI/buffered HTTP equality and G12 Abort/error tests. |
| CLI | **None.** Independent app tool reuses existing emit grammar/main_with; maintenance shutdown composed in app entry as needed. | Tool builds with app/output absent; finite awaited cleanup. |
| Idea transformer | Add a checked public contextual-cause constructor for TransformError, retaining typed arbitrary Send+Sync source and explicit kind. Current public IO/Idea conversions alone misclassify framework callback errors. | Callback→SchemaError downcast/source chain and one explanation per Display/source boundary; default disclosure remains safe. No parser/composer/output semantics change. |
| Idea parser/wrapper | **None expected.** Reuse current typed reference-aware composition/finalization and pure config adapter. | Literal placeholder/reference/import/default fixtures remain intact. Any new source-span/context need returns as a separate request. |
| SQL core (shared with Slice 7) | Add read-only Transaction::dialect() and typed Insert conflict/upsert clauses, with explicit per-dialect supported semantics. No raw connection getter, broad batch constructor, general ORM or weakened MySQL execute_plan gate. | No unsafe/aliasing change; compile Send/exclusive session and hostile-bind upsert, concurrent uniqueness/correct ID results in SQLite; network execution gated with Slice 7. |
| SQLite adapter | Add explicit maintenance owner/session with typed catalog/FK/sequence inspection and policy transitions before BEGIN; same-lease validated query execution/cleanup. | M0 real file rebuild + ledger/rollback/retirement/cancellation/competing process proofs. Normal ordinary-query gate unchanged. |
| Root Cargo.toml/lock/src/lib.rs | Actual framework crates/example members, opt-in facade edges/features and reviewed dependencies. | Coordinator sole integrator, --locked after each update; pure contracts/default facade tree excludes tool/server/driver. |
| Existing examples and other slice design files | **None.** New example/unique receipts only. | Scope audit; no hand edits of prior generated output. |

The adapter request is substantial enough to be the first M0 gate, not hidden as a migration helper. If the coordinator declines it, the sound fallback is pure migration planning plus native supported ALTER only, with SQLite reconstruction explicitly incomplete; that would require an owner scope change to task 00011.
