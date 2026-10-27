//! Native top-layer popovers and modal drawers with root-local lifetime ownership.
use super::{Dialog, DialogMode};
use leptos::prelude::*;

/// Closed fixed role set for an interactive nonmodal top-layer surface.
#[derive(Clone, Copy, Debug, Default)]
pub enum PopupRole {
    /// Labelled nonmodal dialog.
    #[default]
    Dialog,
    /// Menu with application-supplied menuitems.
    Menu,
    /// Combobox listbox; focus stays on its input.
    Listbox,
}
impl PopupRole {
    /// Stable ARIA role token.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dialog => "dialog",
            Self::Menu => "menu",
            Self::Listbox => "listbox",
        }
    }
}
/// Native auto popover: outside click/Escape, nested top-layer stack and unmount cleanup.
///
/// Keep initial state closed for deterministic SSR/hydration. Native light-dismiss
/// synchronizes the controller. This does not trap focus or make the page inert.
///
/// # Examples
/// ```
/// use leptos::prelude::*;
/// use stackpress_framework_ui::components::Popover;
/// let owner = Owner::new();
/// owner.with(|| {
///     let open = RwSignal::new(false);
///     let _view = view! {
///         <button id="help-trigger" type="button" aria-controls="help"
///             on:click=move |_| open.set(true)>"Help"</button>
///         <Popover id="help" anchor="help-trigger" label="Help" open
///             initial_focus_id="help-close">
///             <button id="help-close" type="button"
///                 on:click=move |_| open.set(false)>"Close help"</button>
///         </Popover>
///     };
/// });
/// owner.cleanup();
/// ```
#[component]
pub fn Popover(
    /// Unique popup identity, referenced by its trigger's aria-controls.
    #[prop(into)]
    id: String,
    /// Native anchor identity; must be connected before opening.
    #[prop(into)]
    anchor: String,
    /// Accessible surface name.
    #[prop(into)]
    label: String,
    /// Root-owned controlled visibility.
    open: RwSignal<bool>,
    /// Fixed role; interactive tooltip content is deliberately absent.
    #[prop(optional)]
    role: PopupRole,
    /// Optional descendant element ID, otherwise no focus move.
    #[prop(optional, into)]
    initial_focus_id: String,
    /// Explicit top-layer content.
    children: Children,
) -> impl IntoView {
    let element = NodeRef::<leptos::html::Div>::new();
    #[cfg(target_arch = "wasm32")]
    super::overlay_dom::install(element, anchor, open, initial_focus_id, false);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (anchor, initial_focus_id, open);
    view! {<div id=id node_ref=element class="sp-popover" popover="auto" role=role.as_str() aria-label=label
    on:toggle=move |_event|{#[cfg(target_arch="wasm32")] super::overlay_dom::sync_toggle(&_event,open);}>
    {children()}</div>}
}
/// A modal drawer retains the native dialog's focus trap, Escape and return-focus rules.
#[component]
pub fn Drawer(
    /// Stable native dialog ID.
    #[prop(into)]
    id: String,
    /// Root-owned controlled state, initially closed.
    open: RwSignal<bool>,
    /// Accessible visible title.
    #[prop(into)]
    title: Signal<String>,
    /// Descendant focus target.
    #[prop(into)]
    initial_focus_id: String,
    /// Deterministic target when the opener is removed.
    #[prop(into)]
    fallback_focus_id: String,
    /// Drawer side, default trailing edge.
    #[prop(optional)]
    leading: bool,
    /// Drawer contents.
    children: Children,
) -> impl IntoView {
    view! {<Dialog id open title initial_focus_id fallback_focus_id mode=if leading{DialogMode::DrawerStart}else{DialogMode::DrawerEnd}>{children()}</Dialog>}
}
/// Validated timer duration, bounded to ten seconds; zero executes on the next turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DelayMs(pub(super) u32);
impl DelayMs {
    /// Admit a bounded root-owned timer, without scheduling native work.
    /// # Errors
    /// Rejects delays exceeding ten seconds.
    pub fn new(value: u32) -> Result<Self, super::ControlError> {
        if value > 10_000 {
            Err(super::ControlError)
        } else {
            Ok(Self(value))
        }
    }
    /// Borrow the validated duration.
    pub fn get(self) -> u32 {
        self.0
    }
}
impl Default for DelayMs {
    /// A small explicit hover delay.
    fn default() -> Self {
        Self(150)
    }
}
/// Root-owned hover/focus dismissal state, independently native-testable.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct TooltipActivity {
    hovered: bool,
    focused: bool,
    dismissed: bool,
}
impl TooltipActivity {
    /// Record pointer presence. A new entry permits a previously dismissed tooltip.
    pub fn hover(&mut self, present: bool) {
        if present && !self.hovered {
            self.dismissed = false;
        }
        self.hovered = present;
    }
    /// Record keyboard focus independently of pointer presence.
    pub fn focus(&mut self, present: bool) {
        if present && !self.focused {
            self.dismissed = false;
        }
        self.focused = present;
    }
    /// Escape dismisses until the next entry/focus, while keeping trigger focus.
    pub fn dismiss(&mut self) {
        self.dismissed = true;
    }
    /// Whether delayed presentation is requested.
    pub fn requested(self) -> bool {
        (self.hovered || self.focused) && !self.dismissed
    }
}
/// Noninteractive text description shown on hover/focus, dismissed by Escape.
/// No links, buttons or arbitrary child HTML can enter this tooltip surface.
#[component]
pub fn Tooltip(
    /// Prefix for trigger and text relationship.
    #[prop(into)]
    id: String,
    /// Visible trigger text.
    #[prop(into)]
    label: String,
    /// Escaped descriptive text.
    #[prop(into)]
    text: String,
    /// Bounded hover/focus delay.
    #[prop(optional)]
    delay: DelayMs,
) -> impl IntoView {
    let activity = RwSignal::new(TooltipActivity::default());
    let open = RwSignal::new(false);
    let popup = NodeRef::<leptos::html::Div>::new();
    let anchor = format!("{id}-trigger");
    let description = format!("{id}-tip");
    #[cfg(target_arch = "wasm32")]
    {
        super::overlay_dom::delayed(
            open,
            Signal::derive(move || activity.get().requested()),
            delay,
        );
        super::overlay_dom::install(popup, anchor.clone(), open, String::new(), true);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = delay;
    view! {<span class="sp-tooltip"><button id=anchor type="button" aria-describedby=description.clone()
    on:mouseenter=move |_|activity.update(|a|a.hover(true)) on:mouseleave=move |_|activity.update(|a|a.hover(false))
    on:focus=move |_|activity.update(|a|a.focus(true)) on:blur=move |_|activity.update(|a|a.focus(false))
    on:keydown=move |event:leptos::ev::KeyboardEvent|{if event.key()=="Escape" && activity.get().requested(){event.prevent_default();activity.update(TooltipActivity::dismiss);open.set(false);}}>{label}</button>
    // Native hint close-watching also dismisses a hover-only tooltip on Escape.
    <div id=description node_ref=popup class="sp-popover" popover="hint" role="tooltip"
    on:toggle=move |_event|{#[cfg(target_arch="wasm32")] {super::overlay_dom::sync_toggle(&_event,open);if !open.get() && activity.get().requested(){activity.update(TooltipActivity::dismiss);}}}
    on:mouseenter=move |_|activity.update(|a|a.hover(true)) on:mouseleave=move |_|activity.update(|a|a.hover(false))>{text}</div></span>}
}
/// Explicit scoped toast; dismissible notices never become a process-wide singleton.
/// Hover/focus pauses expiration and restarts a full duration on leaving; None is manual.
#[component]
pub fn Toast(
    /// Stable notice identity.
    #[prop(into)]
    id: String,
    /// Root-owned visible state.
    visible: RwSignal<bool>,
    /// Escaped application message.
    #[prop(into)]
    message: Signal<String>,
    /// Optional bounded expiration; errors should normally be manual.
    #[prop(optional)]
    duration: Option<DelayMs>,
    /// Error uses alert; ordinary notices use status.
    #[prop(optional)]
    urgent: bool,
) -> impl IntoView {
    let active = RwSignal::new(TooltipActivity::default());
    #[cfg(target_arch = "wasm32")]
    if let Some(duration) = duration {
        super::overlay_dom::expire(
            visible,
            Signal::derive(move || active.get().requested()),
            duration,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = duration;
    view! {<div id=id class="sp-notice" hidden=move ||!visible.get() role=if urgent{"alert"}else{"status"}
    on:mouseenter=move |_|active.update(|a|a.hover(true)) on:mouseleave=move |_|active.update(|a|a.hover(false))
    on:focusin=move |_|active.update(|a|a.focus(true)) on:focusout=move |_|active.update(|a|a.focus(false))>
    <span>{move ||message.get()}</span><button type="button" on:click=move |_|visible.set(false)>"Dismiss"</button></div>}
}

/// Opt-in neutral styles for M2 semantic states, overlays, navigation and collections.
pub const BREADTH_STYLES: &str = include_str!("breadth.css");
