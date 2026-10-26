# Link and safe_href

Checked navigation with escaped labels. Source: [controls.rs](../../src/components/controls.rs).

```rust
use stackpress_framework_ui::{
    components::{Link, safe_href},
    leptos::prelude::*,
};
pub fn navigation() -> impl IntoView {
    assert!(safe_href("/ui/profiles?id=0"));
    assert!(!safe_href("javascript:alert(1)"));
    view! { <Link href="/ui/profiles" label="Profiles"/> }
}
```

## Props

Use with Leptos `view!`. Required props have no default; signal props accept the
conversion indicated below.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `href` | `String` via Into | Required destination, checked once. |
| `label` | `Signal<String>` via Into | Required reactive label. |

**Returns** A Leptos `impl IntoView`. Unsafe destinations produce an anchor without href and with `aria-disabled=true`.
Safe anchors work before hydration and can participate in the host's client routing.

## safe_href

`safe_href(href: &str) -> bool` checks navigation hygiene. It allows nonempty local relative,
absolute-path, query, and fragment links, plus explicit lowercase `https://` or `http://`
with nonempty authority and no `@`. It rejects whitespace/control characters, backslashes,
protocol-relative URLs, and a colon in a local path's first segment.

**Returns** True for the admitted forms, false otherwise. This is neither a full URL parser
nor origin authorization. `PublicLocation` is stricter and has a different purpose.

Related: [Button](button.md), [public locations](../../../framework-view/docs/api/public-location.md).
