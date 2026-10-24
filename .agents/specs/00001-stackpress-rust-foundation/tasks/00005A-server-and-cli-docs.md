# Task 00005A: Server and CLI usage and API docs

Follow-up of [task 00005](00005-slice-2-server-and-cli.md) in [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The Slice 2 design planned it, and the owner accepted it on 2026-10-24, to run in parallel with Slice 3 ([task 00006](00006-slice-3-idea.md)).

## Task summary

Write developer documentation for `stackpress-server` in `crates/server/docs/` and for `stackpress-cli` in `tools/cli/docs/`, covering how to use them plus an API reference. Follow the installed `chrisai-docing` skill and the same conventions as [task 00004](00004-lib-usage-and-api-docs.md):

- guided-learning pages first, then lookup-oriented reference pages;
- examples as plain Markdown, compile-checked in a disposable consumer outside the repository;
- no source changes;
- API rough edges reported as DX observations.

## Implementation steps

1. Server docs:
   - `README.md` with reading paths;
   - `getting-started.md` (a server with a plugin, calling a capability);
   - `concepts.md` (plugins and dependency order, config layering, services, startup phases, the top-level lifecycle and its G12/G17 rules, raw versus top-level dispatch, reports and correlation, the HTTP adapter);
   - `guides/` (by task intent);
   - `api/` (an index plus one page per public surface).
2. CLI docs: the same structure, scaled to the crate. Cover an application-owned `stackpress` binary via `main_with`, operator flags versus capability arguments, the stdout/stderr contract and exit codes, `Terminal`, `Control`, and prompts with the native TTY driver.
3. Verify every claim against source and tests. Use `examples/capability-host` as the canonical integration example.
4. Apply copy-editing and formatting, and run the skill's review gates.

## Verification process

The coordinator reviews the pages against the `chrisai-docing` review gates, spot-checks accuracy against source, runs a mechanical format and link scan, and confirms that no source, tests, or manifests changed. Workspace checks must still pass.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Implemented on `trial/3` at `bbbcd3d`, using the current public Rust source, contract tests,
accepted G12–G17 decisions, Slice 2 designs/receipts, and canonical capability-host application.
Only the two assigned docs directories and this Implementation notes section were edited.
Concurrent Slice 3 spec changes belong to the other implementer and were left untouched.

- Created 39 pages / 4,839 lines: server 21 pages / 2,740 lines; CLI 18 pages / 2,099 lines.
  Both sets follow the Lib family: entry page, visible first success, concepts, intent-based guides,
  API index, and separate public-surface lookup pages. Lib concepts are linked rather than copied.
- Server coverage includes plugin preflight/order, explicit config layers and typed validation,
  services/provenance, phase gates, all registration/route helpers, capture injection, raw versus
  handled entry, G12/G17 error/Abort/finalization, retained reports/correlation, OutputGuard, and
  buffered HTTP policy/limits/framing/availability.
- CLI coverage includes the app-owned main_with binary, injected entry/run seams, operator versus
  target argv, machine/human sinks and exit precedence, Terminal/Control/color/NO_COLOR, prompt
  options/validation/state/rendering/cancellation, and native TTY ownership/cleanup limitations.
- API references contain 144 method entries with purpose, examples, parameter tables where
  applicable, and Returns prose including errors. A source-name audit found entries for all 63
  distinct server and 51 distinct CLI public inherent/free method names; shared registrar methods
  are documented once for both receivers. Public traits, fields, variants, defaults, aliases, and
  feature boundaries are also covered; private helpers/registries are omitted.
- Followed chrisai-docing sequencing: guided learning and API ownership, then an editorial pass
  for technical truth/assumptions/terminology/transitions, then Markdown normalization and rustfmt
  for snippets. Author review covered first-success flow, explained examples, separate lookup
  material, method schema, and public-only scope.
- Examples remain plain fenced Markdown with named async handlers first. No doctest wiring or
  tracked harness was added. The disposable consumer at `/private/tmp/stackpress-docs-00005a/consumer`
  compile-checks all 178 Rust blocks, using independently wrapped API snippets with each page's
  stated setup. Its lock was seeded from the workspace lock and checks run offline/locked.
  All 12 noninteractive learning walkthroughs pass. A second consumer omitting TTY-only API
  fragments compiles the remaining examples and passes the same 12 walkthroughs with CLI default
  features disabled. Native TTY examples were compile-checked, not interactively run by this harness.
- Both complete quick starts were compiled and executed as separate binaries: server printed
  `Hello, Ada`; CLI exited 0 with exactly one public JSON line containing that result and empty
  stderr. Existing workspace PTY/watchdog tests passed independently of these doc examples.
- Author checks passed on Rust 1.96.0 / native aarch64-apple-darwin: cargo fmt --all -- --check;
  cargo test --workspace --all-targets --locked (143 passed, zero failed, 15 intentional child
  markers invoked by parent tests); cargo test --workspace --doc --locked (394 passed, zero failed).
  A fresh external CARGO_TARGET_DIR kept workspace builds separate from concurrent work; cargo
  clean and cargo llvm-cov clean --workspace ran against that external target before workspace
  verification. No coverage claim is made for this documentation-only change.
- Mechanical scan passes for all 39 pages and 235 relative links/anchors: one H1, tagged fences,
  no empty parent headings, no consecutive blank lines, and all link targets/anchors resolve.
  Check scripts, consumers, build artifacts, and logs remain outside the repo under
  `/private/tmp/stackpress-docs-00005a/`. These are implementer self-checks; coordinator verification
  remains in the section below. Agent Workspace validation passed with warnings only; git diff
  --check passed, and the source/test/manifest/lockfile diff audit is empty.

Page inventory (paths relative to each docs directory; counts include all lines):

| Server page | Lines |
| --- | ---: |
| README.md | 43 |
| getting-started.md | 105 |
| concepts.md | 185 |
| guides/writing-a-plugin.md | 71 |
| guides/typed-config.md | 58 |
| guides/services.md | 61 |
| guides/operation-routes.md | 60 |
| guides/errors-and-recovery.md | 77 |
| guides/testing-in-process.md | 51 |
| guides/using-http.md | 66 |
| api/README.md | 22 |
| api/builder.md | 180 |
| api/plugin.md | 79 |
| api/config.md | 96 |
| api/startup.md | 302 |
| api/registration.md | 430 |
| api/context.md | 271 |
| api/server.md | 233 |
| api/reports.md | 151 |
| api/output.md | 64 |
| api/http.md | 135 |

| CLI page | Lines |
| --- | ---: |
| README.md | 37 |
| getting-started.md | 94 |
| concepts.md | 120 |
| guides/application-binary.md | 45 |
| guides/arguments.md | 43 |
| guides/human-output.md | 43 |
| guides/prompts.md | 61 |
| guides/native-prompts.md | 82 |
| guides/testing.md | 65 |
| api/README.md | 20 |
| api/entry.md | 104 |
| api/terminal.md | 187 |
| api/control.md | 259 |
| api/execution.md | 108 |
| api/prompt-options.md | 280 |
| api/prompt-state.md | 176 |
| api/prompt-driver.md | 211 |
| api/tty.md | 164 |

DX observations, documented without source changes:

- HTTP has no explicit cookie-list codec option; a list-valued session revision produces a named
  Output diagnostic and sanitized empty 500. Scalar cookies and scoped removal work.
- main_with/main_with_args use neutral TerminalOptions defaults and expose no brand/color defaults
  hook. Custom automatic-diagnostic presentation needs parse_with_options plus run in an app-owned
  entry, which must also own any executable cancellation-to-130 mapping it wants.
- ServerBuilder::extend moves selection/config/listeners/routes/tasks but does not transfer the
  donor correlation prefix. The receiver's prefix remains the host policy.

No Rust source, tests, manifests, lockfile, other task/spec sections, commits, pushes, listeners,
publication, or KB MCP/indexing were changed by this task. No additional product decision was needed.

## Verification notes

Coordinator review on 2026-10-24. Result: **verified**.

- **Scope:** 39 pages: `crates/server/docs` 21 pages, `tools/cli/docs` 18 pages. `git diff` confirms no changes to server/CLI source, tests, or manifests.
- **Mechanical scan** of all 39 pages: one H1 each, tagged fences, no heading directly above a lower-level heading, no double blank lines, and every relative link target resolves. 0 issues.
- **Guided learning:** both `getting-started.md` pages were read in full.
  - The server quick start reaches a printed `Hello, Ada` through a named-handler plugin and `Server::call`.
  - The CLI quick start builds an app-owned `stackpress` binary with `main_with`, mirroring `examples/capability-host/src/main.rs`.
- **Accuracy spot checks** against the accepted decisions:
  - `server/docs/concepts.md` states G12 correctly: a failed request skips the operation; Request/Operation abort stops the lifecycle including Response; errors start from a fresh public error response with a sanitized 500 fallback.
  - It states G17 correctly: a 400 with no callbacks and no fabricated Request.
  - It states the `delegate` mapping correctly, including NoListener → public 404.
- **Example evidence** (from the implementer, not rerun by the coordinator): 178 Rust blocks compile-checked in a disposable consumer outside the repo; 12 walkthroughs pass with and without `tty`; both quick starts executed.
- **DX observations recorded for later**, with no code changes:
  - no HTTP cookie-list codec option (the known G13 limitation);
  - no brand/color defaults hook on `main_with`;
  - `ServerBuilder::extend` keeps only the receiver's correlation prefix.

## Acceptance notes

Documentation has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
