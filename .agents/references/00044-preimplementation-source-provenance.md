# Historical context: preimplementation source provenance

Owner: [source catalog](../context/source-catalog.md). Load only to audit the pre-sync setup text, provenance, superseded plans or resolved questions.

This is a lossless demotion of the context at branch base `34be3146ffc3279595595b665bf371205744a3a2`, preserved by Task 00059. Statements about missing implementation, open decisions, versions and support describe that older text and have no current authority. Use the [current context index](../context/index.md) for accepted truth. Only prose links are rebased; fenced examples are unchanged.

Original owner: `.agents/context/source-map.md`.

# Foundation and package source map

Use when locating a semantic owner before implementation. Source roots below are canonical sibling checkouts; source code was not copied into resources. The [manifest and hash ledger](../resources/provenance/ingestion-manifest.json) records capture revisions and file hashes.

For the Rust target, read [accepted source structure](../context/source-structure.md) for the root facade, grouped crate tree, package/path catalog, and responsibility boundaries. Upstream names below remain source locators: Ingest maps to `server/server`, Inquire to `sql`, and Stackpress framework packages to the `framework` family.

The existing Rust server family is grouped beneath `crates/server/`, which has no package manifest. Start with the [capability-host manifest](../../crates/server/server/Cargo.toml), [source](../../crates/server/server/src/lib.rs), and [docs](../../crates/server/server/docs/README.md) for transport-neutral hosting and buffered HTTP. The optional owned HTTP/1 IO adapter has its own [manifest](../../crates/server/http/Cargo.toml), [source](../../crates/server/http/src/lib.rs), and [docs](../../crates/server/http/docs/README.md). The facade retains separate buffered `server-http` and network-host `server-http-host` / `server-http-host-tls` features.

## Foundations

| Repository | Inspect first | Owns |
| --- | --- | --- |
| Lib | [public exports](../../../lib/src/index.ts), [Exception](../../../lib/src/Exception.ts), [emitters](../../../lib/src/emitter), [router](../../../lib/src/router) | Data, queues, emitter/matching, request/response/session, files, terminal, error primitives |
| Idea | [parser](../../../idea/packages/idea-parser/src), [transformer](../../../idea/packages/idea-transformer/src/Transformer.ts), [language server](../../../idea/language/server/src/server.ts) | Grammar/AST/compiled schema, composition, transform runner, editor tooling |
| Ingest | [Server](../../../ingest/ingest/src/Server.ts), [Router](../../../ingest/ingest/src/Router.ts), [Route](../../../ingest/ingest/src/Route.ts), [Loader](../../../ingest/ingest/src/Loader.ts) | Capability host, plugin loading, handler facets, lifecycle, transport adapters |
| Inquire | [Engine](../../../inquire/packages/inquire/src/Engine.ts), [builders](../../../inquire/packages/inquire/src/builder), [adapters](../../../inquire/packages) | SQL/build/dialect/transaction/diff mechanics and native connectors |
| Reactus | [rendering](../../../reactus/reactus/src/DocumentRender.ts), [helpers](../../../reactus/reactus/src/helpers.ts), [Builder](../../../reactus/reactus/src/Builder.ts) | Existing React rendering, hydration/manifests, build/dev/assets |
| Frui | [manifest](../../../frui/package.json), [source](../../../frui/src) | Existing granular React behavior/components and notifications |
| r22n | [manifest](../../../r22n/package.json), [source](../../../r22n/src) | Existing React phrase translation and interpolation |

Read [full foundation behavior details](../references/00034-foundation-source-map.md) for the public families, edge cases, source tests, and port boundaries. This reference should be loaded before substituting a Rust library for an existing behavior.

## Framework packages and applications

Read [the complete package and test-location inventory](../references/00035-upstream-package-source-inventory.md) to locate the package; it records all foundation/framework packages rather than equating a repo with one package.

Read [package behavior supplements](../references/00041-framework-package-supplements.md) for explicit per-package runtime/generator responsibilities and optional-package limits. The [upstream KB router](../context/upstream-stackpress.md) links all detailed APIs, configuration, examples, and generation contracts.

Framework roots are [packages](../../../stackpress/packages). The primary end-to-end application is [templates/blog](../../../stackpress/templates/blog); [store](../../../stackpress/templates/store) and [website](../../../stackpress/templates/website) demonstrate other composition. Generated directories and application data are not the durable owner of a fix.

## Existing Rust Idea parser

Pinned source: [feature/rust-parser-2](https://github.com/stackpress/idea/tree/6c289d1f3fba17c758baafb2d4dcefd87fd448ea/packages/idea-rust).

- `idea_parser_core`: Serde-based tokenizer/AST/parser/finalization and structured `IdeaError` with code/message/span.
- `idea_parser_node`: separate N-API cdylib binding/error conversion.
- `idea-node` and per-target packages: Node distribution, not mandatory dependencies of a Rust generator.

Read [the complete parser and diagnostic assessment](../references/00008-discussion-diagnostics-existing-parser.md) before extraction. The branch was inspected without checkout/merge or compile/parity tests; use current accepted fixtures to assess gaps. Do not let platform-specific npm packaging prevent using the native Rust core directly.

## Locate the right layer

Trace declaration/config → semantic owner → generator/runtime producer → generated output → consumer → adapter/surface → verification. A package-local workaround is inappropriate when the primitive belongs in a foundation. For Rust this can remain one coordinated monorepo while preserving independent crate ownership.

Inspect corresponding tests and current manifests, then perform meaningful target checks. Existing docs/API exports show intent/implementation, not automatically support or production readiness. Do not assume a sibling checkout version equals the dependency version consumed by an application.

Original owner: `.agents/context/source-catalog.md`.

# Source catalog and authority

Load for provenance, historical references, conflict resolution, or KB refresh. Capture date: 2026-10-23, Asia/Manila.

## Authority order

1. Current explicit project-owner instructions govern target scope/decisions/style/coverage and override older proposals or imported contributor policy.
2. Rust context records the accepted reusable target direction and rules. New implementation truth requires source/test evidence.
3. Existing source code and manifests establish current source-system behavior, scoped to their checked revision/hash. The imported KB routes detailed understanding but can drift.
4. Complete discussion references retain proposals, rationale, and open questions. They do not freeze a crate tree or fictional API.
5. Raw resource copies preserve source evidence. They do not independently establish current target authority.

The owner's later explicit approval establishes the [accepted source structure](../context/source-structure.md); load it for the current root workspace/facade, grouped crate layout, names, and boundaries. The [conversation extract](../resources/conversation/2026-10-23-source-structure-acceptance.txt) preserves the proposal, accepted response, and exact approval. This supersedes earlier layout and Ingest/Inquire naming proposals without changing archived discussion bodies. The acceptance record is a later source, separate from the original setup ingestion manifest/receipt; those historical hashes and receipts remain unchanged.

## Preserved discussion

All seven original note files were preserved byte-for-byte before migration. Their relative links are interpreted against the original directory recorded in the manifest. Live reference copies rebase prose links into this workspace or to canonical retained sources; fenced examples remain literal.

- [Discussion Working Summary](../references/00004-discussion-working-summary.md): load for complete historical detail in this topic, including superseded suggestions and explicit uncertainties.
- [Discussion Feature Feasibility](../references/00005-discussion-feature-feasibility.md): load for complete historical detail in this topic, including superseded suggestions and explicit uncertainties.
- [Discussion Crate Plugin Proposals](../references/00006-discussion-crate-plugin-proposals.md): load for complete historical detail in this topic, including superseded suggestions and explicit uncertainties.
- [Discussion Frontend Decision History](../references/00007-discussion-frontend-decision-history.md): load for complete historical detail in this topic, including superseded suggestions and explicit uncertainties.
- [Discussion Diagnostics Existing Parser](../references/00008-discussion-diagnostics-existing-parser.md): load for complete historical detail in this topic, including superseded suggestions and explicit uncertainties.
- [Discussion Cargo Namespace](../references/00009-discussion-cargo-namespace.md): load for complete historical detail in this topic, including superseded suggestions and explicit uncertainties.
- [Discussion Source Index](../references/00010-discussion-source-index.md): load for complete historical detail in this topic, including superseded suggestions and explicit uncertainties.

[Current setup request](../resources/conversation/2026-10-23-setup-request.txt) records the authorized monorepo/skills/KB/style/refactoring/coverage work. The [ingestion manifest](../resources/provenance/ingestion-manifest.json) records original paths, hashes, raw/reference mappings, headings, source repositories, source anchors, skills, and package facts.

## Upstream KB and source coverage

Preservation scope is the eight topic Context Files and all fifteen domain Reference Files (00004–00018), plus the original index. The complete 23 documents are available as rebased live references, not summary-only replacements. Managed Agent Workspace rules were installed separately from the requested knowledge skill instead of copying the old project operating surface.

The raw non-code KB snapshot lives under `.agents/resources/upstream-stackpress-kb/`. Source code stays in the eight canonical sibling checkouts. No source trees, generated client, database, dependencies, or secrets were archived. Related Frui/r22n manifest facts and source pointers support UI contract understanding.

Read [the upstream contract router](../context/upstream-stackpress.md) for all imported contract owners and [the source map](../context/source-map.md) for exact code locations. [Ingestion and verification detail](../references/00038-ingestion-provenance.md) explains the checks and their limits; load it when auditing or refreshing this KB.

## Checked repositories

| Source | Checked HEAD | Worktree |
| --- | --- | --- |
| `lib` | `e2cc6b8cad77b5ad18be8a94e0d40e72e2f6183a` | Clean at capture |
| `idea` | `cf8e922ba6e57746e15a05c26df0546dda2cc5c9` | Clean at capture |
| `ingest` | `477ec47f8b5dafa5713b13f9c964d06c61c22938` | Clean at capture |
| `inquire` | `039a1b5f41500aa63286a9f9b34f4c7f71b990a0` | Clean at capture |
| `reactus` | `357569a4923821d9086adf565c5ed22f0e082631` | Clean at capture |
| `stackpress` | `a71d683051ba8350fdd12d6b5a33f268fdcc285f` | Dirty; see per-file hashes |
| `frui` | `096fd14580f0f49b6e159fa0aa5ae2a5bce8fb0e` | Clean at capture |
| `r22n` | `48c50dd1ba57a0555c9afbd4bfa4cb41e702cb68` | Dirty; see per-file hashes |

Main-branch manifests in the six original foundation/framework repositories report version 0.10.8. Existing KB text can retain older version literals and examples. Source HEAD plus current file hash is the relevant observation, especially in a dirty checkout.

The Rust parser branch is separately pinned at `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`; no branch merge, parser extraction, compile, or parity test occurred.

## Skill installation

Installed complete `chrisai-coding` and `chrisai-knowledge` packages from `cblanquera/chrisai` main at `c3be0b52e797f48ba8400893bb4aae693eaeb221` using the official skill installer, at `.agents/skills`. The knowledge installer was dry-run before apply. Package file counts/hashes are in the ingestion manifest.

On 2026-10-24 the owner also requested `chrisai-docing`, installed at `.agents/skills/chrisai-docing` from the same `cblanquera/chrisai` revision `c3be0b52e797f48ba8400893bb4aae693eaeb221` (8 files, byte-identical to the source tree). It supplies the technical-writing standards for crate documentation: API reference, guided learning, formatting, and copy editing. The provenance verifier's integrity check still covers only the original two packages.

The coding skill currently scopes itself to existing JS/TS/React/HTML/CSS. The owner explicitly requests its principles for Rust. The project style/workflow adapts those principles while preserving installed packages. The user-authorized repeated refactor pass supersedes audit-only stops inside the requested coding task; unrelated scope still requires a decision.

No MCP publisher/index/service was requested or installed. This is a Markdown KB; there is no automatic semantic-indexing or external-serving claim.

## Refresh and evidence limits

Run `.agents/scripts/verify-kb-provenance.py` for preservation/import/skill integrity and optional `--check-sources` for live source drift. Run `.agents/scripts/validate-agent-workspace.py` for links, routing, naming, and line caps. Neither proves semantic completeness of a Rust implementation.

Preserve older evidence before refreshing; update accepted owners and references without promoting speculation. If a source hash changes, inspect/reconcile the changed contract instead of regenerating a manifest to hide drift.

