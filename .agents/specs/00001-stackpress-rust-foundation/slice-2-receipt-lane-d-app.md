# Lane D: profiles application proof

Spec file of [00001 Stackpress Rust foundation](index.md), recording lane D of [task 00005](tasks/00005-slice-2-server-and-cli.md). Load for the `examples/capability-host` proof application (profiles plugin, customization, compile-time experimental exclusion, lifecycle reporting), its coverage method, and proof-item status. Moved losslessly from `examples/capability-host/LANE-D-RECEIPT.md` (lane branch commit `340750d`, merged in `253734a`).

Implemented on 2026-10-24 in branch `trial/3-lane-d-app`, based on accepted M1/M2
commit `f19f37d`. This is the initial, in-process application deliverable for coordinator
review. The app binary and buffered HTTP adapter proof await the coordinator's next brief.

## Scope and behavior

- Added `stackpress-example-capability-host`, explicitly `publish = false`.
- Added only its workspace member and package lock entry outside `examples/capability-host/`.
  No existing dependency version changed. No `crates/`, `tools/`, facade, or KB file changed.
- `build_application(ConfigLayers)` owns explicit compiled plugin selection and returns
  `BuildResult<Server>`. This matches the future `main_with` callback shape without a stub binary.
- `profiles::register` is an ordinary function queuing named Config, Listen and Route functions.
  Configuration is typed, denies unknown fields, defaults to limit three, and accepts `1..=100`.
  Config constructs an `Arc<Profiles>`; Listen registers `profile-search`; Route registers
  `GET /profiles/:id` and calls `inv.delegate` from its named route handler.
- Search uses case-sensitive, nonempty ID prefixes, preserves seed order, then limits results.
  Three public demo rows are held immutably. `rows` reports the returned-row count as `total`.
  Missing/empty/non-string IDs fail; unknown prefixes return successful empty rows and total zero.
- `customization::register` independently installs a priority-100 listener that reuses the
  service/input policy, copies public rows with customized names, and returns Abort. The original
  listener cannot overwrite these results. G12 also skips response hooks after this Abort;
  direct and route-delegated calls prove that consequence explicitly.
- The `experimental` Cargo feature defaults off. Both its module and selection are cfg-gated.
  Enabled builds add a `profile-preview` operation delegating to `profile-search`.
  Disabled builds exclude the entire implementation, rather than only omitting registration.
- No plugin trait, registration macro, runtime plugin loader, application I/O, spawned task,
  socket, or listener was introduced. Standard serde derives and the feature-proof compile guard
  are used. The only handwritten data trait implementation supplies config defaults.

## Integration evidence

The final package suite has **12 tests per feature mode**, with no ignored package tests:
seven profiles/composition tests, four lifecycle tests and one mode-specific experimental test.

| Contract | Evidence |
| --- | --- |
| Direct and routed public result | One host serves `call("profile-search", {"id":"abc123"})` and `handle_route` for GET `/profiles/abc123`; entire JSON envelopes match, including status/results/total. |
| Capture precedence and manual values | Captures override query, POST, explicit and manually edited aggregate IDs; original query/POST remain intact. A manually edited Response header survives handling, and headers remain editable before native output. |
| Config defaults → JSON → code | Rebuilding with successive layer prefixes returns three, two, then one row. A later seed row still matches under limit one, proving filtering precedes truncation. |
| Validation | Zero, excessive, negative and nonnumeric limits, unknown fields and unclaimed sections reject startup. Limit 100 is accepted. Missing/empty/non-string/null IDs fail; unknown/case-mismatched prefixes return empty rows. |
| Customization | A separately selected dependent plugin changes names through both call and route paths, preserves the configured limit despite request data `limit:100`, and leaves the shared service's seed rows unchanged. Missing prerequisite fails preflight. |
| Feature exclusion | Disabled-build compilation would hit `experimental.rs`'s `compile_error!` if that module were accidentally included. A default-build compile-fail doctest additionally proves the module API is absent. Default runtime selection contains only profiles and preview is 404. |
| Enabled experimental plugin | The enabled test references its real registration function, observes both selected plugins and compares preview/search public envelopes and lifecycle record counts. |
| Lifecycle/reporting | Separate test-plugin hooks observe Request → Operation → Response once, including the nested route capability, sharing the report's nonempty correlation ID. Separate roots get different IDs. |
| Error listener | Operation failure reaches a real error listener with the same correlation and original failing stage. The report retains profiles plugin/event/source frames; completed observation does not recover the public 400. |
| Guard/error/input boundaries | Request failure skips operation and is offered once to Error. G17 pre-normalization failure records Input/400 with no callbacks or fabricated Request. |

## Baseline and review rounds

Fresh effort began with `cargo clean` and `cargo llvm-cov clean --workspace`, before any build.
The initial unlocked package test updated Cargo.lock; all subsequent build/test/lint/doc commands
used `--locked`.

Baseline: 11 integration tests passed in each feature mode; correctly scoped default production
coverage was **108/108 executable lines (100%)**. An initial ordinary cargo-llvm-cov invocation
produced an empty report and failed its threshold because its default ignore rule excludes every
`examples/` directory. That was an invalid measurement, not a passing coverage result. Explicitly
including the example's production source corrected it; see the final exact commands below.

1. **Logic/complexity:** reviewed prefix matching, filter/limit order, validation boundaries,
   capture collisions, error entry and Abort propagation. Added the later-row/limit assertion
   and G17 malformed-input test. No production simplification was justified: guards and iterator
   ordering already expose the policy. Both feature suites reran: **12 passed each**.
2. **Responsibility/abstraction:** reviewed app selection versus plugin/service/config ownership,
   independent customization and feature boundaries. Kept cohesive profiles code in one module;
   customization and experiment each own their registration/handler code. There is no generic
   helper layer or new trait to remove. No-change production round; both feature suites reran:
   **12 passed each**. Normal dependency tree contains only the approved direct dependencies.
3. **Consistency/Rustdoc/comments:** checked named async handlers, default/phase naming, errors,
   borrowing, module/field/function docs and block/branch comments. Added an executable crate
   example and the conditional disabled-module compile-fail doctest. Both feature suites passed;
   default doctests **2 passed**, experimental doctests **1 passed**; scoped Clippy passed with
   `-D warnings`. All seven Rust files have function Rustdoc and no lines above 100 columns.
4. **Safety/soundness:** inspected immutable service borrows across await, explicit Arc sharing,
   copied customization output, trusted config versus request data and pre-output mutation.
   Added checks for unchanged service seeds, request limits unable to replace host policy, and
   manual Request/Response edits. Production code uses no unsafe, locks, globals, native handles,
   blocking operations or detached work. Test-only Mutex guards cover synchronous trace accesses
   and never survive an await. Both feature suites and fresh coverage runs passed after these
   assertions. No soundness veto or framework API change was needed.

## Final verification

Compiler: rustc **1.96.0** (`ac68faa20`, LLVM **22.1.2**).
Cargo: **1.96.0**; cargo-llvm-cov: **0.8.6**. Target: **aarch64-apple-darwin**.
Logs and JSON reports are under ignored `target/lane-d-*` paths in this worktree.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `cargo test -p stackpress-example-capability-host --all-targets --locked` | 12 passed. |
| `cargo test -p stackpress-example-capability-host --all-targets --features experimental --locked` | 12 passed. |
| `cargo test --workspace --all-targets --locked` | 89 passed, 0 failed; six existing helper entrypoints are ignored by ordinary enumeration and exercised by their watchdog/PTY parent tests. |
| `cargo test --workspace --doc --locked` | 272 passed, including the app's two default-build doctests. |
| `cargo test -p stackpress-example-capability-host --doc --features experimental --locked` | 1 passed. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed warning-free. |
| `cargo clippy --workspace --all-targets --features server-http,stackpress-example-capability-host/experimental --locked -- -D warnings` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --features server-http,stackpress-example-capability-host/experimental --no-deps --locked` | Passed warning-free. |
| `cargo tree -p stackpress-example-capability-host --edges normal --locked` | Approved direct edges only: lib, server, serde, serde_json; no Tokio/CLI/network edge. |
| `cargo tree -p stackpress-example-capability-host --features experimental --edges normal --locked --depth 1` | Same direct edges; experiment is compiled application code, not another dependency. |
| `git diff --check`; function-doc/100-column source scan | Passed. |

### Coverage scope and reproduction

Both final measurements were independently preceded by `cargo llvm-cov clean --workspace`.
The report disables the tool's automatic `examples/` exclusion and excludes only dependencies,
other workspace packages, tests and build artifacts. **No production file in this package is
excluded.** The experimental source is absent from the default compilation through cfg, and all
of it is measured in the enabled build. Ordinary doctests run separately, without nightly coverage.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-capability-host --all-targets --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/|/(crates|tools|tests|target)/' \
  --json --output-path target/lane-d-coverage-default.json --fail-under-lines 70

cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-capability-host --all-targets \
  --features experimental --locked --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/|/(crates|tools|tests|target)/' \
  --json --output-path target/lane-d-coverage-experimental.json --fail-under-lines 70
```

| Package feature scope | Executed / executable lines | Line coverage |
| --- | --- | --- |
| Defaults, experiment excluded | 108 / 108 | **100%** |
| Experimental enabled | 128 / 129 | **99.22%** |

Both exceed the 70% development floor. The enabled build's remaining line is the defensive
plugin-selection error return at `src/lib.rs:64`; the application supplies a fixed valid plugin
ID and prerequisite. Zero-count regions also include defensive registration-error returns on
lines counted as covered by their successful paths. Line coverage is not a branch, adapter or
publication guarantee; meaningful validation/ordering/Abort/error combinations are tested above.

## Proof checklist disposition

Against `.agents/references/00006-discussion-crate-plugin-proposals.md`:

| Item | Current disposition |
| --- | --- |
| 5: CLI and HTTP invoke profile-search | **Partial:** direct host call and host route delegation agree. Actual application CLI and buffered HTTP adapter comparison await lanes B/C and the extended brief. |
| 6: manually constructed mutable requests/responses | **Demonstrated in process:** request data edits, response headers before/after handling, capture injection and public results; no native output is performed. |
| 7: Result errors reach Error with explicit outcomes | **Demonstrated:** real error listener and retained failure/report, unrecovered public error, missing-listener paths and skipped-operation guard failure. |
| 8: disabled experimental plugin excluded from selected build | **Demonstrated:** cfg module boundary, disabled-build compile guard, absent-API compile-fail doctest and enabled runtime proof. |
| 9: customization without editing original | **Demonstrated:** independent plugin overrides behavior on both host entry paths with unchanged original profiles source/service. |
| 14: explicit lifecycle/abort; nested calls do not repeat it | **Demonstrated:** single Request/Operation/Response sequence and inherited correlation, error sequence, request guard failure, and nested customized Abort skipping Response. |

Proof items 1/10 (Idea/generation), 11/12 (UI/Tauri) and platform expansion are outside this brief.
The brief's R26–R28 reference points to prompt/TTY risks in the current verification document;
those belong to lane B. This package addresses the application portion of M4 and the R30/R31
feature/evidence boundaries without claiming prompt or terminal-driver verification of its own.

## DX observations and remaining work

- The current Server API supports all assigned app contracts. `inv.service`, typed config lookup,
  named phase functions and `inv.delegate` fit the design without application trait implementations
  or boxed-handler ceremony. No missing API or dependency expansion blocked this lane.
- High-priority customization via Abort intentionally skips Response hooks as well as the original
  operation. The source docs and tests make this consequence visible; the example does not imply
  that Abort only replaces a handler. Handle-based replacement is a different available approach.
- Tests returning lib `Result` explicitly unwrap `BuildError.error` with `map_err`; the public app
  builder retains the complete `BuildResult` and startup metadata for future CLI integration.
- Future changes are limited to adding the app-owned `stackpress` main/dependency and an in-process
  buffered HTTP test after coordinator integration. No placeholder binary or HTTP stub was added.
- This receipt verifies only the native target above. No network, streaming, auth, database,
  generator, serverless, Leptos/Tauri, cross-platform or publication claim follows from these tests.

No commit, push, merge, listener, socket, deployment, toolchain installation, KB edit/indexing,
other trial source read or external message was performed. Stopped for coordinator review.

## HTTP-path proof receipt

The following lane D HTTP receipt was moved losslessly from `examples/capability-host/LANE-D-HTTP-RECEIPT.md` (lane branch commit `1f1a319`, merged in `93654d7`); headings are demoted one level.

## Lane D: application HTTP proof

Date: 2026-10-24. Branch: `trial/3-lane-d-app`, based on the coordinator's fast-forward
to `727e5e2`. This extends the accepted application proof using the merged buffered HTTP adapter.
The app-owned CLI binary remains deferred until the coordinator releases the `run`/`main_with`
integration brief. No CLI stub was added.

### Changes and scope

- Enabled `stackpress-server/http` in this package's dependency declaration.
- Added `tests/http.rs`: four integration tests covering six HTTP scenarios against the real
  profiles application, its service and its independently selected customization plugin.
- Updated two crate-overview Rustdoc lines to reflect the available HTTP proof and pending CLI.
- Used the adapter's exported `BufferedRequest` alias to construct native `http::Request<Vec<u8>>`
  values. No direct `http` dependency, new crate, dependency version change or lockfile edit was
  needed. No production Rust behavior changed.
- All edits are under `examples/capability-host/`. The root manifest, lockfile, framework, tools,
  facade and KB remain unchanged. Source inspection stayed in this worktree and its KB; no other
  trial branch or other lane worktree was read.

### Proved behavior

| Case | Observable result |
| --- | --- |
| GET `/profiles/abc123` and direct `profile-search` | One compiled application serves both entries. The native HTTP status is 200, and the entire decoded public envelope matches `Server::call`, including the Ada Lovelace row and total one. Report event/correlation and final Output/Sent state are checked independently from JSON. |
| Valid route with unmatched ID | Both entries return status 200, an empty result array and an explicit total zero. This remains distinct from an unknown route. |
| Unknown HTTP route | `/unknown?event=profile-search&id=abc123` produces native status 404 and a public JSON 404/error envelope with no results. The report selects only `GET /unknown` and records NoListener; query data cannot select the internal operation. |
| G17 asterisk-form input | A representable native URI of `*` cannot normalize into the adapter's pathname contract. It returns native/JSON 400 with only Input/Output records, one Input failure, no error offer and no operation. |
| G17 normalization-changing path | `/ignored/../profiles/abc123` is rejected with the same no-callback G17 contract, rather than silently selecting the valid profiles route after URL normalization. |
| Customized HTTP entry | A separate dependent plugin changes names through HTTP and direct calls identically. Captured `id` overrides query data, configured limit one overrides request `limit=100`, and delegated Abort skips Response while still producing Output/Sent HTTP output. |

The shared decoding helper explicitly matches `HttpOutput::Response`; Suppressed and Unavailable
fail the proof. It validates native status before decoding actual native response bytes. The
malformed-input host attaches a named panic-on-entry handler to Request, profile-search, Error
and Response: any fabricated callback input fails immediately, supplementing the exact report
assertions. These tests construct buffered values and open no sockets or listeners.

### Baseline and scoped rounds

Before the first build, ran `cargo clean` and `cargo llvm-cov clean --workspace`; Cargo removed
14,203 stale files / 1.5 GiB from this worktree. Every build/test/lint/doc invocation used
`--locked`, and Cargo.lock remained unchanged.

Baseline: the four new HTTP tests passed, and the complete example suite passed **16 tests in
each feature mode**. No package tests are ignored.

1. **Logic/complexity:** reviewed successful versus empty results, missing route versus malformed
   input, query/capture precedence, configured limits and delegated Abort. Added the successful
   empty-result scenario and a normalization-changing path that could otherwise enter a real
   profiles route. Both full example suites reran: **16 passed each**.
2. **Responsibility/abstraction:** reviewed native construction, output decoding and scenario
   ownership. Kept two focused local helpers and explicit per-scenario app composition. Using the
   public buffered alias avoids unnecessary dependency coupling; no production helper, generic
   wrapper or shared test-fixture extraction was justified. No-change round; HTTP tests reran in
   both modes: **4 passed each**.
3. **Consistency/Rustdoc/comments:** consolidated lib imports and updated the crate overview for
   the integrated HTTP adapter. Reviewed Rustdoc for every new function and purposeful setup,
   branch, conversion and assertion-block commentary. Both HTTP suites passed; app doctests
   passed **2 default / 1 experimental**; scoped all-feature Clippy passed with `-D warnings`.
   The final line audit also shortened an overlong comment; final suites below passed afterward.
4. **Safety/soundness:** reviewed owned native buffers, partial move of HttpReply output while
   retaining its report, explicit publication variants, malformed-input callback denial, immutable
   service use and absence of native I/O. The new tests add no unsafe, locks, globals, workers,
   blocking callbacks or cancellation machinery. Panic-on-entry is an intentional test failure
   oracle, not recovery logic. No-change round; both HTTP suites reran: **4 passed each**.

No in-scope material findings remain. All new test functions/helpers have Rustdoc; new test source
and the edited crate source have no lines over 100 columns.

### Final verification

Toolchain: rustc **1.96.0** (`ac68faa20`, LLVM **22.1.2**), native target
**aarch64-apple-darwin**. Coverage tool: cargo-llvm-cov **0.8.6**.
Logs and coverage JSON are retained under ignored `target/lane-d-http-*` paths.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `cargo test -p stackpress-example-capability-host --all-targets --locked` | **16 passed**, including four HTTP tests. |
| `cargo test -p stackpress-example-capability-host --all-targets --features experimental --locked` | **16 passed**, including four HTTP tests. |
| `cargo test --workspace --all-targets --locked --features server-http` | **127 passed**, zero failed. Ten existing helper child entrypoints are ignored in ordinary enumeration; their watchdog/PTY parent tests passed. |
| `cargo test --workspace --doc --locked --features server-http` | **322 passed**: facade 1, CLI 48, app 2, lib 268, server 3. |
| `cargo test -p stackpress-example-capability-host --doc --features experimental --locked` | **1 passed**. |
| `cargo clippy --workspace --all-targets --locked --features server-http -- -D warnings` | Passed warning-free. |
| `cargo clippy --workspace --all-targets --locked --features server-http,stackpress-example-capability-host/experimental -- -D warnings` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --features server-http` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --features server-http,stackpress-example-capability-host/experimental` | Passed warning-free. |
| `cargo tree -p stackpress-example-capability-host --edges normal --locked --depth 2` | Existing approved dependencies; server adds the already locked `http` 1.5.0 edge. No CLI, Tokio or network runtime normal edge. |
| `git diff --check`; function-doc and 100-column scan | Passed. |

Enabling HTTP in this example is unconditional; the two measured feature modes differ only in
the `experimental` plugin. The server crate's own optional feature declaration was not changed.

### Coverage

Each final collection was independently preceded by `cargo llvm-cov clean --workspace`.
The corrected command disables the tool's default `examples/` exclusion, then excludes only
dependencies, other workspace packages, test source and build output. All production source in
this example is measured; no shipped example implementation is excluded. Doctests run separately.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-capability-host --all-targets --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/|/(crates|tools|tests|target)/' \
  --json --output-path target/lane-d-http-coverage-default.json --fail-under-lines 70

cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-capability-host --all-targets \
  --features experimental --locked --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/|/(crates|tools|tests|target)/' \
  --json --output-path target/lane-d-http-coverage-experimental.json --fail-under-lines 70
```

| Example scope | Covered / executable lines | Line coverage |
| --- | --- | --- |
| Defaults, HTTP enabled, experiment excluded | **108 / 108** | **100%** |
| Experimental plugin plus HTTP | **128 / 129** | **99.22%** |

Both exceed the 70% development floor. The remaining enabled-build line is the defensive error
return for the app's fixed valid experimental-plugin selection at `src/lib.rs:64`, as in the
accepted initial proof. These are example-package production metrics, not measurements of the
unchanged HTTP adapter or assertions about branch/platform coverage.

### Disposition and DX feedback

- Proof checklist item 5 now has the real application's **HTTP-to-capability** path in addition to
  direct-call/route evidence. Its **CLI-to-capability** portion still awaits lane B and the next
  coordinator brief; the complete CLI/HTTP claim remains pending.
- Items 7, 9 and 14 gain native HTTP error, customization and Abort/output evidence. Existing
  compile-time experimental exclusion and the original profiles implementation remain intact.
- The HTTP API supplied every required seam. Exported buffered aliases allow dependency-light
  construction; HttpOutput and retained reports express native availability and diagnostics
  clearly. No missing API, dependency expansion or framework change is requested.
- Verification is in process on the native target above. It establishes no listener, network
  buffering, streaming, deployment, other platform or publication support.

No commits, pushes, listeners, sockets, KB edits or publishing were performed. Stopped for
coordinator review; the next work item is the separately released application CLI proof.

