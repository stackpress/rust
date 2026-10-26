# Profile UI M0

This bounded proof uses the actual generated Profile source and `profile-search` operation. A Send native loader delegates inside the page's existing lifecycle, then renders owned public data. The same Leptos component hydrates that markup or runs as packaged Tauri CSR.

## Build

Use Rust 1.96, the `wasm32-unknown-unknown` target, `wasm-bindgen 0.2.129` and Tauri CLI 2.12.4. The scripts check versions and never install tools or start a listener.

```sh
python3 examples/profile-ui/build-client.py hydrate
python3 examples/profile-ui/build-client.py csr
cd examples/profile-ui/native
cargo tauri build --debug --bundles app --features app --no-sign --ci
```

Client outputs are ignored under `dist/hydrate` and `dist/csr`. The unsigned `.app` is under the selected Cargo target directory's `debug/bundle/macos/`.

## Native SSR startup

Call `stackpress_framework_ui::render::initialize_ssr()` inside the host's Tokio runtime before rendering. It is idempotent, including when Leptos already has an executor. Missing initialization or an absent Tokio runtime produces `ViewError::ExecutorUnavailable` before a dependency can panic. Buffered Head assembly needs this executor even though component rendering is synchronous. General asynchronous component SSR remains deferred.

Facade features are `framework-ui-ssr`, `framework-ui-hydrate` and `framework-ui-csr`. Select one mode per separate build. The facade's defaults remain unchanged.

## Finite verification

Build the web fixture with `cargo build -p stackpress-example-profile-ui-web`. Set `PROFILE_UI_HOST` to that executable and `PROFILE_UI_PLAYWRIGHT` to an existing Playwright module, then run `node examples/profile-ui/proofs/browser.cjs`. It fulfills requests at a synthetic HTTPS origin over Chromium pipes and invokes the real HttpAdapter fixture for each search. It opens no listener and closes Chromium in `finally`.

Run `python3 examples/profile-ui/proofs/launch.py <path-to-app>` for the real packaged WKWebView proof. The opt-in `PROFILE_UI_PROOF` environment flag enables the finite main/ungranted window checks. Only the two app commands are exposed; no test command is added. The wrapper kills/reaps the app after 25 seconds if it does not exit.

Run `python3 examples/profile-ui/proofs/non-send.py` for the compiler rejection of an Rc retained across await in an actual registered native handler.

Coverage uses a clean isolated target with `cargo llvm-cov`. That tool's default `examples/` exclusion hides these real application crates. `proofs/coverage.py` exports the actual LLVM profiles, counts production `src/` files per crate, and includes the native executable when present. Execute the instrumented web fixture and native app before exporting. WASM-only code is exercised in the browser and WKWebView proofs; native LLVM coverage does not measure it.

## Scope

This is M0 evidence, with basic search markup. Full page registration APIs, router/detail navigation, reusable controls, locale/theme behavior, bounded network gateway, broad lifecycle/failure tables, visual acceptance and the later milestones remain open. No development listener, persistent offline store, remote fallback or other-platform support is provided by this proof.
