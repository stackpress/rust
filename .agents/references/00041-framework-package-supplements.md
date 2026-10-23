# Framework package behavior supplements

Owner: [upstream KB](../context/upstream-stackpress.md). Load to map all framework packages to runtime/generated responsibilities and target-specific redesign. This is source-system knowledge; current Rust direction/policy takes precedence.

## Aggregate and server

- [Aggregate plugin](../../../stackpress/packages/stackpress/src/plugin.ts) registers server, schema, language, CSRF, SQL, view, session, API, and admin in explicit order. It exposes a facade without moving implementation ownership. AI and desktop are explicitly optional.
- [Server package](../../../stackpress/packages/stackpress-server/src) owns terminal bootstrap, develop/serve/emit operations, transport facades, and generation host integration. Event modules adapt request/response/status to reusable scripts.
- Current terminal bootstrap resolves config/listen/route after plugin bootstrap. Terminal requests are manually constructed. CLI operational errors have specific error-event behavior; serve/develop errors can be rethrown.
- Source CJS/ESM exports, root dependency-free npx payload, TypeScript config, and npm release machinery are existing packaging contracts, not instructions to reproduce them in Cargo.

## Schema and SQL

- [Schema](../../../stackpress/packages/stackpress-schema/src) owns normalized Schema/Fieldset/Model/Column/Attribute, semantic facets, dictionaries, assertions/defaults/serialization, generation and revisions. Unknown syntax metadata has no effect without an owning semantic consumer.
- [SQL](../../../stackpress/packages/stackpress-sql/src) generates stores/actions/listeners/scripts and orchestrates query/install/upgrade/push/migrate/populate/purge/uninstall.
- Generated selectors traverse relations and JSON; aliasing/scalarization reconstructs nested results. Create/update validation and soft remove/restore semantics belong to generated operations, not a generic ORM assumption.
- Migrations reconcile adjacent revision signatures and clear one-to-one field rename intent through Inquire builders. Ambiguity/destructive upgrades require explicit force; raw migration artifact generation has a different warning/application boundary.
- Revision history is not an applied-migration ledger; population lacks an automatic outer transaction. Generated client presence is executable runtime state, not passive types.

Load [schema API](00021-upstream-schema-api-contracts.md), [SQL API](00025-upstream-sql-api-contracts.md), and [operational examples](00030-upstream-operational-examples.md) before implementing their consumers.

## View and admin

- [View](../../../stackpress/packages/stackpress-view/src) configures Reactus, generated role components, shared server props, providers/layouts/theme/notification helpers, asset/dev/build integration, and browser-safe exports.
- [Admin](../../../stackpress/packages/stackpress-admin/src) generates model-specific page/view/route workflows: create/search/detail/update/copy/remove/restore/import/export and related records. It composes owned capabilities; generation alone does not prove accessible or authorized interaction.
- `field`, `filter`, `list`, `span`, and `view` are distinct component roles. Metadata/dictionaries select granular imports, prop contracts, labels, and fixed attributes. Existing fixed attributes override colliding Idea-provided props.
- Current server snapshots include configured/response data, public session, normalized request fields, and status/errors/results/totals. Browser wrappers are readonly helpers, not live server state.
- Existing rendering skips redirects/no-view requests/existing string bodies. Development paths use native Node resources and are not automatically portable.
- Rust should preserve selected behavior through Leptos, not expose raw native resources/secrets to browser props or port React APIs literally.

Load [view API](00027-upstream-view-api-contracts.md), [components](00031-upstream-idea-component-catalog.md), and [interface recipes](00032-upstream-interface-exposure-examples.md) for detailed shape and evidence requirements.

## Session, authentication, profile, and access

- [Session package](../../../stackpress/packages/stackpress-session/src/plugin.ts) composes auth, email, and session subplugins. Email is therefore indirectly reached through session even though it is not a direct aggregate registration.
- Auth listeners include signup/signin/signout; configured auth routes/views cover username/email/phone, link/OTP/2FA, account/profile/security/password/export/removal behavior.
- Signed identity Session is distinct from the low-level request cookie/session map. Current JWT signing/verifying, expiry, role grants, and cookies have exact source-specific behavior; do not assume a generic session middleware is equivalent.
- `can` requires every requested permission to match a grant. Event hyphen patterns and route segment patterns have distinct wildcard semantics; method ALL and regex strings are supported.
- Detailed Session source says an empty access map permits all. Config catalog wording describes an allowlist but is not a universal default-deny guarantee. Decide the Rust default explicitly and verify denials.
- Client permission checks are UX only. Internal resolve/emit does not automatically inherit caller authorization, identity, transaction, or audit context.

Load [session and language contracts](00028-upstream-session-language-contracts.md) and inspect current auth/session helpers, actions, guards, pages, config, and tests when implementing security-sensitive behavior.

## API and OAuth

- [API plugin](../../../stackpress/packages/stackpress-api/src/plugin.ts) exposes configured method/path→event mappings for public/app/session callers, fixed request data, priorities, scope policy, CORS, OAuth routes, and outgoing webhooks.
- App/session endpoint adapters resolve their own identity and secrets; scopes are per surface. Non-GET handling differs from GET. Event capability existence does not imply safe public exposure.
- Webhook listeners run after successful operation state by their registered priority and can filter/map results. External delivery/retry/idempotency guarantees must not be inferred from fetch source alone.
- Current CORS implementation needs concrete header/origin/method verification; do not promote an existing example or helper into a correctness claim.

Load [configuration](00023-upstream-configuration-catalog.md) and [surface examples](00032-upstream-interface-exposure-examples.md); inspect token/pages/helpers and generated models for exact OAuth/credential contracts.

## CSRF, email, and language

- [CSRF plugin](../../../stackpress/packages/stackpress-csrf/src/plugin.ts) registers clear/generate/valid mechanisms during config. It creates a random token, stores it in response cookie/session and SSR data, checks supplied/request token shapes and equal length before timing-safe comparison, and reports 419 on failure. Its availability does not automatically mean every route invokes validation.
- [Email plugin/events](../../../stackpress/packages/stackpress-email/src) register configured email-send behavior. The event skips existing result/error state, creates a NodeMailer transport from config, sends supplied options, and sets response results or propagates failure. A Rust transport must preserve needed event/result semantics rather than NodeMailer object types.
- [Language](../../../stackpress/packages/stackpress-language/src) selects/persists locale and can rewrite recognized locale-prefixed routes. Browser phrase translation belongs to r22n in the source architecture; server locale policy and client reactive translation remain separate concerns.

## Optional AI/MCP

- [AI plugin](../../../stackpress/packages/stackpress-ai/src/plugin.ts) activates with MCP config, registers the factory/transports during listen, contributes model tool generation during idea, and can attach HTTP/SSE to the existing server or use a dedicated configured listener.
- Serializable tool config includes event/plugin mode, public/app/user identity type, metadata/method, JSON Schema input/output, scopes, and fixed data. Generated tools supply candidate definitions; configuration selects exposure.
- Runtime normalizes/validates tool definitions, filters listing and call permissions for caller auth, validates input, merges fixed/caller/auth data, invokes the event, and converts results to MCP output.
- Stdio, streamable HTTP stateful/stateless, and legacy SSE have distinct lifecycle/session/shutdown contracts. Do not assume one adapter proves all transports or that tool authorization secures direct event calls.

Inspect helpers, transport scripts/events, generated tool listeners, and tests before choosing a Rust MCP SDK. No Rust MCP implementation is selected here.

## Optional desktop and planning-only Studio

- [Desktop package](../../../stackpress/packages/stackpress-desktop/README.md) currently uses Electron plus a local HTTP runtime. It owns app/window/preload/menu/route rules/build/packaging adapters while Stackpress owns operations/pages/data.
- Protocol mode and update events are reserved/incomplete. Current-platform unsigned directory packaging does not establish signing/notarization/updating or cross-platform parity.
- Route rules allow all when empty, switch to allowlisting when present, and validate the configured open route. Menu contributions have deterministic ordering and event-backed dispatch; contributions close when initialization is complete.
- The Rust target selects Tauri, so retain intended contribution/capability/route policy while redesigning Electron-specific mechanics. Do not blindly embed a local HTTP server or port old native defaults.
- `packages/stackpress-studio` contains future planning, not an active package or completed schema editor. Its archive/plans are not implementation authority.

## Source-version and verification scope

Current manifests report 0.10.8 while imported KB passages can retain 0.10.7 assumptions. Native/default/error behavior must be rechecked against current sources when coding. Every package's manifest/source anchors are in the [source inventory](00035-upstream-package-source-inventory.md) and [hash ledger](../resources/provenance/ingestion-manifest.json).

Source review does not mean tests/builds were run in the source projects. No source code was changed or archived. The Rust policy is 70% development / 90% publication coverage, plus meaningful contract evidence; old JS contributor thresholds do not govern the new monorepo.
