# Foundation and package source map

Use when locating a semantic owner before implementation. Source roots below are canonical sibling checkouts; source code was not copied into resources. The [manifest and hash ledger](../resources/provenance/ingestion-manifest.json) records capture revisions and file hashes.

For the Rust target, read [accepted source structure](source-structure.md) for the root facade, grouped crate tree, package/path catalog, and responsibility boundaries. Upstream names below remain source locators: Ingest maps to `server/server`, Inquire to `sql`, and Stackpress framework packages to the `framework` family.

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

Read [package behavior supplements](../references/00041-framework-package-supplements.md) for explicit per-package runtime/generator responsibilities and optional-package limits. The [upstream KB router](upstream-stackpress.md) links all detailed APIs, configuration, examples, and generation contracts.

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
