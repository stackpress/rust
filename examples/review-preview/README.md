# Slice 8 owner review preview

This debug-only example opens the Slice 8 gallery, generated SQLite admin,
preferences, formatting and streamed pages in a normal browser. The owner uses
one main session, then a compiled plugin session for control 013. Installed iOS
review is Part B; Android remains unverified.

From the repository root, using already installed Rust, wasm-bindgen 0.2.129 and
Playwright tools:

```sh
# Point this at the main checkout's store when running in a worktree.
REVIEW_BENCH_STORE=/path/to/main/checkout/.agents/resources/build-benchmarks
python3 .agents/scripts/build-benchmark.py --store "$REVIEW_BENCH_STORE" run --label "review preview build" --kind build --scope selected --cache unknown -- python3 examples/review-preview/review.py build --directory /private/tmp/s8-review
python3 .agents/scripts/build-benchmark.py --store "$REVIEW_BENCH_STORE" run --label "review preview setup" --kind other --scope other --cache unknown -- python3 examples/review-preview/review.py setup --directory /private/tmp/s8-review
```

Builds use one measured eight-GiB-capped `target` directory. Native executables,
matching browser modules, manifests and evidence are copied outside it. Setup is
an explicit separate step: it claims a new manifest and uniquely named SQLite
file, installs the current generated tables and seeds real generated operations.
A second setup refuses to overwrite the fixture. A failed setup retains its
partial files for inspection; choose another fresh directory rather than deleting
or reusing caller data automatically. Serving never creates or resets a database.
The operator must own the directory exclusively during build/setup/startup;
portable metadata/open checks cannot prevent hostile concurrent filesystem edits.

The coordinator starts the main session through devmetrics:

```sh
devmetrics list
devmetrics start --summary "Slice 8 main owner review" -- '/private/tmp/s8-review/bundle/disabled/slice8-review serve --port {port} --base "" --render hydrate --directory /private/tmp/s8-review --variant disabled --duration-seconds 7200'
```

Open `http://127.0.0.1:<assigned-port>/review` using the port reported by devmetrics,
a fresh visitor defaults to Editor. Keep that compact panel in a second tab;
choose Editor to reset an earlier caller selection. Follow the [owner checklist](../../.agents/specs/00001-stackpress-rust-foundation/tasks/00013-review-preview-owner-checklist.md).
Every scenario names its control; the status line reports only public fixture
state. Incoming actor headers do not override the selected caller. These are
labelled review fixtures, not authentication. Lost acknowledgements occur after
actual SQLite commit and remain visibly uncertain. Faults never survive restart.

After stopping the assigned devmetrics port, start the plugin session:

```sh
devmetrics stop <assigned-port>
devmetrics start --summary "Slice 8 plugin owner review" -- '/private/tmp/s8-review/bundle/plugin/slice8-review serve --port {port} --base "" --render hydrate --directory /private/tmp/s8-review --variant plugin --duration-seconds 7200'
```

The build reuses generated-admin's disposable workspace generation path: select
the plugin, regenerate and compile; deselect it, regenerate and compile again.
Tracked generated outputs are never rewritten. The review builder makes an
explicit isolated `schema-review.idea` input that changes only the profile name
label to **Full name**; each bundle records both source hashes in
`review-schema.json`. Normal generated-admin proofs retain the hostile source
label. The compiled plugin still replaces it with **Plugin field label**.
`bundles` reuses existing example assets; wrap it with the same benchmark
recorder (`--kind build --scope selected`), choosing the actual cache state. A build-time variant marker
prevents mixing the wrong executable and bundle. The host and devmetrics both
limit the session to two hours. Stop the assigned port when finished. Optional
`--base /demo` and `--render csr` sessions are extra checks, not required owner
repetitions. Streaming always uses actual streamed SSR and hydrates only complete
pages; its delayed, late-failure and pending links are on the panel.

The new package is not a dependency of normal examples or the production facade.
Its handlers require `review-preview` and debug assertions; release compilation
with the feature is refused. `pipe` uses the exact prepared HTTP host over duplex IO and never binds.
`stream-pipe` accepts one bounded pipe request and emits native response headers
and base64 body frames as NDJSON, ending with a separate terminal/interrupted
report. It preserves native HTTP truncation for progressive browser proofs. Browser interception proves interaction and real cookie jar
behavior, while native duplex tests separately prove progressive delivery and
cancellation. Neither substitutes for the owner's review over the real managed
loopback listener. No test or build installs tools or starts a development server.

The consolidated owner checklist includes inline collection actions, plain tag
chips, the 48×12 overflow table, native star ratings, readable role lists, labelled
Safety samples and HTML denial/error pages. Typed nested rows is proof-only.
Streaming packages the fixed external async `stream-status.js` separately from
the completion-gated bootstrap. A parsing end/failure marker uses “didn't finish
loading”; an unobservable end falls back after six seconds to “hasn't finished
loading”. Both offer the checked Reload link and never hydrate a partial page.

## Installed iOS review

The coordinator runs mobile builds and launches. The build's temporary Tauri
options service is part of the existing iOS path; the implementer does not run
it. No frontend development server is used. First prepare both matching CSR
catalogs with this listener-free command:

```sh
python3 .agents/scripts/build-benchmark.py run --label "00013 installed review assets" --kind build --scope selected --cache unknown -- python3 examples/review-preview/installed.py assets --directory /private/tmp/s8-review
```

This reuses the isolated **Full name** schema, builds plugin then disabled,
and cleans the shared/client WASM cache at each boundary. It writes
`ios-assets/{plugin,disabled}` with hash inventories and schema provenance.
Ordinary Profile UI CSR output is unchanged. Only the explicit debug
`review-preview` build selects these folders; normal iOS builds replace their
separate `gen/csr-bundle` from ordinary assets. Release review builds refuse to
compile. The role catalog mounts the actual generated-admin CSR component with
local drafts. It has **no admin mutation or IPC service and is not installed
admin CRUD**. Raw fixture snapshots remain available to proofs and appear only
inside the collapsed **Debug state (review only)** disclosure. Hostile labels
belong to **Safety samples**.

Use two installed sessions: plugin first for 011 and the plugin half of 013,
then disabled to check that **Full name** returns. Stop any existing simulator
display and booted device yourself before starting; the launcher refuses to
reuse or close them. From the repository root:

```sh
IOS_REVIEW_VARIANT=plugin
python3 .agents/scripts/build-benchmark.py run --label "00013 iOS $IOS_REVIEW_VARIANT build" --kind build --scope selected --cache unknown -- env PROFILE_UI_IOS_LOGS="/private/tmp/s8-review/ios-logs/$IOS_REVIEW_VARIANT" CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 python3 examples/profile-ui/native/proofs/build-ios.py --directory /private/tmp/s8-review --review-variant "$IOS_REVIEW_VARIANT"
python3 .agents/scripts/build-benchmark.py run --label "00013 iOS $IOS_REVIEW_VARIANT manual review" --kind test --scope other --cache unknown -- env PROFILE_UI_IOS_LOGS="/private/tmp/s8-review/ios-logs/$IOS_REVIEW_VARIANT" python3 examples/profile-ui/native/proofs/launch-ios.py --manual-review --variant "$IOS_REVIEW_VARIANT" --duration-seconds 1800
```

The build copies its actual unsigned simulator app into `ios-apps/<variant>`;
the launcher checks that variant's build receipt and that its options service
has closed. It creates one disposable simulator using the existing installed
runtime/device type. On the Duo display, use **primary / LCD 1**. The two labelled
fixed buttons above Profile UI exercise denial and a cancellable four-second
read through the existing native commands. Follow checklist 011, then open
**Generated admin review** to compare the plugin label. No automated proof
driver manipulates this manual session.

In a second terminal, restart the same app/container to check saved settings,
or finish the session. Keep `IOS_REVIEW_VARIANT=plugin` in that terminal too:

```sh
python3 .agents/scripts/build-benchmark.py run --label "00013 iOS restart" --kind other --scope other --cache unknown -- env PROFILE_UI_IOS_LOGS="/private/tmp/s8-review/ios-logs/$IOS_REVIEW_VARIANT" python3 examples/profile-ui/native/proofs/launch-ios.py --manual-review --variant "$IOS_REVIEW_VARIANT" --control restart
python3 .agents/scripts/build-benchmark.py run --label "00013 iOS finish" --kind other --scope other --cache unknown -- env PROFILE_UI_IOS_LOGS="/private/tmp/s8-review/ios-logs/$IOS_REVIEW_VARIANT" python3 examples/profile-ui/native/proofs/launch-ios.py --manual-review --variant "$IOS_REVIEW_VARIANT" --control stop
```

Wait for the first terminal's cleanup report, set `IOS_REVIEW_VARIANT=disabled`
in both terminals, and repeat the build/launch commands above. Open the local
catalog and check **Full name** returns; finish with the matching stop command.
Each launch allows 30–7200 seconds, default 1800, including setup. Deadline,
Ctrl-C, SIGTERM, app failure and normal stop all enter cleanup; the parent reaps
its worker and retries recorded device/display cleanup. Only the launched
display and observed descendants are claimed. Failed cleanup retains its
ownership record for inspection rather than declaring success. Restart keeps
settings inside the same session; finishing deletes that disposable device.

Mobile build, real simulator lifecycle and owner acceptance remain coordinator
checks. Listener-free verification is available through benchmark-wrapped
`installed.py tests`, `quality`, `isolation`, `coverage` and `browser`, each with
`--directory /private/tmp/s8-review`. These proofs never launch an app/simulator.
They check actual CSR bytes in Chromium, native cancellation, asset exclusion
and cleanup policy with platform effects mocked. Android remains unverified.

Use the recorder for every Cargo/proof/clean invocation; wrap an outer proof
once instead of its nested commands. For the composed gateway recovery proof:

```sh
python3 .agents/scripts/build-benchmark.py --store "$REVIEW_BENCH_STORE" run --label "review progressive stream" --kind test --scope other --cache unknown -- python3 examples/review-preview/review.py run --directory /private/tmp/s8-review --name review-stream --timeout 180 -- env REVIEW_DIRECTORY=/private/tmp/s8-review REVIEW_PLAYWRIGHT=/path/to/installed/playwright node examples/review-preview/stream-browser.cjs
```

This proof reads actual native HTTP frames through an intercepted service worker,
including real truncated-body errors, and serves assets through the gateway
manifest under a strict no-inline-script CSP. It covers success, late failure and
pending deadline at root and `/demo`; each owns/reaps its finite children. It
proves progressive parser behavior without binding a socket. Real managed
loopback review and installed-target acceptance remain separate.
