//! One real component tree for each independent render mode.
use crate::{Initial, dictionary};
use leptos::prelude::*;
use stackpress_leptos::{
    components::{Input, LayoutProvider, NotifierContainer, Popover},
    language::LanguageContext,
    leptos_meta::{Style, Title},
    state::{NoticeKind, Notifications, ThemeState},
};

/// Mount root-local state from explicit values; no host registration occurs.
#[component]
pub fn IndependentRoot(
    /// Checked explicit root choices and the adopted native draft.
    initial: Initial,
) -> impl IntoView {
    // All handles belong to this Owner, so repeated independent roots cannot share state.
    let theme = ThemeState::new(initial.theme);
    let language = LanguageContext::new(initial.language);
    let notifications = Notifications::new();
    let draft = RwSignal::new(initial.draft);
    let echoed = RwSignal::new(String::new());
    let count = RwSignal::new(0_u32);
    let open = RwSignal::new(false);
    provide_context(language);
    view! {
        <Title text="Independent < & >"/>
        <Style id="independent-styles">{concat!(
            "body{margin:0;font:16px system-ui}main{max-width:50rem;margin:auto;padding:1.25rem}",
            "button,input{margin:.35rem}output{display:block}label{display:block}"
        )}</Style>
        <Style id="independent-component-styles">{format!("{}{}",
            stackpress_leptos::components::CORE_STYLES,
            stackpress_leptos::components::BREADTH_STYLES)}</Style>
        <LayoutProvider theme notifications>
            <main id="independent-main" data-language=move || language.get().locale().to_owned()>
                <h1>{move || language.text("Welcome", &[])}</h1>
                <p>"A component root with explicit values."</p>
                <label for="independent-draft">"Draft"</label>
                <Input id="independent-draft" name="draft" value=draft/>
                <button id="echo-draft" type="button" on:click=move |_| echoed.set(draft.get_untracked())>
                    "Read current draft"
                </button>
                <output id="draft-value">{move || echoed.get()}</output>
                <button id="increment" type="button" on:click=move |_| count.update(|value| *value += 1)>
                    "Increment"
                </button>
                <output id="count">{move || count.get()}</output>
                <button id="theme" type="button" on:click=move |_| theme.toggle()>"Toggle theme"</button>
                <button id="language" type="button" on:click=move |_| {
                    let next = if language.get().locale() == "en" { "fr" } else { "en" };
                    // Both choices are fixture constants; an unexpected dictionary error is visible.
                    if let Ok(next) = dictionary(next) { language.change(next); }
                }>"Change language"</button>
                <button id="notify" type="button" on:click=move |_| {
                    let _ = notifications.notify(NoticeKind::Info, "Action completed".into());
                }>"Notify"</button>
                <button id="open-popover" type="button" aria-controls="independent-popover"
                    aria-expanded=move || open.get().to_string() on:click=move |_| open.set(!open.get())>
                    "Open details"
                </button>
                <Popover id="independent-popover" anchor="open-popover" label="Details" open
                    initial_focus_id="close-popover">
                    <p>"Root-local details"</p>
                    <button id="close-popover" type="button" on:click=move |_| open.set(false)>"Close"</button>
                </Popover>
                <NotifierContainer notifications/>
                {samples()}
            </main>
        </LayoutProvider>
    }
}

/// Feature samples use the same source in native rendering and both browser modes.
fn samples() -> impl IntoView {
    view! {
        {async_sample()}
        {rich_sample()}
        {format_sample()}
    }
}

/// Resolve through a real serializable Resource, reusing native replay during hydration.
#[cfg(feature = "async")]
fn async_sample() -> AnyView {
    let resource = Resource::new(
        || (),
        |_| async {
            #[cfg(target_arch = "wasm32")]
            {
                // A fixture-owned counter distinguishes replay from a duplicate loader execution.
                let global = js_sys::global();
                let key = wasm_bindgen::JsValue::from_str("independentLoads");
                let count = js_sys::Reflect::get(&global, &key)
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let _ = js_sys::Reflect::set(&global, &key, &(count + 1.0).into());
            }
            crate::RESOURCE_TEXT.to_owned()
        },
    );
    view! {<section aria-label="Safety samples">
        <h2>"Safety samples"</h2>
        <p>"Deliberately hostile Resource text remains inert."</p>
        <Suspense fallback=|| view!{<p>"Loading optional sample"</p>}>
            {move || resource.get().map(|text| view!{<p id="resource">{text}</p>})}
        </Suspense>
    </section>}
    .into_any()
}
/// Disabled helper features leave no hidden Resource or loader work.
#[cfg(not(feature = "async"))]
fn async_sample() -> AnyView {
    ().into_any()
}

/// Checked rich HTML removes executable markup before insertion in every target.
#[cfg(feature = "rich")]
fn rich_sample() -> AnyView {
    use stackpress_leptos::rich::{RichContent, SafeHtml};
    let safe = SafeHtml::html("<p><strong>Checked rich sample</strong><script>bad()</script></p>")
        .expect("fixed short rich fixture is within the sanitizer budget");
    view! {<section id="rich"><RichContent content=safe/></section>}.into_any()
}
/// No sanitizer work is requested when the helper is disabled.
#[cfg(not(feature = "rich"))]
fn rich_sample() -> AnyView {
    ().into_any()
}

/// Decimal editing keeps native draft, committed value and locale policy distinct.
#[cfg(feature = "format")]
fn format_sample() -> AnyView {
    use stackpress_leptos::format::{
        DisplayLocale,
        edit::{DecimalPolicy, EditState, FormattedInput},
    };
    use std::ops::Bound::Unbounded;
    let policy =
        DecimalPolicy::new(2, 24, Unbounded, Unbounded).expect("fixed valid decimal policy");
    let edit = EditState::new(policy, None, DisplayLocale::English).expect("empty nullable editor");
    let state = RwSignal::new(edit);
    view! {<section id="format">
        <FormattedInput id="independent-amount" name="amount" label="Amount" state nullable=true/>
        <output id="committed">{move || state.with(EditState::canonical).unwrap_or_default()}</output>
    </section>}.into_any()
}
/// Ordinary consumers do not acquire formatting state or compiled locale data.
#[cfg(not(feature = "format"))]
fn format_sample() -> AnyView {
    ().into_any()
}
