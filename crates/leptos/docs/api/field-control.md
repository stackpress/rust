# FieldControl

Label, help, and error slots for a native control. Source: [controls.rs](../../src/components/controls.rs).

```rust
use stackpress_leptos::{components::{FieldControl, Input}, leptos::prelude::*};
pub fn field() -> impl IntoView {
    let value = RwSignal::new(String::new());
    view! { <FieldControl for_id="query" label="ID prefix" help="Enter a prefix">
        <Input id="query" name="id" value described_by="query-help query-error"/>
    </FieldControl> }
}
```

## Props

Use with Leptos `view!`. Required props have no default; signal props accept the
conversion indicated below.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `for_id` | `String` via Into | Required target input ID. |
| `label` | `Signal<String>` via Into | Required visible label. |
| `help`, `error` | `Signal<String>` via Into | Empty. |
| `children` | `Children` | Required actual control. |

**Returns** A Leptos `impl IntoView`. Help/error IDs are `{for_id}-help` and `{for_id}-error`; the error paragraph has
role `alert`. Targets remain in the tree when empty. Pass those IDs to Input's `described_by`
and pass validation state to its `invalid`; FieldControl does not wire those props implicitly.

Related: [Input](input.md), [Alert](alert.md).
