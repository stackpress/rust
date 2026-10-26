# Task 00007A: SQLite documentation receipt

Implementation evidence for [task 00007A](00007A-sql-docs.md). Load for the SQLite page
inventory, public API coverage, executable examples, review gates and DX observations.
Coordinator verification and task status remain coordinator-owned.

## Scope and authority

Date: 2026-10-26. Checkout: `trial/3`, starting HEAD
`4100eab315115e1967e065200466bab3c8e3d33c`, clean before drafting. Only the current branch
was read. This lane owns `crates/sql/sql-sqlite/docs/` and this specifically requested
receipt. The concurrently authored `crates/sql/sql/docs/` belongs to the other implementer;
its shared concepts/guides/API index are linked without editing those files.

Ground truth: current adapter source and tests, [sql-profile](../../../../examples/sql-profile/README.md)
source/tests, decisions G24 and G26 in [decisions](../decisions.md), accepted
[Slice 4 decisions](../slice-4-design-decisions.md), execution/API/integration design notes,
and receipts [M0](../slice-4-receipt-m0.md), [M1](../slice-4-receipt-lane-b-m1.md),
[M2](../slice-4-receipt-lane-b-m2.md), [M3](../slice-4-receipt-lane-b-m3.md), and
[rounds](../slice-4-receipt-lane-b-rounds.md). Current source supersedes proposal sketches
and earlier-milestone omissions. Server/Idea docs and task 00006A supplied the structural bar.

The installed chrisai-docing skill governed guided learning, then API lookup, copy editing
and formatting. No source implementation/refactor rounds or new coverage measurement were
appropriate for this documentation-only task. No source, tests, manifests, Cargo.lock,
task status or other spec files were edited by this lane.

## Page inventory and public coverage

Created **8 pages / 964 lines** under `crates/sql/sql-sqlite/docs/`:

| Page | Lines | Reader outcome |
| --- | ---: | --- |
| `README.md` | 39 | Reading paths, runtime/feature/target scope. |
| `getting-started.md` | 104 | Temp file, explicit table plan, bound insert/select, awaited close. |
| `concepts.md` | 211 | Options, pool/admission/leases, transaction/cancellation/shutdown, arity, limits, disclosure. |
| `guides/server-service.md` | 109 | Named handlers, Config registration, borrowed Engine service, explicit shutdown. |
| `guides/sql-profile.md` | 97 | Explicit setup, read-only startup, byte-prefix TrustedSql, actual reference parity. |
| `api/README.md` | 35 | Public-surface and operation lookup. |
| `api/options.md` | 145 | All option fields, defaults/bounds/serde behavior and four methods. |
| `api/database.md` | 224 | Open, four Database methods, returns/errors and lifecycle integration. |

Public-name audit (`api-audit.json`) covers both exported types, **7 fields**, **5 inherent
methods** and **4 implemented Database operations**:

| Surface | Covered items |
| --- | --- |
| SqliteOptions | `path`, `create_if_missing`, `read_only`, `max_connections`, `acquire_timeout_ms`, `busy_timeout_ms`, `max_in_flight`; `file`, `create_if_missing`, `read_only`, `validate`. |
| SqliteDatabase | `open`; Database `dialect`, `run`, `begin`, `close`. |
| Traits and shared contracts | Options Clone/Debug/Serialize/Deserialize and absence of Default; adapter Send/Sync/static and non-Clone; core-owned Engine/Transaction/Query/QueryLimits/QueryResult/values/errors linked to core docs. |

Each public operation has a named lookup section, executable example, parameters where
applicable, Returns prose and error behavior. Private leases, collectors and session
implementations are explained as lifecycle mechanics, not extension APIs.

Accuracy review explicitly covers no URI/memory/fallback mode; finite option ranges and
unknown-field rejection; eager first connection versus maximum pool size; WAL/FULL/foreign
keys/read-only policy; admission held through acknowledged retirement on a live runtime;
exclusive Rust transaction ownership versus deferred native BEGIN; drop/abort/unwind;
rollback-only after session failure versus pre-session validation; unknown commit outcome
and no retries; cancel/re-await close and the no-runtime fallback; same-lease prepared arity;
retained payload versus native allocation limits; optional rowid observation versus RETURNING;
schema-refresh/cache invalidation; and safe default error formatting versus explicit source
inspection. G26 is described as renderer validation, not production network adapter support.

## Example evidence

Disposable consumer, scripts and logs: `/private/tmp/stackpress-docs-00007a-sqlite/`.
All database files were under TempDir/TemporaryDirectory there or under the system temporary
root; no repository database or documented operator file was created. No listener was started.

The consumer lockfile was seeded from workspace Cargo.lock, then projected with
`cargo generate-lockfile --offline`. All **136 external package identities** match the seed:
no external name/version/source drift. Every example compilation used `--offline --locked`.
The workspace lockfile was untouched.

`verify_examples.py` extracts and rustfmt-formats **all 13 Rust fences**. Each named example
is compiled in its own module and executed as an independent test; only the complete main
program is additionally adapted to an async test entry. The original main program is also
compiled unchanged as `quickstart` and run as a real binary. Final runs have no warnings.

Results: **13 passed, 0 failed/ignored**. The actual quick-start binary prints
`profile-name: Ada`. Other executed cases cover four option methods, native open/dialect/run,
consuming session commit and rollback, repeated close/new-work rejection, a real native
failure and rollback-only state, read-only Config service registration and response lookup,
and setup/read-only application calls compared against the actual capability-host envelopes
for `abc`, `abc4` and `ABC`.

`run_cli.py` separately builds and runs the real `sql-profile-setup` and `sql-profile`
binaries against one private temporary file/config. Setup succeeds; CLI searches `abc`,
`abc4`, `ABC` return totals **3 / 1 / 0**, Grace Hopper for `abc4`, newline framing and exit 0.
Each binary run has a 10-second process timeout. `cli-walkthrough.json` retains the envelopes.
The full canonical hostile-prefix/HTTP/cancellation suites were read as source evidence,
not newly claimed as a full all-target test run in this docs task.

## Verification gates

Toolchain: rustc/Cargo 1.96.0, native aarch64-apple-darwin. All builds reuse the single target
root `/private/tmp/stackpress-docs-00007a-sqlite/target` with incremental output disabled.
Before the first build, cargo clean and cargo llvm-cov clean --workspace ran against that
fresh external target; no repository or other agent target was cleaned. The target is
deleted after verification; scripts, projected lock and logs remain outside it.

| Gate | Result / evidence |
| --- | --- |
| Guided-learning review | PASS: first visible result, explicit prerequisites, explained examples, task-specific follow-ups; tutorials/concepts separated from lookup. |
| API/copy-editing/formatting review | PASS: all public operations/fields/returns, native-vs-core boundaries, oriented headings, TOCs for long pages, tagged fences and consistent wording. |
| `python3 .../audit_api.py` | PASS: two exports, seven fields, five inherent methods, four implemented operations. `api-audit.json`. |
| `python3 .../verify_docs.py` | PASS: 8 pages, 97 relative links/anchors, one H1, tagged/closed fences, no empty parent headings/double blank lines, including core links. `mechanical.json`. |
| Consumer `cargo test --all-targets --offline --locked -- --nocapture` | PASS: 13 example tests, 0 failures/ignored; original quickstart compiles. `consumer-test-final.log`. |
| Consumer `cargo run --bin quickstart --offline --locked` | PASS: expected visible output. `quickstart.log`. |
| `python3 .../run_cli.py` | PASS: explicit install plus three real CLI searches. `cli-walkthrough.log/json`. |
| `CARGO_NET_OFFLINE=true cargo test --workspace --doc --locked` | PASS: **855 passed, 0 failed/ignored**, 23 result groups. `workspace-doc.log`. |
| `cargo fmt --all -- --check` and `git diff --check` | PASS. |
| Scope audit | No tracked source/test/manifest/lock diff. Only this lane's docs and receipt authored; concurrent core docs preserved. |
| Agent Workspace baseline validator | PASS, 0 errors / 21 pre-existing warnings. `kb-before.log`. |

The scanner checks links against real files and GitHub-style heading slugs after removing
fences. No source refactor, coverage increase, network fixture execution, platform expansion
or acceptance claim is inferred from docs checks. Coordinator review remains the next gate.

## DX observations, no fixes

1. `QueryResult::insert_id` is an optional observed rowid transition. Successful inserts can
   return None, including reused explicit IDs/WITHOUT ROWID tables. The docs direct callers
   needing every generated key to RETURNING; no field/API semantics changed.
2. Every transaction connection is retired, including acknowledged commit/rollback, and
   each statement refreshes schema/discards cached metadata. These correctness policies
   have connection/reprepare costs; no performance benchmark or optimization is claimed.
3. Awaited shutdown is application-owned. The ordinary main_with CLI retains RAII cleanup;
   there is no universal host async-finalization hook. The guide explicitly retains and
   closes the Engine before runtime teardown.
4. Budget tuning uses public fields rather than consuming setters. Options construction
   and deserialization can hold invalid settings until validate/open; the reference makes
   that timing and the read-only/create conflict explicit without changing the API.
5. Finite acquire/busy settings and retained-result caps are separate controls. There is no
   universal statement deadline/native-memory cap; cancellation of commit can lose its
   acknowledgement. Documentation preserves these boundaries instead of implying retries.

No commit, push, publication, deployment, KB promotion/indexing or agent messaging occurred.
No acceptance artifact applies. Stop for coordinator review after the approved receipt write.
