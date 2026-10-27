# Slice 8 example review preview — design checkpoint

Status: approved with coordinator amendments on 2026-10-28; Part A web delivery precedes Part B installed review. Scope is the [owner controls](../slice-8-design-verification.md#proposed-owner-visual-acceptance), not owner acceptance itself.

## Host and boundaries

- Add `examples/review-preview` (`stackpress-example-review-preview`), one process, one `stackpress-server-http::HttpServer`, one configured `ServerBuilder`; reuse profile-ui's explicit port/base, asset validation, finite admission and shutdown pattern rather than its buffered-only wire response.
- Add example-only registration seams to gallery/preferences/stream and admin so they share the builder; retain existing finite entry points/defaults. Reuse shared components, generated operations, real stream bodies and checked assets; never proxy through `.invalid` or buffer the streamed page.
- `serve --port N --base PATH --render hydrate|csr --directory DIR --variant disabled|plugin --duration-seconds N` uses an already prepared bundle/database; startup never creates a database. Port has no fallback; only `127.0.0.1` binds. Derive the canonical origin from the explicit port, reject unexpected Host/Origin, and preserve real browser headers.
- New package/module code, scripts and additive wiring stay in examples; root Cargo member/lock additions are approved without existing dependency version changes. No framework, generator, tracked generated-root, production facade or release application changes; stop if a framework fix becomes necessary.

## Examples and routes

`B` means the selected outer base, empty or `/demo`. Each example's assets/actions live under its own mount; `--render csr` selects matching CSR assets globally except streaming, which always streams SSR and hydrates on completion.

| Mount / human entry | Rendering and data | Controls |
| --- | --- | --- |
| `B/ui-gallery/gallery` | Existing shared Gallery; async buffered SSR + hydrate / CSR; fixed public component, table, dialog, Resource and hostile-content fixtures. | 001, 002, 003, 009, 010 |
| `B/generated-admin/profile/search`, `project/search`, `task/search`; `B/generated-admin/roles` | Actual generated pages/stores with SQLite for CRUD/relations/CSV; existing fixture-only role catalog for plugin labels; SSR + hydrate / CSR. | 001, 003–006, 010, 013 |
| `B/ui-preferences/preferences` and `/en/preferences`, `/fr/preferences` within that mount | Existing shared preference UI, host-selected cookies, public flash and fixed sample phrases; SSR + hydrate / CSR. | 007, 008, 010 |
| `B/ui-format/`, `B/ui-format/edit/en`, `edit/fr` | Existing rich/format Proof and editing component; deterministic public fixtures, pinned formatters; SSR + hydrate / CSR, no database. | 007 supplement, 009, 010 |
| `B/ui-stream/stream`, `late`, `pending` | Existing StreamDemo, real Resource/Suspense and replay; stream-only SSR + hydrate, no database. | 012 |

## Setup and review controls

- `python3 examples/review-preview/review.py build --directory /private/tmp/s8-review` builds/copies native gateway, matching hydrate/CSR assets and manifests for every example into `DIR/bundle`; uses installed tools only. `setup --directory DIR` separately creates a new uniquely named SQLite file with create-new/no-symlink checks, installs the current generated schema and seeds related Profile/Project/Task records through generated operations (enough rows to page, plus formula-leading export text). It records the exact file in a run manifest, refuses reuse, and preserves caller files. No in-server reset/delete endpoint.
- `B/review` is a visibly labelled **Review fixtures — not authentication** panel. Editor/reader/denied selection persists in a scoped HttpOnly cookie; the host strips supplied actor headers and derives its fixture policy input from this checked selection on every page/API/relation/import request. Missing/invalid selection denies. The panel can change the next request without replacing a dirty page.
- The same panel supplies bounded slow/failed reads, one lost acknowledgement after an actual SQLite commit, one failed preference save, one public flash, and pause/manual-start hydration. Faults are explicit one-shot host state, not operation JSON. Provide downloadable valid/invalid CSV examples. A public review status shows the selected scenario and cancellation/settlement, without SQL/private diagnostics.
- All fixture selectors/handlers live only in the new review package under an explicit `review-preview` feature plus `debug_assertions`; release compilation with that feature is refused. Normal example/release graphs have no dependency on this package. Additive example seams accept trusted host configuration; existing proof defaults remain unchanged.
- Preferences/gallery actions accept the configured loopback origin rather than their fixed HTTPS fixture origin. Use existing CookieScope/SettingsAction APIs: HttpOnly, SameSite=Lax, persistent Max-Age, example-specific Path, and Secure only for HTTPS. Apply/confirm and flash clearing use the identical scope; never strip flags or forge Origin to make tests pass.
- Streaming: ordinary entry visibly delays 800 ms; `late` selects the existing late-render failure with a short finite render/transport deadline and safe recovery; `pending` stays pending until navigation/disconnect or deadline. The owner uses browser Back immediately to leave. Preserve shell/recovery markup and no completion/hydration on failure; test cancellation capture release separately from browser appearance.

## Plugin and installed target

- Reuse generated-admin's existing disposable workspace and plugin/disabled generation helpers through `review.py bundles`: regenerate with `--plugin`, build matching native/hydrate/CSR, then regenerate without it and rebuild disabled artifacts. Also rerun the original `generated-admin/proof.py build --suite plugin` regression. Never regenerate tracked outputs. `--variant` chooses a prepared compiled bundle; the checklist requires rebuilding, not a runtime label toggle.
- For 013's same-label installed comparison, add a review-only packaged `admin-review.html` role-catalog screen using those exact generated-admin CSR assets and generated wrappers, with a local return link to Profile UI. Bundle/link it only for the explicit debug review build; normal/release frontend assets omit it. It has fixture drafts and no admin mutation/IPC service. Prove marker/asset exclusion in ordinary/release graphs. This is additive example wiring, not a claim of installed admin CRUD.
- For 011 use the existing profile-ui iOS `native/proofs/build-ios.py` and `launch-ios.py` path, bundled assets and installed SDKs. Add a bounded manual-review launch mode that keeps only its owned simulator/display available, exposes fixed debug denial/cancel checks, and always cleans up on exit/deadline. Coordinator runs mobile builds/launches: the existing build invokes a temporary Tauri options listener, which the implementer's no-listener rule forbids. Android remains unverified by owner decision.

## Launch, verification and closeout

After build/setup, coordinator checks `devmetrics list`; owner uses the root hydrate disabled session, then the compiled plugin session:

```sh
devmetrics start --summary "Slice 8 review root" -- '/private/tmp/s8-review/bundle/disabled/slice8-review serve --port {port} --base "" --render hydrate --directory /private/tmp/s8-review --variant disabled --duration-seconds 7200'
devmetrics start --summary "Slice 8 review plugin" -- '/private/tmp/s8-review/bundle/plugin/slice8-review serve --port {port} --base "" --render hydrate --directory /private/tmp/s8-review --variant plugin --duration-seconds 7200'
```

- Repeat after stopping the assigned port with `--render csr`; for plugin review select the rebuilt `bundle/plugin` executable and `--variant plugin`. Devmetrics and the host independently cap lifetime at two hours; finite body/header/asset/admission/idle/stream/drain limits and cancellation/reaping apply. No listener is started by the implementer, including smoke checks.
- Verify service/HTTP over buffered requests and injected duplex accepts: mount/asset isolation, actual generated CRUD/CSV, fresh-file refusal, role/header forgery denials, real HTTP cookies/confirm, one-shot faults, streamed timing/failure/drop/shutdown and manual hydration. Use supplied Playwright `/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright` with port-free interception/duplex relay; distinguish this from coordinator's real-browser network review.
- One owned `DIR/target`, capped at 8 GiB, sequential native/WASM/coverage builds, no installs; every long-running child has a parent timeout and kill/reap in finally. Copy reviewable binaries/assets/evidence outside target, then delete only the owned target. Run fmt, scoped Clippy/tests/doctests/Rustdoc, required example builds and ≥70% measured changed-package coverage; record complexity, responsibility, consistency/docs and safety review rounds with relevant tests.
- Deliver [the owner checklist](00013-review-preview-owner-checklist.md), finalized with exact paths and 3–6 steps per control, plus [the receipt](00013-receipt-review-preview.md) with commands, artifacts, checks, release isolation and remaining limitations. Owner visual/iOS acceptance and coordinator verification remain separate gates. Coordinator approved the debug/review-feature packaged role catalog and manual iOS launcher for Part B; normal/release frontend assets must omit it, with evidence. Part A ends with its receipt before Part B begins.
