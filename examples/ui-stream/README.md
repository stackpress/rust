# Opt-in streamed SSR proof

This example uses the real `stackpress-server-http` Hyper host over a 128-byte
Tokio duplex connection. It never binds or listens. One shared Leptos component
supplies a useful shell, an 800 ms Resource/Suspense section, hostile plain text,
and a hydrated counter. Buffered renderers remain unchanged and remain the
default; opt in with `stackpress-framework-ui/stream`.

`StreamLeptosRenderer::render` returns owned `status` and an exclusive Send body.
The host maps `ViewError` into its native error envelope, attaches that body with
`HttpResponseExt::stream`, applies the staged status, and completes one response
lifecycle before publication. The view itself and its root remain on an owned
local runtime thread. Two channel slots bound producer lookahead. Body Drop
signals cancellation and synchronously joins runtime teardown, including
upstream Resource tasks. Loaders must remain cooperative and bounded; blocking
CPU or destructors cannot be preempted. The host must bound admission.

UI `stream::StreamLimits` defaults to a 64 KiB fragment and a 30 s absolute render
deadline; zero/unrepresentable deadlines are invalid. Aggregate HTML/replay
limits come from `PageContract`. A fixed failure marker is reserved inside the
HTML budget. HTTP `StreamLimits` separately bounds bytes and delivery deadlines;
whichever deadline expires first cancels the owned work. Frozen Head values do
not change after publication.

Direct factory/Head/body-poll failures retain native diagnostics and stop the
stream. Leptos 0.8.22 also reevaluates Suspense children in an internal effect
task. A panic there does not propagate through the body future; any_spawner
0.3.0 discards its Tokio JoinHandle. Stable Tokio APIs cannot retrieve that
handle. This path stops at the configurable deadline, preserves a typed timeout
cause and cancels/reaps the entire private runtime. It does not install a global
panic hook, custom global executor or unstable Tokio policy.

The precommitted recovery notice is available even if the connection fails
before a late marker arrives. Failure never rewrites status, retries a write,
reenters the error lifecycle, or adds a completion marker. The bootstrap hydrates
only a completed document. It validates and merges escaped inert replay,
removes replay templates before hydration, and leaves interrupted markup
unhydrated with a safe reload link.

Run finite builds with `python3 examples/ui-stream/build.py`. It requires the
already installed matching wasm-bindgen 0.2.129 and writes assets and a finite
native host to `/private/tmp/s8-stream` by default. Set `UI_STREAM_OUTPUT_DIR`
for both commands to select another artifact directory, including nested paths.
Set `CARGO_TARGET_DIR` to use one owned native/WASM compilation root; the
default remains `target/s8-native` and `target/s8-wasm`. Run the browser proof with
`UI_STREAM_PLAYWRIGHT=<installed-playwright-package> python3 examples/ui-stream/browser.py`.
The parent kills and reaps the owned process group at 120 s. There are no
installs, listeners or remote requests.

The browser intercepts only `https://stream.invalid`, installs an ephemeral
service worker and feeds its navigation `ReadableStream` with exact body
fragments emitted by the native Hyper duplex exchange. Each pull obtains the
next native fragment without inventing HTML, replay or completion markers.
Chromium proves real incremental HTML parsing, visible shell before delayed
content, inert hostile text, unchanged hydrated DOM/node identity, working
interaction, zero initial Resource fetches, safe late recovery, and navigation
cancellation at root and `/demo`. The native tests independently prove producer
backpressure, ordered resolution, byte ceilings, response lifecycle, deadlines,
shutdown and capture release. Browser interception does not prove real TCP/TLS
flush, browser network-buffer backpressure or remote acknowledgement. All worker
registrations and contexts are closed in `finally`.

Review artifacts are `<UI_STREAM_OUTPUT_DIR>/browser.json` (default
`/private/tmp/s8-stream/browser.json`), native/browser logs
and `*-shell.png`, `*-settled.png`, `*-375.png`, `*-recovery.png`. These are a
bounded review packet for control 00013-012; owner acceptance remains separate.
