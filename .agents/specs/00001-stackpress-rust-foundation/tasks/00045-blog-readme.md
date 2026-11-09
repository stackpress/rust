# 00045 — Blog README and tooling boundary

Status: **implemented; clean-copy verification passed; coordinator review pending.**
Based on `f7d7d27dd7ade5fe3b6d5063c57f90c2f0ac6fec`; no commit.

## Task and implementation

- Rewrite the [blog README](../../../../examples/blog/README.md) for a new Rust developer: overview, unchanged original-counterpart table, prerequisites, copied dependency paths, generation, SQLite, browser build, vanilla serve command, features to try, command reference, migration and limitations.
- Preserve the original detailed prose in [design notes](../../../../examples/blog/docs/design-notes.md), rebasing links and applying the owner's tooling corrections. Full verification detail now lives in the [repository contributor guide](../../../../tools/examples/blog/README.md). The commands guide remains linked.
- Follow the owner's final scope: replace only the browser build with `cargo xtask build-browser`; move the other 14 scripts to `tools/examples/blog/`. No script or bytecode cache remains in the example.
- `blog-xtask` uses only the standard library at runtime, inherits `publish = false`, and has no application/generator dependency. A Cargo alias selects it. This keeps `blog-tool` focused on schema generation. Existing workspace `tempfile` supplies tests; no external tuple was added.
- The browser helper preserves separate hydrate/CSR targets, locked/offline Cargo and installed bindgen output. Deadlines kill/reap the owned process group. Unit tests cover target selection, mode isolation, early failure, filesystem failure, process status and descendant cleanup.
- The extra generated Profile test fixture is behind `blog-web/extension-proof`. The moved gate and storage proof enable it, retaining the existing regression. Normal `cargo test --workspace --locked --offline` now works after user setup without repository-only generated fixtures.
- Moved proofs derive the current checkout paths; copied-app browser builds use the Rust command. README proof extracts actual setup/test/migration commands and preserves a separate baseline project for the nickname proof. Coverage includes `blog-xtask`; G56 still exempts only `blog-client`. Playwright can resolve normally or through `BLOG_PLAYWRIGHT`.
- Server changes are neutral comments and the requested missing-port diagnostic. No listener, install, licence change, commit or push.

## Review rounds and implementation evidence

| Round | Finding and evidence |
| --- | --- |
| Logic / complexity | Reviewed first-failure ordering, distinct targets and process ownership; no further production refactor justified. Five tests pass: `20261109T143753.043576Z-a8764b0ea106`. |
| Responsibility / abstraction | Kept build orchestration independent of generation; moved unit tests outside production source so their lines do not inflate coverage. Five tests pass: `20261109T143844.122820Z-3747010f02c4`. |
| Consistency / Rustdoc / comments | Updated command/path/docs and port language. README setup, both renderers, 55 normal tests and the real nullable-column migration pass: `20261109T143926.159150Z-04aa21442bf1`. Strict Clippy found three redundant borrows in tests, then passed after correction: `20261109T144347.093346Z-b86bce4cba0d`. |

- One authorized root-seeded offline lock refresh: `20261109T143429.407583Z-ade5d813eb52`. Root bytes and all 699 ordered registry tuples unchanged; blog's same 437 tuples remain an exact subset. Only local `blog-xtask` was added to the lock.
- Formatting passes: `20261109T143912.422372Z-df697e61516f`. Four documentation files / 12 local links and anchors pass, with source-choice text and original mapping table preserved: `20261109T143955.764322Z-7809a9de9028`.
- KB baseline: **146 inherited errors / 43 warnings**, exit 1 (`20261109T143956.259481Z-4b50c7ffe4aa`); not clean KB validation.
- Local targets retired before the clean proof: `20261109T145337.497987Z-735ac0a97137`.
- Detached clean checkout: `/private/tmp/blog-clean-00045-f7d7d27-01`; initially no `.build` or target. Scoped patch SHA256 `384133539f44b8ccc2b4996d2ac3073bfccc5cce8de99fa677fd59761177bc0e`. Provenance and patch retained under `examples/blog/.build/clean-copy/00045-f7d7d27-01/`.
- Persistent records/logs: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Sccache, incremental 0, dev debug 0, two jobs and owned targets only. Command wall time and observed Cargo-stage time are separate measurements.

## Clean-copy result

- README commands from the detached clean checkout passed: generation/check, fresh SQLite/revision/population/query, both browser builds, **55 normal tests**, and the nullable Article migration preserving both seeded articles. Record `20261109T145413.040172Z-3ff896d8b651`: command wall **653.054466s**, observed Cargo stages **612.86s**.
- Moved full gate ran **once**, with all **26 phases passing**: strict native/hydrate/CSR Clippy; **57 native tests, none skipped**; doctests (0 examples); Rustdoc; graph/freshness checks; six public browser matrix cases; both authenticated/admin/editor/API/export flows; copied Profile migration/browser flows; native coverage and locks. Record `20261109T150716.414251Z-5d098874473e`: command wall **1346.285145s**, observed Cargo stages **1123.47s**.

| Native crate (shipping generated code included) | Executable lines |
| --- | --- |
| blog-tool | 102/113 (90.27%) |
| blog-web | 2325/3310 (70.24%) |
| blog-app | 3698/3882 (95.26%) |
| blog-store | 38/39 (97.44%) |
| blog-assets | 59/66 (89.39%) |
| blog-xtask | 85/100 (85.00%) |

- Only `blog-client` is G56-exempt, exercised by real hydration/CSR browser tests. Xtask unit-test source is outside `src/` and is excluded from production line counts by the existing source grouping.
- LLVM export reported **37 functions have mismatched data**. This existing diagnostic and the unfiltered raw report are preserved; coverage is not described as warning-free.
- Peak sampled targets **11,105,288,192 bytes (10.343 GiB)**, below 12 GiB. The final audit verifies the exact tested patch against the original worktree, all 26 results, coverage, 699/437 lock invariants, no remaining example tooling references/scripts, and scope: `20261109T153027.875204Z-abc0abb5de87`.
- Reports, raw coverage, public/copied browser evidence, exact launch environment, provenance, patch and ledger are retained under `examples/blog/.build/clean-copy/00045-f7d7d27-01/`. Clean worktree and all owned compiled targets were deleted: `20261109T153117.371173Z-c39c0fbe2f51`. No other lane's output was touched.
- Through the full gate: **16 recorded attempts, 14 passed / 2 failed**, accumulated command wall **2745.803317s**, observed Cargo stages **2424.07s**. The failures are the inherited KB baseline and the corrected test-only Clippy finding. No root-workspace build was run. Later audit/cleanup/KB records are additional and retained in the ledger; summed durations are not elapsed session time.

## Verification and acceptance notes

Coordinator verification pending. Acceptance criteria: none for the implementation proofs;
README review remains with the coordinator. No product acceptance or context promotion is inferred.
