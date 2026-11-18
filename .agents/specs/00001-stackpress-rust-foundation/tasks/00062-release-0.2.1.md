# Task 00062: prepare 0.2.1 beta

Status: implemented; required local verification complete. Coordinator review and owner publication remain separate.
Acceptance criteria: none. Scope: versioning, locks, release tooling and docs only.
Worktree `release/0.2.1`, starting HEAD `5436c94`; only this checkout and its ancestry consulted.

## Published baseline and semver

All three published 0.2.0 archives identify **`34be3146ffc3279595595b665bf371205744a3a2`**
in `.cargo_vcs_info.json`, with the expected crate paths and no dirty marker. Downloads
used `https://crates.io/api/v1/crates/<name>/0.2.0/download` and User-Agent
`stackpress-release-prep`, without credentials or personal details. The owner reports
publication of all 23 crates on 2026-11-12. Direct downloads initially hit sandbox DNS
restriction; the authorized read-only retry succeeded.

| Archive | Source files matching `git archive` | SHA-256 |
| --- | ---: | --- |
| stackpress-lib | 77 | bfa9ac8af94c33162b8b27516043c1d090bd0edaab07196770001c42a3480d9e |
| stackpress-server | 64 | 593ae0777f54ef1a308863c14a9c31c10457ca90bc9e5d2d2b904eb23e39bcf3 |
| stackpress-leptos | 180 | 24889e41111b11a5a4933eba40bb8e3a951d4f4c4512d173557ad6f910641830 |

All source/assets/README/licence bytes and `Cargo.toml.orig` match. Cargo's normalized
manifest and generated lock were checked separately: package version/edition/MSRV
match; external registry tuples are subsets of the baseline root. The server archive's
published `stackpress-lib 0.2.0` registry edge has the downloaded library's checksum.
An initial comparison incorrectly treated that legitimate internal registry edge as an
external addition; the corrected assertion checks internal and external edges separately.
[Baseline proof](../../../resources/release-0.2.1-00062/baseline.json).

Reviewed every changed Rust/public declaration, public body and manifest in all 23
candidates against that commit, including generated source/templates. Unchanged rows
below have no source/template diff. Public additions do not alter existing signatures,
required trait methods, exhaustive public fields/enums or bounds. Private fields were
added only to already opaque types; existing feature mappings/defaults and Rust 1.96
remain unchanged. [Per-crate inventory](../../../resources/release-0.2.1-00062/classification-inventory.json).

| Crate | Classification | Review |
| --- | --- | --- |
| stackpress-leptos | Additive | Optional realtime module, EventSource/WebSocket options, messages, handles, state/error/result types and helpers. Existing render features/APIs unchanged. |
| stackpress-framework-language | Unchanged | No source/template/public change. |
| stackpress-sql | Unchanged | No source/template/public change. |
| stackpress-lib | Unchanged | No source/template/public change. |
| stackpress-idea-parser | Unchanged | No source/template/public change. |
| stackpress-sql-sqlite | Unchanged | No source/template/public change. |
| stackpress-sql-postgres | Unchanged | No source/template/public change. |
| stackpress-sql-mysql | Unchanged | No source/template/public change. |
| stackpress-server | Additive | SSE events/options/limits/driver/channel/response extension and cursor helper; sibling native-output types and adapter methods; `HttpBody::empty`, optional SSE inspection and preflight hook. Existing `handle_body` signature, `HttpResponseExt`, output enums and reply fields preserved. |
| stackpress-idea-transformer | Unchanged | No source/template/public change. |
| stackpress-server-http | Additive / Changed behaviour / Fix | Optional bounded SSE/WS hosting, host limits/statistics/builders, `bind_prepared`, WS options/extension/owned connection/message/close/snapshot and Origin helper. Enabled valid WS endpoints return 101 instead of 400; feature-off rejection remains 400. Unsupported WS versions return 426 and malformed handshakes reject before handlers. Early flush avoids unnecessary supervisor wakes and timeout starvation. |
| stackpress-cli | Unchanged | No source/template/public change. |
| stackpress-framework-view | Unchanged | No source/template/public change. |
| stackpress-framework-csrf | Unchanged | No source/template/public change. |
| stackpress-framework-email | Unchanged | No source/template/public change. |
| stackpress-idea | Unchanged | No source/template/public change. |
| stackpress-framework-schema | Unchanged | Generator/templates unchanged. |
| stackpress-framework-sql | Unchanged | Generator/templates unchanged. |
| stackpress-framework-admin | Unchanged | Generator/templates unchanged. |
| stackpress-framework-session | Unchanged | Generator/templates unchanged. |
| stackpress-framework-api | Unchanged | Generator/templates unchanged. |
| stackpress-framework | Unchanged | Generator/templates unchanged. |
| stackpress | Additive | Five opt-in forwarding features; existing exports/default remain unchanged. |

**No breaking change found: 0.2.1 is appropriate** under the binding
[release workflow](../../../workflows/release-versioning.md). G63's explicit endpoint
opt-in, separate finite session clocks, graceful stop and application-owned authorization
are documented behaviour. No new migration guide is needed; the existing 0.1-to-0.2
guide remains valid. No security fix is separately claimed.

## Applied changes and locks

- Workspace version plus all 114 internal version requirements in 18 manifests use
  0.2.1. All 52 workspace package versions inherit it. Release constants, dependency
  controls and the generated-profile consumer's version expectation are updated.
- Root offline `cargo tree` changes only workspace package versions; all **706**
  registry tuples and tracked `registry-baseline.json` bytes are unchanged. All **22**
  nested locks received one root-seeded offline refresh with immediate root-byte and
  subset assertions. Blog is a pure root subset; fuzz retains only the two existing
  G62 extras. [Lock proof](../../../resources/release-0.2.1-00062/locks.json).
- No managed output embeds the package version and no generator/template changed.
  Official freshness checks are part of verification; no generated source was edited.
- Changelog now records 0.2.1 beta dated 2026-11-18, realtime/browser additions,
  observable handshake/clock/stop behaviour and the HTTP flush wake-up fix. Root and
  relevant crate READMEs already describe SSE/WS; root current-version text is updated
  and `"0.2"` installation snippets remain valid. Current KB version references,
  publication runbook and task index are updated; historical decisions/receipts remain.
- Licence texts/notices, production Rust, goldens/oracles and `.stderr` stay unchanged.
  The only Rust edit is the required consumer package-version expectation.

## Verification and handoff

- **Combined release gate: 30/30**, run once with `python3 tools/release/gate.py --allow-dirty`
  and the requested cached `BLOG_PLAYWRIGHT`. Default nextest: 1,066 passed plus the
  separately selected TCP watchdog; compatible nextest: 1,335 plus its watchdog.
  The main runs skipped 91/110 respectively, including those separately run watchdogs;
  the remaining 90/109 skips are not claimed as passes. Default/compatible Clippy,
  doctests, Rustdoc, graphs, consumers and realtime browser checks all passed.
  [Gate results](../../../resources/release-0.2.1-00062/gate-results.json).
- **23/23 standalone Clippy** checks with `--all-targets --locked --offline -- -D warnings`,
  and **13/13 additional official freshness** checks passed.
  [Clippy](../../../resources/release-0.2.1-00062/standalone-clippy.json),
  [freshness](../../../resources/release-0.2.1-00062/freshness.json).
- **23 package archives**, **five leaf publish dry runs**, **44 release-control tests**,
  **12 README doctests**, **86 facade builds / 87 advertised paths**, and the default
  facade dependency tree passed. [Packaging](../../../resources/release-0.2.1-00062/packaging-results.json),
  [archive inventory](../../../resources/release-0.2.1-00062/packages.json),
  [facade](../../../resources/release-0.2.1-00062/facade.json).
- Strict boundaries: **23 candidates**, copies: **46 files / two manifests**, licences:
  **708 registry packages / 23 locks**, **39 expressions**, **two existing dated metadata
  fallbacks**, zero failures. [Strict checks](../../../resources/release-0.2.1-00062/auxiliary.json).
- Blog: **27/27 phases**, including 92 native tests, 92 instrumented tests and browser
  proofs. Its normal coverage gate passed (70.68–97.44% across measured blog crates;
  browser startup glue remains G56-exempt). LLVM reported **92 functions with mismatched
  data**; this is not a warning-free result. [Blog phases](../../../resources/release-0.2.1-00062/blog-results.json),
  [coverage](../../../resources/release-0.2.1-00062/blog-coverage.json).
  Live-inclusive release-crate coverage was not repeated, as explicitly directed for
  this version/docs task; task [00061](00061-sse-websocket.md) owns the prior evidence.
- `./publish.sh --plan`: **23 updates at 0.2.1**, no brand-new names and **zero planned
  waiting**. Only the read-only plan was invoked. [Plan](../../../resources/release-0.2.1-00062/publish-plan.json).
- Final version/source/lock invariants and `git diff --check` passed; all 22 nested
  lock hashes still match their refresh receipts. The four gate-owned historical JSON
  reports were preserved under this task then restored to their starting bytes.
  [Invariants](../../../resources/release-0.2.1-00062/invariants.json).
- Canonical KB validation retains **146 inherited errors / 44 warnings**, with **no errors
  in changed or new files**. It is not a clean KB validation; historical missing links
  were outside this release scope. [KB result](../../../resources/release-0.2.1-00062/kb-validation.json).

Three bounded review rounds covered (1) version propagation and complexity, backed
by release controls and the gate; (2) responsibility, source boundaries and root/nested
lock ownership, backed by strict checks and structural assertions; and (3) consistency,
README/changelog/KB wording and generated output, backed by freshness, documentation
tests and final invariants. No production refactoring was warranted in this version-only
scope; no runtime behavior changed during preparation.

All Cargo/proof commands used sccache, incremental=0, dev debug=0, jobs=2 and the
benchmark recorder at `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`.
The one combined gate took **6,204.73 command seconds**; its summed observed Cargo
Finished durations were **6,384.09 seconds**, which may overlap/nest and are not elapsed
time or CPU time. [All benchmark records and timings](../../../resources/release-0.2.1-00062/benchmark-records.json).

Sampled peak target trees: **native 13.007 GiB**, browser-build **1.020 GiB**, blog
**10.683 GiB**. The native tree transiently exceeded its **12 GiB cap** during the final
facade matrix. Completed independent consumer, trybuild, API/README and facade targets
and completed native test executables were retired; active build targets, main libraries
and metadata, and package archives were preserved. No concurrent `cargo clean` was used.
This is a recorded disk-budget deviation, despite all checks passing.
[Budget and current sizes](../../../resources/release-0.2.1-00062/target-budget.json),
[samples](../../../resources/release-0.2.1-00062/footprint.json),
[overrun cleanup](../../../resources/release-0.2.1-00062/retired-trybuild.json).

Raw gate logs remain gitignored and untracked; the shared benchmark log preserves
the combined output. Failed preliminary baseline attempts remain described above.
No install, development listener, commit, push, login, upload or tag was performed.
The coordinator reviews this local receipt; the owner decides publication.

## Coordinator verification

Committed as `a109a76` and merged into main as `2cc27c6` (2026-11-18). The raw gate logs stayed ignored and were not staged.

Independent check on a fresh clone of main:
- the root lock still has exactly 706 registry tuples, and `registry-baseline.json` is unchanged from `5436c94`;
- all 44 workspace lock entries are at 0.2.1, and no manifest still requires 0.2.0;
- `cargo tree --locked --offline` for the all-features facade resolves;
- the dependency-licence check covers 708 packages in 23 locks with 0 failures;
- the test-boundary check (23 candidates) and test-copy check (46 copies) pass, with 0 errors;
- strict `check_packages` passes;
- `check_facade` passes: 86 builds and 87 advertised paths;
- `check_readmes` passes;
- `cargo check` passes for the facade with the SSE, WebSocket and Leptos realtime features;
- `./publish.sh --plan` lists 23 updates at 0.2.1, with no rate-limit waiting.

Release-crate coverage was not rerun for this version-only change; task 00061's live-inclusive run (all non-facade crates at least 90%) stands. The full gate, standalone Clippy and freshness results are the worker's run above. Publishing and tagging stay with the owner.
