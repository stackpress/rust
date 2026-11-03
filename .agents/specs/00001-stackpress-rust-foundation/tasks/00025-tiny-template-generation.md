# Task 00025: Tiny template engine and source-emitter migration

Owner request, 2026-11-01: add the tiny template engine to `stackpress-lib` and
refactor Rust code-generation formatting to use it. The owner explicitly clarified
"Lib crate". This is local implementation, without commit, push or publication.

## Scope

The requested formatting scope is the production source-code emitters that assemble
Rust with `format!`, `push_str`, and related string writes. It includes schema,
SQL, admin, API generation where source is emitted, the transformer aggregate module
writer, and the generated-profile example generator. Ordinary diagnostics, runtime
formatting, SQL rendering, identifiers/paths, test fixture strings and unrelated
application UI rendering are not a blanket replacement target.

Implement the reusable engine in `crates/lib`, without a host, filesystem, runtime,
Rust language dependency or arbitrary expression execution. Rust-specific checked
names, types, literals and semantic interpretation remain generator responsibilities.
Templates must expose actual source structure, rather than merely wrap the existing
formatter output in a single substitution.

## Source authority and contracts

- User's active request and clarification authorize the engine and emitter migration.
- Original `../lib/src/Template.ts` provides variables, conditional/inverse blocks,
  loops and helpers as source reference, not a requirement to reproduce regex nesting
  bugs, arbitrary JS execution, truthiness surprises or silent missing substitutions.
- The preceding discussion requested clear missing-variable errors, reliable nesting,
  registered helpers and build-time use, with `rustfmt` and managed output retained.
- Existing Rust normalizers, generated public APIs, filenames, module ordering,
  stored metadata and transform-selection/ownership behavior remain binding.
- The task follows Rust programming, coordinated development and build benchmark
  workflows. No production contracts are promoted to context before owner approval.

## Milestones and ownership

1. Design checkpoint: engine syntax/API, limits, escaping, error handling, migration
   inventory and compatibility proof. Coordinator reviews before implementation.
2. Engine proof: parsed nested templates, clear typed failures, bounded rendering,
   literal braces and inert inserted values. Freeze the API for migration lanes.
3. Source migration: independent lanes with exclusive code ownership and no shared
   generated-file writes until coordinated regeneration.
4. Integration: check all generated roots, compare artifacts to the pre-change
   snapshot, and compile/run affected consumers.
5. Three distinct test-verified rounds (logic/complexity, responsibility/abstraction,
   consistency/Rustdoc), dedicated safety review, docs and coordinator closeout.

The coordinator owns this task, task status, shared lockfile and final verification.
The engine lane owns lib/root exports, then schema, transformer module emission and
the Profile example generator after M0 release. SQL and admin have independent
emitter/template/manifest/test ownership. Generated roots remain coordinator-owned.

Detailed migration inventories: [SQL emitters](00025-sql-inventory.md) and
[admin/source boundary](00025-admin-inventory.md). These distinguish structural
source templates from lexical/path formatting and record the compatibility corpus.
The [engine design](00025-template-design.md) freezes variables, boolean conditional
and inverse blocks, ordered loops, compiled helpers, strict typed errors, preserved
whitespace, configurable delimiters and finite render budgets. Coordinator reviewed
and released the lib M0 on 2026-11-01; migration waits for the working API proof.

## Verification process

Prove the engine's valid variables, repeated rendering, same-name nested conditions
and loops, scope lookup, empty/false/null cases, helpers and nonrecursive insertion.
Prove malformed/unclosed blocks, unknown helpers/paths, bad input types, invalid
delimiters and finite template/depth/work/output limits. Preserve whitespace needed
by source generation and reject unsupported expressions rather than evaluating them.

For source migration, compare normalized final artifacts and public output before
and after. An expanded baseline contains hashes of 190 tracked generated Rust/JSON files under
examples; coordinator keeps the temporary baseline path in session evidence.
Generation checks and absent/repeat/rename/disabled/hostile-input proofs also cover
output not present in that baseline. Do not hand-edit generated consumers.

Run relevant per-crate tests during iteration, then the workspace gate, doctests,
Clippy, Rustdoc, appropriate feature graphs, generation/compile proofs, affected
browser proofs and coverage. Keep SSR/hydrate/CSR separate. At least 70% development
coverage is required for every changed crate's measured scope; publication is not
requested and has a separate 90% policy.

Record all build/test/check/lint/doc/coverage/clean commands through the benchmark
recorder. Each lane uses a new task-specific target and the required initial clean
and llvm-cov clean there, then retains it warm under the 12 GiB cap. Preserve the
owner's existing targets and unrelated benchmark records. No listeners are needed.

## Acceptance criteria

Library/code-generation work: no visual acceptance controls. Completion requires
working template-based emitters throughout the declared source scope, generated
compatibility evidence, documentation, review rounds and coordinator verification.

## Implementation notes

Design and library M0 approved after independent verification. Schema/transformer/
Profile, SQL and admin lanes are released for M1 emitter implementation, with
exclusive file ownership. Builds wait for coordinated local dependency/lockfile
synchronization; generated roots remain coordinator-owned.

## Verification notes

2026-11-01: coordinator read current contracts and original Template.ts, inventoried
production emission sites and captured and expanded the 190-file generated artifact baseline.
Existing unrelated tracked changes are two older benchmark records; untracked
benchmark resources and `.DS_Store` files are outside scope.

Baseline generator checks passed without differences for generated-workspace,
generated-admin and generated-profile. Recorded in `target/template-review` after
scoped initial Cargo and coverage cleanup. Benchmark records:
`20261101T023143.857658Z-201e5dcefabb` (workspace),
`20261101T023225.316046Z-7fefa7131724` (admin), and
`20261101T023352.940831Z-3d08ca882d57` (Profile).
These prove the pre-migration corpus, not the pending implementation.

Coordinator M0 re-verification: 74 lib/facade all-target tests passed (five skipped)
in `20261101T024701.445371Z-aeee7ecb161e`; 273 doctests passed in
`20261101T024735.445756Z-af36ad61aef8`; strict Rustdoc passed in
`20261101T025117.440858Z-e0fa1ee14ce6`. Initial Clippy found the manual even-number
predicate (`20261101T024757.181854Z-86f17e9a9b72`); the idiomatic correction passed
Clippy in `20261101T025427.562485Z-adfe5a5ed680` and all 19 focused template proofs
in `20261101T025435.802398Z-7b131fd6b2dd`.

Source review covered UTF-8 scanning, bounded recursion/counters, borrowed data
inspection, strict lookup/types, lazy selected arms, helper snapshots, inert
insertions and facade exports. No unresolved source finding remains at this point.
Implementer whole-workspace gates passed 953 tests (85 skipped), 965 doctests,
Clippy, formatting and Rustdoc. The native lib coverage report records 4,160/4,426
lines (93.99%); the new template module subtotal is 703/714 (98.46%). The coordinator
read the report and checked its engine filenames against production-only scope.
M0 is approved; the detailed receipt and subsequent migration evidence remain
separately tracked in the [M0 proof receipt](00025-template-m0.md). Coverage report output initially failed for a missing parent
directory and was recovered from existing profiles without repeating the tests.
Initial relative target paths were rebased by trybuild into package-local targets;
future commands use absolute task target paths and account those owned nested
directories separately. No unrelated target is cleaned.

M1 dependency synchronization: reviewed codegen-only lib activation for schema,
SQL and admin, plus the transformer's direct build-host dependency. `Cargo.lock`
contains exactly three added local lib edges and no registry/version changes.
Full offline metadata wrote those edges, then failed on the preexisting uncached
`curve25519-dalek-derive 0.1.1` from the unfiltered dependency graph
(`20261101T030511.053533Z-d2a8bc6f8741`). No fetch was attempted. Locked/offline
workspace-manifest metadata then passed (`20261101T030525.276663Z-786afca14db2`).
Scoped lane builds are released with `--locked`; this does not claim every
unselected target dependency is cached or verified.

Early M1 compatibility review: generated-workspace check reported stale output
(`20261101T031046.028591Z-4e53e80f53f5`). Disposable generation and exact file
comparison localized the default corpus differences to three SQL store files:
one missing blank line per update function plus active-policy statement/macro
spacing retained by rustfmt around long literals. Findings sent to SQL lane;
no installed generated root was modified. Disposable generation records are
`20261101T031125.924651Z-4a702f20df18` (workspace) and
`20261101T031206.885532Z-09e42bceea5d` (admin). All default admin UI Rust files,
including pages and host registration, already match the baseline exactly.

The real-source admin size probe exposed template-limit causal classification
as a review concern. Preserve the direct emitter's `Limit` cause and established
outer registry `Callback` wrapper; the initial assertion targeted the wrong layer.
The near-limit normalized-JSON schema passes the actual writer/rustfmt with over
1,000,000 raw source bytes. No shared schema/registry error policy or writer cap
is changed; only local template adapters are being corrected and verified.

After SQL asset whitespace fixes, independent default workspace generation check
passed with no differences (`20261101T031309.359996Z-4fcfc594e4b5`). Independent
admin SSR/codegen/format tests passed 34/34, zero skipped, including the real source
boundary/error-chain proofs (`20261101T031624.838733Z-7c4846cad34e`). Admin lane also
reports exact SHA256 parity for eight formatted UI files in addition to the eight
default UI files; its full M1 coverage/lifecycle receipt remains pending.

Admin M1 approved: coordinator inspected its production coverage lower bound
2,359/2,781 (84.8256%), conservatively removing 85 physical inline-test lines from
the covered numerator while retaining the full denominator. Instrumented 34 tests
and generator lifecycle five tests passed. Three distinct verified refactoring
rounds, safety review and docs are released for the admin lane; shared snapshot
browser/consumer builds wait for stable schema source.

Coverage precision: lib M0's 93.99% is the raw file-filtered metric. Two inherited
inline test modules occupy 53 physical lines. Deducting all 53 from covered lines
gives a conservative production lower bound of 4,107/4,426 (92.79%), still above
the development floor. The new template module's 703/714 (98.46%) is unaffected.
This is an evidence clarification, without code/test changes or a gate reopening.

Independent SQL selected-feature tests passed 32/32, zero skipped, including
SQLite maintenance and real near-limit generator cases
(`20261101T032237.497321Z-1739a4f5d442`). SQL's first raw 71.01% report included
the test-only migration fault module; coordinator source audit caught this and
held its coverage approval. The lane is excluding that module/test hooks and
adding existing native generated-consumer suites; no production exclusions or
trivial tests are added to meet the floor.

SQL M1 approved after the corrected native-consumer coverage run: coordinator
audited all 24 SQL production-file rows, 4,995/5,803 lines (86.0762%). Deducting
all six test-only injection statements from the covered numerator gives the
conservative production lower bound 4,989/5,803 (85.9728%). The faults module is
absent. The lane reports 44 SQL/native-consumer tests passing; independent SQL
32-test verification and the old/new corpus/boundary comparison remain the
separate behavioral evidence. Three verified rounds, safety review and docs
are released within SQL ownership.

Independent schema/transformer/Profile M1 run
`20261101T033105.306003Z-989ff4a9131f` passed 127 tests and skipped seven; its
single failure was the new fixture selecting the raw maximum before rustfmt
expansion. The corrected test separately asserts formatted refusal, raw aggregate
refusal and an accepted large installed artifact, with no production changes.
It passed independently in `20261101T033214.827746Z-744f46cef05f`. The prior run
also passed final-format fingerprints, module ordering and the independent
Profile consumer compilation/removal proof. Schema and transformer production
source are stable for disposable admin consumer snapshots; coverage is pending.

Schema/transformer/Profile M1 approved: independently audited schema 3,189/3,829
(83.29%) and transformer 2,020/2,162 (93.43%) production-file coverage. Profile's
initial export omitted its `examples/` path; a report-only export with that default
exclusion disabled records 174/194 raw lines. Deducting its 24 physical inline-test
lines from covered lines gives a conservative 150/194 (77.32%) production lower
bound. The lane reports 128 instrumented tests passed, seven ignored, and three
disabled-Profile tests passed. Three verified rounds and safety review now cover
the engine as well as all three emitter crates; no product behavior is reopened.

Final integration found two standalone locks requiring the same local edges:
`crates/framework/framework/consumers/build-host/Cargo.lock` and
`examples/api-checkpoint/tool/Cargo.lock`. Eleven other relevant standalone native
locked metadata checks passed unchanged. The initial audit is
`20261101T035419.837187Z-08cfbe0c7ae6`; offline synchronization and locked rechecks
passed in `20261101T035453.283411Z-bb5206bdbfeb`. The build-host lock adds four
local lib edges (admin/schema/SQL/transformer); the API-tool lock adds three
(admin/schema/transformer). All package identities and registry
version/source/checksum tuples are identical before and after; no fetch occurred.

KB rename maintenance: replaced one historical task 00018 link to the renamed,
byte-identical host registrar asset, adding explicit old/new path provenance.
No earlier behavior/evidence changed. Validator returns the inherited eleven
missing screenshot-link errors and 32 warnings; the migration adds no broken
link. Agent File preferred-size warnings remain below the 500-line hard cap.

## Final coordinator verification

Implementation, all three rounds, safety review and documentation are complete.
The [integration receipt](00025-template-integration.md) records independent
workspace gates, per-crate production coverage, all eleven generation selections,
190-file byte integrity, standalone locks and the separate browser evidence.
Load it for the final task result and evidence limits. The engine/emitter rounds
are preserved in the [engine/schema/transformer/Profile receipt](00025-template-rounds.md)
and the SQL/admin inventories linked above. No material review finding remains.

Task 00025 finishes at `verified`: this library/generator task has no visual
acceptance criteria. Existing rendered admin proof cases verify compatibility,
without reopening product acceptance. No commit, publication or context promotion
was performed. Historical intermediate measurements and retries above remain
checkpoint evidence; the integration receipt identifies the final measurements.
