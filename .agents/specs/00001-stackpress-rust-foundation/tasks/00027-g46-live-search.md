# G46 live source-search receipt

Status: scoped tests, three review rounds and verification complete; **PostgreSQL passed; MySQL search remains blocked by the existing install contract**. Baseline: clean confirmed **8699aef**; coordinator fast-forwarded to **16f5255** with edits intact during closing verification. The only intervening committed changes are decisions.md, status.md and the G47 receipt rename to 00026; executable/manifest/root-lock source is unchanged. This task's receipt number is 00027, per coordinator correction. No G48 implementation is included. G48 is recorded in [decisions.md](../decisions.md); the separately requested [00028 design checkpoint](00028-g48-mysql-schema-setup.md) awaits review/release. This receipt does not claim all live tests are green or authorize a commit.

## Search and fixture contracts

New opt-in live_search executables in session and API mirror the existing g46_sqlite_ascii_searches_execute_* regressions through the actual actions. Session installs Profile then Auth and proves Profile q/like, ordered p1/p3 identities/count, Auth-to-Profile like and Auth ID like/count. API installs Profile/Application/Session, proves Application q/like and Session ID like/count; a nonmatching Session q remains a source no-op. All use real store install(), preparation, native codecs and source selects; no handwritten table replaces source DDL. PostgreSQL runs successfully with mixed ASCII case. Arbitrary Unicode/collation, every JSON operator, load capacity and aggregate live-provider wiring are not claimed.

The two gates are STACKPRESS_TEST_POSTGRES_URL / STACKPRESS_TEST_MYSQL_URL. They accept only the documented passwordless local URL forms (localhost or 127.0.0.1; 5432/postgres or 3306/root, administrative database postgres/mysql). No URL, credentials, raw SQL or native diagnostics is printed. Unset gates dynamically report ignored to libtest/nextest: **2 watchdog controls passed / 4 live cases skipped** in the scoped unset run. The targets require sql; they never pass by returning early when a URL is absent.

Adapter M2 Fixture structs are private and unreachable without an API/source change. The new shared test-only owner uses the existing public PostgresDatabase/MysqlDatabase and reuses the exact reachable SQL harness.rs discovery and support/process.rs watchdog files by path. API imports that same owner; no copied second owner, new fixture crate, public library API or production source change was introduced. Each enabled case creates one unique bounded stackpress_g46_* database with CREATE DATABASE, never IF NOT EXISTS. CREATE acknowledgement arms exact-name cleanup before socket close; DROP acknowledgement disarms before close, preventing a repeated DROP after an acknowledged removal. No database enumeration, adoption, prefix sweep, role/global change or listener occurs.

The parent owns creation and cleanup; children own the source workflow. A 30-second external child deadline kills/reaps before DROP. Administration uses a separate worker/runtime with a 9-second IO and 10-second wall deadline. Unwind/failure invokes the armed exact-name Drop guard; cleanup failure fails the executable. A forced-stall negative control requires the watchdog's exact timeout result, not any arbitrary error. Actual MySQL child failures also prove the success/failure cleanup path.

## Actual MySQL boundary (retained verbatim)

Both real MySQL cases create/open their owned database, then fail at Profile install with **Unsupported: dialect does not support transactional DDL plans**. No source table, seed or search is reached. ProfileActions::install at [session actions](../../../../crates/framework/framework-session/src/storage/actions.rs) calls Engine::execute_plan; [Engine](../../../../crates/sql/sql/src/engine.rs) checks transactional_ddl before executing the plan. API's [install](../../../../crates/framework/framework-api/src/models/actions.rs) uses the same contract. This is an accepted lower-crate setup refusal, not evidence against LIKE or the server's collation.

No manual schema, sequential plan bypass, expected-error replacement test or production workaround was added. The enabled MySQL cases remain failing until separately released G48 work. Initial and final combined-feature live runs each report **4 passed / 2 failed / 0 skipped**: two PostgreSQL cases and two watchdog controls pass; both MySQL searches are blocked. Every database below was created by this task and has a matching confirmed drop; none remains to be swept.

| Exact owned database | Result / cleanup | Record |
| --- | --- | --- |
| `stackpress_g46_postgres_27f2_18e3e5b1a83b0650` | PostgreSQL search passed; **drop confirmed** | `20261101T104025.695977Z-8fc4a635b908` |
| `stackpress_g46_postgres_27eb_18e3e5b19368a818` | PostgreSQL search passed; **drop confirmed** | `20261101T104025.695977Z-8fc4a635b908` |
| `stackpress_g46_mysql_27ea_18e3e5b19368b7b8` | MySQL install refused; search not reached; **drop confirmed** | `20261101T104025.695977Z-8fc4a635b908` |
| `stackpress_g46_mysql_27f0_18e3e5b1a4770cf8` | MySQL install refused; search not reached; **drop confirmed** | `20261101T104025.695977Z-8fc4a635b908` |
| `stackpress_g46_postgres_f533_18e3f21f05b1c610` | PostgreSQL search passed; **drop confirmed** | `20261101T142738.555633Z-f9aaba323385` |
| `stackpress_g46_postgres_f53d_18e3f21f1bd56438` | PostgreSQL search passed; **drop confirmed** | `20261101T142738.555633Z-f9aaba323385` |
| `stackpress_g46_mysql_f532_18e3f21f05b4ac40` | MySQL install refused; search not reached; **drop confirmed** | `20261101T142738.555633Z-f9aaba323385` |
| `stackpress_g46_mysql_f539_18e3f21f1891f138` | MySQL install refused; search not reached; **drop confirmed** | `20261101T142738.555633Z-f9aaba323385` |

## Three required rounds

| Round | Finding and verification |
| --- | --- |
| Complexity | Tightened the new fixture's CREATE/DROP acknowledgement state so socket-close failure cannot lose cleanup authority or cause a second DROP. Tightened the forced-stall control to require the exact kill/reap timeout. Unset-gate rerun: 2 passed / 4 ignored; final live replay confirms both PostgreSQL successes and MySQL failure cleanup. No source-store behaviour changed. |
| Responsibility/abstraction | **Nothing further found.** One test-only owner serves both crates; existing adapter/discovery/watchdog owners stay in place. Real stores own DDL/codecs. Existing four G46 regressions rerun: 4 passed. No public test API, new crate or generic framework abstraction. |
| Consistency/Rustdoc/comments | Added precise module/ownership comments and session/API/aggregate docs/claim anchors. Distinguished successful PostgreSQL source searches from the current MySQL setup refusal and aggregate/provider limits. Static docs and full default/combined tests, doctests, Clippy/Rustdoc -D warnings and fmt passed. |

## Final commands and scope

All Cargo/proof/clean commands go through the benchmark recorder; outer scripts record nested Cargo once. Cargo always uses --locked --offline; fmt has no resolution flags. Nextest and libtest use --test-threads 2. Workspace nextest excludes tcp_watchdog. Gate/coverage explicitly unset live URLs so unrelated adapter suites stay opt-in; real G46 live runs are separately recorded above. No renderer all-features was used.

| Command / scope | Result |
| --- | --- |
| cargo nextest run -p session -p api --test main --features session/sql,api/sql --locked --offline --test-threads 2 -E test(g46_) | 4 existing rendering/SQLite tests passed. Actual package names have stackpress-framework- prefixes; full argv is in records. |
| cargo nextest run -p stackpress-framework-session -p stackpress-framework-api --test live_search --features stackpress-framework-session/sql,stackpress-framework-api/sql --locked --offline --test-threads 2 --no-fail-fast --success-output final --failure-output final | Unset: 2 passed / 4 ignored. Initial live: 4 passed / 2 failed. |
| Final live selection, features session/auth-server,session/leptos-ssr,api/sql,api/codegen,api/ui-ssr | 4 passed / 2 current MySQL refusals; full package-qualified features in records. |
| cargo fmt --all --check | Passed after owned-target formatting. |
| cargo clippy --workspace --all-targets [--features F] --locked --offline -- -D warnings | Default + combined passed. |
| cargo nextest run --workspace --all-targets [--features F] --locked --offline --test-threads 2 --no-fail-fast -E 'not test(tcp_watchdog)' | **934 default / 1038 combined passed**, 88 / 93 skips. Live gates unset. |
| cargo test --workspace --doc [--features F] --locked --offline -- --test-threads 2 | **962 default / 967 combined passed**. |
| RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --features F --locked --offline | Passed. |
| session/docs/check.py; aggregate/docs/check.py; aggregate/proofs/check.py | 21 pages / 17 fences / 38 claims / 4 controls; 19 pages / 11 fences / 61 claims / 7 controls; complete ledger passed. |
| New API README / Markdown examples | 7 local links resolve. No pre-existing runnable fence changed; no new Rust fence or docs resolution was introduced. API has no package docs/check.py. |

F = `framework-aggregate-full,framework-aggregate-ui-session,framework-aggregate-ui-api,framework-aggregate-ui-admin,framework-aggregate-ui-view,framework-aggregate-ssr,framework-api-codegen,framework-csrf-server,framework-email-server,stackpress-framework/codegen,stackpress-framework-schema/providers`.

Coverage uses installed cargo-llvm-cov/LLVM, Rust/Cargo 1.96.0, native aarch64-apple-darwin. Per crate: cargo llvm-cov -p PACKAGE --features FEATURES --locked --offline --fail-under-lines 70 --json --output-path /private/tmp/g46-live/PACKAGE-coverage.json -- --test-threads 2. No new exclusions or threshold changes; table isolates each owning first-party src denominator. New fixture code is test-only and excluded by the tool's normal test-path filter; its actual execution/cleanup evidence is above.

| Crate / features | Line coverage |
| --- | --- |
| session / auth-server,leptos-ssr | 5222/5710 = **91.45%** |
| API / sql,codegen,ui-ssr | 3708/4146 = **89.44%** |
| aggregate / full,codegen | 1043/1078 = **96.75%** |

## Exact changed files and lock policy

**17 paths for this task including this receipt**: 11 existing paths modified, five new crate files and this receipt. The separate 00028 design-only note adds one path, making 18 for the combined handoff. All **3563 other baseline tracked files**, implementation source, root manifest/facade, license files, .agents/context, consumer locks and historical records are unchanged. The manifests add only the live targets and dev path edges to existing PostgreSQL/MySQL adapters. Root Cargo.lock changes only four workspace edges in session/API; all **699 external package entries remain byte-identical**, with no new tuple, package fetch or resolution. Nothing was staged or committed.

| Status | File |
| --- | --- |
| `A` | [.agents/specs/00001-stackpress-rust-foundation/tasks/00027-g46-live-search.md](../../../../.agents/specs/00001-stackpress-rust-foundation/tasks/00027-g46-live-search.md) |
| `M` | [Cargo.lock](../../../../Cargo.lock) |
| `M` | [crates/framework/framework-api/Cargo.toml](../../../../crates/framework/framework-api/Cargo.toml) |
| `A` | [crates/framework/framework-api/README.md](../../../../crates/framework/framework-api/README.md) |
| `A` | [crates/framework/framework-api/tests/live_search.rs](../../../../crates/framework/framework-api/tests/live_search.rs) |
| `M` | [crates/framework/framework-session/Cargo.toml](../../../../crates/framework/framework-session/Cargo.toml) |
| `M` | [crates/framework/framework-session/docs/api/storage.md](../../../../crates/framework/framework-session/docs/api/storage.md) |
| `M` | [crates/framework/framework-session/docs/claims.json](../../../../crates/framework/framework-session/docs/claims.json) |
| `A` | [crates/framework/framework-session/docs/live-search.md](../../../../crates/framework/framework-session/docs/live-search.md) |
| `M` | [crates/framework/framework-session/docs/source-behavior.md](../../../../crates/framework/framework-session/docs/source-behavior.md) |
| `M` | [crates/framework/framework-session/docs/storage.md](../../../../crates/framework/framework-session/docs/storage.md) |
| `A` | [crates/framework/framework-session/tests/live/support.rs](../../../../crates/framework/framework-session/tests/live/support.rs) |
| `A` | [crates/framework/framework-session/tests/live_search.rs](../../../../crates/framework/framework-session/tests/live_search.rs) |
| `M` | [crates/framework/framework/docs/api/sql.md](../../../../crates/framework/framework/docs/api/sql.md) |
| `M` | [crates/framework/framework/docs/claims.json](../../../../crates/framework/framework/docs/claims.json) |
| `M` | [crates/framework/framework/docs/source-behavior.md](../../../../crates/framework/framework/docs/source-behavior.md) |
| `M` | [crates/framework/framework/proofs/coverage-map.json](../../../../crates/framework/framework/proofs/coverage-map.json) |

## Resources, attempts and KB

Fresh target/g46-live, warm sccache, two jobs, debug=0, incremental=0; cap 24 GiB / guard 23.4. Existing targets and dependency artifacts retained. Sampled completed-command peak **10.66 GiB**; no cap/deadline trip or later cleanup. Initial clean passed. The first coverage-clean command warned that the new dev edges were not yet in the lock; after the exact authorized edge edit/assertion, a separate initial coverage clean succeeded before the first build. Its earlier tool exit 0 is not represented as a successful inner Cargo clean. The wrong ledger-checker filename was a proof-script error, corrected to existing proofs/check.py; package docs already passed. The MySQL failures above remain deliberately visible.

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`, runs/<id>.json/log. Effort-filtered attempts through final validation: **20**; **6** emitted compilation. Wall time and observed Cargo-stage subtotal are separate; unavailable is not zero. Closing-record timings below were transcribed after completion.

| Record | Stage | Status / exit | Wall s | Cargo-stage s |
| --- | --- | --- | ---: | ---: |
| 20261101T103307.218766Z-08b37e5a98f6 | baseline and bounded fresh target preparation | passed / 0 | 0.64 | unavailable |
| 20261101T103324.182263Z-20e4295ce559 | initial fresh target clean | passed / 0 | 2.07 | unavailable |
| 20261101T103502.226704Z-7a754e8b42c2 | initial coverage clean | passed / 0 | 2.05 | unavailable |
| 20261101T103554.052064Z-e58f8b4f994d | four workspace test edges and immediate frozen registry assertion | passed / 0 | 2.04 | unavailable |
| 20261101T103629.280346Z-2500c6e872a5 | initial coverage clean after authorized lock edges | passed / 0 | 2.05 | unavailable |
| 20261101T103652.522818Z-a5898824fc13 | format owned test targets | passed / 0 | 2.10 | unavailable |
| 20261101T103729.541599Z-79beb68c2a98 | edit loop existing G46 baseline warm sccache fresh target | passed / 0 | 77.39 | 71.00 |
| 20261101T103930.966989Z-90a4aa6149c5 | edit loop unset gates must report ignored | passed / 0 | 6.17 | 2.67 |
| 20261101T104025.695977Z-8fc4a635b908 | real PostgreSQL and MySQL session API searches | failed / 1 | 4.15 | 0.37 |
| 20261101T104304.257695Z-6f14ab4db134 | round 1 complexity ownership acknowledgement and watchdog control | passed / 0 | 6.20 | 2.49 |
| 20261101T135636.659682Z-e595d64d2fd0 | round 2 responsibility shared fixture and real stores | passed / 0 | 2.15 | 0.34 |
| 20261101T135840.774493Z-771125416bce | round 3 documentation claims and ownership comments | failed / 1 | 2.10 | unavailable |
| 20261101T135920.515789Z-7ad772c92a6b | round 3 corrected documentation and ledger checks | passed / 0 | 2.10 | unavailable |
| 20261101T140031.342421Z-997e3127717d | gate workspace default compatible features two workers no live gates | passed / 0 | 1163.22 | 501.24 |
| 20261101T141047.007204Z-a99def18c11c | exact scope frozen registry and canonical KB baseline | failed / 1 | 2.35 | unavailable |
| 20261101T142146.302406Z-d0492b33d844 | gate independent session API aggregate coverage floor 70 | passed / 0 | 246.28 | 222.18 |
| 20261101T142333.287921Z-bd5024395f96 | coordinator 16f5255 renumbering and corrected zero-error KB baseline | passed / 0 | 2.56 | unavailable |
| 20261101T142738.555633Z-f9aaba323385 | final combined-feature real searches current MySQL refusal | failed / 1 | 35.63 | 30.38 |
| 20261101T142851.324892Z-03bb840701d9 | close 00027 receipt exact results database pairs and scope | passed / 0 | 2.60 | unavailable |
| 20261101T143815.497789Z-cc9c94d16570 | final 00027 receipt and 00028 design scope KB verification | passed / 0 | 2.79 | unavailable |

The initial KB check found two missing task links introduced by the coordinator's commit (00025-tiny-template-generation.md and 00025-template-integration.md). The coordinator fixed them and corrected task numbering in 16f5255; they are not inherited errors in the final baseline. A separate baseline rerun was rejected before execution pending that correction. The recorded transition proves only those KB files changed and retains the original snapshot. Final record cc9c94d16570 confirms the canonical KB at **0 errors / 28 warnings** and the raw worktree at the unchanged **146 inherited sibling-layout errors / 28 warnings**, with zero task-link errors. It also rechecks both package docs, the aggregate ledger, 41 local receipt/design links, line caps, all eight database pairs, the exact 18-path combined handoff and unchanged 699 external entries. No commits, staging, installs, listener starts, role/global changes or unowned database cleanup occurred.
