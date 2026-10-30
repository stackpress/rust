# Slice 9 Lane A documentation receipt

Status: **Docs complete; stop for coordinator verification**. Started from confirmed 1ae9129, clean tree, after the accepted rounds merge. Scope is framework-session README/docs and this new receipt only. No source, Rustdoc comments, tests, consumers, manifests, locks, facade or public behavior changed. G42/G43 remain binding.

## Pages and reading paths

The entry routes readers through first identity success, concepts, task guides and API lookup. Existing detailed auth/storage/account pages remain, with current registration examples and evidence links. Lane B's completed docs supplied the structure; the docing skill supplied guided-learning, reference and formatting review. **20 Markdown pages**:

| Page |
| --- |
| [README.md](../../../../crates/framework/framework-session/README.md) |
| [docs/account.md](../../../../crates/framework/framework-session/docs/account.md) |
| [docs/api/README.md](../../../../crates/framework/framework-session/docs/api/README.md) |
| [docs/api/account.md](../../../../crates/framework/framework-session/docs/api/account.md) |
| [docs/api/auth.md](../../../../crates/framework/framework-session/docs/api/auth.md) |
| [docs/api/identity.md](../../../../crates/framework/framework-session/docs/api/identity.md) |
| [docs/api/storage.md](../../../../crates/framework/framework-session/docs/api/storage.md) |
| [docs/api/views.md](../../../../crates/framework/framework-session/docs/api/views.md) |
| [docs/auth.md](../../../../crates/framework/framework-session/docs/auth.md) |
| [docs/concepts.md](../../../../crates/framework/framework-session/docs/concepts.md) |
| [docs/getting-started.md](../../../../crates/framework/framework-session/docs/getting-started.md) |
| [docs/guides/authentication.md](../../../../crates/framework/framework-session/docs/guides/authentication.md) |
| [docs/guides/configuration.md](../../../../crates/framework/framework-session/docs/guides/configuration.md) |
| [docs/guides/consumers.md](../../../../crates/framework/framework-session/docs/guides/consumers.md) |
| [docs/guides/hooks.md](../../../../crates/framework/framework-session/docs/guides/hooks.md) |
| [docs/guides/registration.md](../../../../crates/framework/framework-session/docs/guides/registration.md) |
| [docs/guides/shared-ui.md](../../../../crates/framework/framework-session/docs/guides/shared-ui.md) |
| [docs/source-behavior.md](../../../../crates/framework/framework-session/docs/source-behavior.md) |
| [docs/storage.md](../../../../crates/framework/framework-session/docs/storage.md) |
| [docs/verification.md](../../../../crates/framework/framework-session/docs/verification.md) |

The API pages cover signatures, arguments, returns/defaults/errors and integration links. Configuration distinguishes feature/registrar selection, missing/empty sections and explicit Environment/resources. Facade forwarders, identity-only plugin versus composite auth→email→identity, aggregate placement, all five consumers and API cookie/public-event acquisition are explicit. No request-local carrier or route ownership arbitration is introduced.

## Compiled examples and claims

**17/17 exact Rust fences compile**, isolated by module in four disposable external consumers; **11 finite examples pass** (4 direct, 6 server, 1 facade). The browser fence compiles separately with hydrate and CSR on wasm32-unknown-unknown. Context-dependent handler/Engine functions compile without arbitrary host execution.

Final workspace: /private/tmp/slice9a-docs-ij7zlx67. Its examples.json maps each page/line/body SHA-256; final audit matches current Markdown exactly. Direct/plugin/facade/browser manifests were copied with absolute paths and **byte-identical existing locks**. Separate existing consumer-session-* targets kept dependencies warm without adding a graph or refreshing a lock. No scaffold import or ignored snippet supplies missing names.

| Rust fence | Graph |
| --- | --- |
| docs/api/identity.md:32 | direct |
| docs/api/storage.md:10 | direct |
| docs/getting-started.md:12 | direct |
| docs/guides/configuration.md:33 | direct |
| docs/account.md:11 | server |
| docs/api/account.md:29 | server |
| docs/api/auth.md:56 | server |
| docs/api/identity.md:71 | server |
| docs/api/storage.md:45 | server |
| docs/auth.md:16 | server |
| docs/guides/authentication.md:10 | server |
| docs/guides/hooks.md:10 | server |
| docs/guides/registration.md:10 | server |
| docs/guides/registration.md:37 | server |
| docs/storage.md:9 | server |
| docs/guides/consumers.md:22 | facade |
| docs/guides/shared-ui.md:16 | browser |

The linked account_host SSR example passes an explicit cargo check. Both linked browser entry builds pass fmt, Clippy, build and wasm-bindgen. Plain fences are rustfmt-formatted. Static checks pass: one H1, heading orientation, single blank-line separation, classified/tagged fences and local paths/heading anchors. **36 claim anchors**, plus inline source/test/oracle links, cover documented behavior. Four in-memory negative controls reject omitted classification, missing file, missing heading and stale source anchor. These detect drift; manual semantic review remains necessary.

No implementation bug was exposed. Initial checks caught docs-anchor spelling and an email import absent from the copied plugin manifest. Only docs were corrected: account uses the public composite, auth-only selection stays explicit, and the compiled constituent implementation is linked rather than relying on a transitive import. No dependency/lock change or hidden scaffold was used.

## Source behavior and owner questions

All seven are labeled **open question — current behaviour follows upstream**, with relevant guide/API cross-references: account delete keeps profile; SQLite ILIKE; Auth create-return; positive hasnt; optional phone on update; email sign-in skips send; Moment admin dates. Exact upstream paths/lines and Rust evidence are recorded without selecting a fix. G44 optional signup phone remains distinct from update.

No global guard is added for missing session.access; empty access retains source permissive/zero-status error-offer behavior. Direct events retain their own checks. Password/2fa request flags, errors, redirects, consumed/lifetime behavior, literal signout cookie, partial writes and deliberate enrollment/CSRF values are described as shipped behavior. Regex/Date/typed-resource/Leptos/QR mechanics and provider/backend/installed limits remain explicit.

DX observations, documented without API changes: qualify the foundation cookie Session, JWT Session and API credential Session; distinguish direct empty seed from make fallback abc123; distinguish returned Actions rows/batch attempts from persisted acknowledgements; bind/stop Hooks explicitly; use an explicit email dependency for its standalone registrar. No additional owner question is opened.

## Final gate

| Check | Result |
| --- | --- |
| Workspace fmt; all-target Clippy -D warnings, default/eight-feature | Passed |
| Workspace all-target nextest, default | **928 passed**, 86 skipped, 224 binaries |
| Workspace all-target nextest, eight-feature | **993 passed**, 89 skipped, 224 binaries |
| Workspace compatible-feature doctests | **966 passed**, including existing compile-fail contracts |
| Workspace Rustdoc --no-deps, RUSTDOCFLAGS=-D warnings | Passed |
| Exact Markdown / linked SSR example | 17 fences, 11 finite executions; SSR check passed |
| Session feature/facade/target graphs --check | Native leaves and released WASM/default/auth-server/hydrate/CSR boundaries passed |
| Four native standalone consumers/run.py | Graph, fmt, Clippy, executable assertions and tuple subsets passed |
| Browser hydrate and CSR consumers | Separate WASM fmt, Clippy, builds, bindgen, graph/secret assertions passed |
| Links, claims, negative controls, diff --check and immutable-source/lock audit | Passed |
| KB before/after | **146 inherited errors / 28 warnings**, exact diagnostic-set delta zero |

Eight compatible features: session/leptos-ssr, session/auth-server, csrf/server, email/server, api/sql, api/codegen, api/ui-ssr and schema/providers (full package-qualified names in the gate log). Both workspace nextest runs exclude tcp_watchdog, owned by coordinator verification. Existing deadlock-class tests retain external fail-fast process proofs. The outer wrapper has an 1800-second absolute deadline and process-group termination/reaping.

Gate record 20261030T154723.556969Z-9d939adcbaa6: **180.099800 s command wall**, **8.24 s observed Cargo-stage subtotal**. Final examples/consumers record 20261030T155112.074392Z-00e2ea02e311: **35.207647 s wall**, **19.94 s Cargo subtotal**. Cargo-stage time is incomplete and overlaps wall time; it is not additional elapsed time.

## Scope, resources and limits

Audit: **145 frozen files** byte-identical (142 non-README/docs session files plus root Cargo.toml/Cargo.lock/src/lib.rs). Five repository consumer locks and external copies are unchanged; **699 external tuples**, zero additions. Root lock SHA-256: fb6614c67c54f1a1341b1df24a59fa6f502bb3d2881f0be03a46d158e05adcdc. Changed paths are only session README/docs and this receipt.

Warm target/slice9a uses sccache, two jobs, debug=0, CARGO_INCREMENTAL=0. Cap **24 GiB**, guard **23.4 GiB**; largest sampled docs observation **21.488 GiB**. No clean/deletion, lock reconciliation, download or install. Cargo resolving/build/test/doc commands are --locked --offline; fmt/rustfmt have no lock flags. No renderer all-features, commit/push/GitHub write, other-agent message or development listener.

Coverage was not remeasured for Markdown: the accepted coordinator rounds result remains **91.43% session coverage** in its original native scope, not a new docs measurement. Browser runtime/screenshots were not rerun; unchanged accepted hydrate/CSR matrix evidence is linked separately from current compilation. SQLite executions use real files; PostgreSQL/MySQL have renderer/scripted evidence only. Live SMTP/SES/sendmail/provider/network and installed Tauri acceptance, TCP/TLS flush and remote acknowledgement are not inferred.

## Benchmark records

Store: /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks. Each runs/<id>.json and log retains metadata, exact outer command and printed nested argv. Read-only inspections were exempted by the coordinator; early recorded reads remain included. Failed attempts remain visible. The ledger includes completed records through final audit; the receipt metadata refresh is separately recorded as slice9a receipt metadata refresh after docs final audit.

| Record | Invocation | Status | Wall s | Cargo-stage s |
| --- | --- | --- | ---: | ---: |
| 20261030T152630.907803Z-6f6b7da46ea5 | workflow skill and Lane B documentation evidence | passed | 0.020262 | — |
| 20261030T152954.838385Z-435d309fa66a | model and guidance reads | passed | 0.026914 | — |
| 20261030T153005.121299Z-b532ddb2cca3 | checker and crate surface reads | passed | 0.021013 | — |
| 20261030T153021.965743Z-9785eddf2469 | source inventory and baseline | failed | 0.146676 | — |
| 20261030T153034.614856Z-4488c1e068e6 | inventory corrected paths and owner questions | passed | 0.055325 | — |
| 20261030T153050.323787Z-738a8ff71619 | identity config auth reference | passed | 0.026058 | — |
| 20261030T154433.344964Z-15c8bb3b7006 | structure links claims and fence formatting | failed | 0.483697 | — |
| 20261030T154451.679183Z-d01b5ec4258a | complete claim anchor review | failed | 0.070871 | — |
| 20261030T154526.549407Z-38de9f999476 | external exact fences native and wasm | failed | 6.110684 | 3.59 |
| 20261030T154552.579890Z-8f15f95a9f39 | KB baseline diagnostics | passed | 0.361496 | — |
| 20261030T154642.932933Z-b77425e36300 | exact examples corrected imports and finite execution | passed | 8.754136 | 5.38 |
| 20261030T154723.556969Z-9d939adcbaa6 | final default and eight-feature workspace gate | passed | 180.0998 | 8.24 |
| 20261030T155112.074392Z-00e2ea02e311 | final examples features consumers and browser builds | passed | 35.207647 | 19.94 |
| 20261030T155703.234504Z-6aeebda57602 | prepare final audit and receipt writer | passed | 0.0215 | — |
| 20261030T155723.389056Z-24db4f8c64fd | final scope lock fence link KB audit and receipt | failed | 0.69328 | — |
| 20261030T155748.497598Z-6984254f3136 | final audit receipt corrected path rendering | passed | 0.899646 | — |

Final audit checks exact fence hashes, unchanged implementation/locks/tuple subsets, links/claims, diff whitespace, receipt line cap and KB diagnostic equality. Coordinator verification is next; work stops at docs.

## Coordinator verification

Committed as `22afc64` on top of `1ae9129` (trial/3 unchanged since, so no integration merge was needed). The commit changes only `crates/framework/framework-session/README.md`, the `docs/` tree and this receipt; no source, test, consumer, manifest or lock file changed, and no session source includes the README into Rustdoc (the only `include_str!` is `views/styles.css`). Reran independently on 2026-10-31:
- `docs/check.py`: 20 pages, 17 Rust fences, 36 claim anchors, 4 negative controls, all local links and anchors resolve.
- `docs/check.py --compile` in a fresh empty target (`s9adocs-verify gate docs examples compile`): all 17 fences compiled in external direct, server, facade and browser (hydrate and CSR, wasm32) consumers; 11 finite examples executed (4 direct, 6 server, 1 facade). Tracked tree unchanged afterwards.

Spot review: source defaults (key `session`, seed `abc123`, no expiry) are described as source defaults, not a protection policy; no hardening advice is framed as a framework default (G43). `source-behavior.md` records all seven owner questions as "open question — current behaviour follows upstream". Merged into trial/3 as `ce80fa1`. Lane A (session) is complete; Slice 9 is complete.
