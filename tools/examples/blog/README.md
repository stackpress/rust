# Blog contributor proofs

These repository tools verify `examples/blog`; they are not part of the copied boilerplate.
The application setup and browser build use Rust commands documented in the
[blog README](../../../examples/blog/README.md). Run the commands below from the repository root.

Prerequisites: Python 3.11 or newer, the Rust toolchain with rustfmt and Clippy, the WASM target,
matching wasm-bindgen CLI, sccache, cargo-nextest, and rustc's LLVM tools for coverage.
The gate uses source instrumentation directly; cargo-llvm-cov is only needed for the
repository's initial coverage cleanup. Browser proofs need Node, Playwright and Chromium.
Playwright must resolve as the `playwright` Node package, or set `BLOG_PLAYWRIGHT` to the
absolute path of its installed package. The tools never install dependencies.

## Verify

From the root of a fresh Stackpress Rust checkout, run this exact sequence using the
already installed toolchain, cached dependencies, WASM target, matching `wasm-bindgen`, nextest,
LLVM tools and cached Playwright. As with the other contributor commands, record the sequence
through the repository benchmark recorder and its persistent store. No earlier `.build/`
directory, generated source, database, lock-resolution marker or review patch is required.

```sh
export RUSTC_WRAPPER=sccache SCCACHE_CACHE_SIZE=20G
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="$PWD/examples/blog/target/verify/native"
python3 tools/examples/blog/gate.py
```

The gate first runs the same `generate-app` and `generate` commands described above, then
`tools/examples/blog/extension.py` creates the generated nullable-Profile fixture that the integration tests
compile. It builds actual browser assets before the native HTTP asset test, explicitly builds
the stdin/stdout `probe` executable, and prepares the copied-out nickname project/database and
both browser products. Its copy step changes only existing repository-local dependency paths
to this checkout; every source/copied/root lock stays unchanged. These are finite operations;
the gate opens no listener and installs nothing.

It then checks formatting, freshness, native and separate-renderer Clippy, all native tests,
docs, dependency graphs, public/authenticated/copied-out browser flows, and native coverage
for `blog-app`, `blog-assets`, `blog-store`, `blog-web`, `blog-tool` and `blog-xtask`. Browser graphs select
`blog-app` UI without its server feature or either native plugin.
The gate freezes root lock bytes from entry through exit. Workspace-only package/edge edits may precede the gate.
The lock check compares all 699 registry tuples with HEAD and recomputes the example subset, without
reading a marker. G58 allows exactly tungstenite 0.29.0 and its five enumerated transitive
exceptions in the blog lock only; the check asserts their versions, sources and checksums.
The JavaScript phase also exercises the shipped hot-patch failure guard without a listener;
xtask's tests cover classification, debounce, process cleanup, markup safety and in-memory WebSocket IO.
Logs and per-phase results are in `examples/blog/.build/gate/`; compiled targets stay under
the dedicated `examples/blog/target/verify/` budget. Completed consumer/browser targets may be retired before
coverage; exact browser products and test reports remain in `.build/`.

For later narrow edit loops, `python3 tools/examples/blog/proof.py storage` and `python3 tools/examples/blog/proof.py generation` reuse the prerequisites
created by this sequence. The storage proof enables `blog-web/extension-proof` for the generated extension fixture;
normal copied-app tests do not need that repository-only fixture. The suite includes the
Catalog redirect, native HTTP Date and cookie-scope regressions. `blog-client` retains its G56
coverage exemption, exercised by actual hydration/CSR browser tests.

## Individual proofs

All paths are relative to the repository root; record each invocation through
`.agents/scripts/build-benchmark.py` and its persistent repository store.

| Tool | Purpose |
| --- | --- |
| `readme.py` | Run the README setup, normal tests and migration in disposable copied apps. |
| `format.py` | Check authored Rust, including xtask; `--write` formats it. |
| `freshness.py` | Check both generated trees without changing bytes or timestamps. |
| `graphs.py` | Check native and separate browser dependency boundaries. |
| `locks.py check` | Compare all 699 registry tuples with HEAD and check the example subset plus G58. |
| `extension.py` | Generate/check the extra Profile fixture used by the full gate. |
| `browser.py` | Run public and authenticated browser flows through in-process routing. |
| `nickname.py prepare\|check\|browser` | Prepare/check/test the copied app's Profile migration. |
| `proof.py storage\|generation` | Run narrow nextest suites after the gate's generation prerequisites. |
| `g56.py` | Measure extracted plain-data browser decisions with native instrumentation. |
| `prepare.py` | Prepare a separate smoke database and print a vanilla serve command. |
| `http.py ORIGIN` | Make finite requests to an explicitly supplied smoke server; never launches one. |
| `coverage.py` | Internal native coverage phase, imported by the gate. |

`cargo xtask build-browser` replaces the former build proof and lives inside the example.
Only `blog-client` has the G56 coverage exemption; `blog-xtask` is measured against 70%.
