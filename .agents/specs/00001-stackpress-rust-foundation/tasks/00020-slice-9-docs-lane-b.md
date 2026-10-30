# Slice 9 documentation Lane B receipt

Status: **Docs follow-up complete; stop for coordinator verification**. Started from 595b89b on trial/3-slice9-csrf-email. Scope is the released CSRF/email documentation row. Used the installed chrisai-docing skill's guided-learning, API-reference and formatting guidance; first-use pages lead with local success, while lookup pages stay separate. Acceptance criteria: none for this technical documentation gate; owner/product acceptance remains separate. No implementation issue was discovered or fixed. No further milestone was started.

## Documentation delivered

| Surface | Deliverable |
| --- | --- |
| Both crate README.md files | Short entry points with reading paths to first success, concepts, tasks, lookup and source notes. |
| CSRF docs | Getting started; incoming/outgoing concepts; register/configure and explicit protected-process guides; Service/config/default/error and plugin API reference; source behavior and verification. |
| Email docs | Getting started; configuration/per-call/native/task concepts; all transports, configuration/auth/TLS/content, native content and event/hook guides; seven public-surface API pages plus index; source behavior and verification. |
| Email docs/check.py and claims.json | Explicit fence classification, local path/anchor/structure checks, 18 source/test claim anchors and external exact-example compilation/finite execution. Source evidence links stay local and resolve without network access. |

There are **27 Markdown pages and 21 fenced Rust examples** across the two entry points and docs trees. Source mappings include SMTP, pool, JSON object/string, stream Buffer/Readable, sendmail, SES and connection strings, plus activation/defaults, auth/TLS/client identity, provider bindings, access flags, native slots/results and supervised hooks. Existing detailed README option/capability/native behavior notes were retained in the email source-behavior page, with stale milestone wording removed.

Both required G43 observations are explicit in entry/concept/guide/source pages: **the auth-update results body causes email-send to skip** in the shared auth response, while fresh resolves still send; **the CSRF registrar adds no global guard, and routes call the service themselves**. Neither is described as a bug. G42 priority/FIFO/abort and application-owned route composition remain intact. The accepted lettre provider-or-named-error capability mapping remains a Rust-mechanics adaptation. No response clearing, hardening or new policy is proposed.

## Example and claim verification

Every Rust fence has a direct/server marker and is copied unchanged into its own Rust module. Modules compile in two disposable consumer workspaces outside the repository, using copies of the existing direct/event manifests with absolute path dependencies and **byte-identical existing lock snapshots**. Their targets are absolute target/slice9b/docs-direct and docs-server; they cannot reuse each other's same-named binary. No repository manifest, source or test is generated/edited for the docs.

Final exact-fence record **20261030T101945.563381Z-7faad7113f51** passed: **21/21 Rust fences compile**, with **14 direct and 1 server finite examples executed successfully**. All other functions accepting caller/runtime/host/PEM state are compile-checked without invoking external IO. Final external workspace: `/private/tmp/slice9b-docs-m02iz5fx`; its examples.json maps each source page/line and exact fence SHA-256 to the generated consumer module. No source-example scaffold import supplies a missing name to another example.

The checker verifies one H1, heading orientation, tagged/classified fences, blank-line spacing, every local Markdown path/anchor and all 18 documented claim anchors against implementation/regression-test names. Full manual review covered entry/concept pages, guide sequencing, public method parameters/returns/errors, option selection/defaults, native ownership/cancellation and platform/provider claims. Four in-memory negative controls were rejected: unclassified fence, missing target, missing heading anchor and source-claim drift. No file was modified by those negative controls. Claim anchors detect drift; they are not a claim of automatic semantic proof of arbitrary prose.

The first link pass found a moved consumer link; it was corrected. The first external server compile found a missing ServerInvocationExt import in the CSRF handler example; only the example was corrected. The extension trait and why it supplies service are now explicit. These were documentation defects, not implementation issues; no hidden code fix or product choice was made. All final examples are rustfmt-formatted plain Markdown, without hidden scaffold imports or ignored snippets.

## Final gate

Gate record **20261030T100318.045332Z-115b25c0f8c9** passed, using the six compatible features: csrf/server, email/server, session/auth-server, schema/providers, api/sql and api/codegen (full package-qualified feature names are retained in the recorder).

| Check | Result |
| --- | --- |
| Workspace fmt --check and all-target Clippy -D warnings | Passed |
| Workspace all-target nextest default / six compatible features | 900 / 940 passed; 86 / 88 skipped; 224 binaries; tcp_watchdog excluded in both |
| Workspace doctests with compatible features | 962 passed, including the four email ownership compile-fail contracts |
| Workspace Rustdoc --no-deps with -D warnings | Passed |
| Feature graphs --check | All four CSRF/email default/server combinations and minimal SDK graph assertions passed |
| Separate --no-default-features suites | CSRF 6 passed; email 31 passed |
| Existing standalone direct/event/disabled consumers | Graph/lock subset, fmt, Clippy and executable proofs passed |

All deadlock-class/consumer proofs use the existing external process-group kill/reap wrapper: 900 seconds for each external example run, 1800 seconds for the complete workspace gate. No renderer --all-features was used. No UI/client output changed; browser/installed acceptance is not inferred from this native documentation gate.

All Cargo/proof/clean commands were recorded at the outer script boundary with --locked --offline wherever supported; cargo fmt/rustfmt have no such flags. Gate environment: CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, RUSTC_WRAPPER=sccache, SCCACHE_CACHE_SIZE=20G, CARGO_BUILD_JOBS=2. The effort target remained warm; no fresh-effort clean was repeated. The first external-example record labels the overall effort warm, but its newly added docs-direct/server targets were initially empty with warm sccache. Subsequent external checks reuse those targets. Independent main-workspace and external-consumer checks used different target directories.

Coverage was rerun on unchanged first-party Rust implementation using `cargo llvm-cov -p <crate> --features server --locked --offline --json --output-path /private/tmp/slice9b-docs-<crate>-coverage.json --fail-under-lines 70`. No custom exclusions; plain Markdown consumers are separately compiled/executed, not counted as production coverage.

| Scope | Covered/executable lines | Coverage | Report SHA-256 |
| --- | ---: | ---: | --- |
| framework-csrf/server | 67/69 | 97.10% | `c0a8a807ba3a2dd266902f21d4dcab28bae8cfb8d845ac980b961663ad523ba5` |
| framework-email/server | 2423/2739 | 88.46% | `c8e7b4047e70e62fdcf114b393e7205a8c2c36ef2464a119a33ffbb805ebabe1` |

## Scope, resource and limits

Final audit passed: **67 existing non-README crate files** are byte-identical to 595b89b, including production source, tests, manifests and consumer locks. Root Cargo.toml/Cargo.lock/facade are byte-identical; all 698 external lock tuples remain with no additions. Root lock SHA-256: `290304ae3ac6b038058ce5c2ed3c412f9f31256ac835bb560a5250bbfa5b2c67`. Scope is only the two README/docs trees and this new receipt. Final example hashes match the external compiled report. KB diagnostics remain 146 errors/27 warnings, zero delta; historical records were preserved.

The released **20 GiB cap / 19.4 GiB guard** remains active. Largest recorded sample **16,428,064 KiB (15.667 GiB)**; final sample **16,424,624 KiB (15.664 GiB)**. Sampling is periodic, not continuous. No target clean or manual dependency/incremental deletion ran. Existing consumer targets plus new docs-direct/server targets remain warm for coordinator verification.

Temporary external example workspaces are retained with manifests, lock snapshots, generated modules and examples.json path/line/SHA-256 maps for coordinator review. They are compile/finite-local evidence, not live SMTP/TLS negotiation, SES/sendmail delivery, remote HTTP, pool recycling, PostgreSQL/MySQL, browser/WASM or installed Tauri acceptance. All fake delivery acknowledgements are identified as fakes; no listener, install or external delivery ran. No commit, push, GitHub write, other-agent message, foreign crate edit or historical Agent File edit occurred. The source auth-update/email-send question remains for the owner; current source behavior is documented unchanged.

## Benchmark records

Attributable docs delta: **17 recorded attempts**, 13 passed and 4 failed; **5 compilation-emitting** and **4 checking-emitting** commands. Aggregate command wall total **1327.597191 s** (some commands ran concurrently); observed Cargo-stage subtotal **349.29 s**, incomplete and overlapping command time rather than additional time. Recorded start/end span **2291.103 s**, including writing/review/tool gaps. Failed records are the inherited KB baseline, moved link, missing example extension-trait import and an audit list-order comparison corrected to key by page/line with unchanged exact hashes; all applicable final gates pass. Package-cache waits and shared-machine load are included in wall time.

Store: /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks. Every runs/<id>.json and matching log retains exact command/cache/environment/result; nested Cargo argv are printed by the outer proofs. Other lanes' records are excluded.

| Record ID | Invocation/result | Command s | Cargo-stage s |
| --- | --- | ---: | ---: |
| `20261030T094554.224487Z-50ef016256ff` | baseline target; passed (0) | 2.348736 | none |
| `20261030T094605.580825Z-0ec236ea9cec` | baseline KB; failed (1) | 0.439663 | none |
| `20261030T095558.909843Z-11beca23cb68` | edit links claims structure; failed (1) | 0.081320 | none |
| `20261030T095613.652116Z-d83049cb98f3` | edit corrected links claims structure; passed (0) | 0.104117 | none |
| `20261030T095645.827742Z-4255c1127dba` | edit external examples compile and finite runs; failed (1) | 145.106427 | 82.52 |
| `20261030T095913.486400Z-6a82bd4bf5c9` | check negative controls; passed (0) | 0.634312 | none |
| `20261030T100009.431216Z-4f3af77a412d` | edit example formatting; passed (0) | 0.847196 | none |
| `20261030T100018.516197Z-b24a46a13628` | gate exact formatted external examples; passed (0) | 41.613235 | 34.30 |
| `20261030T100242.320375Z-22a65f8c8090` | gate final Markdown example format; passed (0) | 7.522965 | none |
| `20261030T100318.044554Z-7a893566de32` | gate final external fences; passed (0) | 181.692218 | 101.54 |
| `20261030T100318.045332Z-115b25c0f8c9` | gate workspace consumers docs coverage; passed (0) | 913.164830 | 122.29 |
| `20261030T101401.140540Z-358e869bb9c7` | gate final links claims negative controls; passed (0) | 0.213675 | none |
| `20261030T101408.766066Z-b02fc6fd3740` | gate final links claims structure; passed (0) | 0.157781 | none |
| `20261030T101945.563381Z-7faad7113f51` | gate final external examples and target guard; passed (0) | 18.643213 | 8.64 |
| `20261030T102221.006707Z-d829c4d9a325` | gate final target size; passed (0) | 4.263991 | none |
| `20261030T102243.769894Z-3143aaeb5202` | gate final scope source lock KB audit; failed (1) | 6.648844 | none |
| `20261030T102400.887271Z-f31b754855d6` | gate final keyed scope source lock KB audit; passed (0) | 4.114668 | none |
