# Slice 9 M3 Lane B receipt

Status: **M3 Lane B complete; stop for coordinator verification**. Started from 8f8ced4 (verified M2), branch trial/3-slice9-csrf-email. Scope is the released M3 Lane B row. G43 source behaviour and the accepted M1 provider-capability ruling remain binding. No production implementation, public interface, global guard or other lane's file changed.

## Delivered finite integration

Five new integration tests join the **real** framework-csrf Service, framework-email event/adapters, stackpress-server lifecycle, framework-view PageSnapshot/PageRenderer and stackpress-leptos SSR. All live SDK/process seams are fake. Added only three internal email dev-dependencies: framework-csrf, framework-view (leptos-ssr) and stackpress-leptos (ssr). Existing external dependencies/normal default graphs stay unchanged.

| Owned proof | Behaviour proved |
| --- | --- |
| tests/cases/integration/mod.rs | Original page-boundary trace matrix; generate-before-valid compares incoming cookie/data while outgoing token changes; invalid 419 retains new SSR/cookie token; OTP success clears then redirects; magic success retains the form; missing email/sender no-ops; password page overrides and auth-signin reuse; returned child error propagation versus thrown native error. |
| tests/cases/integration/rendering.rs | Real Leptos SSR and inert PageSnapshot JSON contain the intentionally public CSRF name/token and match the outgoing cookie; original error status is restored after HTML staging; email input uses escaped text/attribute bindings; explicit public projection omits SMTP/native factory resources. No browser/installed or full auth-view acceptance is claimed. |
| tests/cases/integration/transports.rs | Nine source option configurations pass the finite CSRF/form boundary and direct email-send: SMTP; pool provider; JSON object/string; stream Buffer/Readable; sendmail; SES; string SMTP. Exact original config/message fields reach each per-call factory. Real MIME/transport adapters run against fake SMTP/process/SES; the explicit full provider receives the pool's complete options. Separate direct calls prove per-call construction and native stream consumption. |
| tests/cases/integration/support.rs | Inert SMTP/SES/sendmail and complete-provider capture seams retain actual native option/MIME mapping without an external SMTP server, process delivery or external account. No policy or transport variant is suppressed. |

The nine option fixtures include configured auth/TLS choices, pool maxConnections/maxMessages/socketTimeout/extensions, JSON skipEncoding, stream buffer/newline, explicit sendmail path/argv, native SES binding and decoded SMTP URL auth/query. Two sends per configured host prove independent construction. SMTP encoded subject/envelope/options, sendmail program/argv, SES destination, JSON message content and stream MIME are inspected. M1/M2's full source transport/option vectors remain regression tests in the same 55-test Lane B suite.

## Source oracle and fidelity boundary

`tests/source-page-boundaries.cjs` executes the **original** email-signin page and CSRF plugin after checked TypeScript scaffolding removal, with finite auth/session/template/email seams. Its 11 cases cover guest/signed-in GET, invalid CSRF, OTP/magic success, returned child 419, provider throw, missing email config/sender, and password success/401. Unexpected oracle exceptions now fail generation instead of becoming accepted fixture data. The final audit reruns it and compares fixture bytes.

Read-only original root: /Users/cblanquera/server/projects/stackpress/stackpress/packages. Fixture: tests/fixtures/source-page-boundaries.json; SHA-256 `b6bb8430ab8a5fc00b035f39f59c0780cbfafbc822d10d2a506d44d7a9118114`.

| Original file under packages/ | SHA-256 |
| --- | --- |
| stackpress-session/src/auth/pages/signin/email.ts | `147c76378a43fddf8ccdcc05352909bbd7a68f4fcb039a16a066f401d222efbc` |
| stackpress-csrf/src/plugin.ts | `55f589f6d9d9c63d37e751ca1e550b932c4b893de3f65f263a2590286e5792e2` |

The oracle fixes the entropy/random/auth-reload/template seams so its output is deterministic. Rust uses real OS entropy and correlates the generated token across response data, cookie, rendered hidden input and hydration seed. The fake Rust page enters at the frozen auth/mail boundaries; it does not reimplement Lane A's pending storage/challenge/template/page code. Provider info/request shapes and event names stay as accepted at M2.

| Verification mapping | M3 evidence |
| --- | --- |
| R04/R10/R11/R12/R22 | Real incoming/outgoing CSRF isolation, original 419 text/status, explicit clear/redirect order, intentionally rendered token and cookie correlation; existing arbitrary-string/query/post/cookie tests retained. |
| R09/R19/R20/R21/R24/R25/R30 | Existing email skip/order/error tests plus finite form/direct reuse, all transport variants, original options/results and no automatically added CSRF/auth guard on direct email-send. No new cap, retry or producer policy. |
| I01/I02/I04/I05/I06/I07/I08 | Existing owned/supervised task and cancellation proofs retained; new real renderer/host/provider composition, native stream ownership, complete provider routing, private resource exclusion from explicit SSR data and public status retention. Fake success is not remote acceptance. |
| R28/S9-UI01–06 | Only finite Lane B SSR/form/error boundaries are proved here. Actual auth/account page inventory, browser keyboard/layout/200% zoom, QR/TOTP and installed target acceptance remain Lane A integration/owner work, explicitly unreleased or absent from this tree. |

## Rust mechanics and Lane A integration points

- The finite native operation selects status 200 before deferred Response rendering. Existing Server requires an operation body/status before Response; source page code is exercised separately by the oracle. This is test-host wiring, not a production activation or protection rule.
- **Returned status versus thrown error:** a returned child 419 keeps staged CSRF/data for the renderer. A raw Rust Result failure enters the existing G12 fresh-error replacement, clearing partial buffered data/cookies and returning sanitized 500, with one error offer and the original provider cause in the developer report. The JS oracle captures the page *before* that host boundary. The test explicitly distinguishes these stages; it does not claim original Node top-level error middleware was executed.
- Lane A must replace the finite page/auth-signin fakes with its real auth pages/events, account lookup/rotate/reload, native Date/hash challenge and original templates. The proof uses only frozen **auth-signin** and **email-send** shapes; pending auth-search/auth-update and actual page integration are not claimed. No framework-session edit or alternate shared interface was needed.
- Recheck real page error rendering and token/cookie handling around raw provider exceptions when Lane A merges. This is a composition observation about the existing host contract, not a new framework policy or a requested shared-code change. Source composite registration/default CSRF selection and full consumer/feature integration remain at M4.
- Accepted library-capability adaptation is unchanged: pool/maxMessages and other lettre gaps receive the full native provider binding or a named unsupported error. The pool fake proves the complete binding and per-call contract, not real SMTP pool recycling. Native Readable outputs keep M2's explicit one-shot transfer contract.

No new source defect or product choice was identified. No actual SMTP/SES/sendmail/HTTP/TLS acceptance, network listener, browser/WASM or installed platform proof is claimed. Formal refactoring rounds remain after M4 as specified; ordinary review corrected only this milestone's finite test/oracle wiring.

## Verification and resource accounting

Environment: one absolute warm target /Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-slice9b/target/slice9b; original M0 clean receipts retained, no new target/effort reset. Edit loop leaves CARGO_INCREMENTAL unset; gate uses 0. Both use CARGO_PROFILE_DEV_DEBUG=0, CARGO_BUILD_JOBS=2, RUSTC_WRAPPER=sccache, SCCACHE_CACHE_SIZE=20G. Native cargo/rustc 1.96.0, aarch64-apple-darwin. No renderer --all-features.

Every Cargo/proof/size-guard command is recorded once via `python3 .agents/scripts/build-benchmark.py --store /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks run --label 'slice9b M3 ...' --kind ... --scope ... --cache warm -- <command>`. Tests/docs/coverage use tests/run-proof.py with process-group kill/reap deadlines (focused 180–240 s, workspace 1800 s, docs 900 s, coverage 600 s). Ordering uses direct traces/channel barriers, not sleeps.

Early failed integration attempts and oracle corrections are retained below. The first oracle omitted req.data.path<string> erasure and initially captured that unexpected exception; final generation has strict expected-error assertions. The finite host also needed explicit initial status and stage-specific G12 error expectations. These were fixture corrections; production code did not change. The final full workspace run passed with two nextest workers despite one slow existing consumer compile fixture.

| Final gate command after recorder/watchdog prefix | Result |
| --- | --- |
| cargo fmt --all -- --check | Passed. |
| cargo clippy --workspace --all-targets --features stackpress-framework-email/server --locked --offline -- -D warnings | Passed. |
| cargo nextest run --workspace --all-targets --features stackpress-framework-email/server --locked --offline --no-fail-fast --test-threads 2 -E 'not test(tcp_watchdog)' | **870 passed, 0 failed, 85 skipped**, 224 binaries, one slow existing consumer fixture. Includes all **55 Lane B tests** (6 CSRF, 49 email). Only directed tcp_watchdog exclusion plus existing ignored/environment/feature skips. |
| cargo test --workspace --doc --features stackpress-framework-email/server --locked --offline | **955 passed, 0 failed, 0 ignored**; existing four email compile-fail contracts retained. |
| RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps --features stackpress-framework-email/server --locked --offline | Passed. |
| cargo nextest run -p stackpress-framework-email --no-default-features --locked --offline --no-fail-fast --test-threads 2 | 31 core tests passed with server disabled. |
| run-proof.py --timeout 15 -- cargo nextest run -p stackpress-framework-email --features server --locked --offline --no-fail-fast --test-threads 2 -E 'test(integration)' | All five finite integration tests passed within the absolute process-group bound. |
| /private/tmp/slice9b-m3-feature-graphs.py | Passed: root default normal graph remains stackpress-lib only; email core omits lib/server; explicit email/server includes them; no current normal dependent migration. Exact four Cargo tree commands are retained in its recorder log. |

Coverage: `cargo llvm-cov -p <crate> [--features server for email] --locked --offline --json --output-path /private/tmp/slice9b-m3-<name>-coverage.json --fail-under-lines 70`. All first-party implementation source in the selected crate/feature is measured; no custom exclusions; test helpers are not production coverage. Doctests are measured separately. The production files are unchanged, so the earlier native SDK/entropy/coercion failure gaps remain.

| Crate/features | Covered/executable lines | Line coverage |
| --- | ---: | ---: |
| framework-csrf/default | 52/54 | 96.30% |
| framework-email/server | 2401/2721 | 88.24% |

Coverage JSON SHA-256: csrf `cb749e4c6c44f056fd2eda45fe4050f8e7036c8764239f7f477501d10a77d1c5`; email `911e0dc764d01c7fe749bdd0acd788ce61046fff4c5651c94820b7c5d9789f22`. Fresh raw files remain at the /private/tmp paths above.

Only framework-email's manifest/tests, its three internal Cargo.lock dev-edges and this assigned receipt changed. No production src file, root manifest/facade or foreign crate edit. **All 698 original external tuples and every unrelated lock entry are identical; zero new packages.** Base lock SHA-256 `cc44293662cda052599bbab44a9f329515504364a99012265734fb7a377393a6`. No cargo update/generate-lockfile. Final audit passed: source fixture regeneration is byte-identical; all 698 external tuples/every unrelated lock entry are identical; scope, diff whitespace, 12 GiB guard and unchanged KB baseline passed.

Final target sample **10,986,484 KiB (10.478 GiB)**; largest sampled **10,986,484 KiB**, under the coordinator's **12 GiB** cap. The guard trims only idle debug/incremental above 11.5 GiB; incremental trimming occurred: **False**. Sampling was periodic, not continuous. KB baseline remains 146 errors/27 warnings with zero new diagnostics; no historical KB record was edited. No commits/pushes, installs, other-agent messages, GitHub writes or network listeners. Stop at M3.

## Benchmark records

Attributable M3 delta: **24 attempts** (20 passed, 4 failed); **11 emitted compilation**, **1 emitted checking**. Shared store runs/<id>.json and matching .log remain under /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks. Parallel-lane records are not attributed. Command wall total **510.955975 s**; observed Cargo-stage subtotal **211.59 s**, incomplete for nested output and not additional time.

| Record ID | Invocation/result | Command s | Cargo-stage s |
| --- | --- | ---: | ---: |
| `20261030T065520.756042Z-62b66c0e6f7a` | source page CSRF email boundary oracle; passed (0) | 0.363715 | none |
| `20261030T065837.819799Z-da43121cfaf2` | edit loop integration formatting; passed (0) | 0.342612 | none |
| `20261030T065849.850315Z-f54e27191024` | edit loop cross crate integration; failed (100) | 38.503638 | 35.56 |
| `20261030T070022.135031Z-28c47625301b` | edit loop inspect finite lifecycle failures; failed (100) | 5.674920 | 3.67 |
| `20261030T070147.481469Z-7b0b7ab78fd9` | edit loop native lifecycle fixture adaptation; failed (100) | 8.232438 | 4.91 |
| `20261030T070213.365500Z-7b9d70315265` | edit loop render failure diagnostic; failed (100) | 8.620931 | 6.34 |
| `20261030T070317.615936Z-eabc3685228f` | source oracle complete checked type erasure; passed (0) | 0.602336 | none |
| `20261030T070330.753888Z-90d05c1d88a3` | edit loop original source integration corrected oracle; passed (0) | 8.121786 | 4.42 |
| `20261030T070443.315181Z-8ab7502e7a6f` | edit loop final integration format; passed (0) | 0.260318 | none |
| `20261030T070452.298481Z-03575ab08235` | edit loop all Lane B regression; passed (0) | 9.348732 | 5.02 |
| `20261030T070603.564852Z-0b19f8f78085` | edit loop final fixture explanation format; passed (0) | 0.460254 | none |
| `20261030T070610.385045Z-2336660ceb49` | gate workspace fmt; passed (0) | 3.341636 | none |
| `20261030T070618.937308Z-466c7b84de71` | gate workspace Clippy; passed (0) | 6.696800 | 6.4 |
| `20261030T070739.608148Z-dfc4161c370e` | gate workspace nextest; passed (0) | 161.082845 | 5.48 |
| `20261030T071059.911997Z-c09a8b9131a4` | gate workspace doctests; passed (0) | 101.740142 | 2.56 |
| `20261030T071304.293563Z-53eee89c8df5` | gate workspace Rustdoc; passed (0) | 4.590958 | 4.44 |
| `20261030T071418.197599Z-4eaac5807324` | gate pre coverage target size guard; passed (0) | 0.859902 | none |
| `20261030T071425.469547Z-d2fa4fa31fea` | gate email server coverage; passed (0) | 118.341649 | 112.0 |
| `20261030T071649.838057Z-960be3375185` | gate post email coverage target guard; passed (0) | 1.700425 | none |
| `20261030T071657.597757Z-df48e82d32dc` | gate CSRF coverage; passed (0) | 3.169457 | 1.0 |
| `20261030T071730.701794Z-5a3937206e65` | gate server disabled regression; passed (0) | 16.963233 | 14.31 |
| `20261030T071824.621092Z-7eed6fbc3a77` | gate feature and dependent graphs; passed (0) | 1.540941 | none |
| `20261030T071855.236308Z-6aa620a1de0d` | gate finite integration absolute 15 second bound; passed (0) | 8.122139 | 5.48 |
| `20261030T072054.068137Z-7ce9118a8028` | gate final source lock scope KB size audit; passed (0) | 2.274168 | none |
