# Slice 11 risk inventory and verification plan

Status: Draft / Not Frozen. No implementation/build is released. Read [composition](slice-11-design-api.md) and [targets](slice-11-design-targets.md) before interpreting these proposed tests. Each risk concerns an aggregate regression or mechanics gap, not a mandate to harden accepted source behavior.

## Risk and proof matrix

| ID | Risk | Prevention / documented behavior | Proving test after release |
| --- | --- | --- | --- |
| R01 | Source list reduced to available five registrars | Inventory server/schema/language/script/client/admin responsibilities, real adapters, no dummy callbacks | Instrument actual upstream plugin module with inert import spies; compare all nine calls and surviving subsets; independently exercise real Rust constituent contributions. |
| R02 | Phase/tie ordering changes | Preserve schema Config priority 10, Config→Listen→Route, source FIFO at equal priority and session auth/email/identity | Full source trace plus exact Rust listener/route/config phase trace, including immediate account events vs API Listen; fail M0 on unresolved difference. |
| R03 | Route conflict arbitration sneaks in | G42 retains multiple matching handlers and Abort; no aggregate route ownership | Two aggregate/independent handlers on one event/route; priority, equal ties, first/middle/last Abort and raw nested call assertions. |
| R04 | Double registration of constituents or generated SQL | One call per selected constituent; shared catalog/one SQL registration owner; no nested app.plugin | Composite vs direct parity; selecting composite plus its constituent retains existing typed-section/service failure, no partial Server. Multi-model callback order and generated SQL count checked. |
| R05 | Missing configuration becomes startup denial | Preserve optional source service/routes; optional client absence and no-model mode allowed | Missing session, absent/empty access, missing auth/api/email/language/view, no/empty client; inspect full service/listener/route inventories and source zero-status error offer. |
| R06 | Added global CSRF/auth/permission gate | Aggregate adds none; each existing process owns its source checks | Public endpoint/direct event remains callable without aggregate-wide credentials; same guard skip/error/GET/flag traces as Slice 9/API. CSRF registrar alone validates no route. |
| R07 | Session group accidentally replaced by identity-only registrar | Explicit auth-server + composite; no second email registrar | Trace auth-signup/signin/signout, email-send, me,authorize order; real JSON mail event, cookie-only me/API consent flow and literal signout key. |
| R08 | Source accepted edge is silently fixed | Current behavior follows upstream for all seven open questions and request flags/lifetimes | Reuse SQLite/export/account/email source oracles and API Moment/date cases under aggregate composition; compare results byte-for-byte where current goldens are exact. |
| R09 | Catalog becomes a runtime code loader | Static compiled descriptors/functions only; client.module build identity, nullable and required lookup distinct | Unknown module/config names cannot load code; absent client true→None/false→error; missing listen/admin method skipped, real selected callback called once. |
| R10 | Native resource or secret reaches browser/snapshot | Native target gates; explicit public projection; catalog/services not Serialize; keep intentional form values | Compile-fail service/command/catalog serialization, graph denylist, signing/DB/SMTP sentinels absent from WASM and SSR seed, deliberate enrollment secret/CSRF values present. |
| R11 | Feature unification hides disabled/plugin or target leaks | Separate normal-edge standalone graphs; distinguish reachable crate from registrar; weak renderer forwarding | Default facade lib-only; API-disabled absence; session-disabled registrar absent even if API/native needs schema; separate native/SSR/hydrate/CSR and feature-alone builds. |
| R12 | Admin becomes generic unconfigured route installer | Real app-generated registrar last; existing Mount/SQL readiness/resources | Finite generated admin page and API model management routes with actual generated stores/handlers; missing/wrong-order resources fail via current contract, not new global policy. |
| R13 | SQL lifecycle now performs hidden IO or invents optional dependency failures | Source operator adapters explicit; no Engine open/DDL/seed/startup; generated-sql only when explicitly selected | No-client host with no Engine builds; command provider/Engine observation counters zero until invoke; SQLite operator/model calls real and explicitly closed. |
| R14 | Locale-prefix adapter recurses/deadlocks or loses request state | One raw nested dispatch, shorten only recognized prefix, no lock across await, preserve source data/cookie order | Data locale + prefix override, unknown prefix, query/post/headers/cookie/body/resource roundtrip, duplicate handler/Abort, concurrent locales, uncloneable IO mechanics proof under watchdog. |
| R15 | Wrapper swallows command/backend errors or claims unsupported IO worked | Provider/resource Result errors retained, source request/default/result order; Node-only limitations named | Every command success/error/skip with finite real adapters or explicit fake IO boundary; query event exists but no query script alias; no automatic retries/transaction acknowledgement. |
| R16 | Catalog/driver/renderer ownership crosses await incorrectly | Owned Send callbacks/Arc services, copy settings before host work; no process-global active client/locale | Non-Send provider rejected, cancellation/drop/reentrant config, concurrent nested events and suspended child; external deadline/kill/reap negative control. |
| R17 | Generated client build cycle or extra registry tuples | Separate tool/runtime manifests; existing definitions/Registry/OutputWorkspace; locked subset consumers | Fresh/clean/repeat generator --check, disabled generated selection, duplicate provider/partition failure, consumer tuple subset and root byte hash; stop on new tuple. |
| R18 | Aggregate widens host/TTY/provider default graph | Explicit host TLS/CLI default-features=false/presentation/backend/provider selection | Feature-alone normal graphs, default facade/disabled consumer, registration IO counters; no listener/network/TTY during any finite setup. |
| R19 | CSS/public exports break already accepted UI | Qualified module aliases; no global reset/UnoCSS injection; same shared views | Native/browser public-path compile tests, prior session/API/admin screenshot/runtime suites on full aggregate with unchanged public seeds; no new visual design or icon package. |
| R20 | Implementation evidence conflated with live/backend/platform acceptance | Explicit provider/SQLite/renderer vs live/installed limits | Receipt records actual graphs, executable paths, coverage scope and unverified SMTP/SES/sendmail/Postgres/MySQL/PGlite/WHATWG/Tauri rather than inferring support. |

## Consumer contracts

Model on session/email standalone workspaces, copied locks as subsets, distinct targets, run.py and graphs.py --check. No consumer can borrow workspace dev-feature unification or an old same-named binary.

| Consumer | Required evidence |
| --- | --- |
| full native | Real source nine-stage composition with compiled client, SQL data, app-generated admin callbacks and finite command/provider bindings; real file SQLite. |
| headless/source absent config | stock register, no compiled client/Engine/auth/API/email/language/view config; source optional contributions only; zero startup IO. |
| API disabled at compile time | Full minus api feature using no defaults; no API routes/registrar or crate in normal graph. Renderer flags cannot revive it. |
| direct constituents | Same native/config/catalog/resources as full; same ordinary functions in exact source order without aggregate registrar; compare observable phase/event/response inventory. |
| default disabled facade | Root stackpress with no features: only stackpress-lib among first-party normal deps, no aggregate/session/CSRF/email/API/UI/SQL/native host. |
| facade aggregate | Same APIs through stackpress::framework with explicit forwarding features, existing root features unchanged. |
| website/browser | Real SSR host + separate hydrate and CSR artifacts consuming identical public page contracts; source account/OAuth/admin behavior and graph/seed boundaries. |
| build-host | Explicit framework codegen helpers with API built-in schema, real existing emitters/diagnostics/output freshness; final app/browser normal graph has no codegen. |

## Milestones and file ownership

Two lanes are a proposal, not agents launched here. One implementer may perform both serially. Shared files have one coordinator-released integration owner.

| Gate | Outputs / proof | Exclusive file owner / stop |
| --- | --- | --- |
| Design now | Complete package/export/overlap/API/config/feature/risk inventory, owner questions and KB evidence | This implementer: new slice-11-design-* plus task/status only. **Stop.** |
| M0 | Real aggregate crate and minimal CompiledClient/register/register_with; prove nine-stage/phase order, absent client, existing SQL/admin integration fit and locale-prefix/native IO ownership; disposable source oracle outside repo | Lane A: aggregate Cargo.toml/src/native/tests for proof. Shared root files only explicit integration grant. Exact existing-crate helper request first if necessary. **Stop before broad adapters.** |
| M1 | Complete substantive server/schema/language/SQL/view command/client/provider bridges, real registrar reuse and direct APIs, full source input/skip/error mapping | Lane A: aggregate src/**, tests/native/** and oracle fixtures. No lower-crate edits without exact grant. Real SQLite + finite command/provider error/cancellation tests. **Stop.** |
| M2 | Full/direct/disabled/facade/target/build-host consumers and real generated admin/API/session/CSRF/email integration; root optional exports/forwarders | Lane B: aggregate consumers/**, proofs/**, consumer fixtures; Lane A retains production src/native tests. Single integration owner: root Cargo.toml/Cargo.lock/src/lib.rs and explicitly released generated app fixture. **Stop.** |
| M3 | Complete compatible workspace/feature/target/browser gate, every inventory/risk row mapped, >=70% coverage per changed implementation crate, provider/backend/installed limits | Integration owner closes joint receipt. Root re-export-only lines get compile/graph evidence; do not label zero executable lines 100% coverage. **Stop before rounds.** |
| Rounds | Distinct test-verified complexity, responsibility, consistency/Rustdoc and safety/soundness reviews; no ceremonial edit when no finding | Production owner; consumer owner disjoint. Coordinator re-verifies. |
| Docs | README/getting-started/concepts/guides/API, compiled external examples, claim/link checks | Docs owner after code closeout; report bugs instead of hidden fixes. **Stop for coordinator.** |

Keep source/generated Rust out of .agents resources. Existing project libraries/tests/examples and historical Agent Files are read-only unless an exact later grant names them. No other trial branch or stale target output is evidence. A library composition change has Acceptance criteria: none; any new visual behavior found necessary must receive separate owner acceptance scope before implementation.

## Gate and fail-fast execution

At implementation release: one fresh target/slice11 effort with agreed cap (propose prior Lane A 24 GiB / 23.4 guard), recorded initial clean and llvm-cov clean once, then warm sccache/two jobs/debug=0/nonincremental gate. Separate consumer/renderer targets under that cap. No clean/build is run now.

Edit loop: selected affected crate nextest/check/lint, broaden only at milestone. Gate: fmt; default and compatible selected workspace all-target Clippy -D warnings/nextest; workspace doctests; Rustdoc -D warnings; each new/forwarded feature alone; native + WASM hydrate/CSR; consumers graphs --check/run; generator freshness; prior shared-UI browser matrices through the aggregate; coverage >=70% for changed implementation crates. Do not combine renderer all-features. Include API sql/codegen/ui-ssr, session auth-server/leptos-ssr, schema providers and CSRF/email server in the compatible native gate where applicable.

Every Cargo/proof/clean command goes through the shared benchmark recorder, --locked --offline wherever resolving is possible; no cargo update/generate-lockfile, install or new tuple. Workspace nextest excludes tcp_watchdog; coordinator runs it. Listener tests, previews and live providers are not authorized by a milestone test plan.

Deadlock-class tests use a parent process with an absolute deadline, process-group terminate, short bounded grace, kill and wait/reap assertion. A deliberately blocked child must be killed to prove the watchdog works. Exercise reentrant config, concurrent nested dispatch, suspended provider/hook, cancellation and native IO; do not rely solely on an in-executor timeout which a deadlock can block. Avoid timing sleeps as synchronization; use explicit channels/barriers. Record wall time separately from observed Cargo-stage subtotal, exact record IDs/counts, target peaks and any named limitation.
