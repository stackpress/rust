# Source catalog and authority

Load for source provenance, claim verification, historical material or KB maintenance. This context describes the checked-out Rust implementation; it is maintained from source rather than from the old setup narrative.

## Authority and evidence

1. Current explicit owner instructions govern scope, behavior, process and authorization.
2. Current-branch Rust source and manifests establish implemented crates, APIs, features, commands and checks. [Source structure](source-structure.md) inventories packages; [source map](source-map.md) routes implementation and consumers. Verify at the semantic owner before changing a claim.
3. Accepted decisions G01–G62 in [the foundation ledger](../specs/00001-stackpress-rust-foundation/decisions.md) supply approved semantics. Later decisions supersede earlier ones: G41 removes framework-ui, G59 removes premade framework-models, G60 sets licences and G61 sets the 23-package release scope/facade exemption. Current manifests and the accepted release workflow set the version at 0.2.0 beta; the ledger's earlier 0.1.0 choice is not the current version.
4. Verified [task receipts](../specs/00001-stackpress-rust-foundation/tasks/status.md) establish bounded build/test and platform evidence. They do not turn a historical proof into universal support. Source/manifests are authoritative for the implemented state.
5. Imported upstream contracts and discussion references retain their original scope/revision/provenance. Proposals, capture records and old open questions are historical material, not active target decisions.

`.agents/context/` is the accepted reusable layer for future work. The owner has authorized this source-based promotion; leave planning, chronology and task narratives in the spec/references. For another unapproved source/context conflict, report it before silently changing accepted behavior. Do not consult other branches/worktrees merely to fill a claim.

## Current source families

- Root [Cargo.toml](../../Cargo.toml) and [src/lib.rs](../../src/lib.rs): membership, 0.2.0/edition/MSRV, lints, features, direct exports and optional namespace aliases.
- Linked per-crate manifests and sources in [the inventory](source-structure.md): all 23 release candidates, their local feature names and public facade paths, plus independent CLI use.
- [Parser provenance](../../crates/idea/idea-parser/PROVENANCE.md): the implemented extracted/repaired core, grammar/fixture provenance and retained Apache licence, separate from Node distribution.
- [Framework source schema loader](../../crates/framework/framework/src/native/commands/schema.rs): original `.idea` sources and compiled package imports; app-owned generated families replace premade models.
- [Release inventory/tooling](../../tools/release/common.py), [release policy](release-and-licensing.md), [ATTRIBUTION.md](../../ATTRIBUTION.md): coordinated scope, packaging/licences, copied-test inputs and dependency declarations.
- [Sync receipt](../specs/00001-stackpress-rust-foundation/tasks/00059-kb-sync.md): load when auditing this promotion's source/path verification, lossless demotions and workspace-validator limits.

## Historical context demotions

The following references retain the complete prior owner texts with only prose links rebased. Their stale claims and unresolved questions have no current authority; active equivalents live in context. Load only the matching audit topic:

- [Preimplementation direction and runtime](../references/00042-preimplementation-direction-runtime.md): earlier goals, UI rationale, lifecycle/generation proposals and questions resolved by implementation.
- [Preimplementation source structure](../references/00043-preimplementation-source-structure.md): accepted initial tree/namespace proposal, illustrative facade, dependency rationale, earlier publication questions and original technical links.
- [Preimplementation source map and provenance](../references/00044-preimplementation-source-provenance.md): original sibling source anchors, checked HEADs/worktree states, capture-version distinctions, parser-branch assessment, seven discussion routes, source archive/ingestion scope, skill installation and setup evidence.
- [Preimplementation style and quality](../references/00045-preimplementation-style-quality.md): full prior style/quality text, unchosen edition/MSRV wording, proposed coverage tooling and pre-Cargo command templates.

[Setup handoff](../references/00040-session-handoff.md) and [initial open questions](../references/00039-open-design-questions.md) are also historical: load only for initial intent/provenance. Do not use them as a current implementation inventory or re-ask questions resolved by source and the accepted ledger.

## Preserved upstream material and integrity

[Upstream contract router](upstream-stackpress.md) routes the complete imported architecture, APIs, generation, configuration, UI, package inventories and source-behavior supplements. [Ingestion provenance](../references/00038-ingestion-provenance.md) records exact preservation methods and their limits; load for a historical-import audit. Captured source versions/HEADs are dated observations, not freshly verified sibling checkouts.

The [ingestion manifest](../resources/provenance/ingestion-manifest.json) pins original paths/hashes and raw/reference/skill mappings; [setup receipt](../resources/provenance/setup-receipt.json) records that setup's result. Raw discussion and upstream non-code snapshots remain in resources. Code remains in its source location; do not mirror source trees or regenerate capture hashes to conceal drift.

Project-local coding, knowledge and documentation skill packages retain their own provenance. Rust uses the owner's explicit coding adaptation and standing refactor authorization; JS-specific syntax/audit-only defaults do not override it. No KB MCP publisher/index/service is installed by this documentation sync.

Use `python3 .agents/scripts/validate-agent-workspace.py` for routing, references, links and line caps. Use `python3 .agents/scripts/verify-kb-provenance.py` for preserved import/skill integrity; its optional `--check-sources` reads canonical sibling source files and must respect the task's source-access boundary. Neither check proves Rust behavior, platform support or coverage. Preserve prior evidence and reconcile the changed contract before any accepted refresh.
