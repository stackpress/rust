# Slice 8 architecture and cross-slice contracts

Proposal behind [owner decisions](slice-8-design-decisions.md). Preserve current synchronous APIs and wire shapes; add optional capabilities after their proofs. Source-era KB claims that no Cargo code exists are historical, not current implementation evidence.

## Preserve the accepted foundation

- `framework-view` remains typed public page contracts plus its optional native server extension; `framework-ui` contains components, state, language, routing and renderer modules. Components know neither Profile nor an operation endpoint.
- Existing `PageRenderer<T>` returns a bounded complete `RenderedPage`; do not make it secretly stream. `PageSpec` borrows a Send loader, projects allowlisted owned data, preserves Abort, and delegates within one G12/G17 lifecycle.
- Each root has one writer. UI/controller decisions remain natively tested; browser modules only adapt DOM, fetch and fixed IPC. Concurrent independent SSR roots are supported; arbitrary simultaneous writers to one root are not.
- Keep the current renderer's eager owned Head, cleanup-before-assembly, typed Head failure and caller-Owner restoration. Async rendering gets a new explicit boundary; do not quietly lengthen the old renderer's Owner lifetime.
- Use [issue #11 wrapping policy](tasks/00010-receipt-lib.md#11-error-wrapping-policy-for-lane-2): propagate existing framework errors when adding no context; otherwise describe this operation once and retain its concrete source. Public projection never includes developer causes/frames/paths.

## Packages and features proposed for coordinator review

| Owner | Responsibility | Dependency boundary |
| --- | --- | --- |
| `framework-ui` | Typed components; pure controllers; optional rich-content/formatting modules; existing independent ssr/hydrate/csr modes | Optional pinned pure-Rust sanitizer/formatters; target-specific DOM bindings. No schema parser, database, native Tauri host or generator runtime edge. |
| `framework-view` | Existing public snapshot/route/asset policy; additive settings projection and later async/stream renderer registration | Default remains host/renderer independent; native `server` integration remains optional. No arbitrary request/session snapshot. |
| New `framework-language` | Checked locale registry/aliases, selection and public settings projection; optional native cookie/action adapter | UI phrase rendering remains in framework-ui. No process-global mutable locale or identity dependency; browser gets only public dictionaries/settings. |
| New `framework-admin` | Client-safe paging/filter/form/status DTOs, layout/action visibility; optional native route/policy composition | Generated operation contracts are supplied by the app. Default frontend graph contains no store/driver/server. No generic event name from caller input. |
| Native build-time UI generator under `tools/ui-codegen` | Function-registered view/admin transforms consuming Slice 6 metadata and G20 owned outputs | Tool only: schema/Idea transformer dependencies never enter runtime components or generated frontend contracts. Final package name/path requires coordinator approval. |
| Example app contracts/shared/client/web/native | One multi-model generated CRUD vertical, fixed typed adapters, application policy, preferences and proofs | Domain/store source comes from Slice 6. Do not grow the profile example gateway into Slice 7's production host. |

Introduce a package only when its milestone implements real behavior. A shared facade remains optional; root manifests/lock/facade have one coordinator-assigned owner. Review feature-unified client trees as well as individual crate defaults.

## Required interface from Slice 6

These are requests, not claims about the parallel design. Generation cannot freeze until the coordinator reconciles them with task 00011.

| Interface needed | Required semantics / proving handshake |
| --- | --- |
| Versioned normalized schema model/field identities | Stable safe names and source spans; nullable/optional/default/sensitive/read-only/write-only information; typed field paths, composite keys and relation cardinality. Frontend outputs expose no store, credentials, private defaults or compiler AST. |
| Five distinct role selections | `field`, `filter`, `span`, `view`, `list`, plus ordered participating fields. Preserve source distinction: filter/span derive field dictionaries; list derives view dictionaries. No missing-role synthesis. |
| Registered component definition | Rust component key/import path, role, typed prop schema/order, aliases, fixed props and validated serialized values; Idea props first, fixed props second. Never treat an untrusted schema string as Rust source or a runtime import. |
| Compile-time role registry extension | Function-registered definitions with duplicate/unknown component diagnostics; custom app component mapping independent of the framework default dictionary. Unsupported explicit roles fail generation with a span, rather than disappear. |
| Typed operations and public DTOs | CRUD/search/filter/sort/skip/limit/total, relation lookup and related operations; field errors; omitted versus null versus supplied values; safe result projection; actual soft-remove/restore capabilities. Do not infer total from returned row count. |
| Concurrency and write outcomes | Document whether updates support an expected-version condition. UI must report stale data if provided; otherwise disclose last-write behavior. Cancel/timeout can leave write outcome unknown; no automatic mutation retry. |
| Generation ownership | A contracts partition safe for WASM and independent native stores/server registration; deterministic ordered transforms, owned output/clean/check/model-removal behavior. UI consumes schema objects, not a second Idea parser. |

Before M4, compile a tiny role fixture provided through the coordinator, including aliases/fixed-prop collision/missing roles and a hostile label. Before M5, replace every adapter fixture with actual generated contracts/operations. Without these signals, component/gallery work can proceed; generated CRUD remains blocked.

## Required interface from Slice 7

Request an additive, transport-neutral owned streaming response path; never weaken Handler `Send` or retrofit network policy into a component.

Illustrative shape, not a required frozen public spelling:

```rust
// Proposed native-only boundary. No request, response or Invocation borrow escapes.
type PageChunks = std::pin::Pin<Box<dyn futures::Stream<
    Item = Result<bytes::Bytes, stackpress_lib::Error>
> + Send + 'static>>;

struct PreparedStream {
    status: u16,
    headers: http::HeaderMap,
    chunks: PageChunks,
}
```

Need explicit pre-commit staging, one header/status/cookie commit, bounded chunk/total bytes and admission, pull-driven backpressure, first-byte/idle/total deadlines, disconnect/deadline/shutdown cancellation and deterministic Drop/reaping. Response hooks complete before commit; nested operations stay raw. An error before commit follows fresh-response replacement; an error after commit cannot rewrite status or replay hooks. The coordinator must reconcile that rule with G12 before streaming M0.

Static assets need trusted manifest/MIME/hash/base-path handling and explicit HTML no-store when preferences or caller data are present. CSP must support the selected Leptos serialization mechanism without a blanket script exception. If resource replay emits inline scripts, require per-response nonce integration and prove it in real browsers; otherwise keep that mode unavailable.

Tests can consume an owned stream through an injected finite sink and `tokio::io::duplex`; no localhost listener is required. A collected `String` proves async buffering only, never network progress/backpressure.

## Async renderer lifetime proposal

Provide a separate opt-in async renderer, retaining one request-local Owner until its body, resource serialization and metadata are finished or cancelled. It receives only owned public loader results or explicitly owned capability clients; a `'static` Resource must never capture borrowed Request/Response/Invocation.

The native top-level future must remain Send. Enter/restore the Owner at synchronous poll/callback boundaries, including unwind, and never keep an entered thread-local context guard across await. Schedule resource completions through the owning root rather than permitting concurrent writes. A real Resource may spawn work differently from a delayed Suspend; M0 must prove this exact scheduling/disposal contract. If it cannot, stop with evidence and compare a bounded thread-confined render executor with deferral; no unsafe Send assertion, global render mutex or leaked Owner is an acceptable shortcut.

Buffered async is first: await permitted resources, settle owned Head, serialize public resource data under limits, emit one complete document. Then opt-in in-order streaming with a precommitted owned Head/status. Reactive updates to Head after commit are not promised. Out-of-order fragments, indefinite resources, HMR and arbitrary server functions are proposed later work.

## Persistence and permission policy

Website locale selection: recognized route prefix > valid stored preference > configured default. An explicit picker action validates the locale, persists it, then updates dictionary/`lang`/Head/links in one root action. Prefixes and aliases come from a compiled registry; unknown prefixes remain normal routes/404s. Preserve query, mount and return location via existing strict codecs. Expand the compiled catalog for locale paths; do not call a second top-level handler from a request hook to rewrite routes.

Keep legacy unprefixed routes readable. Add a page-settings projector based on the loader's explicit public data; existing renderer-wide fixed settings remain valid defaults. Seed the exact selected locale/theme before hydration; perform no initial cookie/localStorage read or locale fetch.

Recommend same-origin POST `/ui-preferences` with fixed locale/theme fields, exact Origin policy and validated CSRF protection where session context applies; cookie `Secure; HttpOnly; SameSite=Lax`, app mount Path and explicit one-year max age. Local development HTTP can explicitly omit Secure. This is a presentation preference, not authentication. Failed persistence retains current useful content and unsaved fields and announces that the choice was not saved. No implicit retries or redirect to a caller-supplied URL.

Theme uses the same policy. Flash carries a bounded safe public notice/id, is consumed by the host into a snapshot and cleared with the same cookie scope; deduplicate within one root. Cross-tab concurrent cookie reads cannot guarantee exactly-once delivery; durable notification delivery is separate work. No private errors or credentials go into cookies or seeds.

For generated installed-app CRUD, propose an explicitly configured remote typed host with app-supplied authentication and fixed origin/capability policy; preserve existing local read-only search. No generic remote URL proxy, stored server credential in WASM, local writes or offline synchronization is implied. Native local CRUD is a separate owner choice requiring per-target database/migration/policy proofs.

Installed CSR waits for a narrow native `load_preferences` result before first mount. `update_preferences` accepts only registered locale/theme, writes a bounded versioned file under the app config directory and returns approved public values. No arbitrary path, shell, filesystem plugin or event grant. File failure is visible; host restart persistence and corrupted-file recovery require real packaged proofs. Use an app-config file, not the signing key store, for these nonsecret settings.

Permission-aware layout consumes an explicit safe action/route projection from authoritative app policy. Hiding controls never authorizes work. Generated page/mutation adapters require host identity/authorization and web CSRF checks before event admission; absent policy disables protected exposure. Full signed identity, account flows, JWTs, wildcard grants and recovery are a separately proposed authentication task. A test-only policy is labeled and never the production default.

## G23 shared-crate requests

| Request to coordinator | Why it is needed / scope |
| --- | --- |
| Root Cargo/lock/facade | Serialize optional rich/locale/admin/tool dependency additions and explicit facade exports. Do not alter default native/browser isolation or upgrade Leptos/Tauri/bindgen incidentally. |
| Slice 6 schema and generator integration | Agree on the role/operation contracts above; add schema definitions in Slice 6 ownership, UI transforms in Slice 8 ownership. Any Idea/transformer change is a separate sequenced request, not a UI-lane edit. |
| Lib/server streaming path owned by Slice 7 | UI needs owned Send chunks plus commit/cancel/drop semantics. No shared-core change for components, locale pure policy or buffered gallery. UI lanes stop before editing lib/server/HTTP. |
| Additive framework-view settings/async integration | Coordinator assigns one owner for loader-selected settings and later streaming page registration. Preserve existing trait and serde shape; any new wire revision receives drift/recovery tests. |
| CLI/Idea | No change currently justified. Reuse app-owned codegen via existing `emit generate --check true`; request only concrete compiler/fixture-proved gaps. |
| Existing profile example | New proofs must not rewrite accepted search contracts/build identity without coordinator ownership. Prefer a separate generated-admin example once Slice 6 provides its multi-model vertical. |

No source change is requested merely to simplify a speculative sketch. Package creation and dependency approval are part of M0 sequencing, not accomplished by this document.
