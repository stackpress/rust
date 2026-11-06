//! Source error presentation over already captured public status and optional native snippets.
use crate::Page;
use leptos::prelude::*;

/// Production/404 views omit trace presentation while retaining the original error copy.
pub(super) fn body(page: &Page) -> AnyView {
    let missing = page.code == 404;
    let description = if missing {
        page.phrase("The requested resource was not found.")
    } else {
        page.phrase(if page.error.is_empty() {
            "There was an error."
        } else {
            &page.error
        })
    };
    let traces = if page.mode != "production" && !missing {
        page.stack.as_array().into_iter().flatten().enumerate().map(|(index,trace)|{
            let file=trace["file"].as_str().unwrap_or_default().to_owned();
            let line=trace["line"].as_u64().unwrap_or_default();
            let column=trace["column"].as_u64().unwrap_or_default();
            let lines=["before","main","location","after"].into_iter().filter_map(|key|trace["snippet"][key].as_str().map(|text|view!{<pre>{text.to_owned()}</pre>})).collect_view();
            view!{<section class="error-trace"><h2>{format!("#{} {file}:{line}:{column}",index+1)}</h2>{lines}</section>}
        }).collect_view().into_any()
    } else {
        ().into_any()
    };
    view!{<main id="main-content" class="knowledge-app error-page" tabindex="-1"><h1>{page.phrase(if missing{"Not Found"}else{"Oops..."})}</h1><p role="alert">{description}</p>{traces}</main>}.into_any()
}
