# Slice 5 r22n and locale inventory

Companion to [decisions](slice-5-design-decisions.md). Read-only `../r22n/src` at HEAD `48c50dd1ba57a0555c9afbd4bfa4cb41e702cb68`, package 1.0.10, inspected 2026-10-25. Read `components/{R22nProvider,Translate}.tsx`, `hooks/useLanguage.tsx`, context/types/index. No tests directory was found in this checkout; behavior below is source-observed and must receive new meaningful Rust/browser tests.

| Export/type/member or source behavior | Disposition and reason |
| --- | --- |
| `Language` | Now: bounded owned locale key and phrase map as public snapshot data, independent of request/session config. |
| `R22nContextProps` | Now: explicit scoped locale/placeholder/phrase-map/change operation. React Context shape not ported; Leptos context/signals own reactivity. |
| `R22nProviderProps` | Now: checked locale/phrases/placeholder and Leptos children; no ReactNode ABI. Defaults are explicit snapshot values. |
| `R22nContext`, default `context` export | Now: per-root context with useful missing-provider diagnostic; no shared mutable locale singleton. |
| `R22nProvider` | Now: initialize from snapshot, replace locale/phrase map reactively; missing optional phrase map becomes empty. Default locale `en_US`, placeholder `%s` are accepted only as explicit UI defaults, not host locale detection. |
| `useLanguage`, returned `language`, `changeLanguage` | Now: idiomatic Leptos lookup/update handle, deterministic initial state. Changing language with no map clears translations as source does; no network or cookie read hidden in the helper. |
| `template(phrase)` | Now: phrase-key lookup with original phrase fallback; split the selected translation at configured placeholder and compose escaped text plus deliberate view slots. Empty placeholder rejected. |
| returned template function and variables | Now: safe text/node interpolation preserving order; zero/false/empty must not be dropped by source's truthiness check. Deliberate D08 correction; excess/missing slot policy must be explicit and tested (recommend excess ignored, missing slot empty, no HTML evaluation). |
| tagged-template `t` | Now equivalent phrase-plus-slot builder/macro or named Rust helper; JS TemplateStringsArray/tagged-call syntax not ported. Phrase construction/placeholder identity stays stable for dictionaries. |
| string helper `_` | Now explicit text-only formatter returning String; no unchecked cast of mixed React nodes to string. Reject node content on this API. |
| `Translate` component, `values` and children parsing | Now explicit Leptos phrase/key and child slots; automatic ReactNode child-type introspection is not ported. Named typed props remove ambiguity between text and interpolated nodes. |
| attempted leading/trailing `trimStart/trimEnd` without assignment | Not ported as a whitespace mutation guarantee: the source calls do not change the strings. Preserve exact source phrase whitespace by default; a trim option needs a separate contract. |
| client node interpolation/all-string optimization | Now equivalent escaped text-only fast path and explicit node composition; React fragment/spread mechanics not ported. Identical initial server/browser text is mandatory. |
| locale persistence, server language selection/prefix rewriting | Later S5-F4 in framework-language/session. r22n is a rendering/phrase tool, not authorization/session policy. Do not infer locale from browser-only globals before hydration. |
| language picker/EN-TH defaults from LayoutUser | Later S5-F4, configurable and separated from generic language context. No hard-coded account routes/roles in framework-ui. |

## Proving tests

Test missing/current phrase fallback, exact whitespace/Unicode, placeholders at beginning/middle/end, repeated placeholders, custom nonempty placeholder, missing/excess variables, text versus real Leptos nodes, zero/false/empty values, malicious strings and no raw HTML. Verify changing locale/map updates all mounted consumers without global cross-root leakage and unmount releases subscribers.

Render the same translation fixture into SSR HTML, hydrate it without changed text/node structure, and mount it in Tauri CSR. Test two concurrent native page renders with different locales. Translation maps must fit snapshot budgets and contain no executable markup. No claim of plurals, ICU/date/number locale support, automatic dictionary fetching or RTL layout support follows from simple phrase replacement; those need later explicit contracts (S5-F2/S5-F4).
