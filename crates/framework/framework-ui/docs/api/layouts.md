# Semantic layouts

Explicit child slots with native landmarks. Source: [layout.rs](../../src/components/layout.rs).

```rust
use stackpress_framework_ui::{
    components::{LayoutHead, LayoutLeft, LayoutMain, LayoutMenu, LayoutPanel, LayoutRight},
    leptos::prelude::*,
};
pub fn layout() -> impl IntoView {
    view! {
        <LayoutHead><h1>"Profiles"</h1></LayoutHead>
        <LayoutPanel>
            <LayoutLeft label="Browse"><LayoutMenu label="Profiles"><a href="/ui/profiles">"All"</a></LayoutMenu></LayoutLeft>
            <LayoutMain><p>"Results"</p></LayoutMain>
            <LayoutRight label="Help"><p>"Choose a profile"</p></LayoutRight>
        </LayoutPanel>
    }
}
```

## Components and props

Every component takes required `children: Children` and returns `impl IntoView`. The three
named slots additionally require `label: String` via Into. No component infers account routes,
permission decisions, or remote data.

| Component | Additional props | Output |
| --- | --- | --- |
| `LayoutBlank` | None | Minimal `sp-blank` div; caller supplies a main landmark. |
| `LayoutPanel` | None | Responsive `sp-panel` div; compose left/main/right explicitly. |
| `LayoutHead` | None | Visible `header.sp-head`; separate from document Head. |
| `LayoutLeft` | `label` | Leading `aside.sp-left` with aria-label. |
| `LayoutMain` | None | `main#main-content.sp-main` with tabindex -1. |
| `LayoutMenu` | `label` | Native `nav.sp-menu` with aria-label. |
| `LayoutRight` | `label` | Trailing `aside.sp-right` with aria-label. |

**Returns** Each wrapper preserves deliberate child order. Under LayoutProvider's styles,
panel slots collapse into document order on small screens. Use one LayoutMain per document
because its ID is fixed. The caller owns meaningful headings/focus after navigation; these
wrappers do not move focus or create portals.

Related: [LayoutProvider](layout-provider.md), [composition guide](../guides/compose-a-root.md).
