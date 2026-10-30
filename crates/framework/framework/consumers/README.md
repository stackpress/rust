# Aggregate consumers

Each application is a standalone Cargo workspace with its own checked-in lock.
Record the outer Python runner through the repository benchmark recorder; supply the
effort's `CARGO_TARGET_DIR`. The runner assigns a distinct `consumer-aggregate-*`
subtarget to each application. All normal builds use `--locked --offline`.

| Application | Proof |
| --- | --- |
| `build-host` | Real aggregate build helpers plus schema/SQL/admin emitters; generates `generated/`, then checks freshness. No runtime import runs generation. |
| `full` | Static client, nine source stages, real file SQLite, generated Note operations/admin registrar, account storage, public API endpoint, session identity, CSRF, JSON email and finite command binding. |
| `direct` | Calls those public adapter/constituent functions individually in source order, bypassing `register_with`; compares real phase/event/priority and projected responses byte-for-byte to `full`. Source server/schema/language adapters live in the aggregate crate, so this application still imports that crate. |
| `headless` | Stock `register`, absent client/config/Engine/bindings, no startup IO or request guard. |
| `api-disabled` | Compiles every other runtime stage; the API package is absent from its normal dependency graph. |
| `disabled` | Default `stackpress` facade: only `stackpress-lib` among first-party dependencies. |
| `facade` | `stackpress::framework` through opt-in aggregate features; matches `full` inventory. |
| `website` | Native SSR over the same real full host; separate hydrate and CSR artifacts consume only the public page seed. |

`run.py` builds before applying a 45-second external deadline to each executable.
`../proofs/watchdog.py` proves forced termination and reaping with a stuck child.
The application owns its SQLite file, policy, command providers and source hook shutdown.
The generated Note policy is explicitly supplied by this finite application; it is not
a framework default. No listener, watcher, TTY, network delivery or implicit installation runs.

`graphs.py --check` checks every new facade selection, disabled renderer-only forwarding,
the standalone normal graphs and actual WASM modes. Runtime/browser graphs exclude codegen;
browser graphs exclude server, SQL, mail and native identity/crypto providers.
The root default remains unchanged. Existing constituent feature meanings are preserved.

## Facade selection

`framework-aggregate-full` selects the nine runtime stages without a backend, renderer
or codegen. `framework-aggregate-<leaf>` selects a stage independently. UI families are
separate: `framework-aggregate-ui-session`, `-ui-api`, `-ui-admin`, `-ui-view`.
`framework-aggregate-ssr`, `-hydrate`, `-csr` weakly forward a renderer only to selected
UI families. A renderer alone activates no UI family or native stage. Select one renderer
per artifact; native aggregate and browser UI are separate artifacts.

The independent `framework-api*`, `framework-csrf*`, `framework-email*` features expose
`stackpress::framework_api`, `framework_csrf`, `framework_email` without selecting the
aggregate registrar. Existing `framework-session*` features retain their meaning.

The website runner writes three real source pages under the target's `aggregate-artifacts/pages`.
`website/build.py MODE --out-dir PATH` builds one WASM mode and runs the installed
wasm-bindgen CLI. `../proofs/browser.cjs` fulfills all page/module/WASM requests from memory.
Its required `AGGREGATE_PLAYWRIGHT` is exactly
`/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright`.
It uses cached Chromium headlessly, fresh contexts and no system Chrome or listener.

## Generated fixture and limits

`build-host/schema.idea` is private to these consumers; the existing generated-admin example
is read-only. The checked-in generated tree is never hand-patched. Native consumers include
generated contracts/stores plus the actual `ui/pages_host.rs` registrar. UI role source remains
in its portable partition. The finite app-owned page/API/transfer handlers dispatch generated
operations; they do not claim the full generated-admin website's import/preview workflow.

The website/browser proof covers shared source signin, email and account views at desktop
and mobile widths, hydration errors, an interactive email/password form and disposal.
It does not establish live backend, SMTP, production bundler/HMR, or installed Tauri support.
G42 multiple handlers/Abort and G43 per-process/configured security remain unchanged.
