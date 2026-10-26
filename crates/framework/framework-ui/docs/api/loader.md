# Loader

Polite text loading announcement. Source: [controls.rs](../../src/components/controls.rs).

```rust
use stackpress_framework_ui::{components::Loader, leptos::prelude::*};
pub fn pending() -> impl IntoView {
    view! { <Loader busy=true label="Searching…"/> }
}
```

## Props

Use with Leptos `view!`. Required props have no default; signal props accept the
conversion indicated below.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `busy` | `Signal<bool>` via Into | Required pending state. |
| `label` | `Signal<String>` via Into | `Loading…`; localizable text. |

**Returns** A Leptos `impl IntoView`. The `span` has role `status` and `aria-live=polite`. When idle its text is empty.
No timer, acquisition, animation, or operation state is created.

Related: [LoaderContainer](loader-container.md).
