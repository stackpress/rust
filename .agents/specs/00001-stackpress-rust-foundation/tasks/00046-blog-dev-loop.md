# 00046 — Blog development loop

Status: **implemented; clean-copy verification passed; coordinator review pending.**
Based on `65f959e`; no commit, push, install or development listener.

## Implementation

- [README](../../../../examples/blog/README.md#develop-with-live-reload): `cargo xtask dev [--port N] [--reload-port M]`, defaults 3000 and app port + 1; Ctrl-C stops owned processes. The standalone browser build remains finite.
- Content polling watches plugins, web/client source, assets, schema, config and workspace manifests/lock. A 250ms quiet window coalesces bursts, including edits during compilation. Schema changes generate before compiling consumers; native-only changes skip WASM. Browser-only builds reuse/restart the executable to advance the HTML build ID.
- Both browser products are staged; publication waits for all builds to succeed and restores prior output on rename failure. Failed builds preserve the active server and bundles. Replacements receive a new process group; Ctrl-C, deadlines and unwinding kill/reap owned groups. App shutdown gets a bounded SIGINT drain first.
- An independently ticking reload worker owns the loopback WebSocket while Cargo runs and while the app restarts. Tungstenite handles upgrades and framing; the endpoint is `/live_reload`. Leptos receives `{"all":true}`, `{"css":"styles/global.css"}` or `{"view":"<serialized Patches>"}`. Revision feedback rejects stale patch failures; build IDs reload tabs opened against last-good HTML during compilation. New tabs replay sequential patches; 64 retained patches trigger a compiled checkpoint.
- CSS-only changes read fresh authored CSS and swap links without compiling/restarting. Plain serving retains embedded bytes. [Dev document decoration](../../../../examples/blog/web/src/dev.rs) adds the official AutoReload script only with xtask's runtime environment. A controlled public snapshot compares byte-identical HTML with/without dev decoration; actual public/auth lifecycle responses are also exercised.
- [Markup safety](../../../../examples/blog/xtask/src/dev/markup.rs) uses syn plus the real `ViewMacros` parser/differ. Only shared app views qualify. Static trees may change structure; mixed trees allow top-level literal text/attribute edits while Rust groups, component props/children, macro counts/lines and surrounding Rust stay exact. Parse errors and uncertain edits rebuild. Accepted markup is compiled into SSR/WASM before any later full reload.
- The client guard waits for hydration, detects missing markers/CSR and forwards thrown or internally caught patch failures for full rebuild/reload. Finite Node tests execute the shipped guard. The gate retains all 26 phases and covers xtask without a G56 exemption; only the thin blog-client remains exempt.

## Upstream prototype

Cached sources were read-only under `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`:

- `leptos_macro-0.8.19/src/lib.rs:379-389` builds debug stable IDs; `src/view/mod.rs:111-117` attaches them to views.
- `leptos-0.8.22/src/into_view.rs:112-133` emits SSR comments when **compiled** with `LEPTOS_WATCH`; `tachys-0.2.19/src/renderer/dom.rs:161-201` skips these comments during hydration. `View::build` in `into_view.rs:81-82` delegates directly, so CSR creates no markers and takes the rebuild path.
- `leptos_hot_reload-0.8.7/src/patch.js:11-13` locates those comments; its catch at `223-225` warns instead of throwing. The guard preserves the official patcher and converts that warning into rebuild feedback.
- The real hydrated blog prototype had 50 marker comments. Official `ViewMacros` patches changed the homepage brand text and class in place, without warnings or browser errors; all public browser steps passed (20 requests). No listener: native routing used the stdin/stdout probe and cached Playwright interception. Successful record: `20261110T040836.875963Z-42cd070ba206`.
- Earlier prototype attempts included a disposal-only marker assertion and an overwritten probe artifact; these were corrected before drawing the successful conclusion. Raw experiments remain in `examples/blog/.build/dev-evidence/`.

## G58 lock exception

The [owner decision register](../decisions.md) on main authorizes tungstenite in the blog lock only. One root-seeded **offline** resolution ran from cache: `20261110T035037.634333Z-0bd0a036156d`. Immediate assertions confirmed unchanged root bytes and all **699 ordered tuples**, and that every extra belongs to tungstenite's dependency closure. The blog has **448** registry tuples: root subset plus exactly these six exceptions. All use `registry+https://github.com/rust-lang/crates.io-index`:

| Package | Version | Checksum |
| --- | --- | --- |
| data-encoding | 2.11.1 | `4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06` |
| rand | 0.9.4 | `44c5af06bb1b7d3216d91932aed5265164bf384dc89cd6ba05cf59a35f5f76ea` |
| rand_chacha | 0.9.0 | `d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb` |
| rand_core | 0.9.3 | `99d9a13982dcf210057a8a78572b2217b667c3beacbf3a0d8b454f6f82837d38` |
| sha1 | 0.10.6 | `e3bf829a2d51ab4a5ddf1352d8470c140cadc8301b2ae1789db023f01cedd6ba` |
| tungstenite | 0.29.0 | `6c01152af293afb9c7c2a57e4b559c5620b421f6d133261c60dd2d0cdb38e6b8` |

The [lock checker](../../../../tools/examples/blog/locks.py) asserts the exact source/version/checksum allowlist, inventory, root bytes/tuples and handshake-only manifest features. A disposable tampered checksum is rejected. No TLS feature or root lock edit.

## Review rounds and narrow proofs

| Round | Finding / verification |
| --- | --- |
| Logic / complexity | Capture source before the initial build; distinguish native-only plugin code; retain staged bundles on every build failure; validate mixed macro edits, line shifts, props and dynamic attributes. Narrow tests passed (`20261110T041700.555709Z-69b4ace2c5ef`). |
| Responsibility / abstraction | Separate pure protocol/history state from socket IO; keep the reload worker live during Cargo; bind failure feedback to revisions and HTML to build generations. Narrow Rust and actual client-guard tests passed (`20261110T042033.835477Z-ada4713018ba`, `20261110T042038.642188Z-c3a5a52c6c50`). |
| Consistency / Rustdoc / comments | Format authored Rust, document ownership/cancellation, use let chains, and narrowly explain tungstenite's required concrete callback error type. Strict Clippy passed (`20261110T042949.478360Z-49afb33c4f22`). Follow-up HTML tests use identical snapshot data; database timestamps and generated CSRF values remain uncontrolled source behavior rather than being normalized. |

- Native production xtask coverage (canonical source paths, fresh profiles, no listener exclusions): **660/875 = 75.43%**, `20261110T042451.981371Z-985d7db94bb6`. The final full-gate measurement supersedes this edit-loop sample.
- Three README/docs files and 12 local links resolve; tracked example has no excluded tooling mentions or scripts, and G58 tampering is rejected: `20261110T042536.875790Z-ca17a981599b`.
- KB baseline: **146 inherited errors / 43 warnings**, exit 1, `20261110T042711.494362Z-fe258d306ad9`; this is not clean KB validation.
- Every build/test/proof uses the persistent benchmark recorder/store, sccache, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, two jobs and targets under this worktree. Command wall and observed Cargo stages are separate; neither summed measure is session elapsed time.

## Final verification

- Detached clean copy `/private/tmp/blog-clean-00046-65f959e-01`, HEAD `65f959e285ab2827f40acb2b2429e84e3a3ccdcb`, initially had no `.build` or target. Applied patch SHA256: `24323d12451e41d87885ae940ff6adf779bbf59b3428383cd3bfff6f5f2dc02e`. The source target was retired first (`20261110T044445.782041Z-66621b9d0aeb`).
- README user path passed: generate/check, SQLite setup/revision/population/query, both browser bundles, **79 normal tests**, and the nullable Article migration retaining both article IDs with null subtitles. Record `20261110T044616.306155Z-6669a9f7421d`: command wall **647.669944s**, observed Cargo stages **605.83s**.
- The full gate ran **once**: all **26 phases passed**, including **81 native tests, none skipped**, strict native/hydrate/CSR Clippy, doctests, Rustdoc, graphs/freshness, the six-case public matrix, full authenticated/admin/editor/API/export flows, the copied Profile migration and both copied browser modes, native coverage and locks. Record `20261110T050017.186279Z-c2e3d9a73c0f`: command wall **1488.003864s**, observed Cargo stages **1235.14s**.

| Native crate (shipping generated code included) | Executable lines |
| --- | --- |
| blog-tool | 102/113 (90.27%) |
| blog-web | 2371/3358 (70.61%) |
| blog-app | 3698/3882 (95.26%) |
| blog-store | 38/39 (97.44%) |
| blog-assets | 70/78 (89.74%) |
| blog-xtask | 671/888 (75.56%) |

- Only blog-client is G56-exempt. Production xtask source includes unexecuted listener/loop wiring; test files are excluded by canonical resolved paths, with no listener coverage exemption. The gate's instrumented 81-test run and native HTTP/browser coverage phase also passed.
- LLVM export reported **92 functions have mismatched data**. The diagnostic and complete raw report are preserved; coverage is not described as warning-free. Normal test output can also show `No such process` from best-effort group cleanup after the leader has already exited; child waiting and descendant cancellation proofs pass.
- Peak sampled compiled targets: **11,436,949,504 bytes (10.651 GiB)**, below 12 GiB. Exact tested code/README/proof-script hashes, all 26 results, coverage, unchanged root bytes/699 tuples and the blog's 448-tuple G58 policy passed the audit (`20261110T052624.719725Z-264862642a6e`).
- Verified reports, provenance and frozen patch (worker-local, ignored `examples/blog/.build/clean-copy/00046-65f959e-01/audit.json`) are retained with per-phase logs, raw coverage, browser evidence, recorded launch sequence and benchmark records. The detached copy and every owned compilation target were deleted (`20261110T052643.009166Z-9f3aa04a2014`). No other lane's output was touched.
- The final KB check retains exactly the baseline **146 inherited errors / 43 warnings**, with no new diagnostics. All shipping source remains identical to the tested snapshot; only this final receipt's evidence was filled in afterwards.
- A previously absent, ignored source `.build/blog.sqlite` was prepared through the documented finite commands for the coordinator's launch check; its query returns two articles (`20261110T044012.821070Z-f8436a0ff19c`). No existing database was overwritten.

The coordinator must start the real listener check from `examples/blog` with
`cargo xtask dev --port 3000 --reload-port 3001`, then open `http://localhost:3000/`.
No product acceptance, merge or real listener success is inferred from finite proofs.
Process tests ran on macOS; the Windows cleanup branch and actual TCP listener startup remain unexecuted.

## Coordinator verification

Committed as `3ab4127` and merged into main as `89e164a`. Reviewed:
- the G58 lock exception: tungstenite 0.29.0 plus five closure tuples, blog lock only, root unchanged;
- dev-only AutoReload injection, with a byte-identical plain-serve test;
- the markup-patch safety fallback.

Independent run on a detached clean checkout of `89e164a`, 2026-11-10, exit 0:
- **README user path:** generate, setup, populate (2 articles), `cargo xtask build-browser`, `cargo test --workspace` (79 passed).
- **Contributor gate** `tools/examples/blog/gate.py`: 26/26 phases passed. Coverage: blog-tool 90.27%, blog-web 70.61%, blog-app 95.26%, blog-store 97.44%, blog-assets 89.74%, blog-xtask 75.79%; blog-client G56-exempt.

Live check: `cargo xtask dev --port 3000 --reload-port 3001` ran via devmetrics on this machine. The first build took about 2 minutes, then the server became ready and the page carried the `/live_reload` script.
