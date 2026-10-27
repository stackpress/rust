# Generated admin role proof

This M4 example imports the actual generated-workspace Profile, Project, Task, and Address schema. Its independent app-owned tool composes schema, SQL, and UI functions into one output root. The shared frontend includes contracts and generated roles; it excludes stores and performs no database writes. RoleProof is an additional normalized boundary fixture.

From the repository root:

```sh
cargo run -p generated-admin-tool --locked -- emit generate
cargo run -p generated-admin-tool --locked -- emit generate --check
cargo run -p generated-admin-web --locked > /private/tmp/generated-admin.html
```

`--input PATH` and `--output PATH` select disposable inputs/marked roots. `--plugin` selects the compiled app replacement. A check against another selection fails without changing output. The CLI accepts optional `emit generate` words and Boolean flags; it does not accept `--check true`.

For finite browser evidence with the already installed matching wasm-bindgen and Playwright:

```sh
python3 examples/generated-admin/build.py
ADMIN_PLAYWRIGHT="/absolute/path/to/installed/playwright" python3 examples/generated-admin/browser.py
python3 examples/generated-admin/compile_proof.py
```

The browser proof intercepts `https://admin.invalid`, uses file-backed assets and a finite stdout SSR process, and starts no listener. Its parent kills/reaps after 120 seconds. Logs, exact command results, assets, compile failures, and screenshots live in `/private/tmp/s8-admin/`. The build driver preserves baseline/plugin/disabled binaries and restores default generated source in its final step.

The generated wrappers implement five independent placements. Missing roles stay absent; public descriptors omit sensitive/host-only fields. Native drafts distinguish omission/null/value and preserve defaults, false, zero, and empty strings. Browser assertions cover initial SSR/hydrated DOM identity, controlled edits, unique label/control IDs, plugin replacement/removal, and no captured browser errors.

The framework's larger source catalog remains unavailable unless registered with a concrete compiled adapter. Current built-ins are input/textarea/checkbox/switch/select and text/yesno/uppercase/lowercase/capitalize. There are no CRUD routes, writes, authorization implementation, relation services, or installed-target claims here; those are outside M4.
