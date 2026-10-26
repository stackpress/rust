# Slice 5 decisions and recommendations

Design checkpoint for [task 00008](tasks/00008-slice-5-ui.md), assessed 2026-10-25 on `trial/3`. This is a proposal for coordinator review and owner decisions, not implementation approval, Freeze, visual acceptance, or a platform-support claim. [Context](../../context/index.md) and accepted G11–G23 in [decisions](decisions.md) govern. No production source, manifests, lockfile, upstream checkout, or other lane's documents were changed.

## Read by question

- [Assessment and disposable evidence](slice-5-design-assessment.md): load for versions, MSRV, installed tools, actual compiler results, buffered SSR and installation requests.
- [Architecture and boundaries](slice-5-design-architecture.md): load for page/lifecycle/routing ownership, snapshot security, target dependencies, operation transports, cancellation and G23 changes.
- [API and author experience](slice-5-design-api.md): load for a framework-view page, shared component/operation call and ServerBuilder registration examples. New API names are sketches.
- [Reactus and view inventory](slice-5-design-inventory-view.md): load for every Reactus export/type, view contract, desktop responsibility and source behavior disposition.
- [Frui inventory](slice-5-design-inventory-frui.md): load for the per-module export/type inventory and named component follow-ups.
- [Localization inventory](slice-5-design-inventory-language.md): load for r22n exports, interpolation differences and locale ownership.
- [Risks, milestones and acceptance](slice-5-design-verification.md): load for prevention/proving tests, fail-fast policy, two lanes and owner-visible acceptance criteria.

## Recommended first vertical

Use the existing generated Profile `id`/`name` contracts and the existing in-memory, case-sensitive ID-prefix `profile-search` operation. Preserve seed order, configured result limit, empty-ID rejection, row total and plugin customization. No SQL, new schema semantics, authentication implementation or duplicated domain operation is needed.

Website: data-bearing HTML → hydration from the identical public snapshot → typed browser HTTP calls. Installed macOS app: packaged CSR assets → two narrowly exposed native commands for search/cancel → the same generated operation source. UI source, wire contracts and applicable input validation are shared. Platform entrypoints/adapters are separate.

Start with asynchronous page-data loading followed by synchronous Leptos rendering. This keeps live request/response/Invocation values out of reactive state and retains existing Send handlers. A disposable async-render probe also passes, but general component-owned asynchronous SSR/resource serialization needs a separate M0 proof before inclusion. Buffering is adequate; streaming is not required to demonstrate this vertical.

## Structured owner choices

Each row is unresolved until the owner chooses it. Recommendations do not supersede [the ledger](decisions.md). The coordinator should record accepted choices there and link this packet from the spec index/task; those shared files remain coordinator-owned.

| ID / question | Choices | Recommendation and consequence |
| --- | --- | --- |
| S5-D01 / Q01: first installed target? | A: macOS ARM64 desktop. B: iOS simulator first. C: Android first. | **A.** Matches the installed native Rust target and Xcode host. Produce a local unsigned `.app` and exercise WKWebView; Intel/macOS universal, Windows, Linux, mobile, distribution signing and updates require later target proofs. |
| S5-D02 / Q02: execution and offline promise? | A: web uses HTTP; Tauri executes the read-only demo locally. B: both web and Tauri use a remote host. C: local/remote switching plus persistent offline synchronization. | **A.** Proves operation-source reuse without a network dependency for the installed demo. Local seed reads work with networking disabled; this does not promise offline persistence, queued writes, cache synchronization or remote fallback. B is smaller native exposure but does not prove a local capability. C requires a new product/consistency design. |
| S5-D03 / Q05: Tauri lifecycle? | A: permitted native entry calls existing `handle_with_options` once; nested dispatch stays raw. B: raw `emit` only. C: a new desktop-specific lifecycle. | **A.** Preserve G12/G17 exactly, including Abort, fresh error replacement and failed error-handler fallback. IPC denial before capability admission invokes no domain handlers. Public wire output and native diagnostics remain separate. |
| S5-D04 / Q11: routing, Head and SSR data? | A: Stackpress selects HTTP pages; a shared explicit route catalog drives Leptos client navigation; typed public snapshots seed hydration after a Send page loader. B: Leptos/server functions own all routing and operations. C: general component-owned async SSR from the outset. | **A.** Use `leptos_router` for browser/packaged navigation and `leptos_meta` for deduplicated Head. Native page loading and component actions use the same named typed search boundary; the component consumes the prefetched result on initial SSR/hydration. No duplicate initial call. General resource SSR is a named follow-up unless M0 is explicitly expanded. |
| S5-D05 / Q12: upstream UI breadth now? | A: selected primitives, basic layout, typed snapshot/client helpers, notification state and phrase/node translation, with explicit later inventories. B: all Frui controls, schema component dictionaries and admin CRUD generation now. C: only a hard-coded demo. | **A.** Ship useful framework contracts plus a realistic vertical; preserve the complete upstream inventory as tracked work. This explicitly reschedules the general component-generation work mentioned by Slice 3 to S5-F3 rather than claiming it delivered here. B adds several independent semantic/library integrations; C does not establish reusable DX. |
| S5-D06 / tooling: initial build route? | A: explicit Cargo + matching `wasm-bindgen` commands for website hydration and Tauri CSR, plus Cargo Tauri CLI. B: cargo-leptos for web and a separate CSR pipeline. C: latest Trunk. | **A for M0**, with scripts that expose exact commands and never install implicitly. B is viable after the custom host/asset metadata contract is proved; assess DX after M0. C currently requires Rust 1.96.1, above installed 1.96.0. No toolchain upgrade is implied by G09. |
| S5-D07 / visual demonstration: browser gateway scope? | A: add a bounded example-only HTTP gateway outside server, used solely for the live UI proof. B: require the deferred production network-adapter task first. | **A.** Browser hydration/HTTP acceptance needs a real origin. This is an explicit addition to the example's scope; G13's production gateway/streaming deferral remains. Later startup must use devmetrics and receive coordinator authorization; this checkpoint starts no listener. |
| S5-D08 / compatibility corrections? | A: explicit snapshot allowlist; escaped JSON/HTML; preserve zero/false interpolation; no arbitrary HTML/formula execution; explicit compiled exposure. B: reproduce source leakage/truthiness/raw-HTML behavior. | **A.** These are deliberate public-contract differences. Request/session/config forwarding, raw HTML and formula behavior are not silently copied from upstream. |

## Coordinator decisions within the approved scope

- Approve the dependency budget and exact feature/build matrix in [architecture](slice-5-design-architecture.md). Only direct SSR/header primitives and target adapters justified there enter the first implementation; dependency expansion returns for review.
- Keep lib/server/HTTP/CLI/Idea sources unchanged unless an M0 compiler result demonstrates a necessary shared change. Sequence any such request under G23. Root manifest, lockfile and facade changes have a single coordinator-assigned owner.
- Treat missing WASM tooling as a gate: native SSR/Tauri dependency compilation does not satisfy browser or packaged-app acceptance. Obtain installation approval separately.

## Proposed later work names

These are routing labels for proposed follow-up tasks, not approved new slice numbers or automatic commitments:

| Label | Owner/boundary |
| --- | --- |
| S5-F1 advanced components | `framework-ui`: disclosure/overlay/table/navigation controls and remaining input/view families; each with accessibility/behavior proofs. |
| S5-F2 rich content | Evaluated JS/editor/chart/media integrations, sanitized rich HTML/Markdown, complex numeric/date/phone behavior. |
| S5-F3 schema-to-UI generation | `framework-schema` semantic dictionaries plus view transforms; field/filter/list/span/view roles, aliases, prop precedence, generated compile/runtime tests. Requires separate approved task. |
| S5-F4 language/session integration | `framework-language`/session policy, locale routing/persistence, theme/flash cookies and permission-aware account/admin layouts. |
| S5-F5 native targets/integrations | Tauri mobile/other desktops, menus, device features, signing/notarization/updater and remote/native mode choices. |
| S5-F6 asynchronous UI rendering | General component-owned SSR resources, full hydration serialization, suspense/route-loader integration and streamed rendering only with the native streaming task. |
| S2-Network | Deferred production socket gateway, network read limits, disconnect/shutdown and streaming; serverless adapters separately. |

## Gate

Review this packet, resolve owner choices, approve required installs, then release **M0 only**. Stop after M0 for coordinator re-verification; do not proceed to broad implementation on the strength of disposable probes. Required by [workflow §5.1](../../workflows/coordinated-crate-development.md): the design checkpoint ends with “Stop.” Context promotion, commit/push, publication and deployment remain separate authorization.
