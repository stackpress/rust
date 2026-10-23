# Current source package inventory

> Read when locating the existing semantic owner or finding a package manifest/tests. This is the existing JS/TS release set, not the Rust crate plan.

Captured 2026-10-23 from manifests; current publishable Stackpress foundation/framework packages report 0.10.8. Imported KB passages may retain older 0.10.7 literals. Prefer these source facts, then recheck before implementation.

| Repository | Manifest | Package | Version | Existing test directory |
| --- | --- | --- | --- | --- |
| lib | [package.json](../../../lib/package.json) | `@stackpress/lib` | 0.10.8 | tests |
| idea | [package.json](../../../idea/package.json) | `(workspace root)` | — | Inspect scripts/owner |
| idea | [packages/idea/package.json](../../../idea/packages/idea/package.json) | `@stackpress/idea` | 0.10.8 | Inspect scripts/owner |
| idea | [packages/idea-parser/package.json](../../../idea/packages/idea-parser/package.json) | `@stackpress/idea-parser` | 0.10.8 | packages/idea-parser/tests |
| idea | [packages/idea-transformer/package.json](../../../idea/packages/idea-transformer/package.json) | `@stackpress/idea-transformer` | 0.10.8 | packages/idea-transformer/tests |
| ingest | [ingest/package.json](../../../ingest/ingest/package.json) | `@stackpress/ingest` | 0.10.8 | ingest/tests |
| ingest | [package.json](../../../ingest/package.json) | `(workspace root)` | — | Inspect scripts/owner |
| inquire | [package.json](../../../inquire/package.json) | `(workspace root)` | — | Inspect scripts/owner |
| inquire | [packages/inquire/package.json](../../../inquire/packages/inquire/package.json) | `@stackpress/inquire` | 0.10.8 | packages/inquire/tests |
| inquire | [packages/inquire-mysql2/package.json](../../../inquire/packages/inquire-mysql2/package.json) | `@stackpress/inquire-mysql2` | 0.10.8 | packages/inquire-mysql2/tests |
| inquire | [packages/inquire-pg/package.json](../../../inquire/packages/inquire-pg/package.json) | `@stackpress/inquire-pg` | 0.10.8 | packages/inquire-pg/tests |
| inquire | [packages/inquire-pglite/package.json](../../../inquire/packages/inquire-pglite/package.json) | `@stackpress/inquire-pglite` | 0.10.8 | packages/inquire-pglite/tests |
| inquire | [packages/inquire-sqlite3/package.json](../../../inquire/packages/inquire-sqlite3/package.json) | `@stackpress/inquire-sqlite3` | 0.10.8 | packages/inquire-sqlite3/tests |
| reactus | [package.json](../../../reactus/package.json) | `(workspace root)` | — | Inspect scripts/owner |
| reactus | [reactus/package.json](../../../reactus/reactus/package.json) | `reactus` | 0.10.8 | reactus/tests |
| stackpress | [package.json](../../../stackpress/package.json) | `@stackpress/repository` | 0.10.8 | tests |
| stackpress | [packages/stackpress/package.json](../../../stackpress/packages/stackpress/package.json) | `stackpress` | 0.10.8 | Inspect scripts/owner |
| stackpress | [packages/stackpress-admin/package.json](../../../stackpress/packages/stackpress-admin/package.json) | `stackpress-admin` | 0.10.8 | Inspect scripts/owner |
| stackpress | [packages/stackpress-ai/package.json](../../../stackpress/packages/stackpress-ai/package.json) | `stackpress-ai` | 0.10.8 | packages/stackpress-ai/tests |
| stackpress | [packages/stackpress-api/package.json](../../../stackpress/packages/stackpress-api/package.json) | `stackpress-api` | 0.10.8 | Inspect scripts/owner |
| stackpress | [packages/stackpress-csrf/package.json](../../../stackpress/packages/stackpress-csrf/package.json) | `stackpress-csrf` | 0.10.8 | Inspect scripts/owner |
| stackpress | [packages/stackpress-desktop/package.json](../../../stackpress/packages/stackpress-desktop/package.json) | `stackpress-desktop` | 0.10.8 | packages/stackpress-desktop/tests |
| stackpress | [packages/stackpress-email/package.json](../../../stackpress/packages/stackpress-email/package.json) | `stackpress-email` | 0.10.8 | Inspect scripts/owner |
| stackpress | [packages/stackpress-language/package.json](../../../stackpress/packages/stackpress-language/package.json) | `stackpress-language` | 0.10.8 | Inspect scripts/owner |
| stackpress | [packages/stackpress-schema/package.json](../../../stackpress/packages/stackpress-schema/package.json) | `stackpress-schema` | 0.10.8 | packages/stackpress-schema/tests |
| stackpress | [packages/stackpress-server/package.json](../../../stackpress/packages/stackpress-server/package.json) | `stackpress-server` | 0.10.8 | packages/stackpress-server/tests |
| stackpress | [packages/stackpress-session/package.json](../../../stackpress/packages/stackpress-session/package.json) | `stackpress-session` | 0.10.8 | Inspect scripts/owner |
| stackpress | [packages/stackpress-sql/package.json](../../../stackpress/packages/stackpress-sql/package.json) | `stackpress-sql` | 0.10.8 | packages/stackpress-sql/tests |
| stackpress | [packages/stackpress-view/package.json](../../../stackpress/packages/stackpress-view/package.json) | `stackpress-view` | 0.10.8 | packages/stackpress-view/tests |
| frui | [package.json](../../../frui/package.json) | `frui` | 0.2.9 | tests |
| r22n | [package.json](../../../r22n/package.json) | `r22n` | 1.0.10 | Inspect scripts/owner |

## Source boundaries

- Idea, Inquire, and Stackpress are monorepos; Ingest and Reactus also have workspace/example organization. Do not map one repository to one Rust crate.

- `packages/stackpress-studio` is planning-only and has no active package manifest. Desktop is active but optional; AI is optional. The aggregate includes server, schema, language, CSRF, SQL, view, session, API, and admin in its deliberate order. Email is a separate package composed indirectly by the session plugin.

- Frui and r22n are related UI libraries. Their concepts/component and phrase contracts inform the rewrite, while React implementations are not Leptos-compatible components.

- `templates/blog` is the default end-to-end reference; store and website show other application compositions. Generated `.build`/`client_source` output is not durable implementation authority.

- Every manifest and selected source anchor has a hash and repository HEAD in the [ingestion manifest](../resources/provenance/ingestion-manifest.json). Dirty source checkouts are recorded, not assumed committed. No source-code trees were copied.

- The existing parser branch is pinned separately from Idea main. See [parser assessment](00008-discussion-diagnostics-existing-parser.md) when deciding extraction and parity work.
