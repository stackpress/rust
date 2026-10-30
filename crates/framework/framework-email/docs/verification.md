# Verify documentation and consumers

The documentation uses plain Rust Markdown fences. `check.py` copies every marked fence unchanged into an isolated module in a disposable crate outside the repository; it does not add examples to production source or change manifests/tests.

## Check claims and links

Run `python3 crates/framework/framework-email/docs/check.py` from the checkout through the project's benchmark recorder. It checks one H1, heading orientation, tagged/classified fences, spacing, every local link/anchor and the [claim evidence map](claims.json). There are no external Markdown links to probe.

The map binds specific documented statements to implementation and regression-test anchors. Those checks detect drift; they do not independently prove arbitrary prose. Review defaults, option precedence, mutation/error order and platform claims against the linked implementation/tests as well. [Source behavior](source-behavior.md) retains the native/provider evidence limits.

## Compile every example

Set the absolute effort `CARGO_TARGET_DIR` and invoke the same checker with `--compile` through the recorder and the existing [finite proof wrapper](../tests/run-proof.py). It copies the existing direct/event consumer manifests with absolute dependency paths and byte-identical locks to owned temporary workspaces. All check/test commands use --locked --offline.

Each fence selects its graph through an invisible `example: direct` or `example: server` marker. Every Rust fence is a separate module, so examples cannot supply each other's missing imports. Each consumer has its own `docs-direct` or `docs-server` subdirectory beneath the chosen absolute `CARGO_TARGET_DIR` (target/slice9b for this effort). The checker also executes the listed finite zero-argument examples: local CSRF/JSON/stream, fake delivery/provider sends, content-factory construction and non-IO inspection. Host handlers and functions accepting caller state are compile-checked, not invoked against an external service.

Run workspace `cargo test --workspace --doc --locked --offline` with the released compatible features for the source Rustdoc/compile-fail contracts. `cargo doc --workspace --no-deps --locked --offline` with RUSTDOCFLAGS=-D warnings verifies Rustdoc. These source doctests and the Markdown consumer compile checks are separate evidence.

## Verify application selection

The existing [consumer runner](../consumers/run.py) checks direct/event/disabled standalone graphs, formatting, Clippy and executable markers. [Feature graphs](../consumers/graphs.py) with --check builds all four CSRF/email default/server combinations and verifies minimal SDK features. Each workspace keeps a separate target directory.

All Cargo/proof commands go through `.agents/scripts/build-benchmark.py` with the shared store. The released effort uses sccache, bounded jobs and a warm absolute target under the 20 GiB cap, with a 19.4 GiB headroom guard. Workspace nextest excludes tcp_watchdog, which is coordinator-owned. No listener or live SMTP/SES/sendmail is part of these checks; do not infer browser/WASM, installed-host, remote timing or delivery acceptance from native finite tests.
