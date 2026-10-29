//! Explicit semantic layout slots and scoped presentation state.

use crate::state::{Notifications, ThemeState};
use leptos::prelude::*;
use leptos_meta::Style;

/// Small neutral CSS; no fonts, images, brand palette or runtime style dependency.
const STYLES: &str = include_str!("styles.css");

/// Provide scoped theme/notification state and one stable style resource per root.
#[component]
pub fn LayoutProvider(
    /// Explicit public initial theme, identical on the server and client.
    theme: ThemeState,
    /// Allow the checked document pre-paint choice until the UI root adopts it.
    #[prop(optional, into)]
    initial_theme_pending: Signal<bool>,
    /// Root-local immediate notification state.
    notifications: Notifications,
    /// Layout subtree that owns these providers.
    children: Children,
) -> impl IntoView {
    // Providers are installed before evaluating children; siblings/roots stay independent.
    provide_context(theme);
    provide_context(notifications);
    // Pending roots retain identical SSR/hydration attributes; ancestor CSS supplies the
    // frozen pre-paint color until the caller publishes the effective theme in one batch.
    view! {
        <Style id="stackpress-ui">{STYLES}</Style>
        <div class="sp-root" data-initial-theme-pending=move || initial_theme_pending.get().then_some("") data-theme=move || theme.get().as_str()>{children()}</div>
    }
}

/// Minimal layout without account, menu, cookie or portal assumptions.
#[component]
pub fn LayoutBlank(
    /// Explicit semantic child slots, including the caller's main landmark.
    children: Children,
) -> impl IntoView {
    view! { <div class="sp-blank">{children()}</div> }
}

/// Responsive panel: left/main/right slots collapse into document order on mobile.
#[component]
pub fn LayoutPanel(
    /// Compose LayoutLeft, LayoutMain and LayoutRight explicitly.
    children: Children,
) -> impl IntoView {
    view! { <div class="sp-panel">{children()}</div> }
}

/// Shared page header, separate from metadata Head ownership.
#[component]
pub fn LayoutHead(
    /// Visible heading and optional explicit controls.
    children: Children,
) -> impl IntoView {
    view! { <header class="sp-head">{children()}</header> }
}

/// Leading panel content, before main in both visual and keyboard order.
#[component]
pub fn LayoutLeft(
    /// Accessible landmark name.
    #[prop(into)]
    label: String,
    /// Sidebar content; permission decisions remain host/application responsibilities.
    children: Children,
) -> impl IntoView {
    view! { <aside class="sp-left" aria-label=label>{children()}</aside> }
}

/// One main landmark; applications supply a stable focusable content heading.
#[component]
pub fn LayoutMain(
    /// Main page content.
    children: Children,
) -> impl IntoView {
    view! { <main id="main-content" class="sp-main" tabindex="-1">{children()}</main> }
}

/// Named native navigation slot; does not infer account routes or permissions.
#[component]
pub fn LayoutMenu(
    /// Accessible navigation name.
    #[prop(into)]
    label: String,
    /// Explicit native links supplied by the application.
    children: Children,
) -> impl IntoView {
    view! { <nav class="sp-menu" aria-label=label>{children()}</nav> }
}

/// Trailing supplementary content, after main in both visual and keyboard order.
#[component]
pub fn LayoutRight(
    /// Accessible landmark name.
    #[prop(into)]
    label: String,
    /// Supplementary content.
    children: Children,
) -> impl IntoView {
    view! { <aside class="sp-right" aria-label=label>{children()}</aside> }
}
