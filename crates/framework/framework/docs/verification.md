# Verify documentation and composition

Documentation claims link to source/tests and compiled examples; they are not a substitute for runtime, backend or installed-platform proof. The accepted [rounds receipt](../../../../.agents/specs/00001-stackpress-rust-foundation/tasks/00023-slice-11-rounds.md) records the implementation gate and its exact feature/coverage scope.

## Check these pages

Record the outer `python3 crates/framework/framework/docs/check.py` command with the repository benchmark recorder. It checks every Markdown page's headings, spacing, tagged/classified Rust fences, local links/heading anchors and claims.json source/test anchors. In-memory negative controls demonstrate rejection of omitted graph classification, missing link/heading and stale evidence; see the checker for additional inventory/execution controls. Anchor presence detects drift, not automatic semantic correctness.

Use `--format` to format only Rust fence bodies, then `--compile` to compile those exact bodies in isolated external modules. The compiler checker copies existing full/headless/facade/website/build-host manifests with absolute path substitutions and byte-identical locks. Every resolving Cargo command is --locked --offline; no dependency resolution, root/consumer lock refresh or hidden scaffold import is permitted. Browser fences compile separately for SSR, hydrate and CSR. Finite native examples execute; context-dependent handler/provider functions are compile-only, explicitly listed in examples.json.

The checker prints its external workspace, fence hashes and executions. Supply an absolute CARGO_TARGET_DIR under the bounded effort target. It uses separate consumer subtargets and a finite external deadline for each subprocess, with process-group termination/reaping on timeout. Fresh-target results are separate from warm-target timings.

## Runtime evidence

Existing proofs remain the source for behaviour broader than small documentation examples.

| Capability | Evidence |
| --- | --- |
| Source order, Config priority 10, callback FIFO | [Phase tests](../tests/cases/mod.rs), [catalog tests](../tests/cases/catalog.rs), [source-order oracle](../tests/fixtures/source-order.json). |
| Multiple handlers / Abort, duplicate config errors | [G42 tests](../tests/cases/mod.rs), [closing tests](../tests/cases/closeout.rs). |
| Command input/default/skip/error rules | [Command tests](../tests/cases/commands.rs), [script oracle](../tests/fixtures/source-scripts.json). |
| Real SQL, lazy revisions, rollback and native result | [SQLite tests](../tests/cases/sql.rs). |
| Owned futures, nested dispatch, cancellation | [Ownership tests](../tests/cases/ownership.rs), [language tests](../tests/cases/language.rs), [compile-fail cases](../tests/ui). |
| Non-awaited packaging / observable failures / shutdown | [View tests](../tests/cases/view.rs). |
| All inventory/export/risk rows | [Ledger](../proofs/coverage-map.json), [checker](../proofs/check.py), 43 upstream source hashes. |
| Feature/target/default boundaries and eight recipes | [graphs.py](../consumers/graphs.py), [run.py](../consumers/run.py). |
| Shared UI | [Session replay](../proofs/session-browser.cjs): 352 cases plus two no-JS checks; [aggregate smoke](../proofs/browser.cjs): 12 cases across hydrate/CSR. |

Deadlock-class cases run under external process deadlines, including a SIGTERM-ignoring forced-kill/reap negative control. The final rounds receipt records 13 cases; an executor timeout alone is not the evidence. Workspace nextest excludes tcp_watchdog for implementer runs because the coordinator owns that listener-binding proof.

## Evidence limits

The accepted rounds aggregate coverage is 1043/1078 executable lines (96.75%) with full,codegen; documentation changes do not create a new implementation coverage result. Native line coverage does not measure browser execution. Root re-export lines receive compile/graph evidence without an executable coverage denominator.

The session matrix runs through the aggregate's real finite SQLite host; the API browser backend now uses generated families. API-BUILD-HOST is closed by the source-client build-host; API-MATRIX retains the narrower aggregate-routing gap, and EDGE-REPLAY remains bounded by owning-crate tests. See [source behaviour](source-behavior.md) for the ten resolved G46 decisions and provider/backend/packaging/WHATWG/installed-Tauri boundaries. PGlite is not required. No live delivery, remote acknowledgement, listener or installed application acceptance is inferred from compilation or a renderer.
