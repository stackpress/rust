# 00044 — Blog plugin crates

Status: **implemented; clean-copy Verify passed; awaiting coordinator verification.** Based on 8dd37cb1ccedf2ffc8d60de4bea218077b9fad08; no commit.

## Implementation notes

- [Project layout](../../../../examples/blog/README.md#project-layout) mirrors original `templates/blog/plugins/{app,assets,store}`: unpublished `blog-app`, `blog-assets`, `blog-store`, selected directly in `web/src/client.rs`. New crate edges are paths; workspace `publish = false` is inherited. No plugin manager.
- `plugins/app/src/` owns the former shared UI/data decisions, generated admin selection, helpers/views and moved lifecycle tests. `server` gates pages/hooks/registration; `ui` and separate SSR/hydrate/CSR passthroughs gate portable code. No shared crate remains.
- `blog_app::register` takes the host's plain render function; immutable callback forwarding keeps config/projection/SSR in `web/src/render.rs` without a reverse dependency. Route priorities, response/error guards and operation bodies are retained.
- `plugins/assets/` owns registration, CSS/brand/bootstrap bytes; `plugins/store/` owns registration and `connect.rs`. Generated DDL/setup moved to `web/src/storage.rs`; `web/src/commands.rs` stays with its generated catalog and durable History, because these bind host schema/revision policy. The original store only connects.
- Existing tests follow app code; cross-plugin/CLI/SQLite/auth/HTTP tests remain in web. Three plugin-local fixture tests add standalone connection/config and authored/build-asset/skip evidence. README, commands guide and format/graph/coverage/nickname proofs use the new paths. Tool, schema, config, seed, routes and output behavior are unchanged.

## Review rounds

| Round | Review and evidence |
| --- | --- |
| Logic / complexity | Source parity audit; guards, FIFO/priority and callback error forwarding retained; no policy simplification justified. 16 plugin tests pass: `20261107T023430.430360Z-8e05c0319b5f`. |
| Responsibility / abstraction | Removed shared/plugin facade; explicit connection versus host-generated command ownership. Featureless app and standalone native plugins compile; native and both browser graphs pass; official generate/check and 11 affected host tests pass: `20261107T023458.279751Z-f7d4c47ccefb`. |
| Consistency / Rustdoc / comments | Documented features/render lifetime and ownership; updated paths, formatted authored Rust. 16 plugin tests and doctests pass (0 doctest examples): `20261107T023821.728098Z-126003036098`. |

Safety review: no new unsafe code, synchronization or mutable global state, listener or background task. Callback borrows/future remain bounded by the original invocation; file setup/shutdown and resource ownership are unchanged. Initial narrow failures exposed an omitted source-server feature and two new fixture assumptions; all corrected before rounds/final gate.

## Final clean-copy gate

- Detached HEAD plus scoped diff and 33 new source files at `/private/tmp/blog-clean-00044-8dd37cb-01`; `.build` and all targets absent. Extracted and ran the README Verify block verbatim **once**. All 26 phases green: fmt; native/hydrate/CSR Clippy `-D warnings`; 52 native tests; Rustdoc `-D warnings`; doctests; official generation/freshness; standalone-selection graphs; real SQLite; six public browser matrix cases; both authenticated/admin/API/export and copied-out nickname browser flows. Cached Playwright only, in-process routing, no listener.
- Native graph selects all three plugins. Browser graphs each have 225 packages and exclude store/assets, SQL/server/mail/identity/codegen. G56 exempts only blog-client startup/mount/disposal and thin DOM/navigation/fetch/AbortController glue; plain decisions are natively measured in blog-app.

| Native crate (shipping generated code included) | Executable lines |
| --- | --- |
| blog-tool | 102/113 (90.27%) |
| blog-web | 2325/3310 (70.24%) |
| blog-app | 3698/3882 (95.26%) |
| blog-store | 38/39 (97.44%) |
| blog-assets | 59/66 (89.39%) |

- One approved root-seeded offline refresh, record `20261107T022944.213508Z-1b5ba4c82a54`: exact 437 registry tuples unchanged, root 699 subset and root manifest/lock bytes unchanged immediately after resolution. All later Cargo commands locked/offline. `locks.py check` recomputes both committed inventories, without a marker.
- Peak sampled target 10.360 GiB, below 12 GiB. Completed edit-loop target retired before clean proof; disposable target/worktree removed afterward. Evidence: `examples/blog/.build/clean-copy/00044-8dd37cb-01/{source.json,fix.patch,verify.sh,gate,browser,nickname-browser,ledger.json}`.
- Final gate recorder `20261107T023906.654249Z-b2151d8b958a`: command wall 1609.510197s; observed Cargo stages 1385.50s. Canonical store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Detailed attempts/status/timings are retained in ledger.json; accumulated command/stage durations are not session elapsed time. LLVM export reported `37 functions have mismatched data`; the warning and unfiltered raw report are retained as measurement diagnostics.
- Through that gate: 11 recorded attempts (9 passed, 2 failed), 6 emitting compilation, no root-workspace rebuild; accumulated command wall 1989.503749s versus observed Cargo stages 1580.06s. The two failed narrow attempts above preceded the passing rounds; closing audit/receipt validation is additional.

No new behavior gaps; original web-only/MCP/delivery boundaries remain. Source audit preserves all 25 upstream source files and unchanged tool/schema/config/seed/root bytes. Closing audit `20261107T030630.611381Z-18ba3ab189b8` passed. KB validation retains the inherited 146 errors / 43 warnings (exit 1), with no new diagnostics; this is not clean KB validation.

## Verification notes

Coordinator review pending.
