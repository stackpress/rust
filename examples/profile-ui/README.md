# Shared Profile UI

Build the generated Profile capability once and use its public output in native SSR, browser hydration and packaged CSR. The example web gateway serves the same pages, typed search endpoint and checked client assets through the existing HttpAdapter. It is an example preview host, with finite buffers and deadlines.

## Check the complete web build

From the repository root, use already installed Rust 1.96, Python 3.11+, the `wasm32-unknown-unknown` target and `wasm-bindgen 0.2.129`:

```sh
python3 examples/profile-ui/check.py
python3 examples/profile-ui/check.py --sqlite
```

Each command checks generated freshness with `--check true`, builds native SSR, builds hydrate and CSR separately, verifies their asset/seed manifests, checks the facade feature matrix and browser dependency graphs, then runs the web/gateway tests, doctests and warning-denied Clippy. The default selects in-memory data; `--sqlite` selects the read-only database backend and checks its native dependency edges. Its final service probe opens no port and uses an explicitly installed temporary database for SQLite. It never installs a tool or starts a listener. Missing prerequisites or the first failing command stop the sequence with a log path.

Expected result: `All checks passed`. Logs and `summary.json` are under `examples/profile-ui/dist/check-logs`. Cargo uses exactly two build directories: `target/profile-ui-native` and `target/profile-ui-wasm`. Set `PROFILE_UI_NATIVE_TARGET`, `PROFILE_UI_TARGET` and `PROFILE_UI_CHECK_LOGS` to use other locations; retain those same two targets across checks. `CARGO_NET_OFFLINE=true` uses only cached dependencies. Cargo otherwise fetches the locked dependencies as needed; tool installation is always separate.

## Preview through devmetrics

After the checks pass, an authorized operator can run this exact managed command from the repository root:

```sh
devmetrics start --summary "Stackpress profile UI preview" -- 'CARGO_TARGET_DIR=target/profile-ui-native cargo run --locked -p stackpress-example-profile-ui-gateway --bin stackpress-example-profile-ui-gateway -- --port {port} --base /demo'
```

Use the port assigned by devmetrics and open `/demo/ui/profiles?id=abc`. Initial HTML contains Ada, Grace and Margaret; hydration preserves it without a second initial query. Search for `abc456`, open Grace's detail page, then return to the original search. Direct refresh and back/forward use the shared route/return-query codec.

If you changed `PROFILE_UI_NATIVE_TARGET`, use that same path for `CARGO_TARGET_DIR` in the managed command. Never run the listener command directly. The gateway accepts a port only from `--port` or `PORT`; it has no default port. It binds loopback, stops after at most two hours (or `--duration-seconds 1..7200`), and drains on Ctrl-C or Unix SIGTERM. devmetrics also manages its lifetime. `--assets DIR` selects an existing hydrate output; `--base /demo` selects the trusted public mount.

## Select SQLite explicitly

The web host and gateway have a compile-time `sqlite` feature. It reuses [sql-profile](../sql-profile)'s database and profiles plugins, including literal case-sensitive byte-prefix search and stable seed order. The in-memory generated capability stays the default; the Tauri app stays in-memory. Shared/client/contracts and visible UI are unchanged.

After building the assets with `check.py --sqlite`, create a temporary database and JSON configuration from the repository root:

```sh
PROFILE_UI_DEMO_DIR=$(mktemp -d /private/tmp/profile-ui-demo.XXXXXX)
CARGO_TARGET_DIR=target/profile-ui-native cargo run --locked -p stackpress-example-sql-profile --bin sql-profile-setup -- "$PROFILE_UI_DEMO_DIR/profiles.sqlite"
python3 -c 'import json,pathlib,sys; p=pathlib.Path(sys.argv[1]); (p/"config.json").write_text(json.dumps({"sqlite":{"path":str(p/"profiles.sqlite")},"profiles":{"limit":3}}))' "$PROFILE_UI_DEMO_DIR"
devmetrics start --summary "Stackpress SQLite profile UI preview" -- "CARGO_TARGET_DIR=target/profile-ui-native cargo run --locked -p stackpress-example-profile-ui-gateway --features sqlite --bin stackpress-example-profile-ui-gateway -- --port {port} --base /demo/v1 --config '$PROFILE_UI_DEMO_DIR/config.json'"
```

Open `/demo/v1/ui/profiles?id=abc` on the assigned port. Keep the temporary directory until the managed preview stops, then remove it. Setup is an explicit one-shot transaction and refuses an existing installation. Startup opens the file read-only and checks its schema; a missing file or missing schema fails before the gateway binds. Startup and requests never install, migrate or seed. Relative `sqlite.path` values resolve from the process working directory; `profiles.limit` accepts `1..=100` and cannot be overridden by requests.

Embedding callers pass `ConfigLayers` to `web::application_with_config(base, layers)` or `Gateway::with_config(base, assets, limits, layers)`. The managed gateway reads only an explicit `--config FILE`; finite web/probe executables use `PROFILE_UI_CONFIG=FILE`. An omitted config uses the selected plugins' defaults (`profiles.sqlite` for SQLite), without creating a file. Invalid JSON, unknown sections and invalid policy fail startup.

## Asset and version agreement

The finite build steps can also be run separately:

```sh
cargo run --locked -p stackpress-example-profile-codegen -- emit generate --check true
CARGO_TARGET_DIR=target/profile-ui-native cargo build --locked -p stackpress-example-profile-ui-web
python3 examples/profile-ui/build-client.py hydrate
python3 examples/profile-ui/build-client.py csr
python3 examples/profile-ui/asset_manifest.py check hydrate
python3 examples/profile-ui/asset_manifest.py check csr
```

Each client output has `manifest.json`: wire revision, build ID, a fingerprint of the actual shared contract sources, pinned bindgen version, file sizes/MIME/SHA-256 values and a content identity. The gateway checks that inventory before any bind and retains verified bytes in memory. A changed or missing file fails startup with its path; it is never silently served. Runtime URLs use `/assets/<content identity>/...`, so imports and WASM share the same immutable asset directory. HTML and API responses use `no-store`.

The external bootstrap catches missing JS/WASM and rejected seed/asset versions. It preserves useful server HTML when possible and adds **Reload page**; recovery requires an explicit click and never loops automatically. A valid host error remains distinct from a failed transport in the shared reply decoder. The browser demonstrations below prove those states and successful recovery.

## Port-free browser and HTTP proofs

Set `PROFILE_UI_PLAYWRIGHT` to an existing Playwright module. No script installs browsers or packages. With default check targets:

```sh
PROFILE_UI_GATEWAY_PROBE=target/profile-ui-native/debug/probe node examples/profile-ui/proofs/browser-gateway.cjs
PROFILE_UI_HOST=target/profile-ui-native/debug/stackpress-example-profile-ui-web node examples/profile-ui/proofs/browser.cjs
PROFILE_UI_HOST=target/profile-ui-native/debug/stackpress-example-profile-ui-web node examples/profile-ui/proofs/browser-m2.cjs
PROFILE_UI_HOST=target/profile-ui-native/debug/stackpress-example-profile-ui-web node examples/profile-ui/proofs/browser-navigation.cjs
```

Chromium uses a synthetic HTTPS origin and request interception over browser pipes. `probe` calls the actual gateway service API and emits one response, then exits. It cannot bind a socket. The gateway proof covers hashed assets, initial seed, version/build mismatch, missing JS/WASM, explicit reload recovery and a second typed search. The other proofs cover unchanged DOM/focus, hostile seed and literal hostile prefixes, errors/retry, disposal, responsive/keyboard behavior, navigation and initial French/dark agreement. `PROFILE_UI_BASE=/demo/v1` exercises mounted behavior for every proof.

Run the complete eight-proof matrix; the wrapper rebuilds both finite hosts for the selected backend in the shared native target before using them:

```sh
python3 examples/profile-ui/proofs/browser.py
# Select SQLite; the wrapper also builds the explicit setup binary:
python3 examples/profile-ui/proofs/browser.py --sqlite
```

List rows expose their stable application key as `data-item-id`, matching the generic framework List contract.

The runner invokes all four scripts at the root and `/demo/v1`, creates and installs SQLite only inside a temporary directory when selected, and retains proof logs and a machine receipt in `dist/browser-logs`. Before setup, the SQLite runner checks both finite executables against missing and uninstalled files: startup must explain the setup remedy and leave each file untouched. Set `PROFILE_UI_BROWSER_LOGS` to retain separate backend receipts. `PROFILE_UI_NATIVE_TARGET` selects the same native target used by checks. Rust parity tests additionally compare complete SSR HTML, typed HTTP bytes/headers and detail/error statuses against the actual generated in-memory host, including empty, `%`, `_`, quotes, backslash, Unicode and NUL prefixes.

Gateway tests use `tokio::io::duplex` and the same Hyper connection/runner code as the executable. They exercise raw chunked requests, bounded body/header acquisition, incomplete/slow input, disconnect, excess admission and graceful shutdown without a TCP listener.

## Change one selected plugin

The generated Profile type and search operation remain unchanged. `customization.rs` is one shared native registration source. The `demo-customization` Cargo feature selects `profile-ui-labels` after the generated `profiles` plugin; its priority `-10` runs after the generated operation's priority zero. It prefixes each returned name with `Demo: ` while preserving IDs, order and totals.

```sh
CARGO_TARGET_DIR=target/profile-ui-native cargo test --locked -p stackpress-example-profile-ui-web --test reuse
CARGO_TARGET_DIR=target/profile-ui-native cargo test --locked -p stackpress-example-profile-ui-web --features demo-customization --test reuse
CARGO_TARGET_DIR=target/profile-ui-native cargo build --locked -p stackpress-example-profile-ui-gateway --features demo-customization --bins
PROFILE_UI_CUSTOMIZED=1 PROFILE_UI_GATEWAY_PROBE=target/profile-ui-native/debug/probe node examples/profile-ui/proofs/browser-gateway.cjs
```

The parity test compares direct dispatch, the actual CLI adapter, typed HTTP and the SSR seed/visible HTML. The browser proof confirms `Demo: Ada Lovelace` and `Demo: Grace Hopper` using that same operation output. Add `--features demo-customization` to the Cargo arguments of the managed preview command to preview this selected plugin. Rebuild without the feature to remove it. Lane B owns the matching native feature/registration and packaged Tauri parity proof; native M3 integration is tracked separately.

## Runtime boundaries

The gateway defaults are 64 KiB request body, 16 KiB semantic header bytes/HTTP head buffer, 64 headers, eight admitted connections, three-second headers, two-second total body acquisition, five-second total request, 30-second connection lifetime and two-second graceful drain. Oversize/slow acquisition fails before HttpAdapter. Shutdown stops admissions, signals HTTP graceful stop and aborts/reaps work after the drain allowance. Body cancellation does not roll back completed work and never retries automatically.

Static assets are buffered, capped at 64 MiB per file and 128 MiB total; concurrent responses can copy those buffers. Page snapshots retain their separate one-MiB logical and two-MiB complete-HTML ceilings. This demonstration provides no production streaming, TLS, authentication, arbitrary file serving, CPU sandbox or persistent offline store.

Native render startup calls the idempotent `initialize_ssr()` inside Tokio. Missing startup/runtime returns a typed error before dependency executor code can panic. Reactive Owners are local to synchronous component evaluation; only owned Head/HTML buffers cross the final await. SSR, hydrate and CSR are separate valid builds; never combine them with `--all-features`.

During SSR, Head values and formatters must use eager owned data: use `<Title text=title.get()/>`, resolving the signal while its Owner is active. A deferred getter such as `<Title text=move || title.get()/>` can read disposed state; the render boundary converts a Head evaluation panic into a safe typed render error. Reactive SSR Head remains deferred with **S5-F6**. Browser Head reactivity is separate from this native SSR restriction.

For native executable-line coverage, use a clean one-native/one-WASM target pair and cargo-llvm-cov; `proofs/coverage.py` includes real example source that cargo-llvm-cov's default example filter omits. Browser-only adapters use real Chromium/WKWebView proofs under the accepted numeric-coverage disposition. Packaged app/permission/cancellation evidence belongs to native M3 and is separate from the web gateway checks.
