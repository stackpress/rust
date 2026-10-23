# Ingestion provenance and verification

Owner: [source catalog](../context/source-catalog.md). Load for auditing the handoff, refreshing knowledge, or investigating a drift warning.

## Sources and preservation

- Seven complete discussion notes from the original rust-notes workspace were archived byte-for-byte, then rendered as seven linked historical references with current-authority banners.
- Eight complete existing Stackpress topic Context Files and fifteen domain Reference Files (00004–00018) were archived byte-for-byte, then rendered as twenty-three linked upstream references. The original context index was also archived.
- The current setup request was preserved as text. It supplies authority for the monorepo, skill installation, KB migration/population, Rustdoc/comments, multi-pass refactoring, and coverage policy.
- Root and package manifests across the six requested repositories plus Frui/r22n were inventoried. Selected source anchors were hashed and retained in their canonical checkout; no source-code tree, generated runtime, dependency, database, or secret was archived.
- Both requested skill packages were installed from the same pinned ChrisAI main revision using the official skill installer. Their complete file lists/hashes and provenance are recorded.

The [ingestion manifest](../resources/provenance/ingestion-manifest.json) records exact original/raw/reference paths, SHA-256 hashes, original/rendered heading inventories, rendered-body hashes, repository HEAD/worktree facts, package inventory, and skill files. The [setup receipt](../resources/provenance/setup-receipt.json) records final validation and note-migration state; read it for the actual closeout result.

## Lossless reference rendering

The importer preserved original file bytes in resources. It added a scope/provenance header to live references and rewrote prose Markdown links so they point to migrated references or canonical retained sources. Fenced examples were not rewritten. Content outside this relocation was retained, including tables, code examples, caveats, old defaults/version literals, original decisions, and proposals.

Historical relative paths inside verbatim resources are interpreted against the original source location in the manifest; they are not active KB links. Live owner/reference links are validated separately. Headers/routers prevent older notes or upstream contributor rules from silently overriding the accepted Rust policy.

This is not summary-only ingestion: the complete source documents remain directly retrievable as both exact originals and linked live references, alongside current accepted owners and supplementary source maps.

## Checks

```sh
python3 .agents/scripts/validate-agent-workspace.py
python3 .agents/scripts/verify-kb-provenance.py
python3 .agents/scripts/verify-kb-provenance.py --check-sources
python3 .agents/skills/chrisai-knowledge/scripts/install_agent_workspace_rules.py --target .
```

The provenance verifier checks raw hashes, rendered-body hashes/headings, local skill integrity, note migration, and optional live source hashes. The workspace validator checks managed completeness, reference ownership/naming, context routing, links, and file caps. The installer dry-run must be idempotent after project-owned additions outside managed blocks.

No Rust test/coverage result is implied. No Cargo implementation or manifest was created. No MCP publisher/index was installed or served. No commits/pushes/publications/deployments occurred.

## Known source differences

- Current source package versions are 0.10.8; some imported KB references retain 0.10.7 literals.
- Old JS contribution policy says >90% on new work; the owner's Rust instruction is >=70% development and >=90% publication preparation.
- Existing Reactus/Electron/JS APIs are source evidence, not the Rust target frontend/host or mandatory literal API.
- The permission config catalog's general allowlist description must be reconciled with detailed empty-access allow-all source behavior before deciding Rust defaults.
- An existing Rust parser is present on the pinned feature branch; only source separation was inspected, not completeness/buildability.
- Existing source repositories with dirty worktrees are recorded as such; HEAD alone is not a promise that a file is committed.

## Refresh rules

Do not overwrite an old source snapshot to conceal drift. Reinspect changed sources, preserve needed prior evidence, refresh current accepted owners, and record the new observation. Retain requirements and examples unless explicitly superseded. Keep open proposals outside authoritative target contracts.

If source checkouts are unavailable in another environment, the archived non-code KB remains usable; code-level validation must obtain the recorded repositories/revisions. Do not claim fresh source verification from an archive alone.
