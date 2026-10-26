# Slice 6 risks, milestones and verification

Companion to [decisions](slice-6-design-decisions.md). Load for the implementer's risk inventory, proving tests and about two implementation lanes. This checkpoint delivers specifications and one disposable mechanism probe only; the following checks are plans, not passed production evidence.

## Open-ended risk inventory

Each risk needs an observable failure/control, not merely line coverage or SQL snapshots. Add new material findings as they arise. Existing foundation lifecycle, aliasing, poison and cancellation regressions remain mandatory.

| Risk | Prevention | Required proof |
| --- | --- | --- |
| Unbounded recursive metadata/embedded types | Validate direct config as well as parsed inputs; hard depth/node/bytes/collection limits, reject embedded value cycles. | Deep/wide/one-over/manual config tests in child process, finite diagnostic, no stack overflow during clone/drop/encode. |
| Reference strings mistaken for references | Existing typed reference-aware finalization before semantic normalization. | Imported actual references resolve while literal `${...}` values survive unchanged; no reparsing. |
| Global dictionary contamination | Per-tool frozen registry, typed IDs/versions, immutable descriptors. | Concurrent different registries/schema clones independent; duplicate definitions reject before callback/IO. |
| Rust/name/path injection or collisions | Checked naming, literal escaping, portable case/keyword/length/alias policy. | Compile keyword/Unicode/quote fixtures; reject colliding normalized names and traversal before output changes. |
| Unsupported metadata silently accepted | Consumer ownership and exact supported matrix; error on unimplemented selected semantics. | Misspelled is.*, crypto, missing default provider, unsupported type/role cases fail generation with unchanged output. |
| Missing/null/default or numeric/date loss | Tri-state inputs/patch, finite/exact values, canonical pure codecs. | Missing/null/empty/false/zero, i64 and browser-safe bounds, invalid/nonfinite dates/numbers, JSON NULL distinctions; storage↔wire roundtrip. |
| Validation-only uniqueness race | Database constraints and atomic conflict operations; typed error categories. | Concurrent creates/upserts same unique key; one intended row, safe conflict classification, no text matching. |
| Untrusted validation/disclosure | Bounded errors/projections; no rejected values/SQL/secret config in public output. | Secret sentinels in fields, driver causes, paths/env/config absent from public DTO/HTML; concrete causes privately downcastable. |
| Private fields leaked through relations or selectors | Explicit exposure projection propagated at every include node. | Denied hidden field/filter/sort/lookup paths fail before IO; approved DTO contains only selected fields. |
| UI permissions mistaken for host authorization | App policy at exposed handler; direct stores documented trusted. | Allowed/denied same inputs via CLI/direct exposed operation/HTTP; UI field visibility alone cannot authorize. |
| Wrong pool executor inside transaction | DataSession wraps exact &mut Transaction, no captured Engine. | Two generated stores across await see same uncommitted writes; outside engine doesn't, rollback removes both. Compile-fail Clone/alias/escape/non-Send and service registration of session. |
| Nested dispatch deadlock | No ambient transaction; finite pool/busy budgets, release before delegation. | Size-one nested reacquire fails finitely and recovers; explicit same-session domain composition succeeds. Outer watchdog catches stuck negative control. |
| Session failure/cancellation hides effect | Existing rollback-only/retirement policy; consuming completion, no retries. | Barrier-controlled cancel begin/query/commit; next connection clean, ledger reconciliation handles unknown commit, no duplicate replay. |
| N+1 or join fanout breaks pagination | Bounded explicit batched includes, base-first page, typed key attachment. | 1 versus 20 parents query-count bound, two levels, bind chunk boundary; correct base order/count/pages without duplicate rows. |
| Cyclic/missing/ambiguous relations | Validated finite graph and explicit owning/reverse mapping. | Self-cycle depth limits, multiple inverses, missing/nullable FK, nonunique target, compound unsupported mapping; typed errors not recursion/null guessing. |
| Count and rows falsely called consistent | Explicit read transaction and adapter isolation capability. | SQLite concurrent writer snapshot fixture; network READ COMMITTED doesn't pass repeatable-read claim. Skip/refuse unsupported consistent mode. |
| Output/feature/generator drift | G20 ownership/check plus fingerprints/contract versions and strict selected inputs. | Clean→repeat byte identity, edited owned file conflict, rename/remove/prune+compile, disabled plugin and old contract mismatch; sentinel files intact. |
| Generated app needed to run generator | Independent app tool with static manifests. | Delete generated fixture output, build tool alone, generate then compile app; dependency graph no back-edge. |
| Contracts pull native/tool crates to wasm | Feature/package separation and actual target graph. | Native contracts-only build and real Slice 8 wasm consumer; trees exclude server/CLI/Idea/fs/SQLx/Tauri. Record unrun target limits. |
| History mistaken for applied state | Immutable artifacts plus authoritative DB ledger and locked recheck. | Multiple pending revisions, older applied head, empty/no ledger, missing parent/edit/hash/format/branch mismatch; no accidental install/drop. |
| Concurrent migrators race | Same dedicated DB writer transaction/lock, finite timeout, check after lock. | Two child processes barrier-start same plan; one application plus AlreadyApplied/stale refusal, rows/head correct. |
| Destructive/ambiguous plan bypass | Exact plan hash+data-loss option, explicit rename/backfill resolution. | Wrong hash/generic force/refusal unchanged state; approved loss executes only listed objects; unsafe conversion still errors. |
| SQLite inbound cascade destroys children | FK enforcement off before BEGIN on owned connection, checked reconstruction. | Parent rebuild with inbound CASCADE/RESTRICT/self/cyclic references retains rows; foreign_key_check and actual later cascade behavior correct. |
| Reconstruction loses objects/sequence/data | Bounded catalog allowlist, mapped columns, complete constraints/indexes, sequence policy. | Rows/IDs/defaults/NULL/index/unique/FK and high-water mark survive forward/reverse; unknown trigger/view/custom object refuses unchanged. |
| Partial reconstruction or ledger write | One SQLite transaction, validate all plans first, ledger/head in same commit. | Fault at each create/copy/drop/rename/index/check/ledger/commit transition; rollback leaves exact original schema/data/head or explicit unknown commit reconciled. |
| FK policy leaks into pool | Maintenance-only connection, typed transitions, retire on error/cancel, awaited close. | Before/after/cancel policy readback; ordinary later connection foreign_keys=1; raw PRAGMA still rejected. |
| Fake reversible migration | Explicit reverse metadata/preconditions and irreversible reason. | Safe rename/index and applicable nullable-add reverse; data-bearing drop/lossy conversion refuses; backup/custom reverse distinguished from recovery guarantee. |
| Unknown live schema or unowned data altered | Catalog/ledger signature checks, namespace ownership, no automatic adoption. | Existing unmanaged table/constraint/ledger collision and out-of-band drift refuse; unrelated objects unchanged. |
| Seed partial work/side effects | Typed bounded DB seed steps on one session, explicit modes/idempotency. | Failure at second model rolls back first; repeat stable seed doesn't duplicate; arbitrary event/external side effect not accepted as atomic seed. |
| MySQL transaction overclaim | Reject DDL apply until separate recovery protocol. | Pure render works; atomic runner rejects before effects. Network CRUD tests don't count as migration application. |
| Error wrapping duplicates or loses cause | Context-only Display, concrete source, propagate existing error. | Multi-layer schema→transform→host and SQL→DataError→host chains print cause once and retain original type/frames/public policy. |
| Resource/worker leak or blocking async executor | Finite aggregate budgets, adapter worker, awaited tool shutdown/cleanup. | Current-thread responsiveness, bound+1 rows/bytes/statements/includes/seeds; child exit/reopen under watchdog, no detached successful work. |
| Panic/poison/unsafe workaround | Owned state, no global store lock, unsafe forbid; unwind is not success. | Panicking typed plugin/seed child fails with no successful receipt/committed partial state; any added mutex gets explicit poison handling and no guard across await/callback. |
| Misleading support/coverage evidence | Scope-specific receipts and real generated execution. | Separate native/wasm/network/render/probe results; skipped G26 tests reported skipped; per-crate production-line coverage and critical branch audit. |

## M0: prove the riskiest contracts, then stop

Only after owner choices and shared-change approval. Coordinator gives one integration owner exclusive shared files; no broad parallel edits begin until signatures/partitions freeze.

1. Prove real framework Schema→three-model Rust→consumer compilation from absent output; re-run generation byte-for-byte. Compile contracts without native edges. Prove named Send handler over current service API and DataSession transaction borrowing with actual generated store calls and compile-fail alias/Clone/escape cases.
2. Implement the smallest real SQLite maintenance owner/session. On independent temporary files, reproduce the probe using typed production queries, ledger/head, parent/child FK and one failed copy. Prove readback before BEGIN, cancellation/retirement, one writer and full rollback; deliberately stuck negative control proves the watchdog.
3. Plan/apply/reverse one explicit safe rename and one native addition, compare rows/schema/ledger; reject ambiguous mapping, invalid backfill and stale hash. Simulate lost commit acknowledgement and reconcile applied state without replay.
4. Re-run with final proposed public names/feature graph, document raw-policy gate unchanged and actual limitations. If reconstruction, Send ownership or ledger guarantees cannot be achieved soundly, stop with counterexample and a revised choice; do not hide it in raw SQL/unsafe or weaken the stated contract.

M0 completion is not full generated functionality. **Stop for coordinator re-verification** before M1. Source-only design and the present direct-SQLx probe do not pass this production gate.

## About two implementation lanes after M0

| Lane | Exclusive proposed files | Responsibilities/dependencies |
| --- | --- | --- |
| A schema/generation | `crates/framework/framework-schema/**`; new example tool generator modules/contracts tests; unique lane-A receipt | Normalization/type/rule/role descriptors, pure contracts/value codecs and registered schema generation. Own descriptor version after M0; changes go through coordinator for Slice 8. |
| B data/migrations | `crates/framework/framework-sql/**`; new example app/stores/operation/maintenance tests and tool DB modules; unique lane-B receipt | Runtime stores/relations/handlers, SQL generation, history/planner/ledger/maintenance orchestration. Consumes frozen schema; no edits to Idea/SQL/SQLite/shared tool entrypoints independently. |

Coordinator assigns one integration owner for the new example tool manifest/lib/main/selection and root manifest/lock/facade; it may be either implementer. The coordinator separately sequences shared Idea/SQL/SQLite changes with Slice 7 and other active lanes. This design creates/messages no agent and owns no slice-7/slice-8 design files. Total active implementers remains within G23's cap; two lanes describes logical ownership, not a demand to exceed available slots.

| Gate | Deliverable and required signal |
| --- | --- |
| M1 schema + generation | Complete supported type/default/rule/name/role matrix, exact unsupported diagnostics, contract fingerprint and public projection; clean/repeat/remove/rename checks, no bootstrap cycle. Freeze descriptor/DTO/query contracts for Slice 8 through coordinator. |
| M2 CRUD + relations | Every supported store/action/filter/sort/page/soft-delete/operation; two-store atomic transaction, real N+1 and row/count tests, direct/CLI/buffered HTTP parity, strict policy/disclosure. Resolve/gate upsert core extension. |
| M3 revisions + migrations | Complete immutable history/artifacts/NoChanges/rename/loss/reverse/ledger/drift, native and reconstruction matrix, lock/cancel/fault/unknown-commit tests, explicit install/purge/uninstall/populate. PostgreSQL/MySQL plan snapshots; SQLite execution is required. |
| M4 integrated DX | Generate new example from Idea to installed/seeded SQLite, query three models/relations, change schema and upgrade/reverse safely, owner-readable commands/SQL/Rust; inventory reconciliation, features/dependency trees and per-crate coverage. Network CRUD contract suite integrated only when Slice 7 adapters available. |
| Before rounds | **Stop** with exact results, per-crate coverage, unresolved critical branches and any API/dependency/scope deviations; coordinator independently re-runs. |
| Rounds 1–4 | Test-verified complexity/branch-risk; responsibility/abstraction; consistency/Rustdoc/comments; dedicated safety/soundness. Repeat targeted rounds for material findings; justified no-change rounds acceptable. |
| Closeout/docs | Scoped receipts plus coordinator verification; separate released crate-docs follow-up using chrisai-docing, real compiled examples, full source/output readability review. No publication/commit/deploy implied. |

Milestone dependencies are real: M2 uses M1 semantics, M3 uses frozen storage snapshots; independent preparation can run concurrently but not claim completion with handwritten substitute contracts. Re-run M0 proofs after replacing any scaffolding with final production types.

## Execution and coverage plan

Native tests use independent tempfile-backed databases and explicit setup/close; no shared fixture filenames or important owner data. Parent processes enforce monotonic 5–10 second deadlines on concurrency/deadlock/poison/cancellation/panic/cleanup tests using ready/go barriers, kill+reap on timeout and retained stderr. Compile/build subprocesses use separately justified finite deadlines. A Tokio timeout inside a blocked executor is not a sufficient watchdog. No sleep-based race proof or ignored critical path.

G26 PostgreSQL/MySQL runs are opt-in through STACKPRESS_TEST_POSTGRES_URL / STACKPRESS_TEST_MYSQL_URL. Create a unique throwaway database per run and drop only that exact owned database; never enumerate/read/alter other databases/roles/settings. Reuse Slice 7's harness through coordinator. Without URLs/services/adapters, report skipped/incomplete, not passed. No network database is needed to complete required SQLite milestones. No local listener, toolchain/package install or important database use follows from this plan.

Before a fresh implementation build, coordinate cargo clean/llvm-cov clean so no other lane's target is removed; use fresh isolated target roots where appropriate. The checkpoint probe had an empty external target and did not clean shared output. Never use another trial branch's artifacts.

After production implementation, relevant commands include:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo run -p generated-workspace-tool -- emit generate --check
python3 .agents/scripts/validate-agent-workspace.py
```

Also run separate valid schema default/codegen, SQL default/server/codegen/migrations/sqlite-maintenance, tool database-tools and facade combinations; do not use an invalid all-features blend to prove browser isolation. Run actual consumer wasm builds with Slice 8, not native-only substitutes. Native renderer snapshots alone establish neither a live network adapter nor native/mobile packaging.

Development floor is **70% executable lines in each changed first-party crate/valid measured scope**, including generated shipped code, new tool/app and shared edits; publication is 90% separately. Record compiler/target/tool/features/numerator/denominator, production versus inline-test treatment and exclusions. No workspace-average or cargo-llvm-cov example exclusions conceal generated code. Audit unmeasured critical cancellation/DDL/loss/disclosure branches even above threshold. This design checkpoint measures no production coverage and runs no workspace Cargo gate.

## Design packet closeout checklist

Coverage inventory for the complete draft: owner choices; source exports/types/test dispositions; concrete Idea→Rust/store/operation/schema-change flow; generated ownership/bootstrap; all task risks and tests; revisions/ledger/reconstruction/operator safety; dependencies on both parallel slices; shared requests/dependencies; M0/two lanes/milestones/rounds/docs/coverage. Preserve all sections when splitting into the routed files; each final file is under about 250 lines.

Run the Agent Workspace validator after the approved spec writes, record actual outcome and separate any unchanged KB warnings from new-file findings. The task file/index/status/decision ledger are coordinator-owned, so return the router path for coordinator linkage rather than edit them. No context promotion is proposed. Acceptance criteria remain none for this library design; visual UI/admin acceptance belongs to Slice 8. **Stop after design and report the owner choices.**
