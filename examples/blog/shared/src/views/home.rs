//! Original publication home: featured first row, recent cards, metrics and orientation.
use crate::{Page, helpers::*};
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::SchemaResult;

/// Build one shared homepage from the real ordered search results.
pub(super) fn body(page: &Page) -> SchemaResult<AnyView> {
    let rows = page.data.as_array().cloned().unwrap_or_default();
    let (featured, recent) = get_featured_article(&rows);
    let signals=get_content_signals(&rows)?.into_iter().map(|(label,value)|view!{
        <div class="knowledge-signal"><span class="knowledge-signal-label">{page.phrase(label)}</span><strong class="knowledge-signal-value">{value}</strong></div>
    }).collect_view();
    let feature=featured.map(|row|entry(page,row,true)).transpose()?.unwrap_or_else(||view!{
        <div class="knowledge-empty-state"><h2>{page.phrase("No published articles yet")}</h2><p>{page.phrase("Populate the template to see the console layout fill with featured writing and recent entries.")}</p></div>
    }.into_any());
    let cards = recent
        .iter()
        .map(|row| entry(page, row, false))
        .collect::<SchemaResult<Vec<_>>>()?;
    let featured_url = featured
        .map(|row| format!("/articles/{}", row["slug"].as_str().unwrap_or_default()))
        .unwrap_or_else(|| "#recent-writing".into());
    Ok(view! {<main id="main-content" class="knowledge-app" tabindex="-1"><div class="knowledge-shell">
        <header class="knowledge-topbar"><div><p class="knowledge-kicker">{page.phrase("Stackpress Publication")}</p><a class="knowledge-brand" href="/">"The Blog"</a></div>
            <p class="knowledge-topbar-copy">{page.phrase("System-minded notes for builders shaping content, tooling, and interface work.")}</p>
            <a class="knowledge-topbar-action" href=featured_url>{page.phrase(if featured.is_some(){"Open Featured"}else{"Browse Writing"})}</a></header>
        <section class="knowledge-hero"><div class="knowledge-hero-copy knowledge-panel"><p class="knowledge-section-label">{page.phrase("Knowledge Console")}</p>
            <h1 class="knowledge-display">{page.phrase("A product-style reading surface for structured ideas.")}</h1>
            <p class="knowledge-lead">{page.phrase("This template treats publishing like an interface: featured context up front, modular entry points below, and calmer reading once you enter an article.")}</p>
            <div class="knowledge-signal-grid">{signals}</div></div>
            <aside class="knowledge-feature knowledge-panel knowledge-panel-strong"><p class="knowledge-section-label">{page.phrase("Featured Entry")}</p>{feature}</aside></section>
        <section class="knowledge-section"><div class="knowledge-section-heading"><div><p class="knowledge-section-label">{page.phrase("Recent Writing")}</p>
            <h2 class="knowledge-section-title" id="recent-writing">{page.phrase("Browse the latest entries")}</h2></div>
            <p class="knowledge-section-copy">{page.phrase("Each card behaves like a compact entry point into the publication rather than a plain chronological line item.")}</p></div>
            <div class="knowledge-card-grid">{cards.collect_view()}</div></section>
        <section class="knowledge-section knowledge-topics"><div class="knowledge-section-heading"><div><p class="knowledge-section-label">{page.phrase("Orientation")}</p>
            <h2 class="knowledge-section-title">{page.phrase("What this surface is built to cover")}</h2></div></div>
            <div class="knowledge-topic-grid">{get_topic_highlights().into_iter().map(|topic|view!{<div class="knowledge-topic-chip">{page.phrase(topic)}</div>}).collect_view()}</div></section>
        <section class="knowledge-section knowledge-closing knowledge-panel"><p class="knowledge-section-label">{page.phrase("Reading Mode")}</p>
            <h2 class="knowledge-section-title">{page.phrase("Built to move from scanning to focused reading without changing visual language.")}</h2>
            <p class="knowledge-section-copy">{page.phrase("The homepage carries more energy so users can orient quickly. Article pages pull the system back into a calmer rhythm for long-form trust and readability.")}</p></section>
    </div></main>}.into_any())
}

/// Featured and recent variants retain the same source image, author/date and summary rules.
fn entry(page: &Page, row: &Value, featured: bool) -> SchemaResult<AnyView> {
    let url = format!("/articles/{}", row["slug"].as_str().unwrap_or_default());
    let title = row["title"].as_str().unwrap_or_default().to_owned();
    let image=row["banner"].as_str().filter(|value|!value.is_empty()).map(|image|view!{
        <a href=url.clone() class=if featured{"knowledge-feature-media"}else{"knowledge-card-media"}><img src=image.to_owned() alt=title.clone()/></a>
    });
    let date = format_article_date(row["published"].as_str())?;
    let author = row["profile"]["name"]
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| page.phrase("Stackpress"));
    let summary = get_article_summary(row["contents"].as_str(), if featured { 240 } else { 140 });
    let reading = get_reading_time_label(row["contents"].as_str());
    let content = view! {{image}<div class="knowledge-meta-row"><span>{date}</span><span>{author}</span></div>
    <h2 class=if featured{"knowledge-feature-title"}else{"knowledge-card-title"}><a href=url.clone()>{title}</a></h2>
    <p class=if featured{"knowledge-feature-summary"}else{"knowledge-card-summary"}>{summary}</p>
    <div class="knowledge-card-footer"><span>{reading}</span><a class="knowledge-inline-link" href=url>{page.phrase(if featured{"Read the full entry"}else{"Open article"})}<stackpress_leptos::components::ChevronRight/></a></div>};
    Ok(if featured {
        content.into_any()
    } else {
        view! {<article class="knowledge-card knowledge-panel">{content}</article>}.into_any()
    })
}
