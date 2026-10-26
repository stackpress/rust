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

Pending.

## Verification notes

Pending.

## Acceptance notes

No new visual acceptance; output parity with the accepted UI is verified instead.
