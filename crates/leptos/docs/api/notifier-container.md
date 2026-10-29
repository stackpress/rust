# NotifierContainer

Root-scoped immediate notifications with keyboard dismissal. Source: [controls.rs](../../src/components/controls.rs).

```rust
use stackpress_leptos::{components::NotifierContainer, leptos::prelude::*, state::Notifications};
pub fn notices() -> impl IntoView {
    let notifications = Notifications::new();
    view! { <NotifierContainer notifications/> }
}
```

## Props

Use with Leptos `view!`. Required props have no default; signal props accept the
conversion indicated below.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `notifications` | `Notifications` | Required root-owned store. |

**Returns** A Leptos `impl IntoView`. The aside has an accessible Notifications label and polite live region. Each row
uses its severity role and a native Dismiss button. It does not construct a global store,
auto-dismiss messages, or require a provider when the store is supplied explicitly.

Related: [Notifications](notifications.md), [LayoutProvider](layout-provider.md).
