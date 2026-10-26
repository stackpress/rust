# Package shared UI with Tauri

A packaged application needs local assets and a deliberate native operation boundary.
Compile the shared tree as CSR and connect it to narrow permitted native commands; keep
platform responsibilities in the host adapter.

## Follow the canonical integration

Start with the [Profile UI README](../../../../../examples/profile-ui/README.md) for prerequisites,
separate mode builds, manifests, and proof boundaries. Its
[native package](../../../../../examples/profile-ui/native/Cargo.toml),
[Tauri config](../../../../../examples/profile-ui/native/tauri.conf.json), and
[capability](../../../../../examples/profile-ui/native/capabilities/main-search.json) are the
current integration source. Build/package commands belong to that example rather than a
second framework-owned copy.

The example frontend selects `csr`; it starts explicit empty public state and mounts new
markup. It uses packaged JS/WASM and a fixed search/cancel bridge. The host reuses the generated
read-only operation and calls the existing `handle_with_options` once after native admission.
Nested dispatch stays raw. It does not embed an HTTP server or use remote fallback.

## Keep the boundaries explicit

The shared framework supplies components, contracts, and applicable validation. Your native
host owns command allowlisting, authoritative window/origin checks, permissions, CSP,
bounded admission, cancellation, deadlines, and shutdown. Hiding UI is not authorization.

Dispose root listeners and cancel active acquisition when a window/root closes. Prevent late
completions from publishing into replaced/disposed state. Cancellation does not prove completed
operation effects were rolled back. The example's shared controller and native proofs show
that policy; it is not a universal operation-client API in framework-ui.

The accepted first target is a local unsigned macOS ARM64 `.app`. This documents that
integration, not mobile/other desktop support, signing/notarization, updates, offline persistence,
or synchronization. You can now locate the full packaging implementation while keeping shared
UI and native authority in their respective owners.
