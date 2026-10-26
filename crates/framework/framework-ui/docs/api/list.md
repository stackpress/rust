# List and ListItem

Keyed native list with explicit empty-state text. Source: [controls.rs](../../src/components/controls.rs).

```rust
use stackpress_framework_ui::{
    components::{List, ListItem},
    leptos::prelude::*,
};
pub fn rows() -> impl IntoView {
    let rows = vec![ListItem {
        id: "abc".into(),
        label: "Ada".into(),
        href: Some("/ui/profiles/abc".into()),
    }];
    view! { <List items=rows empty="No profiles"/> }
}
```

## Props

Use with Leptos `view!`. Required props have no default; signal props accept the
conversion indicated below.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `items` | `Signal<Vec<ListItem>>` via Into | Required ordered rows with unique stable IDs. |
| `empty` | `Signal<String>` via Into | Required safe localized empty-state text. |

**Returns** A Leptos `impl IntoView`. It renders a native `ul`/keyed rows and hides empty-state text when rows exist.
Retained keys read current label/href values rather than stale child-factory data.

## ListItem

The Clone/Debug/PartialEq/Eq row has public fields `id: String`, `label: String`, and
`href: Option<String>`. A present href uses Link's safety policy; absent href renders text.
IDs must be unique within the collection; List does not validate/deduplicate them.
The domain-neutral row markup uses `data-item-id` for the supplied stable key.

Related: [Link](link.md).
