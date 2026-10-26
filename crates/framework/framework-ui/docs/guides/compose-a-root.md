# Compose controls and scoped state

Build a root with explicit state so SSR and the client start from the same values. Create
signals while the root Owner is active, then provide them before evaluating descendants.

```rust
use stackpress_framework_ui::{
    components::{
        Button, FieldControl, Input, LayoutBlank, LayoutMain, LayoutProvider, NotifierContainer,
    },
    leptos::prelude::*,
    state::{Notifications, Theme, ThemeState},
};

#[component]
pub fn SearchForm() -> impl IntoView {
    let theme = ThemeState::new(Theme::Light);
    let notifications = Notifications::new();
    let query = RwSignal::new(String::new());
    view! {
        <LayoutProvider theme notifications>
            <LayoutBlank><LayoutMain>
                <h1>"Profiles"</h1>
                <form method="get" action="/ui/profiles">
                    <FieldControl for_id="profile-id" label="Profile ID prefix" help="Enter a prefix">
                        <Input id="profile-id" name="id" value=query
                            described_by="profile-id-help profile-id-error"/>
                    </FieldControl>
                    <Button button_type=stackpress_framework_ui::components::ButtonType::Submit>
                        "Search"
                    </Button>
                </form>
                <NotifierContainer notifications/>
            </LayoutMain></LayoutBlank>
        </LayoutProvider>
    }
}
```

`FieldControl` derives help/error IDs but the input receives their association explicitly.
The native GET form works before hydration; the submit button opts into form submission.
`LayoutProvider` supplies root-local theme/notifications and neutral styles. It does not add a
notifier automatically, so the example places `NotifierContainer` deliberately.

Use one `LayoutMain` per document and unique IDs for fields and keyed list rows. Use
[`Notifications`](../api/notifications.md) for bounded immediate messages; there are no timers
or cross-thread queue guarantees. Marshal completions into this root's UI context before
calling `notify` or changing theme/language.

This gives you a deterministic initial tree with native semantics. Continue with
[translation](translate.md) to supply checked language state, or the [component API](../api/README.md)
for props and defaults.
