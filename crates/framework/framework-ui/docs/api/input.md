# Input

Controlled native text input. Source: [controls.rs](../../src/components/controls.rs).

```rust
use stackpress_framework_ui::{components::Input, leptos::prelude::*};
pub fn search_input() -> impl IntoView {
    let value = RwSignal::new(String::new());
    view! { <Input id="query" name="id" value described_by="query-help query-error"/> }
}
```

## Props

Use with Leptos `view!`. Required props have no default; signal props accept the
conversion indicated below.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `id` | `String` via Into | Required unique native DOM ID. |
| `name` | `String` via Into | Required form key. |
| `value` | `RwSignal<String>` | Required controlled root-owned value. |
| `described_by` | `Signal<String>` via Into | Empty; space-separated help/error IDs. |
| `invalid`, `disabled` | `Signal<bool>` via Into | False. |
| `placeholder` | `String` | Empty; supplementary to a label. |

**Returns** A Leptos `impl IntoView`. It seeds SSR with the value attribute, updates the browser property, and writes input
events back to the signal. It does not add a label, infer validation, decode form data, or
automatically associate FieldControl help/error IDs.

Related: [FieldControl](field-control.md), [theme and root ownership](../concepts.md#client-state-belongs-to-one-root).
