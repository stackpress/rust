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
