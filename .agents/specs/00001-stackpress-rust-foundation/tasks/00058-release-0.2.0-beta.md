# Task 00058: prepare 0.2.0 beta

Status: local implementation and required verification complete. Coordinator review and owner publication remain separate.
Acceptance criteria: none. Scope: versioning, locks, release tooling and user documentation; no runtime behavior change.
Worktree branch `release/0.2.0-beta`, starting commit `c8631d6`.

## Published baseline proof

Published from `d563f8d` (pre-email-rewrite), tree-identical to `25567fb` on main.
Both trees are `340953fc0691d09425b637b9c104faac4049c80b`; all release comparisons use
`25567fb`. Only the named candidate objects and this worktree were consulted.
Read-only downloads used `https://crates.io/api/v1/crates/<name>/0.1.0/download`
and User-Agent `stackpress-release-prep`, without credentials or personal details.

| Published archive | Files matching each candidate git archive | SHA-256 |
| --- | ---: | --- |
| stackpress-lib | 77 | b50764ee43ec30a95e503a7d3b716705afb156bb275efbda8a70489235b264ff |
| stackpress-leptos | 180 | d39f7373766eebdef1ac6612e9a529e679f1b3b7997ab87618e255426adcfbbe |
| stackpress-framework-email | 51 | e38ddc0f08d2e391c354227fa6b387ed02f0198c2d8677fb9b6a3615f315e32a |

Every source/asset/README/licence byte and `Cargo.toml.orig` matched. Cargo's normalized
`Cargo.toml` and generated lock are excluded from the source-byte comparison. Their
resolved package fields, 0.1.0 internal requirements and external registry subsets
were validated separately; `.cargo_vcs_info.json` supplies the published source identity.
All three VCS records name the full `d563f8d` commit and correct
crate path, distinguishing it from the earlier candidate `84010d7`, whose relevant
payloads are identical. [Download and comparison results](../../../resources/release-0.2.0-00058/baseline.json).

## Change classification since 25567fb

All 23 roots gain the G62 six-lint policy. The table classifies public signatures,
public-body changes and generated templates, including retained tests/fixtures as
supporting evidence. Private cache/layout changes do not alter a public constructor.
No removed feature, export, tightened bound or raised MSRV was found.

| Crate | Classification | Change |
| --- | --- | --- |
| stackpress-leptos | Breaking / Added / Changed / Fixed | Fallible EditPolicy::display; Language/SelectOptions defaults; editor draft/error ordering; completed render quiescence; invariant/paging/icon fixes. |
| stackpress-framework-language | Changed | No-panic lint policy only; no signature or supported-behavior change. |
| stackpress-sql | Changed | No-panic lint policy only; no signature or supported-behavior change. |
| stackpress-lib | Breaking / Changed / Fixed | Nest::entries returns Result; fallible internal serialization/mutation; structurally nonempty invocation ancestry; existing Request default constant exception. |
| stackpress-idea-parser | Changed | No-panic lint policy only; no parser signature change. |
| stackpress-sql-sqlite | Changed | No-panic lint policy only; no driver signature change. |
| stackpress-sql-postgres | Changed / Fixed | No driver signature change; no-panic policy; live-fixture CREATE-error cleanup in tests. |
| stackpress-sql-mysql | Changed / Fixed | No driver signature change; no-panic policy; live-fixture CREATE-error cleanup in tests. |
| stackpress-server | Changed / Fixed | Existing config Result paths now carry invariant failures; no signature change. |
| stackpress-idea-transformer | Changed / Fixed | Durable staged writes and journal sync; best-effort Unix parent sync; existing public error channels retained. |
| stackpress-server-http | Changed / Fixed | Typed rejection statuses; completed observed futures stay quiescent; supported HTTP status/framing behavior retained. |
| stackpress-cli | Changed / Fixed | Invalid internal completion action returns configuration error; no signature change. |
| stackpress-framework-view | Added / Changed | PublicLocation::session_asset; no-panic policy. |
| stackpress-framework-csrf | Changed | No-panic lint policy only; no signature change. |
| stackpress-framework-email | Changed / Fixed | Redirect-loop invariant removal; tested service/MIME constant exceptions; no signature change. TLS provider fix predates this baseline. |
| stackpress-idea | Changed | No-panic lint policy only; no facade declaration change. |
| stackpress-framework-schema | Changed / Fixed | Existing generator errors replace invariant panics; source column order retained; tested describe literals; private datetime cache preserves wire/debug output. |
| stackpress-framework-sql | Breaking / Changed / Fixed | SourceStore::create and NativeStore create/paths/selectors/joins return Result; relation/unique metadata errors; private migration dialect cache. |
| stackpress-framework-admin | Breaking / Added / Changed / Fixed | Upload.csv becomes CsvBytes; byte-array serialization; conversions/methods; RoleContext::from_draft; tolerant bounded encoding/delimiter import; rejected malformed children/catalogs; fallible formatted display. |
| stackpress-framework-session | Changed / Fixed | Updated entries callers; typed token/TOTP/record invariants; fixed conventional asset; no new signature; live-fixture CREATE-error cleanup. |
| stackpress-framework-api | Breaking / Changed / Fixed | import_csv changes from &str to impl AsRef<[u8]>; native raw-byte upload path retains JSON batches; entries callers; Language default; no other signature break; live-fixture cleanup. |
| stackpress-framework | Breaking / Changed / Fixed | Generated client definition changes SQL Result to lib Result; generated store errors propagated; language formatting invariant removal; live-fixture cleanup. |
| stackpress | Breaking / Changed | Re-exports the changed contracts; no new facade feature or direct export. Lockstep release and no-panic policy. |

The parameter/return and exhaustive public-field changes require the pre-1.0 minor
bump under [release versioning](../../../workflows/release-versioning.md): **0.2.0**,
not 0.1.1. Beta describes maturity; no semver prerelease suffix is used.
[Per-crate source review inventory](../../../resources/release-0.2.0-00058/classification.json).
G57 HTTP dates and the standalone email TLS provider fix are already in the published
baseline. The changelog's short 0.1.0 entry identifies them there. The live-fixture
leak fix, no-panic policy, durable writes and tolerant CSV import follow the baseline.

## Version, locks and documentation

- Workspace version and 114 existing internal version requirements across 18 manifests
  are 0.2.0. All 52 inherited workspace package versions follow the root version.
  Feature/default-feature/MSRV/licence settings are unchanged. The generated-profile
  consumer test's expected inherited package version is updated; no assertion is removed.
- Root `cargo tree --workspace --offline` changes only those workspace package versions.
  All 22 nested locks use a fresh root seed and one offline `cargo tree` resolution each,
  immediately asserting root byte identity and subset membership. Each registry set
  remains identical to its previous set; root stays at 699 tuples. Blog retains exactly
  six G58 extras; fuzz exactly the two G62 extras. [Lock evidence](../../../resources/release-0.2.0-00058/locks.json).
- `tools/release/common.VERSION`, dependency tests and packaging docs use 0.2.0;
  boundary controls use the 0.2 series. No managed source embeds the crate version,
  so no generated source rewrite is needed. Official freshness checks verify output.
- Added [changelog](../../../../CHANGELOG.md), [migration guide](../../../../docs/migrating-0.1-to-0.2.md),
  and all 23 README beta/support notices. Root install snippet uses `"0.2"` and links
  both guides. The migration guide includes before/after code for every identified
  break, including generated client errors, custom traits and JSON byte uploads.
- Updated [current publication runbook](00047-publication-plan.md#current-020-beta-runbook),
  preserving its historical audit. The owner still handles commits, push, login,
  uploads and tags. The existing publication script reads the root version correctly.

## Verification notes

The combined gate ran **once: 27/27 phases passed**, with `--allow-dirty` and the
specified installed Playwright module. Default nextest: **1,066 + 1 watchdog passed**,
90 genuinely ignored cases after the isolated watchdog run. Compatible: **1,304 + 1
watchdog passed**, 106 genuinely ignored cases. No test retry; only the package-version
expectation changed.
Both warning-denied Clippy/Rustdoc sets and doctests passed; native/WASM aggregate
and session graphs, all eight aggregate/four session consumers and gate generation
checks passed. [Combined phases](../../../resources/release-0.2.0-00058/gate-results.json).

All **23/23** candidates passed standalone all-target Clippy with `-D warnings`.
All **13/13** additional official managed-output freshness selections passed; the
remaining selections are checked by the gate/consumers. No managed version is embedded,
so no generated source edit was needed.
[Standalone Clippy](../../../resources/release-0.2.0-00058/standalone-clippy.json),
[additional freshness](../../../resources/release-0.2.0-00058/freshness.json).

README doctests: **12/12**, covering every release README containing Rust examples.
Release Python controls: **44/44**. Strict boundaries: **23/23**; registered copies:
**46/46**; licences: **707/707** declarations in 23 locks, 39 expressions and three
existing dated metadata fallbacks. Both workspace package passes and strict archive
checks passed for all 23 crates; all five leaf publication dry runs passed with uploads
aborted. Facade matrix: **81 builds / 87 advertised paths**, plus the lib-only default.
[Package results](../../../resources/release-0.2.0-00058/packaging-results.json),
[facade](../../../resources/release-0.2.0-00058/facade.json).

The blog's required nested gate passed native/browser/nickname/coverage/lock phases.
LLVM export reported 92 functions with mismatched data. The coverage report
passed its floors; this is not a warning-free export.
[Blog phases](../../../resources/release-0.2.0-00058/blog-results.json).
`./publish.sh --plan` passed with 23 `update` entries at 0.2.0 and zero planned waiting.
The coordinator explicitly authorized its read-only status requests.

KB validation: **146 inherited missing-link errors and 44 warnings**; no new release
receipt or runbook link error. This is not a clean KB validation claim. `git diff --check`
and final ownership/version/lock/public-link assertions passed. Historical 00050 reports
were restored after copying this run's reports to its own evidence folder; raw logs
remain under ignored `target/evidence-00058` and the shared recorder.

Review rounds: complexity/version propagation checked with the tooling controls;
responsibility/generated ownership checked with strict boundaries/copies and lock
assertions; consistency/docs checked against every changed public contract and local
links. No behavior-preserving Rust refactor was warranted in this version/docs task.

## Recorder and invariants

All Cargo, required verification and cleanup commands use sccache, incremental=0,
dev debug=0, jobs=2 and the shared recorder at `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`.
Task labels start `00058`. Main target: `target/00058-release`; gate consumers and blog
retain their official isolated subtargets in this worktree. Failed attempts, if any,
remain evidence. End-to-end command time and observed Cargo stage time are separate;
captured child output can make compilation totals incomplete.

Main target sampled peak: **11.843 GiB**, below 12 GiB.
Blog phase-sampled peak: **10.677 GiB**, also below 12 GiB.
Completed task-owned build outputs were retired at budget boundaries, retaining
libraries and metadata. Two auxiliary storage accounting attempts failed when Cargo replaced temporary files during `du`;
both helpers were corrected and successful retries retained the evidence. The release
gate itself had no failure.
[Target budget](../../../resources/release-0.2.0-00058/target-budget.json).

Recorder snapshot: **21 attempts**, statuses `{'passed': 19, 'failed': 2}`;
end-to-end command time **12829.940s**, observed Cargo stages
**7018.350s**; 18 records have no observed Cargo timing.
Monitoring overlaps the gate, so these sums are neither elapsed time nor complete
compilation cost. No successful cold-workspace rebuild count is inferred from unknown
cache labels. [All attempt metadata](../../../resources/release-0.2.0-00058/benchmark-records.json).

Production Rust and all managed source bytes, licences, notices, goldens, oracles and
`.stderr` files are unchanged. [Invariant audit](../../../resources/release-0.2.0-00058/invariants.json).
Full live-inclusive release coverage is not repeated for this version-only task;
[00057](00057-hardening-integration.md) records its earlier passing measurement.
No install, development server, commit, push, login, upload or tag was performed.

## Coordinator verification

Committed as `f7e0f3a` and merged into main as `b817f05`. One coordinator edit: the changelog intro now says "beta release: the API may still change before 1.0" instead of an internal note. Reviewed: version, lock, docs and test-version changes only; no behaviour or managed-output change.

Independent checks on main after the merge (2026-11-12):
- 699 registry tuples identical; `cargo tree --workspace --locked --offline` consistent;
- no release-crate requirement left at 0.1.0. The three `examples/api-checkpoint/*` own package versions are unpublished standalone examples;
- `check_dep_licenses.py`: 707 packages, 23 locks, 0 failures;
- strict `check_test_boundaries.py` and `check_packages.py` pass;
- `./publish.sh --plan`: 23 new versions of existing crates, no rate-limit waiting.

The worker's single gate (27/27, 23 standalone Clippy, README doctests) is recorded above. Publication is the owner's step.
