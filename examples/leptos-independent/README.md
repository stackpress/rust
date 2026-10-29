# Independent Leptos consumer

This fixture uses `stackpress-leptos` in its own Cargo workspace. Its only shared dependencies
are upstream `leptos = "=0.8.22"` and path `stackpress-leptos`, both with defaults off. The root
workspace explicitly excludes it. Normal graphs contain no view, Stackpress server/facade,
preference integration, SQL, schema, admin, Idea or Tauri package.

The [same root](src/app.rs) takes explicit theme, language and draft values in native SSR,
WASM hydration and CSR. It contains an Input, counter, popover, notifications and optional
Resource/Suspense, checked rich content and decimal editor. The [native shell](src/native.rs),
[inert bootstrap](bootstrap.js) and [WASM adapter](src/browser.rs) belong to this consumer.
They do not copy a page snapshot, route catalog or application status protocol.

## First result

With the existing approved toolchain/cache, run from the repository root:

```sh
cargo run --manifest-path examples/leptos-independent/Cargo.toml --locked --offline \
  --features ssr --example document -- hydrate light en
```

It prints one real HTML document and exits without a listener. Change the last choices to
`dark fr` for another independent root. The native example uses target-specific development
runtime dependencies; it does not add a runtime/host dependency to portable UI.

## Feature selection

Select exactly one render mode per artifact. `default=[]` compiles shared UI/value code;
`ssr`, `hydrate` and `csr` forward to both libraries. `async`, `rich` and `format` forward to
the reusable helpers. `stream` implies native SSR. Mixed modes and SSR on WASM are rejected.

WASM ABI/browser dependencies are target-specific, and native test/example runtime dependencies
are development-only. The fixture lock was seeded from the approved root lock. Offline resolution
may remove unused tuples or add local entries; every retained external identity/source/checksum
must already exist in the root lock. Never generate a fresh lock, install or upgrade to satisfy a proof.

## Finite proof commands

Run each Cargo/proof/clean through the benchmark recorder from the repository root. One explicit
target contains all native/WASM/doc/coverage output; keep aggregate use below 8 GiB and remove
that task target when the milestone ends. For example, set your already installed tools/output:

```sh
export CARGO_TARGET_DIR="$PWD/target/issue17"
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=2
export INDEPENDENT_OUTPUT_DIR=/private/tmp/leptos-independent-proof
export INDEPENDENT_PLAYWRIGHT=/absolute/path/to/already-installed/playwright
python3 .agents/scripts/build-benchmark.py run --label 'independent native' \
  --kind test --scope consumer --cache unknown -- \
  python3 examples/leptos-independent/proof.py native
```

The recorder retains argv, results and timing in the shared benchmark store. `proof.py` bounds
subcommands and the task target; no action installs tools or starts a listener. Use the action
matching your gate, with the appropriate recorder kind/scope:

| Action | Proves | Recorder kind/scope |
| --- | --- | --- |
| `native` | Default/SSR/helper tests, lifecycle watchdog, lint, doctests, Rustdoc and fmt | `test` / `consumer` |
| `build` | Real separate base/helper hydrate/CSR artifacts, matching native HTML and bindgen | `build` / `consumer` |
| `browser` | Chromium identity/early editing, interactions, helpers, disposal/remount | `test` / `consumer` |
| `graphs` | Selected normal graphs, forbidden edges, invalid modes and isolated import absence | `check` / `selected` |
| `docs` | Literal Markdown Rust fences compiled in isolated consumer/integration workspaces | `check` / `consumer` |

`build` requires an already installed `wasm32-unknown-unknown` target and `wasm-bindgen 0.2.129`.
`browser` requires those built artifacts and an installed Playwright package with its matching
browser. It intercepts only `https://independent.invalid`, serves copied files, and aborts unexpected
network requests. There is no localhost server. A helper Resource counts loader executions to
prove hydration replay rather than duplicate acquisition.

The browser matrix covers two themes/languages, both modes, base/helpers and desktop/phone widths.
It retains early native node/focus/selection, exercises root state and optional editing/sanitization,
and checks repeated disposal plus a clean CSR remount. Composition events are synthetic; they
exercise the real DOM adapter but do not claim an installed OS IME. Numeric coverage is native
production-line evidence; it does not measure WASM or prove installed desktop/mobile support.

Start with the library's [quick start](../../crates/leptos/docs/getting-started.md) or
[ownership concepts](../../crates/leptos/docs/concepts.md). For an actual Stackpress application,
use [view's explicit integration](../../crates/framework/framework-view/docs/integration-leptos.md).
