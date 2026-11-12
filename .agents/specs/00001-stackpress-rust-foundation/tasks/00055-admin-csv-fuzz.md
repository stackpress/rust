# Task 00055: admin no-panic, tolerant CSV and parser fuzzing

Status: implementation and local code gates passed after targeted follow-ups. Coordinator review/merge and owner acceptance remain pending. Acceptance criteria: none.

## Implementation

- Framework-admin denies all six G62 panic-family lints. Library assumptions now return errors or the existing visible adapter rejection; valid-input behavior and G42 lifecycle rules remain unchanged.
- G62 supersedes G43 for crash cases: metadata/list/table object children and malformed nested fieldset rows visibly reject instead of unwinding. Regression tests cover these cases and mismatched public source catalogs that formerly reached generator `expect` calls.
- `Upload.csv` carries raw `CsvBytes`, with `From<String>`, `From<&str>`, `From<Vec<u8>>`, `as_bytes`, `len`, `is_empty` and `clear`. JSON accepts legacy strings and new byte arrays. The handwritten generated-admin browser/HTTP bridge preserves the original bytes. API-owned call-site changes were routed to the framework worker.
- Own bounded import decoder: UTF-8 BOM, strict UTF-16 LE/BE BOM, valid unmarked UTF-8, then Windows-1252 or a documented Arabic-evidence Windows-1256 heuristic. Raw and decoded byte ceilings both apply. No new root dependency and no copied craft-csv implementation.
- Delimiter sampling counts comma/semicolon/tab outside quotes over sixteen nonblank logical records. Candidates must occur on every sample; rank modal agreement, then count range, then comma/semicolon/tab order. No qualifier means comma. Header, sentinel, typed conversion, append budgets, error and export policies stay in place; export remains UTF-8 comma.
- Source templates bind controls through `RoleContext::from_draft`, preserving the former lookup/error order and root signal. Direct tests prove success, both rejection paths, error precedence and no mutation. This removes repeated fallible lookup policy from every emitted control.
- Updated transfer reference and example text. Officially regenerated affected API/aggregate/checkpoint/build-host managed output. No manual managed-output edits and no goldens/oracles/`.stderr` changes.
- Added unpublished nested `fuzz/` workspace, independent lock, three targets, thirteen small fixture-derived seeds, [run instructions](../../../../fuzz/README.md) and a [release note](../../../../tools/release/FUZZING.md). Generated corpus mutations remain under ignored `fuzz/coverage/discovered-corpus`.

## No-panic exception

One template entry: [source-roles.rs.tpl](../../../../crates/framework/framework-admin/src/codegen/templates/source-roles.rs.tpl), generated `model()`. The narrow `#[expect(clippy::expect_used)]` parses a generator-serialized compile-time UiModel literal. Generation round-trip validates it, and `generated_source_descriptor_constant_roundtrips` proves the constant. No runtime-dependent exception or new `#[allow]` escape.

## Refactoring and safety review

1. Logic/complexity: reviewed BOM precedence, expansion accounting, quote states, sample ceilings and error order; added direct fieldset/metadata crash regressions. Focused suite: 70 passed, one existing skip.
2. Responsibility/abstraction: separated encoding, sniffing, structure and raw upload representation; extracted consistency ranking and proved the descriptor constant. Source/format suite: 75 passed, one existing skip.
3. Consistency/Rustdoc/comments: formatted authored code, documented byte/error/heuristic boundaries, retained valid source rendering. Source/format suite: 75 passed, one existing skip.
4. Follow-up/soundness: covered list/table child rejection and inconsistent generator catalogs (both focused pairs passed); reviewed append budgets and generated callback errors. Gate findings added byte length/clear compatibility operations and changed a cfg-gated formatted test fixture helper to return Result. No production behavior change from that helper cleanup.
5. Responsibility/coverage follow-up: centralized the exact generated role binding policy, officially regenerated all affected trees, and proved public source table envelopes/callback rejection, transform prerequisites, layout/root ownership, submitted issue escaping and public error wording. Focused source/format suite: 84 passed, one existing skip. The affected API/framework suite also passed 148 tests, ten existing skips. No threshold reduction, coverage exclusion or golden edit.

## Fuzz evidence and lock invariant

Installed by the approved commands: cargo-fuzz 0.13.2 and nightly rustc 1.102.0-nightly (3ef0c22cd 2026-11-11). Stable rustc 1.96.0; sccache 0.19.0. Each target used `-max_total_time=300 -max_len=1048577 -timeout=10 -print_final_stats=1`.

| Target | Executions | Result | Benchmark ID |
| --- | ---: | --- | --- |
| idea_parser | 3,994,693 | completed, no crash | 20261112T042202.085982Z-620a75b869f5 |
| admin_csv | 1,379,502 | completed, no crash | 20261112T043159.684103Z-ade31d9eee33 |
| lib_query_form | 1,998,157 | completed, no crash | 20261112T043714.704987Z-a0ad32603bca |

No crash needed routing. Nightly reports an inherited stackpress-lib `AtomicU64::fetch_update` deprecation warning; stable lint results are separate. Finite runs do not prove exhaustive correctness or other-platform support.

Root `Cargo.lock` remains byte-for-byte identical to HEAD, with 699 registry tuples. The fuzz lock has exactly these two approved extra tuples:

| Package | Version | Checksum |
| --- | --- | --- |
| libfuzzer-sys | 0.4.10 | 5037190e1f70cbeef565bd267599242926f724d3b8a9f510fd7e0b540cfa4404 |
| arbitrary | 1.5.0 | 3bc62ac97cc33321f50863d514c3bc38a453947a8f9e781137e47c7401020aed |

## Final verification

- Formatting, default and compatible workspace Clippy `-D warnings`, admin Clippy alone, doctests and Rustdoc `-D warnings` passed. Final workspace Clippy IDs: `20261112T060534.911010Z-1235f8b89c19` / `20261112T060549.191460Z-8fabec71f511`; admin default alone: `20261112T060639.988707Z-f943459b774e`. Selected source/format Clippy also passed.
- Default nextest: 1,057 passed, 91 skipped. Compatible: 1,281 passed, 101 skipped and one unchanged ten-second preferences child timeout. That exact case passed its isolated compatible rerun in 0.774 seconds (`20261112T045420.951349Z-1d73383e5096`); no timeout/source change. Isolated authorized TCP fixture: one passed (`20261112T043108.674370Z-f564baea64fc`). Final affected API/framework tests: 148 passed, ten skipped (`20261112T060641.926108Z-21d64541ec53`).
- All eight aggregate consumers passed. Build-host `--check`, generated-admin ordinary/format checks and all officially managed API/aggregate/checkpoint source trees are fresh. Final freshness ID: `20261112T060809.195710Z-8c4179a77ae6`.
- The full blog gate's behavior, browser matrix, nickname extension and README migration phases passed. Its first coverage run exposed repeated unexecuted error closures in emitted controls. After round 5, the affected generator/freshness, 90 native tests, instrumented native HTTP browser and native Clippy all passed (`20261112T060020.029176Z-b43027fa6e85`). Final per-crate native coverage: tool 90.27%, web 70.64%, app 87.64%, store 97.44%, assets 89.74%, xtask 78.73%; browser glue keeps the existing G56 exemption. LLVM retained a 68-function mismatched-counter warning across the combined instrumented binaries (the first run also warned); the report is threshold-passing, not warning-free.
- Generated-admin's actual transfer browser proof passed hydrate and CSR at desktop and narrow/zoomed sizes, including raw legacy encodings and semicolon import (`20261112T055142.237706Z-9524bb21162e`).
- Framework-admin native coverage **90.02585% (4,179 / 4,642 executable lines)**, with 84 passed / one existing skip, features `codegen,source-ssr,format` (`20261112T060658.524652Z-c96e055ce71e`). No claim for unmeasured platforms or DOM event coverage.
- All 23 release package lists passed (`20261112T060827.516572Z-38a76e5b5b7c`). The incidental release-manifest report was restored; final evidence was directed to a temporary report instead of changing another lane's manifest.
- Original failing lint, coverage and fixture attempts remain in the ledger. Only affected phases were rerun after fixes. No unresolved code-gate failure; the existing preferences deadline remains a recorded intermittent result.

The KB validator retains the same 146 existing missing-link errors plus warnings as before this receipt existed; zero new receipt-link errors. This is not a clean KB validation. No unrelated KB repair is included.

Benchmark store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Labels begin `00055`. Commands use own targets, sccache, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`. Preserve failed records and distinguish measured command duration from observed Cargo stage duration. The generic runner marked the initially empty target warm; timings are operational records, not controlled cache-performance evidence. Live ledger: 80 completed attempts (69 passed, 11 failed), 40 with observed compilation activity; zero additional successful cold workspace rebuilds. Summed `command_seconds` 8,095.498 and observed `cargo_build_seconds` 4,901.090; 30 records lack a Cargo stage timing. Overlapping commands and hidden child output make these sums neither elapsed time nor complete compile cost.

## Merge note

**Pending foundation merge: formatted.rs:137-138.** Leave the current branch's String-returning trait implementation unchanged. After merging the foundation trait change, the coordinator will apply this exact compatibility edit:

```rust
fn display(&self, value: &IsoValue, locale: DisplayLocale) -> Result<String, EditError> { Ok(temporal_text(&self.inner.display(value, locale)?, self.precision)) }
```

No commit/push, publication, deployment or development listener. The finite TCP fixture was separately authorized. Local-only `clippy.toml` supplied the three approved test allowances during checks and was removed afterward, excluding it from delivery.
