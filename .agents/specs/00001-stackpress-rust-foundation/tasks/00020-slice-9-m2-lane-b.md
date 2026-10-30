# Slice 9 M2 Lane B receipt

Status: implementation/gate complete; **stop at M2 for coordinator verification**. Base: d740f5cb114d1039011df5512b4b703786ed0bc4, branch trial/3-slice9-csrf-email. Authority: released Lane B M2 row and G42/G43/G44; the accepted M1 provider-gap ruling remains binding. No M3/M4 or formal post-M4 rounds are claimed.

## Delivered interface and source parity

- Optional email `server` feature exposes `plugin::register`, `plugin::register_with_factory` and reusable `events::EmailSend::handle`. The frozen event remains **email-send**, incoming aggregate request data remains the source SendMailOptions object, and provider info remains response results. No session/API/schema/shared-lib or root facade change.
- Exact source order: truthy body skip → truthy non-200 status skip → missing/falsy config return → original message options → a new transport → await callback-equivalent send → provider results. Every normal source return is Continue; errors propagate through existing dispatch without response/status invention, retries or an outbox.
- Registration contributes email-send only for truthy config, including an empty object. Missing/null/false/zero/empty-string modes are inactive. The handler also retains its independent config gate for direct use. No authorization/CSRF/method/recipient/egress/sender/global request gate was introduced.
- All per-call transports remain available: SMTP, pool, JSON, stream, sendmail, SES, string and custom provider. Existing source vectors are replayed through the production event: 18 JSON, 3 stream, 4 SMTP, 4 sendmail and 3 SES cases, plus complete-provider pool/string/LMTP cases. SMTP/process/SES delivery boundaries are fake.
- `TaskSupervisor::resolve_on(runtime, Arc<Server>, event, owned_input)` submits an owned raw resolve. Source signup names remain **email-email-send** then **auth-phone-verify**; there is no automatic alias to email-send. Explicit application handlers can reuse email-send. Parent results complete while the child send remains pending. Missing extension handlers resolve normally; child responses are discarded, while thrown errors/panics remain supervised. Shutdown aborts and reaps jobs; dropping the owner requests abort.
- Native bytes/readers stay at original content paths. `events::NativeReader` transfers a Send, potentially non-Sync reader exactly once; returned Readable stays at results.message as a native handle that JSON conversion rejects; consume it explicitly. Buffer JSON and arbitrary provider extensions retain M1 mapping. Caller-selected larger request/response data budgets are preserved.

Lane A assumptions: enable email/server when selecting the ordinary registrar; the application owns any retained host/runtime/task supervisor. No new input field, event name, response envelope or native caller-carrier contract is required. Lane A's real signup/storage integration remains its implementation responsibility; this lane proves the source-derived submission/support boundary.

## Source-derived invocation evidence

`tests/source-events.cjs` strips checked TypeScript import/type scaffolding and executes original email event/plugin, root session composite and signup handler control flow against inert seams. It generates **37 email traces, 8 registration traces, 4 signup traces** and auth → email → session order. AuthActions/phone normalization are inert boundary stubs here; full account validation/storage is not claimed from this oracle. Trace snapshots record arguments at the call boundary, before later source mutations.

Read-only source root: /Users/cblanquera/server/projects/stackpress/stackpress/packages. Fixture: crates/framework/framework-email/tests/fixtures/source-events.json; SHA-256 `39690a43434ff99356fb9ce22bfd869ae2bb9f1b64ff6a304b75df6949ba86f7`. Final audit regenerates and compares bytes.

| Original file under packages/ | SHA-256 |
| --- | --- |
| stackpress-email/src/events.ts | `4061c0714fc3fff61e65cd05d916332629bcede7f39df0fd55aac8cf7dddc65d` |
| stackpress-email/src/plugin.ts | `4ce2b3cc8a1ad5348309415ab715d278cb1d4882c1b9cf956cb5ace527410759` |
| stackpress-session/src/plugin.ts | `aade8d295a5910f7ccc4f9ac56a7c7a6ac7a8da44bd8b544c8259859bbc4a84f` |
| stackpress-session/src/auth/events/signup.ts | `db7be4e9174a25f0ac090c9f20ceffb43726ce50e2030ad007393d931682557f` |

| Verification IDs | Observable evidence |
| --- | --- |
| R09 / R21 / R25 | Original traces cover prior truthy/falsy bodies, 0/200/non-200 codes, every missing-config mode, success/falsy provider info, create/send errors and skip-before-invalid-config/error. Real host tests preserve priority/FIFO, explicit abort, normal Continue and no unrelated global listener. |
| R19 / R20 / R21 / R30 | Event replays the transport fixtures through real adapters/fake SDKs; native reader/byte slots, unfiltered results/errors, JSON-only legacy transports, provider capability errors and deliberate larger workload budgets are retained. |
| R24 / I01 / I02 | Source signup hook names/payloads/submission order, real same-event handler order, nested/direct event reuse, independent child mutation, parent non-awaiting result, separate per-call transports, root cancellation and supervisor abort/reap. Channel barriers establish order without sleeps. |
| I04 / I06 / I07 / I08 | Existing CSRF/source/adapter regression suite retained; typed provider cause chains downcast correctly. Accepted provider bindings still receive complete options or return a named capability error; no silent drop. |

## Rust-mechanics adaptations and limits

1. **Registration mechanics:** ServerBuilder has no registration-time raw config.has view and freezes typed config before Listen. The package claims an EmailSection defaulting to null, then checks source truthiness in Listen. Missing and null share this inactive default; missing config leaves a no-op startup callback rather than skipping callback insertion. Runtime listener inventory and direct-event skips match the source.
2. **Ownership/Send:** native Readables use explicit synchronized one-shot transfer; snapshots alias the transfer slot, not a cloned cursor. Owned child inputs snapshot submission-time data rather than borrowing JavaScript object aliases. Submission order and within-event registration order are preserved; independent jobs may execute/complete in either scheduler order. An owned Arc host replaces a borrowed Invocation at the spawn boundary. Cancelling cannot roll back external effects already performed.
3. **Error mechanics:** Rust Result replaces callback rejection/throw. lib's generic cause API needs a sized wrapper around boxed provider errors; the concrete provider error remains in its source chain. Existing lifecycle public-error policy is unchanged.
4. **Library capability, accepted at M1:** lettre gaps (pool maxMessages, LMTP, callback sockets/auth, separate DNS/greeting/socket timeouts and other previously recorded unsupported compiler/SDK options) still route to the configured complete native provider or return a named error. No unsupported source option is silently dropped. No new external dependency or provider restriction was added.

Fake SMTP/SES/process boundaries do not prove live delivery, inbox acceptance, HTTP/TLS interoperability or installed/browser targets. No network listener was opened. The source oracle isolates control flow; it is not original end-to-end auth/database execution. Cross-crate pages/rendering, full consumer matrix, formal refactoring rounds and docs follow-up remain at their released design milestones.

## Verification and build environment

One retained absolute warm target: /Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-slice9b/target/slice9b. Initial M0 clean/llvm-cov-clean evidence is retained in its receipt; no fresh target or mid-M2 clean. Cap remains the coordinator-raised **12 GiB**. Edit loop leaves CARGO_INCREMENTAL unset; gate uses 0. Both use CARGO_PROFILE_DEV_DEBUG=0, CARGO_BUILD_JOBS=2, RUSTC_WRAPPER=sccache, SCCACHE_CACHE_SIZE=20G. Native cargo/rustc 1.96.0, aarch64-apple-darwin; compatible workspace features only, no renderer all-features.

All Cargo/proof commands use the outer recorder exactly once: `python3 .agents/scripts/build-benchmark.py --store /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks run --label 'slice9b M2 ...' --kind ... --scope ... --cache warm -- <command>`. Tests/docs/coverage use tests/run-proof.py to kill/reap their process group on deadline. The source oracle never calls a live provider.

Initial edit-loop failures are retained: boxed cause requires a sized adapter; oracle needed its remaining TypeScript generic stripped; a native JSON test expected Buffer JSON where the existing source adapter base64-encodes attachments. Final review also caught default-budget reapplication in event conversions; the selected-budget correction and regression passed, then the full gate reran. Formal rounds remain at the accepted post-M4 gate, not advanced by this ordinary implementation review.

| Final gate command after recorder/watchdog prefix | Result |
| --- | --- |
| cargo fmt --all -- --check | Passed. |
| cargo clippy --workspace --all-targets --features stackpress-framework-email/server --locked --offline -- -D warnings | Passed. |
| cargo nextest run --workspace --all-targets --features stackpress-framework-email/server --locked --offline --no-fail-fast --test-threads 2 -E 'not test(tcp_watchdog)' | **865 passed, 0 failed, 85 skipped**, 224 binaries. Includes all **50 Lane B tests** (6 CSRF, 44 email). Only directed tcp_watchdog exclusion plus existing ignored/environment/feature-gated skips. |
| cargo test --workspace --doc --features stackpress-framework-email/server --locked --offline | **955 passed, 0 failed, 0 ignored**; includes four email compile-fail ownership/Send contracts. |
| RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps --features stackpress-framework-email/server --locked --offline | Passed. |
| cargo nextest run -p stackpress-framework-email --no-default-features --locked --offline --no-fail-fast --test-threads 2 | Passed; 31 tests with the server feature disabled. |
| run-proof.py --timeout 15 -- cargo nextest run -p stackpress-framework-email --features server --locked --offline --no-fail-fast --test-threads 2 -E 'test(event)' | 14 event/ownership tests passed within the absolute process-group deadline. |
| /private/tmp/slice9b-m2-feature-graphs.py | Passed: root default normal graph still only stackpress-lib; email core omits lib/server; explicit email/server adds both; no current normal dependent requires migration. Exact nested cargo tree commands retained in the recorder log. |

Coverage commands are `cargo llvm-cov -p <crate> [--features server for email] --locked --offline --json --output-path /private/tmp/slice9b-m2-<name>-coverage.json --fail-under-lines 70`, gate environment, 600-second watchdog. All first-party implementation files in each selected crate/feature are measured; no custom exclusions. Doctests are verified separately.

| Crate / features | Covered/executable lines | Line coverage |
| --- | ---: | ---: |
| framework-csrf / default | 52/54 | 96.30% |
| framework-email / server | 2387/2721 | 87.73% |

New event/support file coverage: events.rs: 34/34; events/native.rs: 102/111; plugin.rs: 32/34; tasks.rs: 34/34. Remaining gaps include native poison/failure paths, OS entropy/provider errors, coercion branches and existing live SDK/IO branches. Coverage is native line evidence, not branch or live-provider acceptance.

Raw JSON SHA-256: csrf `952168197a87703ec740f773e950b2f04463738625cacfb3560f66a0cb855820`; email `92fadd31b0c3fa2bcf61b3bf832f6ed4244e8216198595ff0307d55d2d46fa2e`. Raw files remain at the /private/tmp paths above.

## Scope, resources and benchmark records

Only framework-email implementation/tests/README, its two internal Cargo.lock edges and this assigned receipt changed. CSRF code is unchanged and regression/coverage verified. No root Cargo.toml/src/lib.rs or foreign crate edit. **All 698 existing external package tuples and every unrelated lock entry stay identical; 0 new packages.** Base lock SHA-256 `d9d0d29f1e637ffbf3bb38a3971f4971c893300e5b70a766dc330cadcdcc3f56`. No cargo update/generate-lockfile.

Final target sample **9,694,336 KiB (9.245 GiB)**; largest sampled allocation **9,694,336 KiB**, below 12 GiB. Sampling was periodic, not a continuous peak measurement. Existing KB baseline remains **146 errors/27 warnings**; historical records were preserved. Final comparative audit passed: 0 new KB diagnostics, all 698 external tuples and every unrelated lock entry identical, exact owned scope, reproducible source fixture, diff whitespace clean and the 12 GiB guard passed.

No commit/push, install, GitHub write, other-agent message, live SMTP/SES/sendmail or network listener was performed. No owner/product question or shared interface change is outstanding. Stop at M2.

Attributable M2 recorder delta: **27 attempts** (24 passed, 3 failed), **12 commands emitted compilation**, **4 emitted checking**. Shared store: /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks/runs/ with matching JSON/log files. Records from parallel lanes are not attributed here. Recorded command wall total **607.207466 s**; observed Cargo-stage subtotal **239.55 s**, incomplete for nested proof output and not additional elapsed time.

| Record ID | Invocation / result | Command s | Cargo-stage s |
| --- | --- | ---: | ---: |
| `20261030T060435.924265Z-7381a2cb44f2` | edit loop event compile; failed (101) | 10.921055 | none |
| `20261030T060558.908887Z-1bde0ff7ecf4` | original source event invocation oracle; failed (1) | 0.160312 | none |
| `20261030T060631.090039Z-177fb3d844d9` | source invocation oracle corrected type stripping; passed (0) | 0.061450 | none |
| `20261030T060742.980556Z-f9cce9634a48` | edit loop formatting; passed (0) | 0.248286 | none |
| `20261030T060807.541475Z-81de46bfb6c9` | edit loop event traces and registration; passed (0) | 6.657325 | 4.04 |
| `20261030T060932.630765Z-5eb8b18d0b2e` | edit loop native hooks format; passed (0) | 0.215083 | none |
| `20261030T060946.629059Z-1217c1db4960` | edit loop all event and transport proofs; failed (100) | 5.918896 | 3.6 |
| `20261030T061044.433205Z-6bd731993cbd` | edit loop source JSON expectations and docs format; passed (0) | 0.222201 | none |
| `20261030T061054.963695Z-97b39ad4dd64` | edit loop complete parity and ownership; passed (0) | 5.138186 | 2.85 |
| `20261030T061151.739303Z-e6d81d4594bf` | gate workspace fmt check; passed (0) | 2.363013 | none |
| `20261030T061205.552209Z-2565bf43d1d9` | gate workspace Clippy with email server; passed (0) | 62.347484 | 62.0 |
| `20261030T061337.297585Z-6c2d7cc27f25` | gate full workspace nextest with email server; passed (0) | 265.426794 | 92.0 |
| `20261030T061814.039274Z-a42eb8d31671` | gate workspace doctests with email server; passed (0) | 41.838385 | 3.09 |
| `20261030T061930.231689Z-cb11316fc787` | gate workspace Rustdoc with email server; passed (0) | 38.596777 | 38.43 |
| `20261030T062122.658578Z-746cfb69e972` | edit loop preserve caller budgets format; passed (0) | 0.249752 | none |
| `20261030T062133.366211Z-375dd6c771f1` | edit loop caller budget regression; passed (0) | 5.954671 | 3.4 |
| `20261030T062211.728311Z-b4bebc23f143` | gate final workspace fmt; passed (0) | 2.370604 | none |
| `20261030T062219.394733Z-4969e70af9cc` | gate final workspace Clippy; passed (0) | 3.358201 | 3.12 |
| `20261030T062234.481547Z-aaf7b65b2efc` | gate final full workspace nextest; passed (0) | 72.249957 | 3.69 |
| `20261030T062402.725841Z-d7e619d2130c` | gate final workspace doctests; passed (0) | 43.681266 | 1.36 |
| `20261030T062459.418126Z-e1bed25625b2` | gate final workspace Rustdoc; passed (0) | 3.551928 | 3.42 |
| `20261030T062508.181958Z-f3962bef66de` | gate email server coverage; passed (0) | 9.825730 | 7.19 |
| `20261030T062630.888636Z-7a929b0a7a8a` | gate CSRF coverage; passed (0) | 3.940209 | 1.69 |
| `20261030T062705.348975Z-c0b997112875` | gate server-disabled email regression; passed (0) | 9.686235 | 5.81 |
| `20261030T062724.580186Z-8632b1b825d6` | gate optional feature and dependent graphs; passed (0) | 4.979431 | none |
| `20261030T062736.177109Z-7e1447d22d9d` | gate event ownership proofs absolute 15 second bound; passed (0) | 6.160835 | 3.86 |
| `20261030T062831.058774Z-0bb27e7f45bd` | gate final source lock scope KB and size audit; passed (0) | 1.083400 | none |
