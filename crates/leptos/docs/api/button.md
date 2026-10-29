# Button and ButtonType

Native button or checked anchor. Source: [controls.rs](../../src/components/controls.rs).

```rust
use stackpress_leptos::{components::{Button, ButtonType}, leptos::prelude::*};
pub fn actions() -> impl IntoView {
    view! { <Button button_type=ButtonType::Submit>"Search"</Button>
    <Button href="/ui/profiles".to_owned()>"Profiles"</Button> }
}
```

## Props

Use with Leptos `view!`. Required props have no default; signal props accept the
conversion indicated below.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `button_type` | `ButtonType` | `ButtonType::Button`; native button semantics. |
| `href` | optional `String` | Absent; when supplied selects anchor mode. |
| `disabled` | `Signal<bool>` via Into | False. |
| `id`, `name`, `value` | optional `String` each | Absent; name/value apply to button mode. |
| `class` | `String` | Empty, added after `sp-button`. |
| `children` | `Children` | Required deliberate label/views. |

**Returns** A Leptos `impl IntoView`. Unsafe/disabled anchors lose href and their tab stop and expose `aria-disabled`.
Native buttons use the disabled attribute. Attach ordinary Leptos event attributes to choose
application actions; no operation is inferred.

## ButtonType

The Copy/Clone enum has `Button` (default), `Submit`, and `Reset`. `as_str(self)` has no
additional parameters and returns `&'static str`: `button`, `submit`, or `reset`. It accepts no
arbitrary HTML type token. `Button` defaults to a non-submitting action.

Related: [safe navigation](link.md), [forms guide](../guides/compose-a-root.md).
