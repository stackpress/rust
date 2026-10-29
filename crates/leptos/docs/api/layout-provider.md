# LayoutProvider

Theme/notification contexts and neutral root styles. Source: [layout.rs](../../src/components/layout.rs).

```rust
use stackpress_leptos::{components::LayoutProvider, leptos::prelude::*, state::{Notifications, Theme, ThemeState}};
pub fn root() -> impl IntoView {
    let theme = ThemeState::new(Theme::Light);
    let notifications = Notifications::new();
    view! { <LayoutProvider theme notifications><p>"Welcome"</p></LayoutProvider> }
}
```

## Props

Create these state handles inside the active root Owner.

| Prop | Type | Meaning |
| --- | --- | --- |
| `theme` | `ThemeState` | Required explicit initial theme. |
| `initial_theme_pending` | `Signal<bool>` | Optional false; admits the frozen document theme until the application adopts it. |
| `notifications` | `Notifications` | Required root-local immediate store. |
| `children` | `Children` | Required subtree, evaluated after contexts are provided. |

**Returns** `impl IntoView`: one neutral `Style` resource with ID `stackpress-ui` and a
`sp-root` wrapper whose `data-theme` follows the state. Provide one per root. It supplies no
fonts/images/brand palette, cookie/persistence, language context, or automatic notifier.
For Head-using components outside native rendering, supply one client meta context.

Context consumers use `use_theme` and `use_notifications`, which return useful missing-provider
errors. The provider shares the supplied handles; it does not create a process-global registry.

Related: [layouts](layouts.md), [theme](theme.md), [notifications](notifications.md).
