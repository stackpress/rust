# Frontend options: Leptos, Tauri, and Vite

Current preference and research snapshot: 2026-10-23.

## Author's latest position

- "Leptos looks good actually. I can see myself using it."
- The author asked why Leptos would be recommended over Tauri + Vite.
- The author expressed concern about Dioxus's direction, describing a perceived pivot by its owners.
- The author also dislikes Dioxus's syntax.

Treat the Dioxus direction statement as the author's concern, not independently verified project news. No investigation into its maintainers' plans was performed. The syntax preference alone is a sufficient reason to evaluate another option.

The author subsequently said "okay sold with leptos + tauri." This is now the selected architecture. The historical comparison below explains the choice; it is not an unresolved selection task. No comparative application or integration build has been performed.

The author also requires the same application code to be usable across web/mobile/desktop except hardware-specific code. Shared components, contracts, domain operations, and suitable validation should be authored once; platform and execution-host differences belong in adapters. Code reuse does not mean a browser can directly execute secret-bearing server/database code or use an identical native binary.

## Compare the layers correctly

| Layer | Purpose | Examples |
| --- | --- | --- |
| Reactive UI framework | Components, state/reactivity, templates, browser interactions | Leptos, React, Vue, Svelte |
| Frontend development/build tool | Compile/bundle/watch assets and support development | cargo-leptos, Trunk, Vite, depending on stack |
| Installed-app host | App lifecycle, WebView, native commands, packaging | Tauri |
| Server capability framework | Events, plugins, routing, data/auth/config | Proposed Rust Stackpress |

Leptos can be used with Tauri. "Tauri + Vite" does not identify a reactive template language until its UI framework is named. Vite can build a React/Vue/Svelte/plain-TS frontend; it does not supply their component/reactivity model itself.

The meaningful comparison is:

- Rust Stackpress + Leptos, with Tauri for installed applications.
- Rust Stackpress + a named JS/TS UI framework built with Vite, with Tauri for installed applications.

Leptos does not inherently require Vite. Exact frontend tooling remains open; the documented Tauri Leptos integration uses Trunk, while the SSR web build can use Leptos-specific tooling.

## Reasons to favor Leptos for this project

These are engineering judgments based on the author's priorities, not measured wins:

1. **One principal implementation language.** Framework, schema-generated types, UI components, and selected local host code can use Rust. Avoid a mandatory Rust-to-TypeScript type-generation layer for shared values, while keeping real serialization boundaries explicit.
2. **HTML-like templates.** Leptos's `view!` style presents markup in an HTML-like shape rather than Dioxus's `rsx!` style. This appears closer to the author's stated syntax preference, but should be validated with real forms and conditional/list rendering.
3. **Website SSR and hydration.** Leptos provides a Rust-based way to render initial HTML and make it interactive in the browser, matching the behavioral goal behind Reactus.
4. **Potentially reusable UI source.** Components and public presentation models can be shared by web and Tauri builds, with different data/host adapters.
5. **Generation alignment.** Idea transforms could emit Rust model types, Leptos form/list/detail components, and server operations within a common Rust codebase.
6. **Explicit framework boundaries.** Stackpress can remain the domain/capability owner, with Leptos managing UI state and rendering.

None of these means the frontend and backend execute in one address space or can share arbitrary live objects. Browser/WASM and Tauri IPC still serialize data. Shared Rust types do not eliminate versioning, authorization, schema compatibility, or client/server trust boundaries.

## Reasons Tauri + Vite + a JS UI framework may be preferable

1. **Preserve existing components.** React/Frui/r22n components can potentially be reused with a React frontend; Leptos requires rewriting their UI implementations or deliberate interop.
2. **Reuse existing JS libraries.** Rich editors, charts, complex grids, and other packages can integrate more directly into their native JS ecosystems. Verify the actual required libraries rather than assuming ecosystem availability.
3. **Familiar frontend workflow.** For a JS-experienced team, a Vite-based project may reduce the initial transition burden.
4. **Template choice stays flexible.** Vue or Svelte may satisfy the desire for HTML-like templates while retaining JavaScript/TypeScript tooling. Do not reduce the alternative to React only.
5. **Separate adoption decisions.** The Rust backend can preserve Stackpress's architecture while the frontend remains JS. Backend language and UI language need not migrate together.

A JS frontend needs public API/IPC contracts and may benefit from generated TypeScript types. That is additional coordination, but it can be an acceptable and inspectable design.

Rust/WASM rebuild times, browser integration ergonomics, hydration debugging, and the availability of required components should be measured in a proof rather than assumed superior. No performance or productivity benchmarks were run.

## Website and installed applications use different hosts

### Website

```text
HTTP request
  → Stackpress request/auth/plugin pipeline
  → operation results and explicit view data
  → Leptos SSR
  → initial HTML + safe serialized data + browser assets
  → hydration and reactive browser interactions
```

This retains the server-first website behavior. Decide how Stackpress page selection cooperates with Leptos routing and metadata so they do not become competing routing authorities.

### Installed application

```text
Tauri application
  → packaged HTML/CSS/JS/WASM assets
  → Leptos client rendering
  → operation interface
    → remote Stackpress API
    → explicitly permitted Tauri command for local Rust capability
```

Tauri is a system-WebView application host. Its ordinary packaged frontend does not natively run the website SSR server. A separate embedded-server arrangement would be a different architecture requiring its own justification and proof; it is not assumed here.

Leptos client rendering and web hydration are distinct builds/modes. Reuse source where possible, but do not claim the exact same entrypoint or build output serves both automatically.

Installed apps also require platform integration for signing, permissions, navigation, notifications, file access, and other device behavior. Shared UI does not mean no platform-specific code or testing.

The author has not yet decided whether local capabilities, remote capabilities, or offline operation are required.

## Comparison table

| Question | Leptos + Tauri | JS UI + Vite + Tauri |
| --- | --- | --- |
| UI authoring | Rust and HTML-like view templates | Chosen JS framework's syntax |
| Existing React components | Rewrite or deliberate interop | Potential reuse if React is selected |
| Shared Rust model types | Directly import client-safe shared crate | Generate/map TS contracts as needed |
| Packaged host | Tauri | Tauri |
| Native widgets | WebView UI, not automatic OS-widget rendering | Same |
| Website server-first rendering | Leptos SSR build | Needs a renderer/framework SSR arrangement; Vite alone is not the application renderer |
| Normal packaged app rendering | Local/client rendering | Local/client rendering |
| External browser/IPC boundary | Remains explicit | Remains explicit |
| Third-party JS UI code | Requires evaluated integration | Usually closer to the library's intended environment |
| Rust port scope | Backend plus UI/generators | Backend, with JS UI retained |

## Selected direction

Use **Leptos as the UI layer and Tauri as the installed-app host**, as selected by the author. Keep view and operation transport boundaries clear.

This selection supersedes the earlier assistant Dioxus recommendation and the earlier tentative Leptos evaluation. The exact integration APIs, build tooling, target order, and offline/local behavior remain open. Selection of the stack is not a request to start implementation.

Use Leptos for Rust types/templates/reactivity and Rust website SSR, not as a claim that it replaces native packaging or guarantees a better development experience than every JS frontend.

## Focused integration proof

If the author requests implementation, use one realistic slice:

- A profile-search screen with query input, loading state, results, validation, an error, and navigation to a detail screen.
- One generated model and one framework event invoked through the same operation interface.
- Website: data present in initial HTML and correct hydration without a duplicate initial fetch.
- Installed app: shared component source renders locally and calls the chosen remote/local adapter.
- One plugin modifies the operation without editing the original plugin.
- Compare code readability, generated output, rebuild/refresh experience, dependency footprint, and one real third-party UI integration.
- Test a chosen first mobile/desktop target, then add the remaining platforms before declaring coverage.

The Vite/JS alternatives are retained as historical rationale, not the current implementation direction. Reopen that decision only if the author asks or a concrete integration blocker requires reconsideration.

## Sources and evidence limits

Checked during this conversation:

- [Leptos introduction](https://book.leptos.dev/): Rust web framework, reactive UI, SSR/hydration.
- [Leptos basic component](https://book.leptos.dev/view/01_basic_component.html): view/component authoring.
- [Leptos async SSR](https://book.leptos.dev/ssr/23_ssr_modes.html): rendering behavior with asynchronous data.
- [Tauri frontend overview](https://v2.tauri.app/start/frontend/): frontend assets, framework choice, normal SSR boundary.
- [Tauri Leptos guide](https://v2.tauri.app/start/frontend/leptos/): documented integration and Trunk example. The page explicitly states it is accurate as of Leptos 0.6; do not copy its version-specific setup as verified-current instructions.
- [Tauri overview](https://v2.tauri.app/start/): installed-app host and platform purpose.
- [Vite guide](https://vite.dev/guide/): development/build tooling.
- [Dioxus renderers](https://dioxuslabs.com/learn/0.7/beyond/project_structure/): earlier candidate's renderer distinctions, not evidence about a maintainer pivot.
- [Dioxus native clients](https://dioxuslabs.com/learn/0.7/essentials/fullstack/native/): earlier comparison of installed-client behavior versus web SSR.
- [Slint FAQ](https://slint.dev/faqs): earlier alternative's declarative GUI/platform scope.

No application builds were performed. Platform documentation establishes candidate capability, not tested integration or exact parity with Stackpress.
