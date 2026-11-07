//! Shared source blog views; document/asset framing remains the native renderer's concern.
mod article;
mod error;
mod home;
mod layout;
use crate::{Kind, Page};
use leptos::prelude::*;
use stackpress_framework_schema::SchemaResult;

/// Create the exact same root on the native and browser execution hosts.
/// # Errors
/// Invalid source date values propagate before a page can be mounted or published.
pub fn page(value: Page) -> SchemaResult<AnyView> {
    let body = match value.kind {
        Kind::Home => home::body(&value)?,
        Kind::Article => article::body(&value)?,
        Kind::Error => error::body(&value),
    };
    Ok(layout::shell(value, body))
}

/// Source Head fields use owned values before the native rendering Owner is disposed.
pub fn head(value: &Page) -> AnyView {
    let title = value.title.clone();
    let description_visible = value.kind != Kind::Article;
    let description = if value.kind == Kind::Home {
        "A simple blog built with Stackpress."
    } else if value.kind == Kind::Error && value.code == 404 {
        "The requested resource was not found."
    } else {
        "There was an error."
    };
    view! {<leptos_meta::Title text=title.clone()/>{description_visible.then(||view!{<leptos_meta::Meta name="description" content=value.phrase(description)/>})}
        <leptos_meta::Meta property="og:title" content=title.clone()/><leptos_meta::Meta property="og:image" content="/icon.png"/>
        <leptos_meta::Meta property="og:type" content="website"/><leptos_meta::Meta name="twitter:card" content="summary"/>
        <leptos_meta::Meta name="twitter:title" content=title/><leptos_meta::Meta name="twitter:image" content="/icon.png"/>
        <leptos_meta::Link rel="icon" type_="image/svg+xml" href="/assets/brand.svg"/>
        {(value.kind!=Kind::Article).then(||view!{<leptos_meta::Meta property="og:url" content=if value.kind==Kind::Home{"https://stackpress.dev/blog".to_owned()}else{value.path.clone()}/>})}
        {(value.kind==Kind::Article).then(||view!{<leptos_meta::Stylesheet href="/styles/article.css"/>})}
        {value.styles.iter().cloned().map(|href|view!{<leptos_meta::Stylesheet href/>}).collect_view()}
    }.into_any()
}
