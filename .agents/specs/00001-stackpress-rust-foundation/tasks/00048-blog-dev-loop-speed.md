# 00048 — Faster blog development loop

Status: **implemented; clean-copy README and full gate passed; coordinator/live review pending.**
Base: `cdfd1dc`. No commit, push, install, lock resolution or listener.

## Changes

- [Development loop](../../../../examples/blog/README.md#develop-with-live-reload): generation precedes concurrent native/hydration builds. Each concurrent compiler gets one Cargo job; solitary native, generation and deferred CSR builds get two. Critical failures retain the last good server and published bundles.
- Publish complete hydration output and restart/reap the owned server; wait for its stdout readiness line before notifying hydration tabs. CSR runs afterward on an independently cancelable worker. Only successful complete CSR output replaces its prior mode directory and reloads CSR tabs. Superseding source batches cancel/join old CSR work before starting critical builds. Ctrl-C also cancels readiness waits.
- The reload worker survives app restarts. Additional `blog_reload: "hydrate" | "csr"` notifications select the tab's renderer before hydration has finished; native-only changes reload both modes when no CSR is pending. Reconnects to stale HTML respect pending CSR. Full native/hydration rebuilds retire prior patch history and revisions.
- Dev Cargo children force `CARGO_INCREMENTAL=1`. The xtask compiler bridge preserves incremental flags for the watched blog workspace, keeps unwatched framework dependencies nonincremental/cacheable, and removes the environment switch only before invoking rustc/sccache. The normal finite build and the gate retain their existing policies.
- Rejected view patches print one reason and source line. Static patches remain available during deferred CSR: completion preserves patch history/revisions, and CSR's existing missing-marker feedback still requests a full rebuilt baseline. AST eligibility remains conservative: translation expressions and component props require compilation. No alternative macOS linker was installed. A 16-codegen-unit native experiment did not improve handler latency (4.202s median, `20261110T074656.822817Z-bc017096d710`); compiler/link settings were left unchanged.
- No dependency, manifest or lock changes. Root lock remains 699 registry tuples; blog retains the existing G58 allowlist only. No excluded tooling mentions or scripts were added to the example.

## Evidence and refactoring

All commands are recorded in `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks` with `00048` labels. Narrow/gate checks use sccache, incremental off, debug off and bounded jobs; real dev timing builds override incremental mode as above. Only owned targets are retired.

1. Logic/complexity: reviewed critical/deferred failure ordering, publication, stale-page revisions and cancellation; added finite success/failure/supersession/readiness tests. `20261110T072952.312942Z-354109e738e1` passed 33 tests, including one slow rapid-cancel fixture. An early cancellation guard plus an explicit child-start barrier now prove prompt cancellation; subsequent runs have no slow case.
2. Responsibility: separated [session ownership](../../../../examples/blog/xtask/src/dev/session.rs) from watch/CLI orchestration and kept build staging and WebSocket state with their owners. `20261110T073300.332697Z-c1dda96f0b73` passed all 33 tests and finite client protocol cases. Follow-up cache-policy tests bring the narrow count to 34.
3. Consistency/Rustdoc/comments: aligned critical/deferred terminology, documented environment and process ownership, checked job policies, formatting, Clippy, doctests and warning-clean Rustdoc. `20261110T073919.776437Z-48f31b794232` passed all 34 tests and client cases.

Preliminary xtask executable line coverage: **955/1218 (78.41%)**, no exemption (`20261110T073953.057717Z-bec03e0d448a`). Final clean-copy per-crate results supersede this preliminary count.

Final follow-up review retained static patches during deferred CSR and verified that completion preserves patch history and fallback revisions. `20261110T080330.775168Z-b0617fcc5e36` passed all 34 xtask tests, client protocol cases, formatting, Clippy, Rustdoc/doctests, the targeted normal/dev HTML comparison, all 12 local documentation links, lock assertions and the exact inherited KB diagnostics. The targeted web run intentionally skipped 30 unrelated cases; the clean-copy gate runs all cases.

## Warm-cache timings

The opt-in [finite harness](../../../../examples/blog/xtask/tests/cases/timing.rs) saves edits in an owned copied app, waits the 250ms debounce, runs real compilers/bindgen and staging publication, then receives the reload/patch notification over an in-memory channel. No HTTP server or WebSocket listener runs. Timings exclude HTTP startup and browser navigation; readiness and real process cleanup have separate finite child tests. Three distinct matching saves per edit family use the same paths and restored compiled baseline in both versions; first warm-up builds are outside the interval. Distinct saves avoid measuring sccache hits for an identical previously compiled local edit. Raw arrays and experiment results are retained in `examples/blog/.build/timing/`.

Final baseline `20261110T075244.818623Z-19e6d34382ad` → final implementation `20261110T075528.670468Z-503763ec898d`:

| Save → notification, seconds | Before | After |
| --- | ---: | ---: |
| CSS swap | 0.263 | 0.258 |
| Literal markup patch | 0.296 | 0.295 |
| View logic, hydration reload | 15.369 | 6.942 |
| Server handler reload | 6.314 | 4.427 |

The matching deferred CSR notification for the view edit arrived at 10.782s median. View edits used three equivalent empty-row fallback expressions; handlers used three valid sort changes. CSS/markup used three distinct comments/literals.

Control experiments are retained: exact repeated edits measured view logic 12.343→7.388s and handler 3.328→3.985s, showing the remaining repeated-local-cache advantage of nonincremental builds. The initial all-dependency incremental experiment regressed handler edits further (5.925s), prompting the framework-cache split. One invalid distinct-measurement setup was explicitly canceled and its owned groups/fixture restored; it is not timing evidence. Failed setup/fixture/lint attempts remain in the ledger.

## Final verification

- Detached copy `/private/tmp/blog-clean-00048-cdfd1dc-01` started with empty `.build`/target and the applied frozen diff, SHA256 `d90a3c07b2a29e6e9cb4da44a4b3550802bba986605b21dc6e03b1dc90777145`. Audit confirmed tested and current implementation bytes match; only this receipt receives post-proof evidence.
- README setup, SQLite population (two articles), both browser bundles, **88 normal tests** and the nullable-column migration passed: `20261110T080550.858501Z-2e5747e3c111`, end-to-end `622.502628s`; observed Cargo stages `584.24s`.
- Full `python3 tools/examples/blog/gate.py` ran **once**, all **26 phases** green: `20261110T081641.057214Z-860db12f5e32`, end-to-end `1345.642612s`; observed Cargo stages `1128.57s`. Both ordinary and instrumented native matrices passed **90/90, zero skips**. Public, authenticated and copied-out browser flows passed in hydration/CSR. Development coverage is below; blog-client alone retains G56.

| Crate | Covered/executable | Coverage |
| --- | ---: | ---: |
| blog-tool | 102/113 | 90.27% |
| blog-web | 2375/3362 | 70.64% |
| blog-app | 3698/3882 | 95.26% |
| blog-store | 38/39 | 97.44% |
| blog-assets | 70/78 | 89.74% |
| blog-xtask | 955/1213 | 78.73% |

- LLVM retained the nonfatal **92 functions have mismatched data** warning; raw coverage and the outer gate log preserve it. Native Clippy and Rustdoc had no warnings. Source-based coverage counts actual generated UI/native families, without a dev-listener exemption.
- Peak sampled compilation footprint: **11,488,497,664 bytes (10.6995 GiB)**, below 12 GiB. Audit `20261110T084030.175648Z-925efeb980b0` confirmed all phases, hashes, locks and per-crate floors. Cleanup `20261110T084106.881947Z-4bb1516a35e3` removed the detached copy and its target; source/timing targets and the copied timing project were also removed.
- Reports, raw coverage, per-phase logs, frozen/final patches, benchmark records and source hashes are retained under `examples/blog/.build/clean-copy/00048-cdfd1dc-01/`; timing arrays and the baseline harness remain in `.build/timing/`. Root lock bytes and 699 tuples are unchanged; blog lock remains 448 tuples with exactly the existing six G58 exceptions.
- Pre-close ledger snapshot through cleanup: **29 records, 22 passed and seven failed/canceled experiments**, `4062.291391s` summed command time. This sum can overlap concurrent commands and is not thread elapsed time. Agent Workspace baseline is **146 inherited errors and 44 warnings**, with exact diagnostics compared after receipt updates.

For the owner's live check, run `cargo xtask dev --port 3000 --reload-port 3001` from `examples/blog`, then refresh existing tabs once to load the updated reload script. First launch warms the incremental targets; the table measures warm saves. No listener, install, commit or push was performed.
