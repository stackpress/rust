# Task 00049 — remove premade framework models (G59)

Status: implemented; final gate green, including live replay. Coordinator verification/merge pending. Base `8d74980`, branch `trial/3-remove-framework-models`. Authority: [G59](../decisions.md). Coordinator reviews/merges; no commit, push, install or development listener.

## Removed public items

Deleted the whole `crates/framework/framework-models` crate, generated output, dedicated tests and `framework-models-generate` binary. Removed the workspace member and every session/API normal dependency/feature edge. There was no root facade models feature/re-export.

| Owner | Removed public items |
| --- | --- |
| Deleted crate | Entire `stackpress_framework_models` namespace: root `generated`; `ProfileSchema`, `AuthSchema`, `ApplicationSchema`, `SessionSchema`; `ProfileModel`, `AuthModel`, `ApplicationModel`, `SessionModel`; each model's `Store` and `Actions`; `Models`. Generated `source`/`stores` modules; source `profile`, `auth`, `application`, `session`, `catalog`; each model module's `describe`, `columns`, schema type and inherent `with_providers`, `columns`, `column`; catalog `describe`, `models`, `EXTERNAL_MODELS`; stores `profile`, `auth`, `application`, `session`, `family`, each model's Store/Actions/Model/`register`, and `family::Models`. |
| Session | `auth::register` (implementation `auth::plugin::register`), `composite::register`; `auth::events::{signup,signin,search,update}`; `account::events::{register,register_listen,auth_create,auth_search,auth_update,auth_remove,profile_detail,profile_update,profile_remove}`; `profile::ProfileSchema`, `auth::AuthSchema`, `profile::{ProfileActions,ProfileStore}`, `auth::{AuthActions,AuthStore}`; storage re-exports `AuthActions`, `AuthModel`, `AuthStore`, `ProfileActions`, `ProfileModel`, `ProfileStore`, `DefaultModels`. |
| API | `models::{ApplicationSchema,SessionSchema,ApplicationModel,SessionModel,ApplicationActions,ApplicationStore,SessionActions,SessionStore}`; `sql::{ApplicationActions,ApplicationStore,SessionActions,SessionStore,register}` (implementation `sql::events::register`); `views::admin::register`. |

Generic paths remain. New `auth::events::{search_with,update_with}` retain the documented direct extension-event flows over the app's family. No new default family. Shipped Idea sources/`SCHEMA_SOURCE` remain the generation contract.

## Fixture layout, regeneration and consumers

Session/API/email/aggregate each own `tests/fixtures/models/`, Idea inputs and `generation.rs`. Session owns extended and session-only external-link families; API owns extended models and management presentation. Freshness tests call the same public PackageLoader, source normalization, contract/store emitters and managed writer in read-only check mode. Fixtures do not compile another crate's tests. The unchanged schema extension oracle JSON moved into schema tests. Session owns a byte-identical copy of the provider Node vectors; both active and historical live helpers use crate-local fixture paths.

Run examples `regenerate_session_models`, `regenerate_api_models` (`codegen` for presentation), `regenerate_email_models`, `regenerate_aggregate_models` (`full,codegen`). Append `-- --check` for freshness. Independent examples can repair missing output before integration-test compilation; existing workspace generator dependencies stay dev-only. Unnecessary direct parser dev edges were removed.

Session account_host owns `examples/account_models`; its plugin consumer owns `src/models` and `regenerate.py`. Aggregate consumers own `generated-models`, emitted by their build-host. Existing source-client/API checkpoint/Typed workspace/admin/blog emitters and freshness gates remain. Documentation applications generate their own models before compiling exact fences. Blog runtime code is unchanged; two stale generator comments were corrected.

Root lock changes are workspace package/edge edits, including approved generator dev edges. All **699** registry identities/versions/sources/checksums remain unchanged. Affected nested locks had their single root-seeded offline refresh with immediate subset/root-byte assertions. Later headless direct-import edges were edited locally without another resolution. Blog retains exactly its six G58 extras (448 registry entries).

## Behavior notes

| Change | Authority/evidence |
| --- | --- |
| Aggregate `register()` without an application-generated session/API binding now fails with a descriptive error when those stages are compiled in (previously used stock models). | G59/coordinator ruling; `selected_session_and_api_require_explicit_generated_bindings` checks both errors and replacement paths. Admin/account keep optional callbacks. |
| Absent-client tests and headless consumer select explicit constituents with their generated family. | Coordinator-approved setup migration; nullable-client, source-order and no-IO assertions unchanged. |
| Blog lock tooling compares all 699 registry tuples with HEAD and freezes root bytes from gate entry through every exit. | Coordinator-approved contributor-tool fix; workspace-edge mutation passes and registry-checksum mutation fails unit controls. Exact G58 check retained. |

G42/G43, G46 search, G53/G54 membership, signup/signin/account/admin, nickname extensions, Date/redirect output and session-only external links retain their assertions. Original/Node goldens and `.stderr` bytes are unchanged. Exclusive-executor compile-fail setup imports local models on the existing first line, preserving spans.

## Rounds and evidence

| Round | Findings / proof |
| --- | --- |
| Complexity | Required generated bindings fail at existing source stages. Registration/order/nullable-client/auth tests: 16 passed (`20261110T100257.253264Z-1207d47a02c2`). |
| Responsibility | Local fixture/model/presentation ownership, dev-only generation; unnecessary parser edges removed. 11 freshness/family/registration tests passed (`20261110T100441.142396Z-a36547d4149e`). |
| Consistency/Rustdoc/comments | Generic paths, current claims/fences, feature-scoped imports, unique example names and explanatory comments. Targeted doctests/Clippy passed (`20261110T100506.355312Z-4b90fd42cabb`, `20261110T100637.891394Z-5c91605f4158`). |
| Safety/soundness | No new unsafe code, runtime shared mutation, hidden IO or concurrency policy. Existing ownership/compile-fail assertions pass unchanged; managed writer owns generation. |

Session/API baseline: **166 passed / 11 skipped**, including compile-fail snapshots (`20261110T095354.619168Z-510fcd9f8ab5`). Failed migration/format/feature-import attempts remain in the recorder. Blog lock unit controls: 2 passed (`20261110T101107.918241Z-4d9ddbf210fc`); actual lock check passed (`20261110T101108.488425Z-21804d70d5e9`).

## Full gate

Rust 1.96.0, native macOS arm64; renderer checks use wasm32-unknown-unknown separately. The default and coordinator-compatible workspace sets are retained verbatim in the core record's argv/log.

| Proof | Result / recorder evidence |
| --- | --- |
| Workspace core | fmt; Clippy both sets `-D warnings`; nextest default **1062 passed / 88 skipped**, compatible **1227 passed / 96 skipped**; both doctest sets; Rustdoc `-D warnings` passed (`20261110T101009.009974Z-34a6bd463569`). |
| Generation / boundaries / consumers | Typed workspace/admin and API checkpoint freshness, session/aggregate graphs, all **8 aggregate + 4 session consumers**, build-host `--check` passed (completed phases in `20261110T102903.019324Z-5f76d9b75ea7`). |
| Exact documentation | **29 fences**, **22 finite executions**, claim/link/negative controls passed (`20261110T110622.725074Z-a1bd3791bb72`). Earlier continuation found one stale storage-alias import; it was migrated to that app's generated family, assertions unchanged. |
| Browser matrices | Cached approved Playwright, in-process routing: API **52**, session **704 + 4 no-JS**, aggregate smoke/source **24** cases passed across separate hydration/CSR builds (`20261110T104805.596579Z-6a46900ad9a6`, `20261110T110108.452759Z-97550b3a7d09`). |
| Final invariants | All **699 exact registry tuples**, every nested-lock subset plus exact G58 extras, unchanged `.stderr`, byte-identical relocated oracles, deleted crate and reference search passed again after live replay (`20261110T114627.144422Z-67b8ea796f63`). Final fmt passed (`20261110T111053.892440Z-86cc827a7c20`). |
| Blog | Full `python3 tools/examples/blog/gate.py` passed: 90 normal + 90 instrumented tests, native/browser lint/docs, public/authenticated/nickname matrices, all six native coverage floors (70.64–97.44%), G56 client exemption, lock controls and invariants (`20261110T110844.005885Z-f37bddaafe5c`). Runtime code unchanged apart from its lock and two stale comments. |
| Live MySQL/PostgreSQL | **9 passed / 0 skipped**, including session/API G46 search, G53 `has`, G54 `hasnt`/NULL/missing-path and aggregate setup/migration/purge. PostgreSQL **12.2**, MySQL **8.0.27**; six exact owned CREATE/DROP pairs confirmed, final listings both empty (`20261110T114304.560976Z-ef54e3661444`). Earlier unavailable-server preflights are superseded. Owner services were never started by this lane. |

Each following database was created by this replay and its exact name was confirmed dropped; both initial and final `stackpress_*` listings were `[]` on PostgreSQL and MySQL.

| Fixture owner | Backend | Confirmed CREATE and DROP name |
| --- | --- | --- |
| session | mysql | `stackpress_g46_mysql_14c36_18e6ac59f3d78d10` |
| session | postgres | `stackpress_g46_postgres_14c38_18e6ac5a049d0210` |
| api | mysql | `stackpress_g46_mysql_14cab_18e6ac5bb1aa51a0` |
| api | postgres | `stackpress_g46_postgres_14cad_18e6ac5bbf07b888` |
| aggregate | mysql | `stackpress_g46_mysql_14d7d_18e6ac5fcdc31878` |
| aggregate | postgres | `stackpress_g46_postgres_14d7f_18e6ac5fd9cde7b0` |

Native executable-line coverage includes each measured crate's `src` under full session/API SSR, source/provider/codegen/server and SQLite-maintenance selections. No production-source exclusions. Fixture/compiler output is not shipped implementation; its behavior/freshness is checked separately. **373 tests passed / 13 skipped** (`20261110T103555.121366Z-894919f03abd`).

| Crate | Covered / executable | Percent |
| --- | --- | --- |
| schema | 6203 / 6959 | 89.14% |
| SQL | 6520 / 8678 | 75.13% |
| session | 3694 / 3957 | 93.35% |
| API | 2229 / 2448 | 91.05% |
| email | 2792 / 3114 | 89.66% |
| aggregate | 1190 / 1245 | 95.58% |

LLVM warned about 121 mismatched functions. A `--dump` audit located every one in external quick_xml/tachys/tokio/foldhash/memchr/tracing; none is measured production code. Independent summary export matched every measured file (`20261110T104755.224429Z-204bdbbcaeef`). Blog export also warned about 92 external functions (including bytes); its independent measured-file summaries match (`20261110T114122.388648Z-4c8b7087d90b`). Completed instrumented/consumer targets were retired; emitted browser products/reports and warm root dependency archives remain. Own targets stay below the approximately 24 GiB budget; blog has its separate 12 GiB gate cap.

KB raw validator retains **146 inherited errors / 44 warnings**, no Task 00049 errors (`20261110T114607.608648Z-0b0513cd4cbe`). This is not a clean-KB claim. Publication plan/index now list **23 candidates**; historical receipts 00035/00036/00038 only gained G59 supersession notes.

Recorder: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`; sccache, jobs=2, debug=0, incremental=0, task-owned targets. Command wall time and observed Cargo-stage time remain separate; captured nested compiler output can make the latter incomplete.

Gaps: no remaining task proof gaps. Coordinator verification and merge remain pending; this receipt does not claim coordinator acceptance. Failed tooling/migration attempts remain in the benchmark ledger; no original expectations or snapshots changed.

## Coordinator verification

Committed as `5f42faf` and merged into main as `50584f3`. Reviewed:
- the crate directory is gone;
- the Node extension oracle moved byte-identically to framework-schema tests;
- the root lock lost only workspace edges, with 699 registry tuples identical;
- no `.stderr`/golden change;
- the in-crate fixture families and their `--check` regenerators;
- the required session/API bindings and their error tests.

Independent gate on a detached clean checkout of `50584f3` (`/private/tmp/g59-verify/run.sh`, 2026-11-10), exit 0:
- fmt; Clippy, default and compatible;
- nextest: default 1,061 passed; compatible 1,226 passed; `tcp_watchdog` alone;
- doctests both sets; Rustdoc with `-D warnings`;
- `regenerate_session_models --check` and `regenerate_api_models --check`;
- aggregate graphs, eight consumers and build-host `--check`; session graphs and consumers;
- generated-workspace/admin and API checkpoint freshness;
- docs and proof ledger;
- live PostgreSQL 12.2 / MySQL 8.0.27 session/API/aggregate `live_search`/`live_setup` (9 passed): six owned databases dropped, none left;
- the blog contributor gate: 27/27 phases.
