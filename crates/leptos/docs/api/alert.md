# Alert

Escaped public status/error text. Source: [controls.rs](../../src/components/controls.rs).

```rust
use stackpress_leptos::{components::Alert, leptos::prelude::*, state::NoticeKind};
pub fn failure() -> impl IntoView {
    view! { <Alert message="Search unavailable" kind=NoticeKind::Error/> }
}
```

## Props

Use with Leptos `view!`. Required props have no default; signal props accept the
conversion indicated below.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `message` | `Signal<String>` via Into | Required public plain text. |
| `kind` | `NoticeKind` | `Error`; severity label/role. |

**Returns** A Leptos `impl IntoView`. The paragraph is hidden when message is empty. Error uses role `alert`, while
Info/Warning use `status`. Visible severity text does not rely only on color. It does not
inspect or stringify private native errors.

Related: [NoticeKind and notifications](notifications.md).
