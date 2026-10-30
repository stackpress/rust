//! Source named consent form/body/head/layout components, with native POST and Deny navigation.
use super::{Consent, ScopeItem, icons, string};
use leptos::prelude::*;
use stackpress_leptos::{
    components::{Button, ButtonType, LayoutBlank, Switch},
    language::Language,
};

/// Original unchecked scope switches, descriptions, Grant submit and Deny anchor.
#[component]
pub fn ApiOauthForm(
    /// Source selected application name, with Unknown selected by the body.
    app_name: String,
    /// Source error=denied destination, kept as ordinary href data.
    revert: String,
    /// Known requested scopes in source order; repeats remain separate controls.
    items: Vec<ScopeItem>,
    /// Existing phrase/interpolation mechanism.
    language: Language,
) -> impl IntoView {
    let instructions = if items.is_empty() {
        language.text(
            "%s is asking permissions to access your personal information.",
            &[app_name],
        )
    } else {
        language
            .phrase("Grant the following permissions:")
            .to_owned()
    };
    let scopes=items.into_iter().enumerate().map(|(index,item)| {
        let id=format!("oauth-scope-{index}");
        let checked=RwSignal::new(false);
        // Hydration adopts a native edit made before WASM arrives, as the earlier shared forms do.
        #[cfg(all(target_arch="wasm32",feature="ui-hydrate"))]
        if let Some(input)=leptos::web_sys::window().and_then(|window|window.document()).and_then(|document|document.get_element_by_id(&id)) {
            if let Ok(value)=leptos::web_sys::js_sys::Reflect::get(&input,&"checked".into()) {checked.set(value.as_bool().unwrap_or(false));}
        }
        let description=if item.description.is_empty(){String::new()}else{format!("{id}-description")};
        // Each source Switch starts unchecked and retains native form/Space-key behavior.
        view! {<div class="scope"><div class="container"><label for=id.clone()>{item.icon.as_deref().and_then(icons::named)}{item.name}</label>
            {(!item.description.is_empty()).then(||view!{<p id=format!("{id}-description")>{item.description}</p>})}</div>
            <Switch id=id.clone() name="scopes[]" value=item.id checked described_by=description/>
        </div>}
    }).collect_view();
    let grant = language.phrase("Grant").to_owned();
    let deny = language.phrase("Deny").to_owned();
    view! {<form class="oauth-form auth-form" method="post">
        <p class="instructions">{instructions}</p>
        {scopes}
        <div class="action"><Button button_type=ButtonType::Submit class="submit".into()>{grant}</Button>
        <a class="deny" href=revert>{deny}</a></div>
    </form>}.into_any()
}
/// The source body selects known permissions but does not reject unknown requested scopes.
#[component]
pub fn OAuthBody(
    /// Complete source-selected public page state.
    page: Consent,
) -> impl IntoView {
    let brand = page.config.get("brand");
    let name = string(brand.and_then(|brand| brand.get("name")), "Stackpress");
    let logo = brand.and_then(|brand| brand.get("logo"));
    let items = page.items();
    let app_name = page
        .application
        .get("name")
        .and_then(serde_json::Value::as_str)
        .filter(|name| !name.is_empty())
        .unwrap_or("Unknown")
        .to_owned();
    view! {<main class="oauth-page auth-page"><div class="container">
        {if let Some(logo)=logo {view!{<img height="50" alt=name.clone() src=logo.as_str().unwrap_or("").to_owned() class="logo"/>}.into_any()}
        else {view!{<h2 class="brand">{name}</h2>}.into_any()}}
        <section><header>{icons::user()}<h3 class="label">{page.language.phrase("Sign Up").to_owned()}</h3></header>
            <ApiOauthForm app_name revert=page.revert() items language=page.language/>
        </section></div></main>}.into_any()
}
/// Source title, favicon MIME choice, global CSS and ordered additional styles.
#[component]
pub fn ApiOauthHead(
    /// Same owned page props as the body.
    page: Consent,
) -> impl IntoView {
    let favicon = string(page.config.pointer("/brand/favicon"), "/favicon.ico");
    let mime = if favicon.ends_with(".png") {
        "image/png"
    } else if favicon.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "image/x-icon"
    };
    view! {<title>{page.language.phrase("Grant Access").to_owned()}</title>
    {(!favicon.is_empty()).then(||view!{<link rel="icon" type=mime href=favicon/>})}
    <link rel="stylesheet" type="text/css" href="/styles/global.css"/>
    {page.styles.into_iter().map(|href|view!{<link rel="stylesheet" type="text/css" href=href/>}).collect_view()}}
}
/// Existing blank layout with source consent body and explicit Light/Dark/unset attributes.
#[component]
pub fn ApiOauthPage(
    /// Same DTO is used by SSR, hydration and CSR.
    page: Consent,
) -> impl IntoView {
    let theme = page.theme.map(|theme| theme.as_str()).unwrap_or("light");
    let pending = page.theme.is_none();
    view! {<div class="sp-root" data-theme=theme data-initial-theme-pending=pending.then_some("")><LayoutBlank><OAuthBody page/></LayoutBlank></div>}
}
