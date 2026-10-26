# Task 00009: Profile UI backed by SQLite

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner requested it on 2026-10-26, as option B after the Slice 5 rounds: connect the UI vertical to the SQL work.

## Task summary

The profile UI example (`examples/profile-ui`) currently searches the in-memory generated `profile-search`. The SQL example (`examples/sql-profile`) serves the same capability from an app-owned SQLite file, with byte-identical envelopes.

Add a selectable SQLite backend to the profile UI's server side (web host and example gateway), so one app runs from a SQLite database through the server to SSR, hydration and the browser.

Coordinator scoping decisions:

- **Backend selection** is a compile-time feature, for example `sqlite`, on the web host and gateway. It reuses `examples/sql-profile`'s database plugin, setup and `profile-search` registration as a library dependency rather than copying them. The in-memory backend stays the default, so existing proofs keep running unchanged.
- **Database setup** stays explicit, using `sql-profile`'s setup binary or helper. Startup never installs or seeds. A missing or uninstalled database is a clear startup error.
- **The macOS Tauri app stays in-memory** in this task. Bundling or locating a database inside the packaged app is a separate owner decision.
- **No changes** to lib, server, CLI, Idea or the SQL crates' public APIs (G23). Any needed shared change comes to the coordinator first.

## Implementation steps

1. Wire the feature-gated SQLite backend into the web host and gateway, with config naming the database file, following `sql-profile`'s config conventions.
2. **Parity:** for the same seed and inputs, the SSR HTML, typed HTTP envelopes and browser-visible results are identical between the in-memory and SQLite backends. Cover hostile prefixes (`%`, `_`, quotes, backslash, Unicode, empty).
3. **Browser proofs:** browser.cjs, browser-m2.cjs and browser-navigation.cjs at the root and `/demo/v1`, plus the gateway proof, pass with the SQLite backend.
4. **check.py** gains the SQLite feature build, test and dependency checks. The browser build graph must remain free of SQLite and SQLx.
5. **README:** setup plus the devmetrics preview command for the SQLite-backed gateway.
6. Rounds per the Rust programming workflow on the new code.

## Verification process

The coordinator reruns:

- check.py;
- workspace tests, doctests, Clippy and the 1.96 check;
- the backend parity tests;
- all browser proofs with the SQLite backend;
- the dependency trees (browser graph driver-free);
- per-crate coverage.

## Acceptance criteria

Acceptance criteria: none. The UI was visually accepted in task 00008; this task must not change visible output.

## Implementation notes

Implemented on `trial/3-profile-ui-sqlite`, based on `9290b40`, on 2026-10-26. Work is uncommitted and awaits coordinator verification.

### Behavior and ownership

- `sqlite` is an opt-in compile-time feature on the native web host and gateway. Default builds retain the generated in-memory operation. Shared/client/contracts, visible UI, Tauri and `crates/**` were not changed.
- `web::backend` selects the existing sql-profile database and profiles plugins with their prerequisite order. Its existing public exports were sufficient: no sql-profile changes or copied persistence code were needed. Cargo.lock adds only the new example/development dependency edges; no registry versions changed.
- Embedders pass `ConfigLayers` to `application_with_config` or `Gateway::with_config`. The managed gateway accepts `--config FILE`; finite web/probe executables read `PROFILE_UI_CONFIG`. JSON retains `sqlite.path` / `profiles.limit`, with database paths relative to the working directory.
- Configuration reads have a finite byte budget and require an operator-owned regular file. Omitted configuration uses registered defaults. Startup reuses sql-profile's eager read-only open/schema check and never creates, installs or seeds a database. Database startup errors preserve their cause and explain the separate `sql-profile-setup` remedy.
- Backend parity tests compare actual generated in-memory and SQLite hosts: complete SSR bytes/headers/status, typed HTTP bytes/headers/status and detail/not-found results at the root and `/demo/v1`, with limits 1 and 3. Inputs include empty, case differences, `%`, `_`, both quote forms, backslash, Unicode, injection-shaped text and NUL.
- Existing web and duplex gateway regressions exercise the selected backend using explicitly installed temporary files. Browser interaction proofs now support both mounts and literal hostile prefixes. `proofs/browser.py` runs every script at both mounts; SQLite preflight also checks both finite executables against missing/uninstalled files without setup effects.
- `check.py --sqlite` covers feature builds/tests/doctests/Clippy, expected native SQLite edges, isolated hydrate/CSR graphs and a temporary-database service proof. The README documents explicit setup, configuration, the proof matrix and the devmetrics-only preview command.

### Test-verified rounds

1. Logic/complexity: reviewed default/SQLite selection, config failure order, explicit setup and fixture lifetimes. Added bounded regular-file configuration checks and cause-preserving startup remedies; tests assert missing/uninstalled files remain untouched. All selected-backend tests/doctests passed (`/private/tmp/task00009-round1.log`).
2. Responsibility/abstraction: reviewed ownership across backend composition, persistence, routes/rendering and adapters. No further extraction was justified: one native backend module reuses the SQL plugins and both hosts share the presentation registration. Tests passed with SQLite plus `demo-customization`, including complete parity (`task00009-round2.log`).
3. Consistency/Rustdoc/comments: updated backend-sensitive declarations/comments, documented config and setup boundaries and tested config-option parsing. Selected-backend tests/doctests passed (`task00009-round3.log`); final formatting, Clippy and warning-denied Rustdoc passed.
4. Safety/soundness: reviewed read-only service publication, cause preservation, temporary-file lifetime, public input boundaries, cancellation inheritance and dependency placement. Added real finite-executable startup checks; all passed. No unsafe code or new concurrency/persistence mechanism was introduced. The existing SQL literal-positive and storage proofs also passed in the workspace suite.

### Verification receipt

All code/proof gates below passed. Commands ran with `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, one reused native target (`target/profile-ui-native`) and one WASM target (`target/profile-ui-wasm`).

- `python3 examples/profile-ui/check.py` and `python3 examples/profile-ui/check.py --sqlite`: both passed. Separate logs and machine receipts: `/private/tmp/task00009-check-memory/` and `/private/tmp/task00009-check-sqlite/`.
- `cargo test --workspace --all-targets --locked`: passed; opt-in/ignored tests remain explicitly ignored. `cargo test --workspace --doc --locked`: passed. Logs: `task00009-workspace-tests.log` and `task00009-workspace-doctests.log` under `/private/tmp/`.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo +1.96.0 check --workspace --all-targets --locked` and warning-denied `cargo doc --workspace --no-deps --locked`: passed. Corresponding logs: `task00009-workspace-clippy.log`, `task00009-workspace-msrv.log`, `task00009-workspace-doc.log`.
- SQLite-specific 1.96 all-target checks and warning-denied Rustdoc also passed for web/gateway; selected-feature doctests/Clippy passed through check.py. Logs: `task00009-sqlite-msrv.log`, `task00009-sqlite-doc.log` and the SQLite check directory.
- `browser.cjs`, `browser-m2.cjs`, `browser-navigation.cjs` and `browser-gateway.cjs` passed at both the root and `/demo/v1` for both backends: 16 runs, zero unexpected browser errors. Receipts: `/private/tmp/task00009-browser-memory/` and `/private/tmp/task00009-browser-sqlite/`. The complete SQLite matrix and executable startup preflight passed again using instrumented binaries in `/private/tmp/task00009-browser-coverage/`.
- Native executable-line coverage used cargo-llvm-cov 0.8.6 / Rust 1.96.0 (`aarch64-apple-darwin`), default and SQLite feature test runs, plus instrumented finite web/probe browser runs. Web: **455/488 = 93.24%**. Gateway: **540/608 = 88.82%**. Both exceed the 70% development floor. LLVM profile diagnostics were empty.
- Coverage includes actual production example source/binaries and the linked route source. Tests, dependencies and non-Rust tooling are outside the changed-crate denominator; optional customization was tested separately. Incidental partial coverage rows for unchanged dependency crates are not their complete coverage results. Browser/WASM behavior is proven by Chromium, not claimed as native numerical coverage.
- Coverage commands: scoped `cargo llvm-cov --no-report --locked` for default and SQLite, followed by `cargo llvm-cov show-env` instrumented binary builds and `proofs/coverage.py`. Set `CARGO_LLVM_COV_TARGET_DIR` to the same native target. Raw export: `/private/tmp/task00009-coverage.json`; summary: `task00009-coverage-summary.log`; machine scope receipt: `task00009-coverage-receipt.json`; build/test logs: `task00009-cov-*.log`.
- Both Cargo targets were deleted after verification; logs, browser assets and coverage export remain reviewable. Every database used for this task was in temporary storage. No listener was started; the gateway listener executable was built, never run. No commit, push, publishing, deployment or KB reindex occurred.

### Boundaries and KB validation

This is the native web/gateway SQLite integration. Tauri database packaging/location, new persistence APIs, production hosting and publication remain outside this task. PostgreSQL/MySQL opt-in servers were not used. Coordinator verification/status remain pending.

Before the task-note write, the Agent Workspace validator reported **146 errors and 21 warnings**, all from existing KB material (chiefly sibling-source links absent in this worktree). The post-write comparison is recorded separately; unrelated KB repair is outside scope.

## Verification notes

Coordinator review on 2026-10-26. Result: **verified**; merged into `trial/3`.

- **Scope:** only the profile-ui web/gateway/proofs/check/README, the root manifest/lock edges and this task file. sql-profile and `crates/**` are unchanged.
- **Checks rerun by the coordinator:**
  - `check.py` passed, in both default and `--sqlite` modes;
  - 443 workspace tests passed, 0 failed; Clippy `-D warnings` clean; `cargo +1.96.0 check` passed;
  - all 16 browser proofs passed (8 per backend, root and `/demo/v1`).
- **First default browser run:** it failed only because it ran after the `--sqlite` check had rebuilt the shared `target/profile-ui-native` web binary with the SQLite feature. The binary correctly refused to start without an installed database. A rerun immediately after a default build passed 8/8.
- **Coverage:** web 93.24%, gateway 88.82% (implementer figures).
- **Follow-ups queued in task 00010's third pass:**
  1. The default and SQLite builds share one binary path, so the proof wrapper should rebuild for its backend or detect a wrong-mode binary.
  2. The startup error in `web/src/backend.rs` interpolates its cause Display into the wrapper message ("…: SQLite operation failed"), which violates the #11 wrapping policy merged afterwards.

## Acceptance notes

No new visual acceptance; output parity with the accepted UI is verified instead.
