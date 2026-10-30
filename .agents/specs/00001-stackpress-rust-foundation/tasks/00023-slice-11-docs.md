# Slice 11 documentation receipt

Status: docs complete; coordinator verification pending. Started from clean confirmed `40da86b`, after the accepted rounds merge. Scope is aggregate README/docs and this receipt only. Acceptance criteria: none for this library documentation checkpoint.

## Pages and reading paths

The [entry](../../../../crates/framework/framework/README.md) leads to a finite first event, then concepts, task guides and lookup pages. The docing skill supplied guided-learning, API-reference and formatting review. **19 Markdown pages**:

| Page |
| --- |
| [README.md](../../../../crates/framework/framework/README.md) |
| [docs/api/README.md](../../../../crates/framework/framework/docs/api/README.md) |
| [docs/api/codegen.md](../../../../crates/framework/framework/docs/api/codegen.md) |
| [docs/api/commands.md](../../../../crates/framework/framework/docs/api/commands.md) |
| [docs/api/configuration.md](../../../../crates/framework/framework/docs/api/configuration.md) |
| [docs/api/features.md](../../../../crates/framework/framework/docs/api/features.md) |
| [docs/api/language.md](../../../../crates/framework/framework/docs/api/language.md) |
| [docs/api/registration.md](../../../../crates/framework/framework/docs/api/registration.md) |
| [docs/api/sql.md](../../../../crates/framework/framework/docs/api/sql.md) |
| [docs/api/view.md](../../../../crates/framework/framework/docs/api/view.md) |
| [docs/concepts.md](../../../../crates/framework/framework/docs/concepts.md) |
| [docs/getting-started.md](../../../../crates/framework/framework/docs/getting-started.md) |
| [docs/guides/codegen.md](../../../../crates/framework/framework/docs/guides/codegen.md) |
| [docs/guides/commands.md](../../../../crates/framework/framework/docs/guides/commands.md) |
| [docs/guides/consumers.md](../../../../crates/framework/framework/docs/guides/consumers.md) |
| [docs/guides/registration.md](../../../../crates/framework/framework/docs/guides/registration.md) |
| [docs/guides/website.md](../../../../crates/framework/framework/docs/guides/website.md) |
| [docs/source-behavior.md](../../../../crates/framework/framework/docs/source-behavior.md) |
| [docs/verification.md](../../../../crates/framework/framework/docs/verification.md) |

Coverage includes source-order composition of all nine stages; register/register_with and static catalogs; SQL Listen model callbacks/admin last; aggregate/facade/renderer features and both compile-error guards; all thirteen commands and direct APIs; ConfigPresence vs truthiness; locale nested dispatch; native render/compiler/SQL/revision/IO providers; supervised non-awaited packaging; and eight consumer recipes. G42 multiple handlers/priority/FIFO/Abort and G43 configured process security are explicit. No global guard, default-deny, route arbitration or hardening was added.

## Exact compiled examples and claim anchors

**11/11 exact Rust fences compile in five external consumer graphs; 10 finite examples pass**: full 7, headless 1, facade 1, build-host 1. The browser fence compiles separately in native SSR and WASM hydrate/CSR; it needs a caller-supplied snapshot, so it is compile-only. Named async handlers precede host setup. No hidden scaffold imports supply missing dependencies.

External workspace: /private/tmp/slice11-docs-nby2f5mv. Its examples.json records current page/line, body SHA-256 and execution selections. Manifest changes in disposable copies are limited to absolute path substitution; all five external locks are byte-identical to their original full/headless/facade/website/build-host consumer locks, with registry subsets of the unchanged root. No resolution, new edge or lock refresh occurred.

| Exact fence | Graph | Finite function(s) |
| --- | --- | --- |
| docs/api/configuration.md:34 | full | generation_options |
| docs/api/language.md:22 | full | translation |
| docs/api/sql.md:39 | full | sqlite_transaction |
| docs/getting-started.md:10 | full | first_aggregate |
| docs/guides/commands.md:10 | full | finite_serve |
| docs/guides/registration.md:10 | full | catalog_host |
| docs/guides/website.md:38 | full | packaging_lifecycle |
| docs/api/registration.md:19 | headless | absent_client |
| docs/guides/consumers.md:21 | facade | facade_host |
| docs/guides/website.md:12 | browser | compile-only public snapshot decoder |
| docs/guides/codegen.md:10 | buildhost | build_registry |

The [checker](../../../../crates/framework/framework/docs/check.py) validates all pages, rustfmt fences, local paths/heading anchors, per-page claim coverage and the exact execution inventory. **59 claim anchors** in [claims.json](../../../../crates/framework/framework/docs/claims.json) point to current implementation/tests/oracles or explicitly historical accepted runtime evidence. **Seven in-memory negative controls** reject omitted graph classification, missing target, missing heading, stale source anchor, duplicate claim ID, missing page coverage and omitted finite execution. Anchor presence detects drift; manual semantic review remains necessary.

Draft checks caught missing API-doc links/SQLite source-test paths, application examples assuming BuildError converts to the handler Error, and a schema error helper qualified under the wrong module. Only documentation/checker drafts were corrected. The final examples retain the distinct BuildError boundary and use the existing codegen helper. An initial final audit compared graph-grouped report order with page order; the audit now matches page/line/graph identities before checking exact hashes. No example body changed. No real implementation bug was exposed; no source/test/manifest fix or grant was needed.

## Source behaviour, questions and limits

The source-behaviour page labels all ten items **open question — current behaviour follows upstream / accepted lower crate**: PGlite; MIT/Apache metadata; SQLite purge/TRUNCATE; account delete keeps Profile; SQLite ILIKE; auth create-return; positive hasnt; optional phone on update; email sign-in skips send; Moment admin dates. Exact upstream provenance and owning Rust evidence are linked without choosing a fix. G44 signup phone remains distinct from update.

SQLite purge retains the accepted stackpress-sql truncate → DELETE mapping, rather than reproducing upstream Sqlite.js:404–408's rejected TRUNCATE … CASCADE. PGlite has no dummy feature/connector. MIT/Apache metadata remains unresolved and no upstream source asset was copied.

API-MATRIX, API-BUILD-HOST and EDGE-REPLAY remain precise accepted evidence gaps. S11-WHATWG, Node packaging, live SMTP/SES/sendmail/Postgres/MySQL, app-owned listener/watcher/bundler/TTY/native IO and installed Tauri remain named limits. Native/WASM compilation does not establish installed platforms, remote delivery/acknowledgement or TCP/TLS flush. Browser snapshot/example compilation is distinct from actual Chromium replay.

## Final verification

| Check | Result |
| --- | --- |
| Documentation structure / links / claims / negative controls | 19 pages, 11 fences, 59 claims, 7 controls; all paths/anchors resolve. |
| Exact external fences / finite executions | 11 compiled / 10 passed; SSR + hydrate + CSR; unchanged external locks. |
| Workspace fmt / default + compatible all-target Clippy -D warnings | Passed. |
| Workspace nextest default / combined | 931 / 1032 passed; 86 / 89 existing skips; tcp_watchdog excluded, coordinator-owned. |
| Workspace doctests default / combined; Rustdoc -D warnings | 962 / 967 passed; Rustdoc passed. |
| Server default / http nextest, http Clippy; isolated API leaf | 36 / 61; Clippy and leaf test passed, source unchanged. |
| Aggregate leaves / target / facade diagnostics | No-feature + 11 leaves, WASM no-feature/crate-alone graph; both intended facade compile-error diagnostics pass. |
| Consumers and graphs | All 32 facade features/target graphs and eight consumers pass; full/direct/facade ordered inventories equal; real generation freshness passes. |
| Browser artifacts | Separate hydrate/CSR fmt/Clippy/build/wasm-bindgen and native-secret checks pass. No browser runtime replay was rerun for docs. |
| Source ledger / linked docs | 24 inventory rows, 5 command rows, 15 export families, 77 declaration rows, R01–R20 and 43 upstream hashes pass; session docs checker passes. |
| Source / manifests / locks | 3547 baseline tracked files byte-identical; root + 8 consumer locks unchanged; 699 external registry tuples, zero additions. |
| Links / KB | Receipt links resolve; canonical layout zero errors / 28 inherited warnings. Raw worktree 146 inherited sibling-path errors / same warnings, exact diagnostic delta zero. |

Exact compatible argv is recorded: aggregate full + four UI families + SSR, API codegen, CSRF/email server, aggregate codegen and schema providers. No renderer all-features. Finite examples/consumer commands use external process deadlines; the consumer watchdog's forced-kill/reap negative control passes.

No Rust implementation changed, so coverage was not remeasured. Accepted rounds aggregate coverage remains **1043/1078 = 96.75%**, in its original full,codegen native scope; root re-exports retain compile/graph evidence without an executable denominator. The accepted aggregate session replay remains **352 session + 2 no-JS + 12 smoke cases** in the rounds receipt, not a new docs runtime result.

## Resources and benchmark records

Same warm target/slice11, sccache, two jobs, debug=0, incremental=0. Cap **24 GiB**, guard **23.4 GiB**; sampled peak 18.36 GiB, final 16.86 GiB. Only completed owned consumer outputs were removed by the existing runner where required; parent debug/dependency artifacts retained. No guard/deadline trip. Every Cargo/proof/clean command was recorded; resolving commands use --locked --offline. No Cargo clean, dependency download, installation or lock exception.

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`; runs/<id>.json/log retain exact outer/nested argv, counts and environment.

| Record ID | Stage | Status / exit | Wall s | Cargo-stage s |
| --- | --- | --- | ---: | ---: |
| 20261030T233138.176502Z-77b371a67882 | baseline and bounded environment | passed (0) | 0.52 | 0.00 |
| 20261030T233933.010089Z-e5cce86df890 | format structure claims and negative controls | failed (1) | 4.22 | 0.00 |
| 20261030T234020.574944Z-071c98f64571 | structure links claims corrected paths | passed (0) | 2.94 | 0.00 |
| 20261030T234052.470575Z-211d7b6a2049 | exact external examples native SSR hydrate CSR | failed (1) | 5.99 | 0.00 |
| 20261030T234149.685525Z-60e27670a14b | exact examples corrected application error boundary | failed (1) | 152.84 | 146.98 |
| 20261030T234223.526571Z-d80ce546e046 | final gate staging | passed (0) | 0.02 | 0.00 |
| 20261030T234418.126515Z-3f8c54673dbd | isolation and guard proof staging | passed (0) | 0.03 | 0.00 |
| 20261030T234434.607601Z-a848bb1b2d15 | exact examples corrected schema error helper namespace | passed (0) | 16.05 | 9.39 |
| 20261030T234512.364985Z-8749e5c1f70a | final compatible workspace gate | passed (0) | 208.41 | 13.86 |
| 20261030T234826.938941Z-8de3de0065a3 | receipt and immutable scope audit staging | passed (0) | 0.02 | 0.00 |
| 20261030T234850.450598Z-11907baec444 | final consumers graphs and browser artifacts | passed (0) | 193.05 | 154.32 |
| 20261030T235302.878331Z-fbe6b5ae9c27 | final scope exact fences links KB and receipt assertion | failed (1) | 2.93 | 0.00 |

Final scope/link/KB/receipt assertion: 20261030T235343.890812Z-d1335389ae68 (outcome in recorder). 12 prior completed attempts, 4 failed draft checks retained. Prior command wall total 587.02 s; observed Cargo-stage subtotal 324.55 s. These overlap and remain separate; end-to-end elapsed 1328.40 s includes drafting/review/waits.

Stop at docs for coordinator verification. No implementation/test/consumer/manifest/lock/facade/history edits, commits, pushes, installs, GitHub writes, listeners, other-agent messages or later-stage work.
