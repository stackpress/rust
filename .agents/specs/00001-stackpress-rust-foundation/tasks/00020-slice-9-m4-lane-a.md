# Slice 9 Lane A M4 implementation receipt

Status: started; implementation complete, pending independent coordinator verification. Stop at M4. Base/HEAD: `99937b067683731828dee7236027dc151b609b97`, branch `trial/3-slice9-session-csrf-email`. G42/G43 remain binding. This receipt does not release post-milestone refactoring, documentation closeout, publication or installed acceptance.

## Task summary and implementation steps

Delivered the opt-in session facade, explicit direct/plugin/disabled consumers, source composite ordering, real API reconciliation and the compatible workspace/target/browser gate. Changes are confined to framework-session, the explicitly granted root manifest/facade/internal lock edge, and this receipt. No framework-api, framework-email, framework-csrf, stackpress-lib, schema provider or historical Agent File was edited.

1. Preserve the existing identity-only `plugin::register`; add `composite::register` behind native `auth-server`.
2. Add only opt-in root session features and the Rust-style `stackpress::framework_session` re-export.
3. Resolve five standalone locks once under the coordinator's offline grant, then audit subsets/root invariance and use locked/offline builds.
4. Exercise real SQLite, session/CSRF/view/API registration, public events, source permissions and missing-config modes.
5. Run isolated graphs, compatible workspace/API checks, the finite account host, separate browser renderers and production coverage.

## Implementation notes and source reconciliation

`stackpress_framework_session::composite::register(&mut ServerBuilder) -> stackpress_lib::Result<()>` calls the ordinary auth, email, identity registrars directly, in that order. Source: `packages/stackpress-session/src/plugin.ts:13–15`. Constituent entry points remain `auth::register`, `stackpress_framework_email::plugin::register` and session `plugin::register`. The latter remains identity-only for existing API consumers. Native `auth-server` now explicitly forwards `stackpress-framework-email/server`; standalone normal graphs prove that the composite does not depend on dev-feature unification.

The main source order is server/schema/language/CSRF/SQL/view/session/API/admin (`packages/stackpress/src/plugin.ts:16–24`). The plugin consumer selects the applicable real constituents with view → session → API → application admin Config order asserted. Rust admin registration is application-generated, not a generic framework registrar; this finite application's final callback exercises the real admin `Draft` snapshot and asserts its response. No empty registrar or route ownership arbiter is invented.

Three new composite tests prove grouped/direct constituent ordering, real JSON `email-send`, config gates, multiple `me` callbacks and cooperative Abort. Missing session contributes no identity service/me/authorize; missing session.access contributes no session request guard; empty access installs the original permissive guard; missing auth omits auth routes while its reusable Listen events remain; missing email omits email-send. The composite does not select CSRF, install SQL or introduce a global gate. Existing same-priority FIFO/source-process cases run in the full gate.

API reconciliation uses the shipped public paths, without a new universal caller abstraction:

- Identity is the native `Service`/per-token `Session`, with `SessionData`/`SessionTokenData` for source claims/authorization. A handler uses `inv.service::<Service>()?.load_request(req)` and `Session::can(&[SessionPermission]) -> async SessionResult<bool>`; all-of and empty-true semantics are unchanged.
- API OAuth emits `me` with the original mutable Request (`framework-api/src/pages/oauth.rs:60–66`), retaining the configured cookie. A bearer JWT alone still follows the guest redirect. Its subsequent application-detail and session-create calls use fresh request data. The consumer pins `[me(cookie), application-detail(no cookie), session-create(no cookie)]`, the issued code/redirect, and SQL profileId override of forged input.
- API endpoint credentials resolve persisted `session-detail` independently. A real GET with source ID-only credentials reaches its target with the stored profileId even though cookie `can(record-read)` is false. No extra `can` gate is imposed. API persisted Session rows are distinct from browser JWT Session instances.
- API `sql,codegen` check and all 81 tests pass against this combined tree, including the public ordered Record/Profile schema surface. Session memoization remains per instance. No request-local carrier, synchronous SQL OperationPolicy change or coordinator-released stackpress-lib/server edit is needed.

The README now describes current registration/facade paths and provider/platform limits. Implementation review passes checked config/order/Abort branches, constituent ownership and isolated dependency selection, then names/Rustdoc/comments and browser entry lint. The meaningful fixes were explicit email/server forwarding and removal of an unused browser import; no behavioral refactor or unreleased formal closeout round was added.

## Standalone consumers and locks

All five manifests are independent `[workspace]` applications under [session consumers](../../../../crates/framework/framework-session/consumers/run.py). Each consumer owns a distinct target below target/slice9a; native consumers deliberately share the same binary/package name to expose accidental target reuse. The browser consumer owns one separate target and builds hydrate/CSR in separate invocations with separate output directories and normal graphs. Renderer all-features is never used.

| Consumer | Executable proof |
| --- | --- |
| direct | Native Service without registration; short seed, absent expiry, null-invalid/Guest, all-of permissions and per-instance memoization. |
| facade | Same direct behavior through `stackpress::framework_session`, without server, SQL, UI, CSRF or email dependencies. |
| plugin | Actual SQLite and session/CSRF/view/API registrars, application admin, Hooks bind/shutdown, source event traces and Engine close; no listener. |
| disabled | Default facade imports/build/run with stackpress-lib as its only first-party dependency; no session/server/CSRF/email/JWT graph. |
| browser | Root facade imports for exact shared pages, checked public seed, hydrate/CSR compilation, graph/secret assertions, explicit disposal and full browser matrix. |

Exactly one initial `cargo tree --offline` resolution per manifest was seeded from a root-lock copy: native four in `e25e5528fd7b`, browser in `f0a8ed822149`. Immediately after each, registry `(name, version, source, checksum)` tuples were a subset of root and root bytes were unchanged. Subsequent resolving/build commands use `--locked --offline`; fmt is nonresolving. No update/generate-lockfile/fetch/install occurred. All **699 existing external package blocks remain byte-identical to HEAD**. The sole root-lock delta is the internal stackpress → stackpress-framework-session edge. Root lock SHA256: `a004327bd625ccbb0ff27022b7b407340b2babb976914ba10a95ce1413536807`.

Every changed root Cargo.toml line follows; no existing root-manifest line/default feature was changed:

- Line 66: `stackpress-framework-session = { path = "crates/framework/framework-session", optional = true, default-features = false }`
- Line 74: `framework-session = ["dep:stackpress-framework-session"]`
- Line 75: `framework-session-native = ["framework-session", "stackpress-framework-session/native"]`
- Line 76: `framework-session-sql = ["framework-session", "stackpress-framework-session/sql"]`
- Line 77: `framework-session-server = ["framework-session", "stackpress-framework-session/server"]`
- Line 78: `framework-session-auth-server = ["framework-session", "stackpress-framework-session/auth-server"]`
- Line 79: `framework-session-leptos = ["framework-session", "stackpress-framework-session/leptos"]`
- Line 80: `framework-session-leptos-ssr = ["framework-session", "stackpress-framework-session/leptos-ssr"]`
- Line 81: `framework-session-browser = ["framework-session", "stackpress-framework-session/browser"]`
- Line 82: `framework-session-leptos-hydrate = ["framework-session", "stackpress-framework-session/leptos-hydrate"]`
- Line 83: `framework-session-leptos-csr = ["framework-session", "stackpress-framework-session/leptos-csr"]`

Root src/lib.rs adds only the documented `#[cfg(feature = "framework-session")] pub use stackpress_framework_session as framework_session`. Root Cargo.lock adds only that internal dependency. Root default features remain empty.

## Verification process and results

Environment: Rust/Cargo 1.96.0, aarch64-apple-darwin and wasm32-unknown-unknown; sccache, two build jobs, debug=0, nonincremental gate. One retained warm effort target, target/slice9a, with separate consumer subtargets. Cap **20 GiB**, guard **19.4 GiB**, measured M4 peak **19.154 GiB**. Recorded cleanup removed idle incremental state and owned session/consumer artifacts only; all dependency artifacts were retained. The wrapper has an absolute 1,800-second process-group deadline with TERM/KILL/reap. Existing deadlock-class negative-control/ownership/watchdog tests pass; tcp_watchdog is excluded and remains coordinator-owned.

| Gate | Result / evidence |
| --- | --- |
| Workspace formatting, full-compatible Clippy -D warnings and Rustdoc -D warnings | Passed; `23b1ee01d733`. |
| Default workspace nextest, all targets, two test threads | **927 passed**, 86 skipped including tcp_watchdog. |
| Seven-feature workspace nextest, all targets, two test threads | **954 passed**, 88 skipped including tcp_watchdog. |
| Compatible workspace doctests | **964 passed**, including the new real composite registration example. |
| API compatibility | sql/codegen check + **81 tests passed**, five skipped; same workspace gate. |
| Session/facade feature and target matrix | All declared session features and root forwarding leaves compile in compatible isolated modes. WASM default/auth-server/hydrate/CSR graphs exclude native JWT/crypto, SQL/server, CSRF/email/lettre and QR/PNG. Native identity/sql/server/auth-server graphs exclude UI/phone additions. `9691b4978b98`. |
| Lane B regression | Frozen graphs.py --check and its disabled standalone consumer fmt/Clippy/run pass without Lane B edits; completed stages of `812ac7aa1fd5`. That record later stopped solely at the new browser's unused import; corrected browser builds pass below. |
| Native consumers | All four standalone graph/fmt/Clippy/run checks pass in `0cbd3170a9f3`; plugin rerun with explicit admin-result assertion also passes in `812ac7aa1fd5`. |
| Finite native host | Existing SQL-backed account_host emitted all 16 pages in Light/Dark/unset and long-content variants and exited; real email JSON event/Hooks/Engine lifecycle, no listener. `812ac7aa1fd5`. |
| Browser artifacts | Each mode's fmt, WASM Clippy -D warnings, build, wasm-bindgen, lock subset and graph/secret checks pass: hydrate `1ec22b09c2af`, CSR `dd6ef3699cdd`. |
| Browser behavior | **176 hydrate + 176 CSR cases**, plus no-JS Light for each. All source pages, 1280/375px, 200% zoom, long content, explicit Light/Dark/unset, keyboard/labels, password/method/phone/copy controls, CSP, disposal, no overflow/runtime/hydration errors. `a69f3e8c1548`, `8e4f5da761af`. |
| Production coverage | cargo-llvm-cov nextest, session/leptos-ssr including auth-server/sql/server: **5,185/5,668 = 91.48%**; 52 tests passed, one externally entered child skipped; composite 6/6. Tests/examples excluded, no production exclusion. `f5b6f9dffa6e`. Browser-only branches have separate Chromium proof. Root facade has no executable production lines (0/0, percentage not applicable); imports/default/forwarding graphs prove its changes. |
| Lock/scope/KB audit | All consumer tuples remain subsets; no external lock change or foreign-source edit. Inherited **146 errors / 28 warnings**, no new diagnostics after this receipt. This is not a clean-KB claim. |

Seven features are stackpress-framework-session/leptos-ssr, stackpress-framework-csrf/server, stackpress-framework-email/server, stackpress-framework-api/sql, stackpress-framework-api/codegen, stackpress-framework-session/auth-server and stackpress-framework-schema/providers. Exact commands are in the recorder logs. No renderer --all-features or further broad rerun was needed after standalone-consumer-only edits.

Artifacts live under `/private/tmp/slice9a/m4`: pages/, hydrate/, csr/, browser/{hydrate,csr}-results.json, coverage.lcov and coverage-summary.json. Visually inspected fresh hydrate-account-normal-375-2-dark.png and hydrate-enroll-normal-1280-1-dark.png: narrow stacked account values stay readable; dark enrollment, inline regenerate focus and desktop navigation retain the accepted layout. All network requests in the browser proof are fulfilled in memory at synthetic URLs.

## Provider, backend and installed limits

Application-owned Engine/table installation, hook binding/shutdown, challenge clock/zone, configured email provider and route/event access remain explicit. Live SMTP/SES/sendmail delivery, remote webhooks, PostgreSQL/MySQL connections, Linux/Windows native runs and installed Tauri desktop/mobile/device behavior were not verified here. Actual SQLite plus native macOS and headless Chromium WASM evidence does not establish installed acceptance. The complete source implementations remain present; nothing is deferred as a new hardening/provider slice. Accepted source policy/flags/expiry/redirect/unchecked-process behavior is unchanged. Existing source questions remain at parity; this milestone introduces no new owner decision.

## Benchmark records

Shared store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks/runs/`. The outer recorder wraps each Cargo/proof/clean sequence once. Completed M4 counter before receipt writing: **39 attempts, 33 passed, 6 failed**. Command-time sum **2053.061s**, observed Cargo-stage subtotal **1031.650s**; these overlap and neither is the other. Recorded start-to-last-completion window **4728.412s**, including approval/review gaps and overlapping browser work. Receipt-writing/final audit has its own subsequent record.

Six retained failed attempts are fully accounted for: duplicate formatter edition; disposable root-lock pruning under --locked before the approved one-time resolution; a guessed inspection path; audit helper byte decoding; totals helper null timing; and the browser unused import. They were corrected/replaced as recorded. No unresolved production/test failure, dependency tuple request or resource-cap failure remains.

| Record | Label after slice9a M4 | Result | Command s | Cargo stages s |
| --- | --- | --- | --- | --- |
| `20261030T104822.293634Z-4b30175d1790` | baseline rules and Lane B consumer reference | passed | 0.024 | 0.000 |
| `20261030T104841.649235Z-51abc3ad6255` | API identity and caller reconciliation inspection | passed | 0.024 | 0.000 |
| `20261030T104847.083950Z-a32bf47e2927` | session and facade surface inspection | passed | 0.026 | 0.000 |
| `20261030T104906.607299Z-b55b912ddf1c` | facade and caller call sites | passed | 0.054 | 0.000 |
| `20261030T104954.537022Z-8bd188f19ca8` | source aggregate order and public design contract | passed | 0.062 | 0.000 |
| `20261030T105107.838396Z-0031ee14306f` | registration DX JWT and API proof seams | passed | 0.032 | 0.000 |
| `20261030T105550.207890Z-e011e88d159d` | reconcile facade internal lock edge only | passed | 0.035 | 0.000 |
| `20261030T105850.567085Z-2351cd57a6b6` | early standalone disabled facade lock boundary | passed | 4.228 | 0.000 |
| `20261030T105922.029972Z-3eb63f47653b` | edit loop format and composite parity tests | failed | 2.707 | 0.000 |
| `20261030T110004.414101Z-c42010fc4515` | edit loop composite tests corrected formatter invocation | passed | 14.545 | 8.740 |
| `20261030T110041.252266Z-2146755fa18e` | disposable locked standalone snapshot derivation proof | failed | 2.688 | 0.000 |
| `20261030T110418.664579Z-ade19cb66939` | consumer and browser contract details | passed | 0.038 | 0.000 |
| `20261030T110630.378017Z-b330e8fb2189` | actual view admin and API SQL registration interfaces | passed | 0.066 | 0.000 |
| `20261030T110801.163415Z-98e97dc6a98f` | minimal aggregate constituent prerequisites | failed | 0.017 | 0.000 |
| `20261030T110816.461455Z-df3d0739de8a` | locate aggregate registrars and explicit view configuration | passed | 0.154 | 0.000 |
| `20261030T112241.358248Z-e25e5528fd7b` | approved one-time offline standalone consumer lock resolution and tuple audit | passed | 4.515 | 0.000 |
| `20261030T112520.088331Z-134b087c9217` | format owned workspace and standalone consumer sources | passed | 3.745 | 0.000 |
| `20261030T112655.898767Z-0cbd3170a9f3` | standalone direct facade plugin disabled consumers locked offline | passed | 541.798 | 529.070 |
| `20261030T113144.651241Z-f0a8ed822149` | approved browser lock and formatting preparation | passed | 6.091 | 0.000 |
| `20261030T113348.606737Z-4345783d86d9` | browser skill and retained proof execution contract | passed | 0.059 | 0.000 |
| `20261030T113712.776699Z-9691b4978b98` | session and facade isolated feature graphs check | passed | 129.204 | 109.870 |
| `20261030T113753.877166Z-452e395f0c39` | execution and writing contracts | passed | 0.037 | 0.000 |
| `20261030T113951.449312Z-8d8b61ae24d9` | target headroom inventory | passed | 4.132 | 0.000 |
| `20261030T114048.325512Z-75c2aef3ee08` | approved incremental and owned package cleanup | passed | 4.134 | 0.000 |
| `20261030T114106.716270Z-23b1ee01d733` | compatible workspace gate and API reconciliation | passed | 609.866 | 179.030 |
| `20261030T114229.905349Z-924cdd52fa2b` | source lock scope and KB baseline audit | failed | 0.128 | 0.000 |
| `20261030T114247.487372Z-503bfa5f152f` | corrected source lock scope and KB baseline audit | passed | 1.806 | 0.000 |
| `20261030T114432.581017Z-e7f255697d2f` | finite browser and process prerequisites | passed | 0.274 | 0.000 |
| `20261030T115140.857586Z-cacadf62b37e` | approved owned session cleanup before browser artifacts | passed | 3.428 | 0.000 |
| `20261030T115201.258200Z-812ac7aa1fd5` | Lane B isolation finite host and facade browser consumers | failed | 159.614 | 56.850 |
| `20261030T115221.512055Z-8f171ea869ac` | recorder totals and gate count audit | failed | 0.555 | 0.000 |
| `20261030T115412.006404Z-2ff72a955a06` | browser build target headroom inventory | passed | 3.139 | 0.000 |
| `20261030T115532.885949Z-f86a09bcaf95` | completed owned consumer and session artifacts cleanup | passed | 4.767 | 0.000 |
| `20261030T115918.261957Z-1ec22b09c2af` | corrected hydrate facade consumer lint build and graph | passed | 68.181 | 63.580 |
| `20261030T120127.505075Z-dd6ef3699cdd` | CSR facade consumer lint build and graph | passed | 54.617 | 51.010 |
| `20261030T120138.596510Z-a69f3e8c1548` | hydrate finite browser matrix through facade | passed | 183.415 | 0.000 |
| `20261030T120306.687966Z-8e4f5da761af` | CSR finite browser matrix through facade | passed | 187.231 | 0.000 |
| `20261030T120326.658992Z-f5b6f9dffa6e` | session SSR production line coverage | passed | 57.064 | 33.500 |
| `20261030T120709.888024Z-624ba985b0f1` | completed recorder totals and gate count audit | passed | 0.565 | 0.000 |

## Acceptance criteria, verification notes and stop

Acceptance criteria: none for this library/composition milestone; unchanged M3 views have fresh supplemental browser evidence. Independent coordinator verification is pending. No context promotion or new Reference File was needed; this complete receipt stays below 200 lines. No commit/push/install/GitHub write, listener, other-agent message or unreleased phase occurred. Stop at M4.

## Coordinator verification

Verified 2026-10-30 by the coordinator on worktree commit `5b87873` merged with trial/3 `9642281` (API M5 and rounds 1–4) as `de0dcef`, fresh target, gate tier, script `/private/tmp/s9am4-verify/run.sh`, eight-feature selection (`api/sql,codegen,ui-ssr`, `session/leptos-ssr,auth-server`, `csrf/server`, `email/server`, `schema/providers`):

- Locks: original external tuples kept, 0 added/removed; all 12 standalone consumer locks (session ×5, email ×3, api-checkpoint ×3, plus root) are registry subsets of the workspace lock and pass `cargo tree --locked --offline` after the merge without refresh.
- Root manifest diff vs trial/3: exactly the optional `stackpress-framework-session` dependency, ten `framework-session*` opt-in features and the `framework_session` re-export; `cargo tree -p stackpress` default graph excludes the session crate.
- `cargo fmt --all --check`, workspace Clippy `-D warnings`: pass.
- nextest default: 928 passed, 85 skipped, including `tcp_watchdog`. nextest eight-feature (2 threads): 993 passed, 88 skipped, including `tcp_watchdog`.
- Doctests 966 passed, 0 failed; Rustdoc `-D warnings`: pass.
- Facade checks: `framework-session`, `-native`, `-auth-server`, `-leptos-ssr` native; `-leptos-hydrate`, `-leptos-csr` on wasm32. Session per-feature checks (native, sql, server, leptos, auth-server, leptos-ssr) and wasm32 hydrate/CSR: pass.
- `consumers/graphs.py --check`: pass. `consumers/run.py`: direct (121 packages), facade (122), plugin (327), disabled (51) graphs and runs passed. Browser consumer hydrate and CSR builds with native-secret graph exclusion: pass.
- api-checkpoint tool `--check` fresh, web Clippy, client Clippy hydrate/CSR; email direct/disabled/event consumers: pass. API nextest (`sql,codegen,ui-ssr`): 92 passed.
- `cargo llvm-cov` session (`leptos-ssr`): 91.48% lines (fail-under 70).
- `composite::register` order (auth, email, identity) matches upstream `stackpress-session/src/plugin.ts`. Tracked tree clean after the gate.

Merged into trial/3 as `cc4b175`.
