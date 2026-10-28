# Issue 15 maintained presentation proofs

The temporary M0 prototype was removed. These proofs run against the actual
ui-preferences example, its generated WASM, and the actual profile native adapter.
No listener or install is needed. Run Cargo/proof/cleanup commands through the
repository benchmark recorder; a supervising parent caps the one target at 8 GiB.

- `golden.rs` / `golden.py`: the identical before/after consumer and pinned
  `golden-v1.json` compare exact cookie flags/scope/lifetimes, HTTP/Tauri envelopes,
  and revision-1 native file bytes. `before` was captured and repeated before the
  source move. It requires the pre-move source checkout; `after` uses current code.
  The separate legacy expectations are `system` before and null after; French
  locale and unchanged file bytes after load/reopen are asserted in both phases.
- `first-paint.cjs`: 40 real-example cases across root/mounted hydrate/CSR,
  OS colors, saved/unset/legacy choices, no-JS, blocked/missing asset and strict CSP.
  Delays application JS, records painted content colors and query counts, preserves
  early input identity/focus/drafts, and asserts no initial writes or settings calls.
- `contracts.py`: exact external lock tuple equality against trial/3, capability
  graphs, the retired prototype, asset fingerprint/size and one-shot constraints.
- `verify.py` and `finish.py`: selected tests/doctests/lint, generators' checks,
  native workspace and WASM/facade checks, asset manifest regeneration/checks.
- `coverage.py`: native production line reports per changed framework crate;
  excludes dedicated test files and keeps browser/platform limits explicit.
- `supervise.py` / `test_supervise.py`: bounded task-owned child groups, including
  success, timeout and termination kill/reap proofs.

Set `CARGO_TARGET_DIR`, `PROFILE_UI_NATIVE_TARGET`, and `PROFILE_UI_TARGET` to the
same task target. `PREFERENCES_LOGS` selects evidence/assets, and
`PREFERENCES_PLAYWRIGHT` selects the already-installed package. Build assets through
`examples/ui-preferences/build.py`. The existing example browser flows remain in
`../browser.py`; profile native and SSR handoff proofs live with that example.
Evidence and benchmark IDs for this run are in the M1 spec receipt.
