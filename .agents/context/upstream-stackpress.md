# Existing Stackpress knowledge and contract router

Authority: source-backed knowledge of the existing TypeScript ecosystem, preserved in full as references. These are accepted descriptions of the captured source system. Current Rust implementations, features and public paths are documented in [source structure](source-structure.md) and [runtime](runtime-and-portability.md); the imported reference alone does not prove a Rust claim.

## How to use

Start with the smallest responsibility/contract. Follow detailed source anchors when tracing an original contract, then inspect the current-branch Rust producer and tests. Captured sibling anchors/revisions are not a fresh inspection of those repositories. Rust target decisions and quality policy override legacy frontend, package, naming, and coverage conventions.

## Architecture and composition

- [Stackpress Identity And Principles](../references/00011-upstream-identity-and-principles.md): load when working on identity and principles; includes complete original detail and provenance.
- [Stackpress Architecture And Composition](../references/00012-upstream-architecture-and-composition.md): load when working on architecture and composition; includes complete original detail and provenance.
- [Stackpress Modeling And Generation](../references/00013-upstream-modeling-and-generation.md): load when working on modeling and generation; includes complete original detail and provenance.
- [Stackpress Runtime And Operations](../references/00014-upstream-runtime-and-operations.md): load when working on runtime and operations; includes complete original detail and provenance.
- [Stackpress Interfaces And Experience](../references/00015-upstream-interfaces-and-experience.md): load when working on interfaces and experience; includes complete original detail and provenance.
- [Stackpress Ecosystem And Portability](../references/00016-upstream-ecosystem-and-portability.md): load when working on ecosystem and portability; includes complete original detail and provenance.
- [Stackpress Extension And Contribution](../references/00017-upstream-extension-and-contribution.md): load when working on extension and contribution; includes complete original detail and provenance.
- [Stackpress Compatibility And Maintenance](../references/00018-upstream-compatibility-and-maintenance.md): load when working on compatibility and maintenance; includes complete original detail and provenance.

## API, generation, configuration, and operation contracts

- [Runtime API Contracts](../references/00019-upstream-runtime-api-contracts.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Server And Transport Contracts](../references/00020-upstream-server-and-transport-contracts.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Schema API Contracts](../references/00021-upstream-schema-api-contracts.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Idea Language Catalog](../references/00022-upstream-idea-language-catalog.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Stackpress Configuration Catalog](../references/00023-upstream-configuration-catalog.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [CLI And Plugin Contracts](../references/00024-upstream-cli-and-plugin-contracts.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [SQL API Contracts](../references/00025-upstream-sql-api-contracts.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Database Adapter Contracts](../references/00026-upstream-database-adapter-contracts.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [View API Contracts](../references/00027-upstream-view-api-contracts.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Session And Language Contracts](../references/00028-upstream-session-language-contracts.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Exports, Types, And Generated Contracts](../references/00029-upstream-exports-types-generated-contracts.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Operational Examples](../references/00030-upstream-operational-examples.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Idea Component Catalog](../references/00031-upstream-idea-component-catalog.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Interface Exposure Examples](../references/00032-upstream-interface-exposure-examples.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.
- [Contributor Source Patterns](../references/00033-upstream-contributor-source-patterns.md): load for the corresponding API/operation; retains methods, data shapes, examples, edge cases, and source anchors.

## Additional source maps

- [Foundation source and behavior map](../references/00034-foundation-source-map.md): load for Lib/Idea/Ingest/Inquire/Reactus/Frui/r22n source ownership, public surfaces, and exact verification boundaries.
- [Package manifest inventory](../references/00035-upstream-package-source-inventory.md): load for every current package, version, manifest, test location, workspace topology, and optional/planning status.
- [Framework package behavior supplements](../references/00041-framework-package-supplements.md): load for API/session/CSRF/email/admin/MCP/desktop semantics and source-specific limitations beyond the original discussion.

## Explicit source-to-target differences

- Existing React/Reactus/Frui/r22n runtime APIs are source behavior. The implemented UI uses stackpress-leptos plus framework-view; the blog has SSR/hydrate/CSR, while desktop is deferred and MCP unsupported (G52). Port exceptions and target limits are explicit in [runtime](runtime-and-portability.md).
- Existing Node transports/module loading, JS callable objects, CJS/ESM/ts-morph/Vite, Electron packaging, and npm release coordination do not automatically become Rust requirements.
- Existing contributor documents demand >90% coverage for all new changes. Rust policy is >=70% development and >=90% before publication; use [testing and quality](testing-and-quality.md).
- The imported KB contains 0.10.7/default literals; the original source capture recorded 0.10.8 manifests. These are historical version observations, separate from Rust's current 0.2.1 manifests. Do not unify them into a false current-version claim.
- The config catalog's allowlist language does not override source empty-access behavior. G42/G43 preserve configured protection and each route/event's responsibility; no new global default-deny policy is implied. See [project direction](project-direction.md) and the Rust session/API implementations.
- Source contracts distinguish architectural, implemented, tested, demonstrated, and supported. Preserve those evidence labels for Rust and each target/database/adapter.

See [source catalog](source-catalog.md) for authority, preserved non-code snapshots, historical source hashes, demotions and current authority.
