# Task 00013: M4 generated role wrappers

Implementer receipt, 2026-10-27, branch `trial/3-s8-admin`. No commit, push, install, listener, publishing, or M5 work occurred. This is implementation evidence for coordinator review, not acceptance or a claim that the entire Slice 8 component catalog is available.

## Scope and frozen interface

Implemented a bounded core M4 generator and a real generated-admin example against [the G34 revision-2 interface](00011-receipt-m1.md#slice-8-frozen-interface-for-coordinator-review). G29/G30 are preserved: one app-owned definition catalog, ordinary compiled transforms, one registry/writer, separate output partitions, and existing Idea composition/provenance. No second parser or UI-only generator binary was introduced.

Write set: new `crates/framework/framework-admin/**`, new `examples/generated-admin/**`, root Cargo members/lock/facade entries, and this pre-approved receipt. No changes were made to framework-schema, framework-sql, generated-workspace, framework-ui/view, ui-gallery, profile-ui, server, SQL, or Idea source. No crates.io package/version was added; the lock diff adds only the five local packages and facade dependency.

The new app-owned `generated-admin-tool` reuses the original workspace tool's definition-construction library. Its schema/SQL/UI functions normalize the same input and declared plugin/diagnostic snapshots. The UI function consumes only `UiSchema`; the writer owns contracts, metadata, stores, and UI in one generated root. The frontend includes contracts/UI directly and never includes stores or the aggregate's native modules.

## Implemented behavior

- `stackpress-framework-admin` has browser-safe root drafts/components and an optional build-host `codegen` feature. The optional facade import is `stackpress::framework_admin`.
- Ten concrete definitions map to eight framework-ui adapters: field input/textarea/checkbox/switch/select; view text/yesno/uppercase/lowercase/capitalize. App replacements are selected before defaults, by canonical identity. Filter/span derive field definitions; list derives view definitions through the frozen schema registry.
- The emitter fixes revision 2, checks complete bounded projections before contributing files, resolves private compiled imports, and emits escaped Rust string literals. It rejects unavailable/missing/invalid imports, keywords/default-export conventions, duplicate roles/fields/modules, incompatible native controls, unsupported filters, and built-in displays of virtual relations without Public DTO values.
- Each model/fieldset module exports public descriptors and only selected role functions. Field/filter/span take a root-owned `Draft`; list/view take the real generated `Public<Model>` type. List/view values have explicit labels. Missing roles create no function; no type-based role synthesis occurs.
- Sensitive and host-only fields are excluded from wrapper descriptors/controls. The conservative M4 UI policy also omits sensitive writable inputs. Public projection, field policy, requiredness, nullability, safe defaults, and role normalization remain schema-owned.
- Drafts preserve `Missing`, `Null`, and literal values, including false/zero/empty text. Only missing creation/span inputs apply approved literal defaults; filters do not inherit creation defaults. Native events update local state; presence selectors choose omission/null explicitly. No mutation endpoint is present.
- `generated-admin` imports the actual Profile/Project/Task/Address schema and renders lists, detail values, and native field forms. An additional normalized RoleProof fixture proves nullable/default/false/zero and excluded sensitive/host fields. The finite SSR executable writes one document and exits.
- The app plugin replaces the input component with handwritten `AccentField` and fixes its label/placeholder. Disabling the selection and rebuilding removes the marker/overrides. Generated files are never patched by hand.

## Review rounds

The initial implementation established native components, actual generation, SSR consumers, and source diagnostics before deliberate rounds. A baseline diagnostic assertion was corrected to traverse the registry's retained error chain rather than assume the immediate source was SchemaError.

| Round | Findings, changes, and verification |
| --- | --- |
| Logic/complexity | Reject duplicate roles and unsupported explicit filters instead of silently dropping them. Narrow built-in prop signatures to implemented props. Prove actual source ranges and JSON structural paths. `round1-final.log`: 12 tests passed. |
| Responsibility/abstraction | Separate import/value/source-position policy into `codegen/bindings.rs`; keep source formatting in the emitter. Decode a model's descriptors once per rendered role function. Reject nonobject seeds and duplicate draft keys. `round2.log`: 12 tests passed. |
| Consistency/Rustdoc/comments | Document public APIs, component props, target/feature boundaries, and data/state sequencing. Filters remain optional independently of creation requiredness. Presence options reconcile selected DOM properties. Pin the frozen descriptor revision explicitly. `round3.log`: 12 tests plus one doctest passed. |
| Safety and visual follow-up | Verify public create/patch/query contracts against the real RoleProof output; private defaults require an explicit host provider, and projection removes private values. Refuse blank built-in virtual-relation displays. Give display values labels/spacing and workspace models semantic lists/forms. Final scoped tests, complete workspace suites, lint/docs, compile proofs, coverage, and Chromium were rerun. |

No unsafe code, dynamic import evaluation, expression engine, arbitrary CSS/HTML interpolation, inferred relation service, write retry, or authorization implementation was added. Errors retain concrete causes under independently authored context, following #11. Public fixture failures render a fixed safe message.

## Verification

All logs/results are under `/private/tmp/s8-admin/`. `verify.py` records exact argv, exit status, elapsed time, and watchdog expiry. Cargo uses four build jobs, incremental disabled, and dev debug info disabled. Build and coverage use at most two owned target roots. CLI generation tests run in killed/reaped children with a 30-second parent watchdog; SSR child tests use ten seconds. The browser parent uses 120 seconds and process-group kill/reap. No test timeout is counted as success.

| Gate / evidence | Final result |
| --- | --- |
| `fmt-final.log` | Workspace rustfmt check passed; diff whitespace check passed |
| `scoped-final.log` | 15 native tests and one doctest passed; no failures or ignored cases |
| `msrv.log` | Explicit `cargo +1.96.0 check` passed for all five new packages with codegen/SSR |
| `clippy-final.log`, `workspace-clippy.log` | Scoped and complete workspace all-target Clippy with `-D warnings` passed |
| `rustdoc.log` | All five packages, no-deps, `RUSTDOCFLAGS=-D warnings`, passed |
| `facade.log` | Optional `framework-admin` facade check passed |
| `workspace.log` | `cargo test --workspace --all-targets --locked`: 623 passed, zero failed, 78 explicitly ignored |
| `workspace-doc.log` | `cargo test --workspace --doc --locked`: 891 passed, zero failed |
| `wasm-hydrate-check.log`, `wasm-csr-check.log`, `*-build.log` | Actual hydrate/CSR WASM checks/builds passed on installed stable Rust 1.96.0 |
| `tree-hydrate.log`, `tree-csr.log` | Normal browser graphs exclude Stackpress server, SQL/driver, Idea/codegen, and Tokio edges |
| `tree-tool.log` | Tool has no generated-admin shared/client or generated-workspace app consumer edge |
| `compile-proof.json`, `compile-*.log` | Absent-root generation; byte-identical check/repeat; actual rename/removal; stale consumer exit 101; updated consumer compiles; hostile normalized-JSON labels/props compile |
| Tool generation tests | Failed unknown/unavailable/out-of-order regeneration preserves every previously owned byte; plugin enable/disable returns exactly to baseline bytes |
| `browser.json`, `browser.log` | Chromium base hydrate/CSR plus plugin/disabled hydrate: all four passed, zero captured browser errors/unexpected external requests |
| `kb-baseline.log`, `kb-final.log` | Identical inherited 146 errors and 22 warnings; no new or removed finding |

The final workspace gates were repeated after the display/safety follow-up; all passed without chasing unrelated failures. Network adapter cases remain explicitly ignored with no opt-in database proof implied. Isolated schema consumers emit the pre-existing `RecordSchema::validate_defaults` dead-code warning; the owned source did not suppress or edit it. Scoped/workspace Clippy passes with the actual host/codegen feature selection.

An initial `cargo +1.96.0` WASM check found that the separately named toolchain lacked the WASM std target. No installation was performed. The installed `stable` toolchain reports the same `rustc 1.96.0 (ac68faa20 2026-05-25)` and has the target; both real WASM modes pass there. Explicit named-toolchain native MSRV checks also pass.

Idea's frozen raw-string grammar does not introduce quote escapes. Hostile closing tags/Unicode are proved from Idea; quotes/backslashes/newlines are additionally proved through actual normalized JSON and consumer compilation. Idea diagnostics retain actual UTF-8 ranges; normalized JSON reports file/structural paths with no invented coordinates. Invalid compiled import configuration can fail at catalog registration; selected keyword/path failures retain the selected Idea attribute location through the UI emitter.

## Chromium and control 00013-013

The finite proof requires `ADMIN_PLAYWRIGHT`, uses the already installed matching module/browser and wasm-bindgen, and intercepts `https://admin.invalid` entirely. Native documents come from a finite stdout subprocess; assets come from retained files. No listener is created. macOS browser launch required sandbox escalation for its Mach port; no packages/browsers were installed.

All four variants prove native control edits, omission/null/empty text, explicit false, literal defaults/zero, 72 unique element IDs, label associations, three workspace lists/forms, no sensitive/host field controls, no private default in DOM, and root disposal. Hydration proves identical initial parsed DOM and preservation of the original input node. The plugin label, fixed placeholder, and component marker are visible in SSR and hydrated web; disabled rebuild removes them. This covers the requested SSR/web portion of control 00013-013; installed UI is not claimed.

Screenshots retained and visually reviewed: `screenshots/base-hydrate.png` and `screenshots/plugin-hydrate.png`; base CSR and disabled hydration screenshots are also retained. The final screenshot includes readable labelled values and semantic fixture forms, not production CRUD acceptance.

## Coverage

Fresh instrumented native run: cargo-llvm-cov 0.8.6, Rust 1.96.0, aarch64-apple-darwin; all first-party implementation in the four native crate/feature scopes, including compiled generated contracts and wrappers. No generated implementation was excluded. Cargo's default report hides paths under examples; `coverage_report.py` reuses the exact LLVM export objects/counters with explicit implementation sources and enforces 70% per measured crate. `coverage-command.log`, `coverage-raw.json`, and `coverage-summary.json` retain the export and counts.

| Crate | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-admin, codegen + SSR | 587 / 666 | 88.14% |
| generated-admin-tool | 160 / 178 | 89.89% |
| generated-admin-shared, SSR | 1842 / 2452 | 75.12% |
| generated-admin-web | 11 / 11 | 100% |

The initial full-source shared measurement was 64.19%, below the floor. Meaningful RoleProof default/null/private-projection/create/patch/query tests raised it to the reported result; files were not excluded. The WASM-only client startup has no executable native lines: it is compiled and exercised in Chromium but has no LLVM/WASM line-coverage claim. Branch, installed-target, and publication-level coverage are not claimed.

Reproduce from the repository root, with the installed Playwright module path supplied through the environment:

```sh
python3 examples/generated-admin/build.py
ADMIN_PLAYWRIGHT="/absolute/path/to/installed/playwright" python3 examples/generated-admin/browser.py
python3 examples/generated-admin/checks.py
python3 examples/generated-admin/compile_proof.py
CARGO_LLVM_COV_TARGET_DIR=/private/tmp/s8-admin/coverage-target \
  python3 examples/generated-admin/verify.py coverage-closeout cargo llvm-cov \
  -p stackpress-framework-admin -p generated-admin-tool \
  -p generated-admin-shared -p generated-admin-web \
  --features stackpress-framework-admin/codegen,generated-admin-shared/ssr \
  --locked --json --output-path /private/tmp/s8-admin/coverage.json
CARGO_LLVM_COV_TARGET_DIR=/private/tmp/s8-admin/coverage-target \
  python3 examples/generated-admin/verify.py coverage-command cargo llvm-cov report \
  -p stackpress-framework-admin -p generated-admin-tool \
  -p generated-admin-shared -p generated-admin-web \
  --json --output-path /private/tmp/s8-admin/coverage.json -vv
python3 examples/generated-admin/coverage_report.py
```

## Limits and handoff

The complete schema vocabulary remains present, but only the ten concrete definitions above are available by default. The broader inventory's numeric/date/formatting, richer collections, fieldsets/relations, service-backed file/media, rich editor, and other adapters are not claimed implemented here; selected unavailable roles diagnose rather than silently degrade. Catalog breadth from the full Slice 8 inventory therefore remains a coordinator follow-up, not an implied completed port.

No actual CRUD, database writes, authorization, relation lookup service, native/installed target, early-user-edit-before-hydration adoption, production host, release, or M5 capability is asserted. The browser proof establishes the declared initial fixture/hydration behavior only. The UI catalog and generated component interface are new M4 surfaces for coordinator review; the frozen schema contract was not changed.

Crate docs and example instructions were added, their links/structure checked, and Rustdoc/doctests verified. No KB context promotion, task acceptance, or MCP indexing/publishing occurred.

Cleanup completed: deleted `/private/tmp/s8-admin/target` (about 3.8 GiB) and `/private/tmp/s8-admin/coverage-target` (about 724 MiB). The checkout's 144 KiB scratch `target/` contained only inherited test logs/fixture directories and llvm-cov-clean metadata, with no Rust build artifacts; preserved it under `/private/tmp/s8-admin/workspace-fixture-artifacts/` and removed it too. All target roots are absent; `cleanup.json` records the checks. Logs, reports, screenshots, WASM assets, and copied finite SSR binaries remain under `/private/tmp/s8-admin/`. Source remains uncommitted; staging remains empty; HEAD remains `cce3a31`. Stop at M4 for coordinator review and the catalog-breadth decision.
