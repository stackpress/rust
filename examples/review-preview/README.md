# Slice 8 owner review preview

This debug-only example opens the Slice 8 gallery, generated SQLite admin,
preferences, formatting and streamed pages in a normal browser. The owner uses
one main session, then a compiled plugin session for control 013. Installed iOS
review is Part B; Android remains unverified.

From the repository root, using already installed Rust, wasm-bindgen 0.2.129 and
Playwright tools:

```sh
python3 examples/review-preview/review.py build --directory /private/tmp/s8-review
python3 examples/review-preview/review.py setup --directory /private/tmp/s8-review
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
select Editor and keep that compact
panel in a second tab. Follow the [owner checklist](../../.agents/specs/00001-stackpress-rust-foundation/tasks/00013-review-preview-owner-checklist.md).
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
Tracked generated outputs are never rewritten. A build-time variant marker
prevents mixing the wrong executable and bundle. The host and devmetrics both
limit the session to two hours. Stop the assigned port when finished. Optional
`--base /demo` and `--render csr` sessions are extra checks, not required owner
repetitions. Streaming always uses actual streamed SSR and hydrates only complete
pages; its delayed, late-failure and pending links are on the panel.

The new package is not a dependency of normal examples or the production facade.
Its handlers require `review-preview` and debug assertions; release compilation
with the feature is refused. `pipe` uses the exact prepared HTTP host over duplex
IO and never binds. Browser interception proves interaction and real cookie jar
behavior, while native duplex tests separately prove progressive delivery and
cancellation. Neither substitutes for the owner's review over the real managed
loopback listener. No test or build installs tools or starts a development server.
