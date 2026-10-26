# Task 00007A SQL core documentation receipt

Implementation notes for the core part of [task 00007A](00007A-sql-docs.md). Load for the
page inventory, public-surface coverage, executable examples, author gates, and DX observations.
Coordinator review and task verification remain pending; no acceptance or publication is claimed.

## Scope and source authority

- Date: 2026-10-26. Checkout: `/Users/cblanquera/server/projects/stackpress/rust`.
- Branch `trial/3`; starting and handoff HEAD `4100eab`. Initial checkout was clean.
- This lane authored only `crates/sql/sql/docs/**` and this separately approved receipt.
  Concurrent `crates/sql/sql-sqlite/docs/**` and its receipt belong to the SQLite implementer.
- Read root/context instructions, the coordinated docs stage, installed chrisai-docing guidance,
  Server/Idea documentation precedents, and task 00006A receipts. Followed guided learning,
  API reference, copy editing, then formatting; no programming refactor rounds apply to prose-only work.
- Ground truth: current core public source, tests, and four runnable examples; decisions
  [G24 and G26](../decisions.md); Slice 4 design API/execution/inventory and the lane-A
  [M1](../slice-4-receipt-lane-a-m1.md), [M2](../slice-4-receipt-lane-a-m2.md),
  [M3](../slice-4-receipt-lane-a-m3.md), and [rounds](../slice-4-receipt-lane-a-rounds.md) receipts.
  Live implementation/tests take precedence over proposal sketches and stale Rustdoc notes.
- Source, tests, manifests, and repository lockfile are unchanged: 303 before/after file hashes
  match. No other trial branch, commit, push, listener, deployment, publication, or indexing used.

## Page inventory and reader flow

Created **32 pages / 2,361 lines** under the
[SQL core docs](../../../../crates/sql/sql/docs/README.md): entry, first-success walkthrough,
concepts, seven intent guides, API index, and 21 public-surface reference pages.

The quick start builds/renders/inspects one SELECT without a database or runtime and prints SQL
plus a bind count. Concepts covers every requested topic: owned builders, capability matrix,
binds/identifiers, strict values, rows/decoding, Engine/transaction ownership, execute_plan,
explicit schema diff, TrustedSql, and per-statement/structural limits. Guides separate tasks from
lookup. Larger runnable examples are linked rather than duplicated. Native SQLite lifecycle,
error disclosure, execution outcomes, and server-service ownership route to the adapter docs;
all cross-lane links and referenced anchors resolve.

| Page relative to core docs | Lines |
| --- | ---: |
| `README.md` | 51 |
| `api/README.md` | 53 |
| `api/alter.md` | 96 |
| `api/create.md` | 81 |
| `api/database.md` | 72 |
| `api/delete.md` | 50 |
| `api/dialects.md` | 55 |
| `api/engine.md` | 96 |
| `api/errors.md` | 68 |
| `api/expressions.md` | 84 |
| `api/identifiers.md` | 55 |
| `api/insert.md` | 73 |
| `api/json.md` | 73 |
| `api/query-plan.md` | 50 |
| `api/query.md` | 100 |
| `api/rows.md` | 106 |
| `api/schema-diff.md` | 80 |
| `api/schema.md` | 117 |
| `api/select.md` | 97 |
| `api/table-command.md` | 66 |
| `api/transaction.md` | 70 |
| `api/update.md` | 71 |
| `api/values.md` | 65 |
| `concepts.md` | 148 |
| `getting-started.md` | 72 |
| `guides/build-queries.md` | 57 |
| `guides/decode-rows.md` | 58 |
| `guides/errors-and-limits.md` | 69 |
| `guides/query-json.md` | 58 |
| `guides/schema-changes.md` | 55 |
| `guides/transactions.md` | 61 |
| `guides/write-rows.md` | 54 |

## Public-item coverage

The source declaration/name audit and manual shape review account for **all 60 exported types,
traits, and aliases**, **170 public inherent/trait operations**, **77 public struct fields**,
and **96 enum variants**. No private module/helper is presented as a public import. The
`api-audit.json` records each operation/field's owner page and the complete export/variant list.

| Public group | Coverage |
| --- | --- |
| Rendering | Query/Render, QueryLimits, DialectId and every capability field, Sql/TrustedSql, dialect identity and validation. |
| Names and predicates | Identifier/TableRef, Compare/Expr/Selector, JSON scalars/documents/segments/path options and all constructors/accessors. |
| DML | Select/Insert/Update/Delete; SelectData/InsertData/MutationData; Join/JoinKind/Sort/Order/WriteValue; every builder operation and inspection method. |
| DDL | Field/SqlType/Length/DefaultValue/Index/ForeignKey/ReferentialAction; Create/CreateData; Alter/AlterData/AlterOperation; QueryPlan; TableCommand. |
| Schema comparison | TableDefinition, Create::definition, SchemaDiff variants and comparison/rename/render operations. |
| Execution | Engine and all nine detached builder conveniences; execute/fetch/execute_plan/begin/close/diff/policy; exclusive Transaction completion and rollback_after. |
| Adapter extension | Database, TransactionSession, SqlFuture, TransactionOptions and required ownership/validation/cancellation duties. |
| Results and errors | SqlValue conversions, Row/FromSqlValue, metadata views, QueryResult; Result/SqlError/ErrorKind, causes, cleanup and disclosure. |

Reference pages include realistic import/usage blocks, parameter/field tables, Returns prose,
defaults, ownership, typed failure behavior, and related source/task links. Detailed runnable
per-method Rustdoc remains linked source rather than copied wholesale. Explicit boundaries
include Select equality replacement versus mutation append, native JSON numeric differences,
SQLite DEFAULT/DDL limits, unknown commit outcome/no replay, caller-owned plan completion,
per-member rather than whole-plan budgets, and trusted authorship rather than sanitization.

## Example evidence

Disposable consumer and evidence: `/private/tmp/stackpress-docs-00007a-core/`.
Cargo.lock was seeded from the workspace lock, then projected with `cargo generate-lockfile
--offline`. The consumer's two external packages have identical versions/checksums to the
workspace lock; there is no external dependency drift and the workspace lock is unchanged.
Every compilation/execution used offline and --locked, with debug info/incremental compilation
disabled to reduce disk use. No runtime/driver dependency was added to the repository.

All **30 plain Rust fences** were extracted verbatim into independent consumer modules after
rustfmt. **30 tests passed, 0 failed/ignored**: 26 complete synchronous programs and four async
helper blocks executed with the existing example RecordingDatabase and Tokio. The recorder
proves orchestration/ownership only, not native SQL/storage/cancellation behavior. The quick
start and every guide walkthrough ran; observed quick-start output exactly matches its page:

```text
SELECT "id", "name" FROM "profile" WHERE "name" = ? LIMIT 5
bind count: 1
```

All four canonical programs also ran successfully from the repository: build_inspect,
capabilities, schema_diff, and execute_plan. Their individual logs are retained. No native
PostgreSQL/MySQL fixture was contacted; G26's renderer-only boundary remains explicit.

## Author verification gates

Toolchain: rustc/Cargo 1.96.0, native aarch64-apple-darwin. The fresh external target was cleaned
with cargo clean and cargo llvm-cov clean --workspace before the first build. Only one target
directory was used for consumer, workspace doctests, and example runs. It was deleted after
verification (about 875 MB); logs, scripts, extracted blocks, consumer sources/lock, and audits
remain outside the repository for coordinator review. No documentation coverage percentage or
new native/platform qualification is claimed.

| Gate | Result / evidence |
| --- | --- |
| `python3 .../verify_docs.py` | PASS: 32 pages, 245 relative links/anchors, tagged/closed fences, one H1, no empty parent headings, double blank lines, or trailing whitespace. |
| Public export/method/field/variant audit | PASS: 60 / 170 / 77 / 96, no omissions; `api-audit.json`. |
| Consumer `cargo test --manifest-path .../consumer/Cargo.toml --offline --locked -- --nocapture` | PASS: 30/30; `consumer-first.log`, `block-inventory.json`, `lock-comparison.txt`. |
| `cargo test --workspace --doc --locked` with CARGO_NET_OFFLINE=true | PASS: 855 passed, 0 failed/ignored across 23 result groups; `workspace-doc.log`. |
| `cargo run -p stackpress-sql --example NAME --offline --locked` | PASS for all four; individual example logs and `example-results.json`. |
| `cargo fmt --all -- --check` | PASS. |
| `git diff --check` and new-page whitespace audit | PASS. |
| Source/tests/manifests/lock hash comparison | PASS: 303 unchanged files; `source-comparison.txt`. |
| Agent Workspace validator before receipt | Exit 0, 0 errors / 21 warnings; `kb-before.log`. Existing KB warnings are outside this lane. |

Editorial review read the complete entry/concept/guide/reference set, checked examples are
explained, reader intent and prerequisites are explicit, headings introduce content, API remains
lookup-oriented, terms/returns are consistent, and limitations are source/test backed. Final
mechanical scan includes the SQLite pages/anchors now on disk.

## DX observations, no code changes

1. Select::where_eq replaces one equality slot while Update/Delete append. Docs make the
   difference explicit and recommend and_where(Expr::eq(...)) for multiple Select equalities.
2. DML build borrows immutable data; DDL build returns a fallible normalized snapshot.
   Create/Alter render are inherent QueryPlan methods rather than Render implementations.
   The reference separates these return/trait boundaries for generic callers.
3. QueryPlan has no public arbitrary-batch constructor. App statements compose through an
   explicit Transaction; execute_plan remains a schema-builder plan surface.
4. Engine has no limits getter, and Transaction has no dialect getter. Configure limits before
   sharing and retain the engine's dialect when rendering a plan for an owned transaction.
5. QueryResult::insert_id Rustdoc still says the M0 adapter does not expose IDs. Current SQLite
   source/tests observe rowid transitions and can return Some(id), while some successful inserts
   return None. Core docs follow current behavior and link to adapter outcome rules; source
   Rustdoc was left unchanged under this docs-only brief.

No unresolved product choice or implementation fix was taken into this lane. Next step:
coordinator review, independent re-verification, and integration of these implementation notes.
Task verification/status and the SQLite receipt remain coordinator/other-lane owned.
