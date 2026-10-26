# Slice 5 risks, milestones and visual acceptance

Proving plan for [checkpoint decisions](slice-5-design-decisions.md). The risk list is the implementer's own inventory and remains open-ended. Compiler/build/coverage results cannot replace rendered interaction, access-policy, cancellation or platform evidence.

## Risk inventory

| ID / risk | Prevention | Proving test |
| --- | --- | --- |
| R01 hydration branch/data mismatch | Identical route/locale/theme/seed and Leptos markers, no time/random/browser-only initial branches | Render real HTML then hydrate in browser with console capture; assert preserved DOM, focus/input, data and zero duplicate initial calls. Include empty/error/Unicode and slow assets. |
| R02 invalid HTML repaired by browser | Semantic markup, stable IDs/keyed rows, correct table/form nesting | Parse SSR in a real browser and hydrate; inspect DOM and fail on hydration warning/mismatch. |
| R03 server/client or asset drift | Explicit wire revision/build ID; locked contract/asset manifest | Serve old client/new host and reverse; missing WASM/chunk/base path; refuse unsafe hydration/call, show actionable reload state without retry loop. |
| R04 Send violation through SSR | Await domain load before creating local render state; no Rc/DOM values across native awaits | Compile real named page Handler; compile-fail Rc-across-await fixture; exercise simultaneous handlers on multithread runtime. General async mode needs an additional real Resource proof. |
| R05 browser/native dependency leakage | Separate packages/target deps and explicit modes; contracts-only import | Real wasm builds for hydrate and CSR; normal/feature tree deny drivers, Tauri, server/CLI, transformer and native services. Build UI via facade too, to catch additive edges. |
| R06 unsafe IPC exposure | Fixed search/cancel commands, explicit Tauri permissions/capability/window origin plus native admission | Allowed local main window succeeds; ungranted/remote/second window, unknown command, forged event/caller, oversized/malformed DTO and other-window cancel denied before handler counters increment. |
| R07 access checks mistaken for universal auth | Each edge owns admission; same capability validated within its policy | Compare direct/page/HTTP/IPC allowed/denied cases; client visibility never authorizes host calls. Demo explicitly exposes only public read seed data. |
| R08 buffered rendering mistaken for streaming | Collect complete data/HTML under finite limits before output | Delayed data appears in first complete response; oversized output errors cleanly with no partial bytes/cookies. Record memory/TTFB limitation. No infinite resource accepted as finite HTML. |
| R09 navigation/new-query cancellation | Abort transport, dispose controller and ignore stale generation | Barrier-controlled A then B; complete B then A; only B renders. Dispose mid-flight; no post-disposal signal write/unhandled rejection. Browser abort and native cancel observed separately. |
| R10 IPC cancel does not stop native work | Scoped in-flight IDs, explicit cancellation/deadline/RAII removal | Hanging read fixture is cancelled by navigation/window close/deadline; no leaked map entry or detached work; cross-window/duplicate ID denial; completion/cancel race remains deterministic. |
| R11 deadlock, reentry or poisoning | No global render mutex; short cancellation lock, no guard across await/callback | Concurrent nested handlers, reentrant dispatch/cancel, intentional poison in a child; fail promptly with sanitized failure and preserve native cause. Parent watchdog kills/reaps on hang. |
| R12 panic inside render | Per-call Owner/owned staging, panic distinct from Result, no success/retry after unwind | Child-process panicking renderer; no output publication/success receipt; cleanup/drop probes; next independent invocation works only if host policy permits. Panic=abort explicitly offers no recovery. |
| R13 snapshot XSS/secrets | Explicit field projection and script-safe JSON escaping; no arbitrary HTML | Sentinels in headers/cookies/session/native/config absent from HTML/wire; hostile `</script>`, quotes, ampersands, Unicode separators roundtrip exactly and execute nothing in browser under CSP. |
| R14 Head/CSP/resource duplication | One meta context/root; trusted asset manifest and explicit policy | Repeated client navigation changes title/description once, no duplicate style/meta tags; malicious metadata escaped; packaged CSP allows required WASM/IPC only and blocks remote navigation/scripts. |
| R15 lifecycle/finalization divergence | Existing handle_with_options/delegate/OutputGuard, no custom finally/retry | Stage decision table: request failure, operation Abort, normal/error response, error-listener failure, missing listener, pre-normalization failure, stopped/foreign-sent output and serialization failure across web/IPC. Compare G12/G17 reports. |
| R16 cancellation implies rollback | Read-only first operation; explicit partial-work semantics | Cancellation during an instrumented staged fixture records completed work without fabricated rollback/success; no automatic retry. Future writes require idempotency/transaction policy. |
| R17 cloned live state aliases unexpectedly | DTO ownership; explicit Arc only for immutable replies/host; Request/Response non-Clone | Mutate independently decoded snapshots and confirm independence; sharing tests assert deliberate Arc ownership, no cross-request signal/context contamination. |
| R18 trusted render blocks allocator/executor | Bounded inputs/HTML, small synchronous renderer, no hidden blocking IO | Largest accepted rows/snapshot/HTML and one-over limits; response latency/peak allocations recorded. Infinite trusted component runs only in killed child; do not claim sandboxing. |
| R19 form/query/route disagreement | One shared public route/input codec; separate API namespace | SSR GET without JS, hydrated form, encoded Unicode/path/query, base-path refresh, back/forward, detail-not-found and Tauri packaged navigation agree. Native duplicate route listeners remain all-matching. |
| R20 locale/theme mismatch or truthiness loss | Explicit seed defaults, safe reactive phrase/node interpolation | SSR/hydration/Tauri equality for locale switch, placeholders/missing phrases/nodes/zero/false/empty; no cookie/window reads in initial render. Persistence deferred explicitly. |
| R21 inaccessible reusable components | Native semantics/labels/focus/live regions, no color-only status | Keyboard-only search/detail/back; screen-reader labels/status/error association; 200% zoom and reduced motion; later overlay controls need their own focus/escape tests. |
| R22 generated contract drift or duplicate domain | Source-file reuse + generator freshness check | Change schema/remove model/disable selection in disposable copy; --check detects stale output and required consumer fails appropriately. Compare native seed results to generated CLI/HTTP/direct baseline. |
| R23 package/target overclaim | Receipt labels compile, link, browser, bundle and visual states separately | Build selected macOS `.app`, launch packaged assets and search with network disabled. Unrun targets/features remain named limitations. Tauri mock tests alone cannot pass native acceptance. |
| R24 third-party framework regression | Exact pins, repro fixtures, minimal adapter | Run known Suspense, route-transition/RefCell and hydration races against selected versions; dependency update reruns affected proof. No framework-internal unsafe modification. |
| R25 example gateway unbounded reads/disconnects | Bounded body acquisition/header budgets/deadlines before HttpAdapter; task ownership | Oversize/chunked/slow body and disconnect tests, finite shutdown; socket listener only under authorized devmetrics. These prove example scope, not a production network adapter. |
| R26 diagnostics disclosure/correlation loss | Public codec separate from developer report; inherited nested context | Secret-bearing handler error/failed error handler and malformed transport tests expose sanitized messages only; native logs retain original causes/stages/correlation. |
| R27 stale caching/cross-user HTML | Public demo data only; explicit no implicit cache in view | Concurrent distinct seeds/locales remain isolated; headers deny accidental shared/private caching if identity later added. No global last-render/snapshot cache. |
| R28 unbounded canceled work/shutdown | In-flight cap/deadline, reject admission after close | Flood to cap+1, cancel all, close window and app; live-count returns to zero inside watchdog. No mutex poisoning or retained Arc cycle. |

## M0: riskiest proof, then stop

One coordinator-assigned integration owner establishes manifests/lock/facade and the actual typed page/operation interfaces before lanes diverge. Required approvals: owner choices, missing installs, dependency budget and any example gateway. No implementation begins from this design alone.

1. Create real framework-view/UI packages containing the smallest accepted page/snapshot/render implementation. Consume the actual generated Profile source and operation. Compile the named loader/Handler with Send, render/Head/seed/Owner cleanup and return HTML through existing HttpAdapter; test Abort/error before output.
2. Build the actual client-only hydration WASM. Render one real result on the server, hydrate unchanged markup, observe no duplicate initial search, then submit a second typed query. Prove hostile seed escaping and contract mismatch. Use only an authorized managed preview or a finite browser harness; no bare listener.
3. Build the same component as CSR and embed it in a minimal real macOS Tauri `.app`, with explicit app-command permissions and local-window-only capabilities. Reuse generated native operation source, verify one lifecycle, test a denied call and a cancellation barrier. Packaged launch must require no local HTTP server.
4. Inspect normal/feature graphs and compile-fail a native non-Send handler. Test a disposed render/cancel path with no retained request/Owner/host cycle. If any step requires weakening Send, unsafe lifetime tricks, unbounded locks or hidden domain duplication, stop with evidence and revise the design.

M0 is not passed by the current probes: real hydration, client graph, permissions, packaged launch and typed API sketches remain unproved. **Stop for coordinator re-verification.** If installations are declined, deliver native-only partial proof with blockers; do not rename it complete.

## Two parallel lanes after interfaces freeze

| Lane | Exclusive write set | Responsibility |
| --- | --- | --- |
| A pages/web | framework-view; framework-ui render/routing; example web host and its route/SSR/HTTP tests | Page/lifecycle/snapshot/Head, browser hydration/HTTP bridge and example-only gateway if accepted. No lib/server/HTTP/CLI/Idea source edits. |
| B shared UI/native | framework-ui components/state/language; example shared UI/contracts/CSR/Tauri host and tests | Search/detail UX, selected component behaviors, localization, native admission/cancel/packaging. No changes to lane A renderer/routes or shared module declarations. |

Coordinator assigns a single implementer for shared framework-ui manifest/lib.rs exports, root manifests/lock/facade, asset metadata and route-contract files. Other lane sends a concrete patch request and stops at dependencies. Coordinator integrates these in sequence alongside Slice 4 under G23. Current Idea docs and all slice-4-design files remain untouched. This checkpoint spawns/messages no other agent.

| Milestone | Required deliverable/signal |
| --- | --- |
| M1 page and contracts | Typed projection/escaping/status/limits; native page extension and real error/Abort/finalization table; fixed HTTP codec; Head/route/base-path agreement and contracts-only graph. |
| M2 shared UI | Selected primitives/layout/notification/translation behavior; complete search/detail/empty/invalid/error/loading states; keyboard/responsive tests; hydration/no-initial-refetch, navigation/cancellation and client dependency proof. |
| M3 native | Local commands/capabilities/CSP, G12 parity, scoped cancel/deadline/shutdown and packaged navigation; actual unsigned macOS ARM64 app with local operation while networking is disabled. |
| M4 integrated DX | Exact finite build/check scripts and optional managed preview instructions; source/plugin reuse, generated freshness, version mismatch/error recovery, feature matrix, per-crate coverage and owner-visible acceptance packet. |

M1/M2 may progress concurrently where files/interfaces are frozen; M3 uses the proven shared controller/contract. Each milestone passes its relevant tests before dependent work. Re-run M0 using final types after interface changes. **Stop before rounds**, report inventory completion/deferrals, per-crate coverage and unresolved risk; coordinator re-verifies before rounds 1–4.

## Verification execution

Deadlock, poisoning, panic, cancel-race, stuck renderer and shutdown tests run as child processes with ready/go barriers and monotonic parent deadlines (normally 5–10 seconds, separately justified longer build deadlines). Parent uses try_wait, kills and reaps on timeout, saves stderr and fails; no infinite joins, unbounded polls or race assertions based solely on elapsed sleep. Browser/IPC waits also have finite deadlines. Do not block the user-facing turn with long silent waits.

Use a fresh isolated CARGO_TARGET_DIR per lane/feature to avoid stale trial artifacts and destructive shared clean races. Before the first repository implementation build, coordinate the workflow's clean and llvm-cov clean with all active lanes. Design probes used a new external target and did not clean anyone else's output.

Required checks after implementation, adapted to accepted package names/features:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo run -p stackpress-example-profile-codegen -- emit generate --check
python3 .agents/scripts/validate-agent-workspace.py
```

Also run separate SSR/hydrate/CSR feature combinations, actual wasm builds and target trees, the actual macOS bundle command and browser/packaged interaction tests. Invalid all-feature mode combinations are not required to compile together. Runtime/tool dependency feature changes need their own locked tree evidence.

Measure at least **70% executable lines per changed crate/valid measured feature scope**, including example/native/contract executable code and changed facade paths; publication requires 90% separately. Report tool/compiler/target/features/numerator/denominator/exclusions. Native cargo-llvm-cov cannot establish browser event-path coverage: run real browser/GUI behavior tests and list unmeasured WASM-specific lines and any approved coverage exception rather than imply native tests covered them. No workspace-average substitution or mock-only platform acceptance.

Rounds: (1) logic/state transitions and complexity, (2) responsibility/ownership/abstraction and dependency direction, (3) consistency/Rustdoc/examples/explanatory block and line comments, (4) open-ended safety/soundness including cancellation, limits, poison, panic, disclosure and target cfg. Each records findings, meaningful changes or justified no-change result and rerun tests. Repeat targeted rounds while material findings remain. Crate docs follow as a separate gated task.

## Proposed owner visual acceptance

All criteria are proposed for owner approval. Supply screenshots plus interaction evidence for the exact browser/build/native target; screenshots alone do not prove hydration/cancellation. No brand palette request was supplied: start with neutral grayscale, readable typography and visible focus/error states.

| Control | Owner-visible criterion / evidence |
| --- | --- |
| 00008-001 | Website initial HTML visibly contains the search label, submitted ID prefix and Ada/Grace/Margaret results for the chosen seed query before JavaScript. Hydration preserves them without flash, duplicate rows or a second initial request; console has no hydration errors. |
| 00008-002 | Labeled ID-prefix input and Search work with keyboard and Enter. Blank input shows an associated clear error; `zzz` shows a clear empty state; an injected host/transport error shows a safe message and retry action. Loading is visible/announced without erasing the previous useful results. |
| 00008-003 | Detail links, back/forward, direct refresh and return-to-search preserve the expected query/results. Missing detail has a clear not-found state and a route back. Search B replaces A even when A completes late. |
| 00008-004 | At 375 px and 1280 px widths and 200% zoom, form/results/navigation remain usable without unintended horizontal scrolling or clipped controls. Focus order and visible focus work; status/error information does not depend on color. |
| 00008-005 | Title/description/language update correctly on navigation and the approved locale fixture; labels/results/safe interpolated text agree before and after hydration. No duplicate Head tags or theme flash. |
| 00008-006 | Packaged macOS ARM64 app shows the same search/detail UI and public results while networking is disabled. No local listener is required. Native denial/cancel produces a clear safe UI state; repeated navigation/search does not freeze the WebView. |
| 00008-007 | Deliberate old-asset/contract mismatch and missing asset show a useful refresh-required/recovery state; no blank screen, stack trace, secrets or infinite reload loop. |
| 00008-008 | One selected plugin customization changes the visible search output consistently over direct/CLI/HTTP/SSR/Tauri without editing the original operation. Owner reviews source readability and finite build/rebuild commands; timings/bundle sizes are reported, with no invented performance target. |

The historical frontend proof also suggests a third-party UI integration. Recommend treating one real external widget as **S5-F2** unless the owner selects a concrete required widget now; Leptos/meta/router/Tauri integration does not prove arbitrary editor/chart compatibility. This is an explicit scope choice recorded under D05, not a silently satisfied criterion.

## Checkpoint closeout

The design's draft coverage inventory comprises owner Q01/Q02/Q05/Q11/Q12, upstream exports/types/behaviors, version/MSRV/tools, actual disposable probes and limits, public API/DX, G23 changes, risks/tests/deadlines, two lanes, visual criteria and installs. Final split preserves these sections through links from the decisions file. Coordinator owns acceptance/verification notes, task status, index links and ledger changes. No context promotion is requested.
