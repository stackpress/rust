//! Shared Profile presentation and state for SSR, hydration and packaged CSR.
//!
//! The controller owns no host service or transport future. Initial data is
//! consumed once; only explicit user actions dispatch the selected adapter.

mod controller;
mod locale;
mod navigation;
mod preferences;
pub use navigation::Navigation;
use preferences::PreferenceController;
pub use preferences::{
    DisplayPreferences, PreferenceChange, PreferenceCompletion, PreferenceTransport,
};

use controller::SearchController;
use leptos::prelude::*;
pub use locale::profile_language;
use stackpress_example_profile_ui_contracts::{
    PublicStatus, SearchFailure, SearchInput, SearchPage, SearchReply, routes, validate_page,
};
use stackpress_framework_view::leptos::routing::use_route_context;
use stackpress_leptos::{
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
pub type SearchCompletion =
    std::sync::Arc<dyn Fn(Result<SearchReply, SearchFailure>) + Send + Sync>;
/// WASM-local adapter; it need not be Send and is never stored by a native host.
pub type SearchTransport = UnsyncCallback<(SearchInput, SearchCompletion)>;
/// Explicit root-local cancellation; no implicit retry or rollback is promised.
pub type SearchCancellation = UnsyncCallback<()>;

/// Checked shared root; native status and catalog identity select the initial page.
#[component]
pub fn ProfilePage(
    /// Owned public data, consumed once by SSR and hydration.
    seed: SearchPage,
    /// Checked snapshot catalog ID, never a browser-only initial branch.
    page: String,
    /// Public native status; missing detail must agree with 404.
    status: PublicStatus,
    /// Explicit locale fixture decoded before component evaluation.
    initial_language: Language,
    /// Explicit initial theme.
    initial_theme: Theme,
    /// Hydrate the SSR Light tree, then adopt the frozen pre-paint document choice.
    #[prop(optional)]
    initial_theme_pending: bool,
    /// Browser-owned text typed before hydration, separate from the last successful search.
    #[prop(optional_no_strip)]
    initial_draft: Option<String>,
    /// Selected fixed operation adapter.
    #[prop(optional_no_strip, into)]
    transport: Option<SearchTransport>,
    /// Root-local cancellation.
    #[prop(optional_no_strip, into)]
    cancel: Option<SearchCancellation>,
    /// Thin host navigation adapter, absent during native rendering.
    #[prop(optional_no_strip, into)]
    navigation: Option<Navigation>,
    /// Optional persist-before-publication adapter; omission preserves existing behavior.
    #[prop(optional_no_strip, into)]
    preferences: Option<PreferenceTransport>,
    /// Native loading recovered to defaults; no warning is emitted by default.
    #[prop(default = false)]
    preference_recovered: bool,
    /// Packaged bootstrap requests its first local read explicitly.
    #[prop(default = false)]
    initial_search: bool,
) -> impl IntoView {
    if validate_page(&page, &status, &seed).is_err() {
        return view! { <p role="alert">"Page data invalid. Refresh to continue."</p> }.into_any();
    }
    view! { <ProfileSearch seed initial_language initial_theme initial_theme_pending initial_draft=initial_draft initial_search
    transport=transport cancel=cancel navigation=navigation preferences=preferences preference_recovered/> }
    .into_any()
}

/// Component input is an owned public seed, never a live invocation or host handle.
#[component]
pub fn ProfileSearch(
    /// Public initial data, consumed without an initial fetch.
    seed: SearchPage,
    /// Explicit browser/IPC adapter, absent for synchronous server rendering.
    #[prop(optional_no_strip, into)]
    transport: Option<SearchTransport>,
    /// Cancels the current adapter request on invalid input or explicit cancellation.
    #[prop(optional_no_strip, into)]
    cancel: Option<SearchCancellation>,
    /// Checked explicit locale/map. Omission selects the example's English fixture.
    #[prop(optional)]
    initial_language: Option<Language>,
    /// Explicit seed theme; browser media queries and cookies are never consulted.
    #[prop(default = Theme::Light)]
    initial_theme: Theme,
    /// Hydrate the SSR Light tree, then adopt the frozen pre-paint document choice.
    #[prop(optional)]
    initial_theme_pending: bool,
    /// Browser-owned text typed before hydration, separate from the last successful search.
    #[prop(optional_no_strip)]
    initial_draft: Option<String>,
    /// Root-local browser navigation adapter.
    #[prop(optional_no_strip, into)]
    navigation: Option<Navigation>,
    /// Optional persist-before-publication adapter; omission preserves existing behavior.
    #[prop(optional_no_strip, into)]
    preferences: Option<PreferenceTransport>,
    /// Announce invalid-file recovery without changing default output.
    #[prop(default = false)]
    preference_recovered: bool,
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
    // Effective root color and the temporary SSR handoff marker are independent of search pending.
    let theme = ThemeState::new(initial_theme);
    let theme_pending = RwSignal::new(initial_theme_pending);
    // Hydration keeps the SSR tree identical. CSS covers it until this one-shot handoff.
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        // Read untracked and clear once, so subsequent OS changes cannot drive this effect.
        if theme_pending.get_untracked() {
            let dark = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.document_element())
                .is_some_and(|html| html.get_attribute("data-theme").as_deref() == Some("dark"));
            // Switch color and remove the CSS handoff marker atomically to avoid a wrong-color frame.
            batch(|| {
                theme.set(if dark { Theme::Dark } else { Theme::Light });
                theme_pending.set(false);
            });
        }
    });
    let base = use_route_context().base;
    let form_action = match routes::search().and_then(|route| route.location(&base, &[])) {
        Ok(location) => location.as_str().to_owned(),
        Err(_) => {
            return view! { <p role="alert">"Page location invalid. Refresh to continue."</p> }
                .into_any();
        }
    };
    let navigation_state =
        match navigation::PageNavigation::new(base.clone(), seed.detail.clone(), &seed.input.id) {
            Ok(state) => state,
            Err(message) => return view! { <p role="alert">{message}</p> }.into_any(),
        };
    let detail = navigation_state.detail;
    let link_base = StoredValue::new(base);
    let controller = SearchController::new(seed);
    // Adopt editable text without replacing the acknowledged query/results or issuing a search.
    if let Some(draft) = initial_draft {
        controller.id.set(draft);
    }
    let preference_controller =
        PreferenceController::new(language, theme, controller.notices, preferences);
    if let Some(adapter) = navigation {
        // Effects run only after hydration; initial native data never dispatches a read.
        Effect::new(move |_| {
            if controller.published.get() > 0 {
                navigation_state.completed(controller, adapter);
            }
        });
        Effect::new(move |_| {
            if let (Some((location, push)), Some(transport)) = (adapter.request.get(), transport) {
                navigation_state.go(location, push, controller, transport, cancel, adapter);
            }
        });
    }
    let title = move || {
        language.text(
            if detail.get().is_some() {
                "Profile"
            } else {
                "Profiles"
            },
            &[],
        )
    };
    let description = move || {
        language.text(
            if detail.get().is_some() {
                "Public profile detail"
            } else {
                "Search public demo profiles"
            },
            &[],
        )
    };
    let SearchController {
        id,
        rows,
        total,
        error,
        failure_kind,
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
    // Initial search clears earlier operation notices synchronously. Publish file
    // recovery afterward so successful bootstrap cannot erase its explanation.
    if preference_recovered {
        let _ = controller.notices.notify(
            stackpress_leptos::state::NoticeKind::Warning,
            "Saved preferences could not be loaded. Defaults are in use.".into(),
        );
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
                id: id.clone(),
                label: name,
                href: routes::detail_location(
                    &link_base.get_value(),
                    &id,
                    &controller.prefix.get(),
                )
                .ok()
                .map(Into::into),
            })
            .collect()
    });
    #[cfg(feature = "ssr")]
    let head = view! {
        // Buffered Head assembly outlives the render Owner, so SSR metadata owns strings.
        <Html attr:lang=language.get().locale().to_owned()/>
        <Title text=title()/>
        <Meta name="description" content=description()/>
    };
    #[cfg(not(feature = "ssr"))]
    let head = view! {
        // Interactive metadata remains reactive while its client Owner is mounted.
        <Html attr:lang=move || language.get().locale().to_owned()/>
        <Title text=move || title()/>
        <Meta name="description" content=move || description()/>
    };
    // No Effect/Resource starts a search: hydration adopts exactly the server seed.
    view! {
        {head}
        <LanguageProvider language><LayoutProvider theme notifications=notices initial_theme_pending=Signal::derive(move || theme_pending.get())><LayoutBlank>
            <LayoutHead>
                <div><h1>{move || title()}</h1><p>{move || language.text("Find a person by profile ID.", &[])}</p></div>
                <LayoutMenu label="Display preferences">
                    <Button on:click=move |_| {
                        // The checked fixture replaces the entire dictionary, without fetching.
                        let locale = if language.get().locale() == "en" { "fr" } else { "en" };
                        preference_controller.change(PreferenceChange::Locale(locale.into()));
                    }>{move || if language.get().locale() == "en" { "Français" } else { "English" }}</Button>
                    <Button on:click=move |_| {
                        // Toggle the current effective Light/Dark choice explicitly.
                        let next = match theme.get() { Theme::Light => Theme::Dark, Theme::Dark => Theme::Light };
                        preference_controller.change(PreferenceChange::Theme(next));
                    }>{move || language.text(if theme.get() == Theme::Dark { "Light theme" } else { "Dark theme" }, &[])}</Button>
                </LayoutMenu>
            </LayoutHead>
            <LayoutPanel>
                <LayoutLeft label="Search guidance"><p>{move || language.text("Use the first characters of a profile ID. IDs are case sensitive.", &[])}</p></LayoutLeft>
                <LayoutMain>
                    <div hidden=move || detail.get().is_some()>
                    <form method="get" action=form_action on:submit=submit>
                        <FieldControl for_id="profile-id" label=Signal::derive(move || language.text("Profile ID prefix", &[]))
                            help=Signal::derive(move || language.text("For example, abc", &[])) error=field_error>
                            <Input id="profile-id" name="id" value=id described_by="profile-id-help profile-id-error"
                                invalid=Signal::derive(move || !field_error.get().is_empty())/>
                        </FieldControl>
                        <Button button_type=ButtonType::Submit>{move || language.text("Search", &[])}</Button>
                    </form>
                    <Loader busy=pending label=Signal::derive(move || language.text("Searching…", &[]))/>
                    <p role="status" aria-live="polite">{move || language.plural_text(total.get(), "%s profile", "%s profiles", &[total.get().to_string()])}</p>
                    <LoaderContainer busy=pending>
                        <List items empty=Signal::derive(move || language.text("No profiles match this ID prefix. Try another prefix.", &[]))/>
                    </LoaderContainer>
                    </div>
                    <div data-search-failure=move || failure_kind.get().map(|kind| kind.as_str())>
                        <Alert message=error/>
                    </div>
                    <div class="sp-menu">
                        <Show when=move || !error.get().is_empty()>
                            <Button on:click=move |_| { if let Some(transport) = transport { controller.submit(transport, cancel); } }>{move || language.text("Retry search", &[])}</Button>
                        </Show>
                        <Show when=move || pending.get()>
                            <Button on:click=move |_| controller.cancel(cancel)>{move || language.text("Cancel search", &[])}</Button>
                        </Show>
                    </div>
                    {move || detail.get().map(|context| {
                        let reply = SearchReply {
                            revision: stackpress_example_profile_ui_contracts::REVISION,
                            rows: rows.get().into_iter().map(|(id, name)| stackpress_example_profile_ui_contracts::Profile { id, name }).collect(),
                            total: total.get(),
                        };
                        let return_href = routes::search_location(&link_base.get_value(), &context.search_prefix).map(|location| location.as_str().to_owned()).unwrap_or_default();
                        view! { <section data-detail-page aria-busy=move || pending.get().to_string()>
                            <Loader busy=pending/>
                            <div hidden=move || pending.get() || !error.get().is_empty()><ProfileDetail id=context.id reply return_href/></div>
                        </section> }
                    })}
                    <NotifierContainer notifications=notices/>
                </LayoutMain>
                <LayoutRight label="About these profiles">
                    <p><Translate language one="Showing %s public profile." phrase="Showing %s public profiles." count=total slots=vec![ViewFn::from(move || view! { <strong>{move || total.get()}</strong> })]/></p>
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
    let language = stackpress_leptos::language::use_language().ok();
    let text = move |phrase: &'static str| {
        language.map_or_else(|| phrase.to_owned(), |language| language.text(phrase, &[]))
    };
    view! {
        <section aria-label="Profile detail">
            {match profile {
                Some(profile) => view! { <h2>{profile.name}</h2><dl><dt>{move || text("Profile ID")}</dt><dd>{profile.id}</dd></dl> }.into_any(),
                None => view! { <h2>{move || text("Profile not found")}</h2><p>{move || text("Try another ID prefix to find a profile.")}</p> }.into_any(),
            }}
            <Link href=return_href label=Signal::derive(move || text("Return to search"))/>
        </section>
    }
}

#[cfg(test)]
mod tests;
