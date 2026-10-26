# Slice 5 lane A — refactor rounds 1–4

Task / crate / feature / target: task 00008; lane A, base `307074d`; branch `trial/3-s5-m0`; macOS ARM64 native SSR and wasm32 hydrate/CSR checked separately. Sources: [round template](../../references/00037-refactoring-round-checklists.md), [Rust workflow](../../workflows/rust-programming.md), [coordinated development §5.4](../../workflows/coordinated-crate-development.md), [accepted verification contract](slice-5-design-verification.md).

This is implementation evidence for coordinator review. The coordinator reported M4/native M3 merged and owner visual acceptance before releasing these rounds. This receipt does not replace that acceptance or authorize commit/publication.

## Intended behavior and baseline

Preserve the frozen public API, public wire shape/build ID, generated Profile operation, plugin order, shared UI, routes, lifecycle/error behavior, independent target modes and checked static gateway. Only confirmed defects and the coordinator-approved Head error disposition change failure behavior.

Read the current context/style/quality/workflows and accepted design; inspected framework-view, framework-ui render/routing, web/gateway, shared route/customization source, build/check scripts, README and collaborators. Current tests and installed dependency implementation were used as evidence. No other trial branch was read or built.

Baseline: `baseline.log` passes view/server, UI/SSR, web and gateway tests/doctests before edits. Started from a clean worktree, and ran `cargo clean` plus `cargo llvm-cov clean --workspace` in the owned native target. All evidence paths below use prefix `/private/tmp/stackpress-s5-a-rounds-`.

## Round 1 — logic and complexity

Findings and changes:

- **Defect:** Python asset validation read whole files before checking sizes and accepted a symlinked manifest/output root that native startup rejects. Regression tests failed first (`r1-regression-before.log`). Metadata/type admission now precedes bounded reads, and a second byte check handles growth after metadata. Nonregular files, including FIFOs, are rejected before opening.
- Counts and remaining total bytes are enforced during inventory traversal rather than after all reads. Directory traversal no longer collects/sorts an unbounded path list before admission; only the bounded accepted inventory is sorted for its deterministic hash. Valid manifest bytes and identities remain unchanged.
- Reviewed native page delivery/Abort/error precedence, one-lifecycle delegation, typed HTTP input, default/selected plugin ordering, percent encoding and checked status/settings. Existing exhaustive lifecycle tests cover these paths; no speculative rewrite was warranted.

Tests rerun / result: `r1.log` passes the initial four Python cases; the unchanged Rust baseline passes. Later rounds expanded the checker table to seven cases and repeated it in the final full sequence.

## Round 2 — responsibility and abstraction

Findings and changes:

- Framework-view remains a renderer/host-independent contract with generic public data, checked identities/status/locations, finite policy and an optional native server extension. Framework-ui render/routing depend on those contracts and Leptos, with no Profile type, operation name, ID default or data schema. Its generic tests use unit, String and now bool page data. The en/light initial defaults are accepted general UI settings, not Profile-domain policy.
- The example owns Profile projections, `abc` fallback, return-query policy, the fixed search endpoint and selected `Demo: ` customization. No domain logic was moved into framework crates. Shared route grammar remains a compiled data definition; its query/percent codec remains example-owned.
- Web startup now calls its documented `initialize_renderer()` helper. Gateway calls that same helper directly; removed a redundant private forwarding wrapper. Asset acquisition, request service, injected task runner and executable-only bind/signal responsibilities remain separate and cohesive.
- Existing framework API is coherent for the accepted synchronous/owned SSR scope. The opaque reactive Head restriction found in round 4 is recorded explicitly below; it is not silently claimed as general reactive SSR support.

Tests rerun / result: `r2.log` passes view/UI/web/gateway tests and doctests. No public symbol, bound, signature, serde field, facade default, manifest dependency or feature changed.

## Round 3 — consistency, Rustdoc and comments

Findings and changes:

- Expanded compressed Tokio selects/admission branches so shutdown, permit lifetime, task ownership and watch-guard release can be read directly. Named the safe HTTP reason before constructing the fixed JSON envelope. Observable order is unchanged.
- Reviewed Rustdoc/private helper comments, actual byte/deadline promises, platform/config boundaries and README commands. Retained explanatory comments at ownership/serialization/async transitions; avoided ceremonial module splits or comment-only rewrites elsewhere.
- Follow-up documentation from round 4 covers eager owned SSR Head on startup/render/prepare/finish and in the README, including S5-F6 and the safe typed failure behavior.

Tests / doctests / lint / docs result: `r3.log` passes UI/web/gateway tests/doctests; `clippy.log` passes scoped all-targets warning-denied Clippy. Final workspace formatting, tests, doctests, Clippy and warning-denied Rustdoc also pass after the safety edits.

## Round 4 — safety and soundness

**Separate defect fix: Owner restoration on unwind.** The existing cleanup guard disposed render signals, but Leptos 0.8.22's `Owner::with` restores its previous thread-local Owner only on normal return. A child-process regression demonstrated that a component panic lost the caller's Owner/context (`r4-owner-before.log`). The guard now captures/restores the previous Owner on every exit. A second local guard restores context even if cleanup itself unwinds. It crosses no await. `r4-owner-after.log` and final cleanup tests pass, retaining the original cleanup/unwind proof.

**Coordinator-approved Head disposition and defect containment.** A synchronous reactive Title getter was evaluated by leptos_meta after signal disposal and panicked (`reactive-title-probe.log`). Coordinator approved documenting eager owned Head and deferring reactive SSR Head with **S5-F6**, conditional on safe request failure and keeping the restoration fix separate.

`PreparedPage::finish` now catches unwinds only while consuming its owned metadata assembly future, discards that future without retry/resumption and returns the existing typed `ViewError::Failure`. Its private native cause explains the eager-owned requirement and retains textual panic diagnostics; public Display remains `Could not render page`. No public API/variant/dependency changed. The AssertUnwindSafe boundary contains only consumed, request-local assembly buffers; it does not resume partially evaluated metadata or mutate/reuse another render's context. Normal Rust panic hooks may still log to operator stderr; response bodies never receive those diagnostics.

Regression tests prove reactive Title → typed error, captured Head state released, then a successful independent eager Title render. The actual HttpAdapter request returns sanitized HTTP 500 and serves the next independent page with HTTP 200 (`r4-head-after.log`, `r4-head-http.log`). Trusted synchronous component panics retain their previously documented propagation policy; their cleanup/context restoration is separately proved. Panic-abort builds or arbitrary trusted code are not converted into a sandbox guarantee.

Safety inventory / evidence:

| Concern | Review and executable proof |
| --- | --- |
| Snapshot escaping and bounds | One-MiB logical/sixfold encoded decoder bounds and two-MiB final HTML; lowerable policies checked before use; inert script escaping includes script terminators, ampersands and Unicode separators. Existing exact-boundary, hostile roundtrip, title/attribute escape and limit tests pass; real browsers retain hostile text inertly. No untrusted raw Head HTML is introduced. |
| Routes and decoding | Validated mount boundaries, literal/encoded dot aliases, encoded slash/plus/percent, Unicode/control data, missing/duplicate/extra fields, exact 1024-byte limit and malformed/overlong/surrogate/out-of-range UTF-8. Added route table in web codec tests; roundtrips decode once and native/browser navigation proofs agree. |
| Request limits | Native known/chunked/unknown bodies, trailers, semantic/raw headers, URI bound, total acquisition/request deadlines; slow/partial heads, body errors and disconnect. Existing gateway adversarial tests pass. Fixed response errors do not echo body/native errors. |
| Shutdown/disposal | Semaphore/JoinSet ownership reviewed; tasks are reaped, watch borrow released before await, pre-stopped notification handled. Extended actual entered-handler Notify-barrier proof to expired graceful drain as well as disconnect; both actual handler stack and connection count reach zero. No detached operation work. |
| Assets | Native bounded regular-file reads, checked sum overflow, exact MIME/path/size/SHA-256/content identity, immutable owned bytes and named startup failures. Python validation now agrees on root/manifest type, admission and output size. Existing tamper/symlink/missing/version/source/unsafe-path tests pass. The local build manifest is trusted inventory, not a signed supply-chain artifact; concurrent hostile filesystem writers are outside the owned build-directory contract. |
| Executor | Explicit/idempotent startup and runtime availability are checked before dependency executor use; fresh-process uninitialized and missing-runtime tests pass. No weakening of startup requirements or browser dependency graph. |
| Render exits | Success, invalid input, component Result error, final HTML error, discarded future, pending metadata cancellation and unwind reviewed. Added a Weak-state/outer-Owner exit table; all states release signal data before owned assembly, and caller context survives. Failed Head getters also release captured state. |
| Locks/RefCell | No first-party registry/global render lock or RefCell exists in these paths. Only a watch read guard is used and released before await. Owner set/cleanup is synchronous and ends before metadata await; dependency metadata title locks are acquired after the initial tick and not carried across await. |
| Send/lifetimes | PageLoader's higher-ranked future family and RenderFuture keep Send; component factory stays Send + Sync; no unsafe/transmute/lifetime extension. Both real ServerBuilder and PageSpec non-Send compile-fail proofs pass. |
| Disclosure/authority | Page DTO is an explicit projection, not request/session serialization. Fixed event and exact asset routes remain application-owned. Native startup may name trusted paths; public render/gateway envelopes omit paths, cookies, diagnostic reports and panic details. New HTTP Head-error regression checks disclosure and recovery. |

## Follow-up rounds

Repeated the logic/safety review after boundary edits. Two further regressions failed first, then passed:

- Generated manifests could exceed native startup's 64 KiB ceiling: serialization size is now checked before publication (`manifest-bound-before.log`, `manifest-bound-after.log`). Exact asset count/total and manifest/FIFO cases are in the checker table.
- A failed recheck could retain a previous `summary.json` claiming success: the sequence invalidates that owned receipt before preflight (`summary-before.log`, `summary-after.log`). No external tool is launched by this regression.

Final targeted review found no further material in-scope issue. Unsupported reactive SSR Head is the named accepted S5-F6 deferral, with safe typed failure rather than host-crashing getter misuse. No unrelated API redesign or native/shared/client edits were made.

## Final verification

| Check / command scope | Result / evidence suffix |
| --- | --- |
| Final full command: `python3 examples/profile-ui/check.py` | Pass: `check-final.log`, `check-final/summary.json`; freshness `--check true`, separate native SSR/hydrate/CSR, asset/seed manifests, feature/default/runtime graph checks, ordinary/customized reuse tests, doctests, Clippy and no-port gateway service probe. |
| `cargo +1.96.0 check --workspace --all-targets --locked --offline` | Pass: `msrv.log`. |
| Workspace all-target tests / doctests / Clippy `-D warnings` | Pass: `workspace-tests.log`, `workspace-doctests.log`, `workspace-clippy.log`. |
| Workspace no-deps Rustdoc, `RUSTDOCFLAGS=-D warnings` | Pass: `workspace-doc.log`. |
| Five browser proofs plus gateway | Pass: `browser-{root,mounted,m2,navigation-root,navigation-mounted,gateway}.log`; mounted paths use `/demo/v1`; `browsers.log` records six zero exits. |
| Real non-Send compile-fail tests | Pass: `non-send-{handler,page}.log` and their compiler logs, bounded offline disposable harnesses in the same native target. |
| Checker regression suite | Seven tests pass: `assets-final.log`; repeated by final check command. |
| Formatting/whitespace and source ownership | `cargo fmt --all -- --check`, `git diff --check`: pass. No G23, native/client/shared/contracts, root manifest/lock/facade or route/customization source changes. |

Browser proofs use existing Chromium/Playwright with synthetic HTTPS request interception over pipes and finite native service/fixture subprocesses. No socket or listener is opened. The gateway executable is built, never launched. Both feature modes still consume the original generated operation and unchanged selected customization source. Native packaged acceptance/parity was coordinator-verified before this release; it was not rerun or edited in this lane.

## Coverage

Tool / command / crate / features / target: cargo-llvm-cov `0.8.6`, Rust `1.96.0`, `aarch64-apple-darwin`. `/private/tmp/stackpress-s5-a-rounds-coverage.py` records the exact finite procedure: clean native output and profiles; `cargo llvm-cov show-env`; instrument selected view/server, UI/SSR, contracts, shared/SSR, web and gateway tests; enable gateway `demo-customization` (which enables web's same feature); build real web/probe/gateway binaries; run finite ordinary/hostile/search/detail/French-dark/invalid web requests; run the actual customized gateway browser proof; export with the checked-in `proofs/coverage.py`.

| Crate / measured scope | Covered / executable production lines | Result |
| --- | --- | --- |
| framework-ui (changed) | 735 / 746 | 98.53% |
| web (changed) | 429 / 453 | 94.70% |
| gateway (changed) | 525 / 582 | 90.21% |
| framework-view | 530 / 547 | 96.89% |
| contracts | 297 / 308 | 96.43% |
| shared | 429 / 466 | 92.06% |

All exceed the 70% development floor; the changed Rust crates also exceed 90%, without implying publication approval. `coverage-run.log`, `coverage-{tests,bins,browser,export}.log`, full `coverage.json` and copied `coverage.profdata` are retained. Export reported no mismatch warning.

Exclusions: dependencies, dedicated test source, non-Rust tooling and out-of-scope workspace crates. Thin WASM browser adapters retain the coordinator's numeric-coverage exception and real browser/WKWebView verification; they were not edited. Shared route/customization source and real executable entry points remain counted. Gateway main is intentionally **0/35**: listener execution is prohibited; its reusable service/runner is exercised through duplex and the browser bridge. No listener code is excluded to increase coverage.

Remaining uncovered lines include a few native asset I/O/fallback, unavailable-output/error, route/fixture failure and panic-payload fallback paths. Render source is 153/157 lines (97.45%); actual reactive-getter failure, restoration, caller context, limits and disposal are directly proved rather than inferred from the percentage. Stable doctest coverage is not claimed; doctests pass separately.

## Closeout and limits

Only `/private/tmp/stackpress-s5-a-rounds-native` and `/private/tmp/stackpress-s5-a-rounds-wasm` were used for build output, and both were removed after preserving evidence. No other lane's directory was deleted. No tool installation, listener, native app launch, external message or commit occurred. Approved finite headless browser launches close in `finally`.

Remaining branch/adapter/platform risks: reactive SSR Head remains S5-F6 with the approved owned/eager contract and typed misuse error. Arbitrary trusted component code, cleanup callbacks and destructors remain outside a sandbox guarantee: a panicking cleanup callback can interrupt dependency cleanup, and abort/double-panic policy cannot be recovered by catch_unwind. The proved unwind case has normal cleanup callbacks; no guarantee of recovery from arbitrary panicking destructors is claimed. Example gateway buffering, trusted build-directory ownership, no production TLS/auth/streaming and other-platform limits are unchanged. No unresolved public API decision remains.

KB validation baseline: 146 errors and 21 warnings from historical/sibling links and existing document warnings. After receipt creation, compare the exact diagnostic multiset; no unrelated KB repair or task/index/ledger edits are authorized. Coordinator re-verification/merge is next; this is the requested round stop.
