//! Explicit public navigation and the existing shared layout primitives.
use crate::Page;
use leptos::prelude::*;
use stackpress_leptos::{
    components::{LayoutLeft, LayoutMenu, LayoutPanel, LayoutProvider},
    state::{Notifications, Theme, ThemeState},
};

/// Native links use original source paths; their visibility never replaces source authorization.
pub(super) fn shell(page: Page, body: AnyView) -> AnyView {
    // Source guests carry id=0; mere field presence must not display account navigation.
    let signed = page
        .session
        .get("id")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|id| !id.is_empty());
    let theme = ThemeState::new(if page.theme.as_deref() == Some("dark") {
        Theme::Dark
    } else {
        Theme::Light
    });
    let pending = RwSignal::new(page.theme.is_none());
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        if pending.get_untracked() {
            let frozen = leptos::web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.document_element())
                .and_then(|element| element.get_attribute("data-theme"));
            if frozen.as_deref() == Some("dark") {
                theme.set(Theme::Dark);
            }
            pending.set(false);
        }
    });
    view! {<LayoutProvider theme=theme initial_theme_pending=Signal::derive(move ||pending.get()) notifications=Notifications::default()>
        <LayoutPanel><LayoutLeft label="Publication"><a class="blog-logo" href="/">{page.brand["name"].as_str().unwrap_or("Stackpress").to_owned()}</a>
            <LayoutMenu label="Publication navigation"><a href="/">{page.phrase("Home Page")}</a>
                <a href=if signed {"/auth/account"} else {"/auth/signin"}>{page.phrase(if signed {"My Account"} else {"Sign In"})}</a>
                <a href="/admin/article/search">{page.phrase("Admin")}</a>
            </LayoutMenu></LayoutLeft>{body}</LayoutPanel>
    </LayoutProvider>}.into_any()
}
