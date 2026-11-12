# Task 00059: sync the KB from source

Status: documentation implementation complete; coordinator review remains separate.
Scope: root AGENTS.md and `.agents/` documentation only. Branch `kb/sync-from-source`, base/verified source `34be3146ffc3279595595b665bf371205744a3a2`. No other branch/worktree was consulted.

## Topics and disposition

Updated project direction, runtime/portability, Rust style, quality/testing, source structure/map/catalog, upstream router and index. Added release/licensing context. The no-panic exception registry remains in place and unchanged. Root AGENTS.md corrects only Project contract and Boundaries; the owner's implementation/review/process rules remain intact.

The source inventory covers all 52 root workspace packages (23 release packages and 29 unpublished examples), plus 28 nested packages / 29 manifests including the blog workspace root. It lists paths, purposes, local features and facade exports or their absence. It includes the seven blog packages, plugin crates/xtask, isolated consumers, release/blog tools and fuzz workspace.

Lossless demotions preserve all seven replaced owner bodies in four numbered references, with only prose links rebased and fenced examples unchanged:

- [Prior direction and runtime](../../../references/00042-preimplementation-direction-runtime.md): load for superseded setup goals, unresolved lifecycle/generation questions and target rationale.
- [Prior source structure](../../../references/00043-preimplementation-source-structure.md): load for the initial tree, illustrative facade, dependency rationale and unresolved release questions.
- [Prior source map and provenance](../../../references/00044-preimplementation-source-provenance.md): load for original sibling anchors, captured HEAD/version/skill details, archive scope and discussion routes.
- [Prior style and quality](../../../references/00045-preimplementation-style-quality.md): load for superseded unchosen toolchain/proposed coverage/pre-Cargo wording and complete former guidance.

References 00038–00040 receive historical-scope banners; their prior bodies remain intact. No spec content/status/decision was changed; this receipt is the sole spec addition. Source catalog routes every demotion. No destination decision remains open.

## Verification method

Opened root/per-package manifests with Python tomllib (no cargo metadata), crate roots, semantic producers, generator/source-family and package-import code, HTTP lifecycle/date/cookie paths, source validation/list predicates, CSV decoding/sniffing, parser provenance and current example/consumer/tool manifests. Read accepted G01–G62 and task status/selected release receipts. Reconciled superseding decisions against actual source. Checked every new Rust/source/tool link and inventory manifest path; features were extracted directly from manifests. Verified release scope, version/licence declarations and text, facade aliases and no-panic lint presence. No source tree was copied into resources.

Compared each demoted body to its saved pre-edit owner after the declared link rebasing; checked unchanged no-panic registry, unchanged root process section, and the documentation-only diff boundary. All new/touched context/reference/receipt files are <=200 lines.

Read-only checker results: test copies **46 / 2 manifests, 0 errors**; packaged-test boundaries **23 candidates, 0 errors**; KB provenance **30 complete imports, 2 original skill packages, PASS**. No optional live sibling drift check, Cargo build/test, coverage measurement, browser/fuzz run or release gate was run for this documentation-only task. Those commands/behaviors were verified from their current code and existing receipts, not claimed as new runtime evidence.

## Workspace validation

Final canonical validation: **0 errors / 44 inherited warnings** (exit 0). Raw worktree validation: **146 inherited sibling-path errors / 44 inherited warnings** (exit 1), matching the baseline. No new warning/error remains. The warnings are 42 advisory 201–500-line files (six historical references, 35 spec files and one managed workflow) and two path-like descriptions inside unchanged imported discussion text. Existing files remain within the hard 500-line cap. No inherited content is shortened or rewritten to clear an advisory.

Canonical validation uses the current worktree's validator and all current worktree Agent Files. Its adapter only rebases external sibling link targets for lib/idea/ingest/inquire/reactus/stackpress/frui/r22n to the canonical `/Users/cblanquera/server/projects/stackpress/` layout and checks existence; it does not read source contents or use another Rust checkout. No line-cap/reference/routing/other link error is suppressed. Logs and the adapter are under `/private/tmp/kb-sync-00059-*` for this local verification.

No install, listener, commit, push, publication, deployment, source/manifest/lock/licence edit or MCP indexing occurred. Next step: coordinator review of the promoted context and this receipt.
