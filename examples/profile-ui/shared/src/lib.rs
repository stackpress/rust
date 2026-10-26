//! Shared Profile presentation and state for SSR, hydration and packaged CSR.
//!
//! The controller owns no host service or transport future. Initial data is
//! consumed once; only explicit user actions dispatch the selected adapter.

mod controller;
mod locale;
// TEMPORARY: coordinator swaps this module for lane A's real routing API at merge.
pub mod m1_compat;

use controller::SearchController;
use leptos::prelude::*;
pub use locale::profile_language;
use stackpress_example_profile_ui_contracts::{SearchInput, SearchPage, SearchReply};
use stackpress_framework_ui::{
    components::{
        Alert, Button, ButtonType, FieldControl, Input, LayoutBlank, LayoutHead, LayoutLeft,
        LayoutMain, LayoutMenu, LayoutPanel, LayoutProvider, LayoutRight, Link, List, ListItem,
        Loader, LoaderContainer, NotifierContainer,
    },
    language::{Language, LanguageContext, LanguageProvider, Translate},
    leptos_meta::{Html, Meta, Title},
    state::{Theme, ThemeState},
};

/// A local transport completes with a validated public reply or safe error.
pub type SearchCompletion = std::sync::Arc<dyn Fn(Result<SearchReply, String>) + Send + Sync>;
/// WASM-local adapter; it need not be Send and is never stored by a native host.
pub type SearchTransport = UnsyncCallback<(SearchInput, SearchCompletion)>;
/// Explicit root-local cancellation; no implicit retry or rollback is promised.
pub type SearchCancellation = UnsyncCallback<()>;

/// Component input is an owned public seed, never a live invocation or host handle.
#[component]
pub fn ProfileSearch(
    /// Public initial data, consumed without an initial fetch.
    seed: SearchPage,
    /// Explicit browser/IPC adapter, absent for synchronous server rendering.
    #[prop(optional)]
    transport: Option<SearchTransport>,
    /// Cancels the current adapter request on invalid input or explicit cancellation.
    #[prop(optional)]
    cancel: Option<SearchCancellation>,
    /// Checked explicit locale/map. Omission selects the example's English fixture.
    #[prop(optional)]
    initial_language: Option<Language>,
    /// Explicit seed theme; browser media queries and cookies are never consulted.
    #[prop(default = Theme::Light)]
    initial_theme: Theme,
    /// Packaged CSR may request its first local read; hydration always leaves this false.
    #[prop(default = false)]
    initial_search: bool,
) -> impl IntoView {
    // Reject a malformed seed before constructing reactive state or publishing rows.
    if seed.input.validate().is_err() || seed.reply.validate().is_err() {
        return view! { <p role="alert">"Page data invalid. Refresh to continue."</p> }.into_any();
    }
    let language = match initial_language
        .map(Ok)
        .unwrap_or_else(|| profile_language("en"))
    {
        Ok(language) => LanguageContext::new(language),
        Err(message) => return view! { <p role="alert">{message}</p> }.into_any(),
    };
    let theme = ThemeState::new(initial_theme);
    let base = m1_compat::use_route_context().base;
    let form_action = match base.join("/ui/profiles") {
        Ok(location) => location.as_str().to_owned(),
        Err(_) => {
            return view! { <p role="alert">"Page location invalid. Refresh to continue."</p> }
                .into_any();
        }
    };
    let controller = SearchController::new(seed);
    let SearchController {
        id,
        rows,
        total,
        error,
        field_error,
        pending,
        notices,
        ..
    } = controller;
    if initial_search {
        // This explicit CSR bootstrap is never selected for seeded website hydration.
        if let Some(transport) = transport {
            controller.submit(transport, cancel);
        }
    }
    let submit = move |event: leptos::ev::SubmitEvent| {
        // The native GET form remains usable before hydration and without an adapter.
        let Some(transport) = transport else {
            return;
        };
        event.prevent_default();
        controller.submit(transport, cancel);
    };
    let items = Signal::derive(move || {
        rows.get()
            .into_iter()
            .map(|(id, name)| ListItem {
                id,
                label: name,
                href: None,
            })
            .collect()
    });
    #[cfg(feature = "ssr")]
    let head = view! {
        // Buffered Head assembly outlives the render Owner, so SSR metadata owns strings.
        <Html attr:lang=language.get().locale().to_owned()/>
        <Title text=language.text("Profiles", &[])/>
        <Meta name="description" content=language.text("Search public demo profiles", &[])/>
    };
    #[cfg(not(feature = "ssr"))]
    let head = view! {
        // Interactive metadata remains reactive while its client Owner is mounted.
        <Html attr:lang=move || language.get().locale().to_owned()/>
        <Title text=move || language.text("Profiles", &[])/>
        <Meta name="description" content=move || language.text("Search public demo profiles", &[])/>
    };
    // No Effect/Resource starts a search: hydration adopts exactly the server seed.
    view! {
        {head}
        <LanguageProvider language><LayoutProvider theme notifications=notices><LayoutBlank>
            <LayoutHead>
                <div><h1>{move || language.text("Profiles", &[])}</h1><p>{move || language.text("Find a person by profile ID.", &[])}</p></div>
                <LayoutMenu label="Display preferences">
                    <Button on:click=move |_| {
                        // The checked fixture replaces the entire dictionary, without fetching.
                        let locale = if language.get().locale() == "en" { "fr" } else { "en" };
                        if let Ok(next) = profile_language(locale) { language.change(next); }
                    }>{move || if language.get().locale() == "en" { "Français" } else { "English" }}</Button>
                    <Button on:click=move |_| theme.toggle()>{move || language.text(if theme.get() == Theme::Light { "Dark theme" } else { "Light theme" }, &[])}</Button>
                </LayoutMenu>
            </LayoutHead>
            <LayoutPanel>
                <LayoutLeft label="Search guidance"><p>{move || language.text("Use the first characters of a profile ID. IDs are case sensitive.", &[])}</p></LayoutLeft>
                <LayoutMain>
                    <form method="get" action=form_action on:submit=submit>
                        <FieldControl for_id="profile-id" label=Signal::derive(move || language.text("Profile ID prefix", &[]))
                            help=Signal::derive(move || language.text("For example, abc", &[])) error=field_error>
                            <Input id="profile-id" name="id" value=id described_by="profile-id-help profile-id-error"
                                invalid=Signal::derive(move || !field_error.get().is_empty())/>
                        </FieldControl>
                        <Button button_type=ButtonType::Submit>{move || language.text("Search", &[])}</Button>
                    </form>
                    <Loader busy=pending label=Signal::derive(move || language.text("Searching…", &[]))/>
                    <p role="status" aria-live="polite">{move || language.text("%s profiles", &[total.get().to_string()])}</p>
                    <Alert message=error/>
                    <div class="sp-menu">
                        <Show when=move || !error.get().is_empty()>
                            <Button on:click=move |_| { if let Some(transport) = transport { controller.submit(transport, cancel); } }>{move || language.text("Retry search", &[])}</Button>
                        </Show>
                        <Show when=move || pending.get()>
                            <Button on:click=move |_| controller.cancel(cancel)>{move || language.text("Cancel search", &[])}</Button>
                        </Show>
                    </div>
                    <LoaderContainer busy=pending>
                        <List items empty=Signal::derive(move || language.text("No profiles match this ID prefix. Try another prefix.", &[]))/>
                    </LoaderContainer>
                    <NotifierContainer notifications=notices/>
                </LayoutMain>
                <LayoutRight label="About these profiles">
                    <p><Translate language phrase="Showing %s public profiles." slots=vec![ViewFn::from(move || view! { <strong>{move || total.get()}</strong> })]/></p>
                </LayoutRight>
            </LayoutPanel>
        </LayoutBlank></LayoutProvider></LanguageProvider>
    }.into_any()
}

/// Shared detail presentation; host/router supplies the exact row and return URL.
///
/// Prefix matches never impersonate an exact detail match. A missing record has a
/// visible not-found state and a safe route back, without starting another search.
#[component]
pub fn ProfileDetail(
    /// Exact requested ID, compared to the supplied public rows.
    #[prop(into)]
    id: String,
    /// Public data obtained by the selected typed operation adapter.
    reply: SearchReply,
    /// Host catalog's query-preserving return destination.
    #[prop(into)]
    return_href: String,
) -> impl IntoView {
    let profile = reply.rows.into_iter().find(|profile| profile.id == id);
    view! {
        <section aria-label="Profile detail">
            {match profile {
                Some(profile) => view! { <h2>{profile.name}</h2><dl><dt>"Profile ID"</dt><dd>{profile.id}</dd></dl> }.into_any(),
                None => view! { <h2>"Profile not found"</h2><p>"Try another ID prefix to find a profile."</p> }.into_any(),
            }}
            <Link href=return_href label="Return to search"/>
        </section>
    }
}

#[cfg(test)]
mod tests;
