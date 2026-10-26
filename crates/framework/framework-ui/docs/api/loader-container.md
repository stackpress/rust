# LoaderContainer

Retain useful content while replacement work is pending. Source: [controls.rs](../../src/components/controls.rs).

```rust
use stackpress_framework_ui::{components::LoaderContainer, leptos::prelude::*};
pub fn results() -> impl IntoView {
    view! { <LoaderContainer busy=true><p>"Previous results"</p></LoaderContainer> }
}
```

## Props

Use with Leptos `view!`. Required props have no default; signal props accept the
conversion indicated below.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `busy` | `Signal<bool>` via Into | Required state, exposed as `aria-busy`. |
| `children` | `Children` | Required content, retained while busy. |

**Returns** A Leptos `impl IntoView`. The wrapper is a section. It does not replace children with a spinner or disable
their controls; callers choose any additional behavior.

Related: [Loader](loader.md).
