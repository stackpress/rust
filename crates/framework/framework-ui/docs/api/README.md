# Stackpress UI API

Import from `stackpress_framework_ui` and its public `components`, `state`, `language`,
`routing`, and native-SSR `render` modules. Related data types/functions share their owning
surface's page. Private helpers, CSS constants, and adaptation modules are not caller APIs.

## Surfaces

Use [the mode matrix](../README.md#select-one-render-mode) before compiling a consumer.

| Surface | Public coverage |
| --- | --- |
| [Startup](initialize-ssr.md) | `render::initialize_ssr (native ssr)` |
| [LeptosRenderer](leptos-renderer.md) | `render::LeptosRenderer; new, prepare, PageRenderer impl (native ssr)` |
| [PreparedPage](prepared-page.md) | `render::PreparedPage; finish (native ssr)` |
| [Routing](routing.md) | `routing::RouteContext, provide_route_context, use_route_context, CatalogPath; View re-exports` |
| [Button](button.md) | `components::Button, ButtonType` |
| [Input](input.md) | `components::Input` |
| [FieldControl](field-control.md) | `components::FieldControl` |
| [Alert](alert.md) | `components::Alert` |
| [Loader](loader.md) | `components::Loader` |
| [LoaderContainer](loader-container.md) | `components::LoaderContainer` |
| [Link](link.md) | `components::Link, safe_href` |
| [List](list.md) | `components::List, ListItem` |
| [NotifierContainer](notifier-container.md) | `components::NotifierContainer` |
| [LayoutProvider](layout-provider.md) | `components::LayoutProvider` |
| [Layouts](layouts.md) | `components::LayoutBlank, LayoutPanel, LayoutHead, LayoutLeft, LayoutMain, LayoutMenu, LayoutRight` |
| [Theme](theme.md) | `state::Theme, ThemeState, use_theme` |
| [Notifications](notifications.md) | `state::NoticeKind, Notice, NotificationOptions, Notifications, use_notifications` |
| [Language](language.md) | `language::Language, PluralCategory` |
| [Language context/provider](language-context.md) | `language::LanguageContext, LanguageProvider, LanguageProviderProps, use_language` |
| [Translate](translate.md) | `language::Translate, TranslateProps` |

## Re-exports and examples

The crate root re-exports pinned `leptos` 0.8.22, `leptos_meta` 0.8.7, and `leptos_router`
0.8.17. Use their upstream APIs through those names when needed; this reference covers the
framework-owned integration rather than reproducing dependency manuals. With the facade,
imports use `stackpress::framework_ui` and a selected `framework-ui-*` mode feature.

Consumers using Leptos macros also need a direct `leptos = "=0.8.22"` dependency, with
default features off and the same selected mode. Macro expansion requires that crate name
even when source imports use the framework re-export.

Every Rust block includes its own imports and is an independent program or named consumer
function. Component helpers run inside a Leptos Owner supplied by the renderer/client root;
they are not global-state constructors. SSR blocks use native `ssr`; the hydration guide's
mount block is checked as wasm32 with `hydrate`. Shared UI examples also compile in WASM
hydrate and CSR consumers. No mode combination or `--all-features` build is valid.

Related: [Getting started](../getting-started.md), [Concepts](../concepts.md),
[View API](../../../framework-view/docs/api/README.md),
[canonical integration](../../../../../examples/profile-ui/README.md).
