# Slice 9 M4 Lane B receipt

Status: **M4 Lane B complete; stop for coordinator verification**. Started from 9ee2ca4 on trial/3-slice9-csrf-email. Scope is the released Lane B M4 row. G42/G43 and the accepted lettre provider-capability ruling remain binding. Acceptance criteria: none for this code/consumer gate; owner UI acceptance remains separate. Formal rounds 1–4 and the full documentation follow-up are not started.

## Delivered integration and consumers

| Owned files/proof | Result |
| --- | --- |
| framework-csrf/src/plugin.rs, Cargo.toml, tests/main.rs | Added opt-in `server` and ordinary `fn(&mut ServerBuilder) -> Result<()>` registration. Config publishes default/configured Service even with absent/null/falsy/nonobject CSRF config; no request listener/routes. String overrides, source 419 text and duplicate typed-registration mechanics are tested. Direct Service APIs are unchanged. |
| framework-email/tests/cases/integration/ | Removed the M3 Rust fake page and fake auth-signin. Real public auth/identity/CSRF/email registrars provide GET/POST /auth/signin/email, auth-signin and auth-signup. Public Auth/Profile Actions install/seed finite SQLite; public auth-search/auth-update adapters execute lookup/rotate/reload. Clock/random are finite; templates/challenge/JWT/CSRF mechanisms are real. No framework-session edit. |
| integration/signup.rs | Real auth-signup submits email-email-send and auth-phone-verify. Application delegation waits on explicit barriers, proving parent nonawaiting, owned snapshot before secret removal, parent mutation isolation, missing email/hook no-ops, per-call construction and supervised provider failures. Parent response remains independent. Hooks are drained/shut down; SQL engines close before owned fixture directories are removed. |
| integration/transports.rs | Nine configurations: SMTP, pool/full provider, JSON object/string, stream Buffer/Readable, sendmail, SES and string SMTP. Real auth issues the source message; independent fresh email-send resolves construct two transports. Original options/message fields, SMTP MIME/options, pool complete config, JSON fields, stream consumption, sendmail argv and SES destination are inspected with fake external IO. |
| consumers/direct | Standalone workspace executes generate/valid/clear plus repeated JSON/stream sends through direct libraries. Normal graph has no server/session package. |
| consumers/event | Standalone workspace imports public registrars and frozen email-send name; missing/null/false/zero/empty-string email modes deactivate the listener, JSON config activates it, selected CSRF defaults remain available, and an application-owned email-email-send delegation uses the supervisor. No global request guard. |
| consumers/disabled | Standalone workspace uses the default stackpress facade with both optional plugins omitted. Its active graph contains neither framework crate nor stackpress-server. |
| consumers/graphs.py --check | Asserts exactly the declared default/server features and builds all four combinations. Inspects lettre builder/hostname/smtp-transport/tokio1-rustls-tls and reqwest rustls-no-provider/gzip/deflate/cookies; neither SDK default feature is selected. SMTP/pool/JSON/stream/sendmail/SES/string are compiled native adapters in both email graphs, without separate transport feature switches. |
| Both READMEs | Added current feature/consumer and source behavior observations; corrected the stale M1 statement that events were future work. Full guides/reference expansion remains the later docs gate. |

Each consumer has its own `[workspace]`, Cargo.lock snapshot and absolute target/slice9b/consumer-<kind> directory. All three deliberately share a package/binary name; run.py asserts their distinct executable success markers. Final runs use --locked --offline, formatting, Clippy -D warnings and cargo run. Consumer external tuples must be a subset of the parent baseline. Initial lock reconciliation ran offline from copies of the existing root lock.

## Source response observation and parity

**The real auth-update adapter leaves a truthy results body on the shared response. The source email-send handler therefore skips factory construction.** The real OTP page still clears CSRF/redirects after code 200; magic retains the update result and outgoing CSRF. Missing email config/sender occurs after rotation/reload. This is preserved source behavior for that adapter composition, not a source defect claim or a new gate. Fresh email-send resolves still send. A priority handler can return 419; a fresh provider exception still follows the existing G12 sanitized 500/developer-cause/error-offer path.

The updated tests/source-page-boundaries.cjs evaluates the original email page, CSRF plugin, email event and templates (using the installed upstream TemplateEngine), with finite SQL/auth/session/clock/random seams. Eleven cases compare observable event order, complete mail/template fields, status/redirect and CSRF retention against real Rust processes. The former M3 magic provider-throw case now proves that a prior update body skips even a failing factory. Source reproduction uses TZ=UTC; entropy is fixed only in the JS oracle, while Rust token/cookie/form correlation uses OS randomness.

| Original under /Users/cblanquera/server/projects/stackpress/stackpress/packages/ | SHA-256 |
| --- | --- |
| stackpress-session/src/auth/pages/signin/email.ts | `147c76378a43fddf8ccdcc05352909bbd7a68f4fcb039a16a066f401d222efbc` |
| stackpress-csrf/src/plugin.ts | `55f589f6d9d9c63d37e751ca1e550b932c4b893de3f65f263a2590286e5792e2` |
| stackpress-session/src/auth/templates.ts | `ed85cb2306886f00616aadd488d6b46f8b0fd41418a76b1b2478c02987dbc871` |
| stackpress-email/src/events.ts | `4061c0714fc3fff61e65cd05d916332629bcede7f39df0fd55aac8cf7dddc65d` |

Changed fixture SHA-256: `f9099a67d8110e242b1d6cadb71a17b3b4b079f10546b61ab5e8c4071cd464b4`. Final audit regenerates it byte for byte. Existing M0–M3 option/event/transport vectors and ownership compile-fail contracts remain in the regression suites.

| Verification mapping | M4 evidence |
| --- | --- |
| R04/R10/R11/R12/R22 | Real generate-before-valid incoming/outgoing isolation; default/overridden CSRF service; source 419, clear/redirect, intentionally public SSR token/cookie/hidden-input correlation; exact incoming cookie remains. |
| R09/R19/R20/R21/R24/R25/R30 | Original skips/order/config absence, all source transport variants, real signup hook names/owned tasks, direct reuse, unfiltered provider binding and no added global protection. |
| I01/I02/I04/I05/I06/I07/I08 | Existing Send/static/borrow compile-fail proofs, per-call ownership, reader/error/cancellation tests plus real SQL/auth/server/renderer integration and isolated feature/consumer graphs. |
| R28/S9-UI01–06 | Finite real Leptos SSR projection verifies escaped public input/token/error data; no application client/hydrate/CSR output changed. Full account/auth views, browser keyboard/layout/zoom, QR/TOTP presentation and installed acceptance remain Lane A M3/owner integration points. No new account-page behavior is faked. |

## Mechanics, remaining integration and limits

- Rust mechanics: CSRF typed string overrides use the existing direct config type at startup; invalid typed overrides/duplicate services propagate native config/registration errors. Raw absent/nonobject sections supply source defaults. Existing cookie-name validation is retained.
- Native test-host wiring selects status 200 before real auth route dispatch so the existing G12 host reaches Response rendering. The finite renderer projects only public Form fields and retains prior body/redirect/status behavior. Native SQL Dates use explicit source/wire projections; snapshots retain rich values instead of forcing JSON.
- Accepted library-capability adaptation is unchanged: lettre gaps, including pool maxMessages, route complete original options/native content to an application provider or return a named error. Fake providers prove mapping/ownership, not remote pool recycling or SMTP/TLS timing/negotiation. No real SMTP/SES/sendmail/HTTP delivery occurred; the existing /bin/cat process fixture is only an inert process seam.
- Existing owned-task snapshots, weak host binding, one-shot native readers and discarded child responses remain the accepted Rust ownership mechanics. Real auth Hooks now consume the frozen Lane B supervisor interface without a change to event names or request/response shapes.
- Lane A M3 account pages/views were absent at release. Recheck their actual view selection/error/CSRF behavior around retained auth-update bodies after merge. No framework-session, framework-api, framework-schema, facade or renderer implementation was edited.
- Declared build/test/coverage target is aarch64-apple-darwin, Rust/Cargo 1.96.0. Browser/WASM/installed Tauri, live PostgreSQL/MySQL and external provider acceptance are unverified here. Core email contains native IO adapters; a server-disabled graph is not a portability claim. No listener, install, commit, push, GitHub write or other-agent message was performed.

## Gate evidence

Edit loop used the warm target, default incremental behavior, sccache and jobs=2. Gate used CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, RUSTC_WRAPPER=sccache, SCCACHE_CACHE_SIZE=20G, CARGO_BUILD_JOBS=2. Absolute target: /Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-slice9b/target/slice9b. M0 initial clean/llvm-cov cleanup was already recorded; it was not restarted. Every Cargo/proof/clean invocation is in the recorder, wrapping outer scripts only.

Compatible features: stackpress-framework-csrf/server, stackpress-framework-email/server, stackpress-framework-session/auth-server, stackpress-framework-schema/providers, stackpress-framework-api/sql and stackpress-framework-api/codegen. Renderer --all-features was never used.

| Gate invocation | Result |
| --- | --- |
| cargo fmt --all --check; cargo clippy --workspace --all-targets --features <compatible> --locked --offline -- -D warnings | Passed, including final standard-library fixture cleanup. |
| cargo nextest run --workspace --all-targets --features <compatible> --locked --offline --no-fail-fast --test-threads 2 -E 'not test(tcp_watchdog)' | 928 passed, 87 skipped, 224 binaries; no failures. Two slow consumer/compile-contract tests passed at 228.116/175.651 seconds. Outer process-group deadline 1800 seconds. |
| Final selected Lane B nextest, both server features | 58 passed: CSRF 7, email 51; includes final fixture-directory cleanup without an added external crate. Outer deadline 240 seconds. |
| Each crate separately with --no-default-features | CSRF 6 passed; email 31 passed. Standalone normal graphs independently exclude the host edges. |
| cargo test --workspace --doc --features <compatible> --locked --offline | 962 passed, including ownership compile-fail examples and new CSRF registrar example; outer deadline 900 seconds. |
| RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps --features <compatible> --locked --offline | Passed. |
| consumers/graphs.py --check; consumers/run.py | All four feature builds and all three final locked standalone graph/format/Clippy/executable proofs passed. Final consumer targets were cold after earlier 12 GiB-bound cleanups; sccache was warm; final deadline 900 seconds. |

Coverage command: cargo llvm-cov -p <crate> --features server --locked --offline --json --output-path /private/tmp/slice9b-m4-<name>-coverage.json --fail-under-lines 70. All implementation files in each selected crate are included, without custom exclusions; doctests are separately verified.

| Crate/features | Covered/executable lines | Coverage | JSON SHA-256 |
| --- | ---: | ---: | --- |
| framework-csrf/server | 67/69 | 97.10% | `ae5d4284689b20d5c59994aaf1efb76455cae5a4ea2b1c3c8705fc3f20845aa1` |
| framework-email/server | 2401/2721 | 88.24% | `46abae14f476f948d944f91574b777d0b0369cdb06ac3f83397f6947779eeef5` |

Only assigned crates/consumers, this new receipt and the two packages' own lock edges changed. Added optional CSRF->server, reused Tokio for CSRF tests, and three internal email dev links to session/auth-server, SQL and SQLite. No new external dependency/package. All **698 original external tuples and their raw lock blocks are byte-identical**, all unrelated lock packages are identical, and consumer tuples are baseline subsets. Root manifest/facade are untouched. Baseline lock SHA-256: `bca8f7ae09e4ce97ee4cd62241324a55a8a7942de4da3744433feb82d2cf6f30`.

Final audit: Passed: source regeneration, exact lock blocks, consumer tuple subsets, scope, KB baseline, whitespace and raised cap. KB diagnostics remain **146 errors/27 warnings**, with zero delta; historical Agent Files were preserved. No context promotion or owner acceptance is claimed.

**Cap ruling:** began under 12 GiB; the coordinator raised it to **20 GiB** during the six-feature build, with a **19.4 GiB** headroom guard. Both guards now use those values and retain dependency artifacts. Before that ruling, idle main debug/incremental was trimmed once (11,767,808 -> 10,936,280 KiB), and the already-verified disposable direct/event/disabled targets were cleaned as recorded; the event cleanup had already started when the new ruling arrived. No dependency cleanup followed the ruling. Final target **15,363,332 KiB (14.652 GiB)**; largest observed sample **15,363,332 KiB (14.652 GiB)**. Samples are periodic, not continuous. Final consumer and dependency artifacts are retained.

## Benchmark records

Attributable M4 delta: **32 attempts**, 28 passed and 4 failed; **12 compilation-emitting** and **7 checking-emitting** commands. Command wall total **1413.036822 s**; observed Cargo-stage subtotal **790.39 s**, incomplete and overlapping command time rather than additional time. Early failures were the known KB baseline, one source type-erasure syntax error, rich-Date test snapshots and a consumer unused import; each applicable final gate passes. Parallel-lane records are not attributed.

Store: /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks; runs/<id>.json and matching .log retain exact argv/output/environment/cache/result.

| Record ID | Invocation/result | Command s | Cargo-stage s |
| --- | --- | ---: | ---: |
| `20261030T075418.499422Z-7ef8ce9a1a78` | baseline KB diagnostics; failed (1) | 0.414406 | none |
| `20261030T075651.934551Z-06a849d66a64` | source real templates and shared response skip oracle; failed (1) | 0.213930 | none |
| `20261030T075710.950227Z-e755497c09f0` | source checked event type removal; passed (0) | 0.074485 | none |
| `20261030T075911.554900Z-f300d002291b` | edit loop format; passed (0) | 0.277799 | none |
| `20261030T075927.978993Z-bd48d7bd1d0e` | edit loop real auth integration; failed (100) | 103.449279 | 91.00 |
| `20261030T080154.348291Z-a462d33297ca` | edit loop near cap incremental trim; passed (0) | 2.061290 | none |
| `20261030T080215.604751Z-a458c56f5f1d` | edit loop native Date snapshot correction; passed (0) | 14.529182 | 10.32 |
| `20261030T080247.148091Z-deb25263adc2` | edit loop consumer formatting; passed (0) | 0.342978 | none |
| `20261030T080312.897131Z-e6175b7525fc` | edit loop isolated consumers cold targets warm sccache; failed (1) | 103.748011 | 65.41 |
| `20261030T080436.488436Z-942bec0f2801` | edit loop final crate format; passed (0) | 0.264947 | none |
| `20261030T080500.244993Z-4e38fd7637cf` | edit loop adapter feature graphs; passed (0) | 2.161275 | none |
| `20261030T080544.867567Z-135dc408ac6f` | gate isolated consumers and lock subsets; passed (0) | 60.886028 | 55.15 |
| `20261030T080712.494105Z-324a9d3bbfab` | edit loop final source formatting; passed (0) | 0.712541 | none |
| `20261030T080736.466683Z-66d97233475c` | gate workspace fmt; passed (0) | 2.581821 | none |
| `20261030T080744.493140Z-0d967099f721` | gate workspace Clippy; passed (0) | 65.117104 | 64.00 |
| `20261030T080904.293838Z-fbf64976e725` | edit loop explicit SQL renderer cleanup format; passed (0) | 0.222338 | none |
| `20261030T080911.288829Z-852c8a745363` | gate workspace nextest compatible features; passed (0) | 637.751495 | 188.00 |
| `20261030T081022.211837Z-34950ef30acd` | gate free verified event consumer target near cap; passed (0) | 0.491950 | none |
| `20261030T081858.811635Z-b2c2fb77596c` | gate final adapter dependency graphs; passed (0) | 2.026994 | none |
| `20261030T082020.111414Z-7da4ccbaa342` | edit loop dependency free fixture directory format; passed (0) | 0.379249 | none |
| `20261030T082028.232406Z-0f32466344d4` | gate final workspace fmt; passed (0) | 9.745525 | none |
| `20261030T082044.274092Z-3250050b49ac` | gate final workspace Clippy after fixture cleanup; passed (0) | 5.112515 | 4.78 |
| `20261030T082103.022130Z-00b02689101c` | gate target guard raised 20 GiB headroom 19.4 GiB; passed (0) | 1.836622 | none |
| `20261030T082149.749019Z-400679cf7047` | gate final Lane B regression without new external dependencies; passed (0) | 28.080785 | 18.99 |
| `20261030T082308.786416Z-2e887016a312` | gate all four adapter feature builds; passed (0) | 13.659699 | 9.56 |
| `20261030T082536.804274Z-170e19fb085f` | gate both server disabled test suites; passed (0) | 35.047188 | 28.16 |
| `20261030T082653.544792Z-50c708cbbf66` | gate workspace doctests; passed (0) | 53.307381 | 3.59 |
| `20261030T082807.533003Z-90ec888e56c7` | gate workspace Rustdoc warnings denied; passed (0) | 65.071835 | 64.00 |
| `20261030T082952.185762Z-d9eb700ad070` | gate email server coverage; passed (0) | 74.455016 | 69.00 |
| `20261030T083124.666028Z-9d43b31d39aa` | gate CSRF server coverage; passed (0) | 15.596132 | 13.07 |
| `20261030T083209.893239Z-93bc08b3bde7` | gate locked isolated consumers cold targets warm sccache 20 GiB cap; passed (0) | 110.592861 | 105.36 |
| `20261030T083554.888653Z-ccab68ece7d8` | gate final source lock scope KB size audit; passed (0) | 2.824161 | none |

Stop at M4 for coordinator review/independent verification. No product question is open.

## Coordinator verification

Committed as `d1492d6`, then trial/3 (API M3) was merged into the branch cleanly at `3d33ff1`. The lock kept every existing external tuple and added none; the removed temporary test helper leaves no trace. Every registry tuple in the three email consumer locks and both API checkpoint locks is in the workspace lock. The CSRF `server` registrar publishes the service only: no request listener, no routes, no global guard. Reran on the combined tree at gate tier (`sccache`, non-incremental, empty target), recorded as `s9bm4-verify gate *`. All passed:
- fmt; workspace Clippy `-D warnings` and Rustdoc `-D warnings` with csrf/server, email/server, api/sql, api/codegen, session/auth-server and schema/providers.
- `cargo nextest run --workspace --all-targets --locked --no-fail-fast` with no exclusions: default features 901 passed; with the six features 941 passed. Both include `tcp_watchdog`.
- Workspace doctests with the six features: 962 passed.
- `--no-default-features`: framework-csrf 6 passed, framework-email 31 passed.
- `consumers/graphs.py --check` (all four server-feature combinations) and `consumers/run.py` (direct, event and disabled consumers).
- Coverage: framework-csrf with `server` 97.10% lines; framework-email with `server` 88.24% lines.
- No tracked file was left dirty by the gate.

Owner question raised: the real auth-update adapter leaves a truthy results body on the shared response, so the source `email-send` handler skips sending during email sign-in. The source oracle reproduces this, so it is preserved under G43 pending the owner's answer. Merged into trial/3 on 2026-10-30; Lane B refactoring rounds 1–4 may be released.
