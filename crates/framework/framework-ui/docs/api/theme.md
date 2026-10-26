# Theme and ThemeState

Explicit root-owned theme without persistence. Source: [state.rs](../../src/state.rs).

```rust
use stackpress_framework_ui::{
    leptos::prelude::*,
    state::{Theme, ThemeState, use_theme},
};
fn main() {
    let owner = Owner::new();
    owner.with(|| {
        let theme = ThemeState::new(Theme::Light);
        theme.toggle();
        assert_eq!(theme.get(), Theme::Dark);
        theme.set(Theme::Light);
        assert_eq!(theme.get().as_str(), "light");
    });
    owner.cleanup();
}
fn toggle_provided_theme() -> Result<(), &'static str> {
    use_theme()?.toggle();
    Ok(())
}
```

## Theme

`Theme` is Copy/Clone, Debug, Default, PartialEq/Eq. `Light` is default; `Dark` is the other
variant. `as_str(self)` takes no additional parameters and returns the stable `&'static str`
token `light` or `dark`.

## ThemeState and context

The Copy/Clone handle owns a signal in the current Leptos Owner.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `ThemeState::new` | `Theme` | Initializes explicit state in the active Owner. |
| `get(self)` | None | Reactive read. |
| `set(self, theme)` | `Theme` | Replaces current root value. |
| `toggle(self)` | None | Switches Light ↔ Dark synchronously. |
| `use_theme()` | None | Looks up the nearest provided ThemeState. |

**Returns** `new` returns Self, `get` returns Theme, mutations return unit. `use_theme` returns
`Result<ThemeState, &'static str>` with `Theme state requires LayoutProvider` when missing.
No API reads/writes cookies, browser media queries, or storage. Copying a handle aliases the
same state rather than creating another root; cleanup disposes it.

Mutate in the owning root's UI execution context. Send/Sync signal handles do not promise a
concurrent cross-thread queue; marshal completions into that context first.

Related: [LayoutProvider](layout-provider.md), [root boundary](../concepts.md#client-state-belongs-to-one-root).
