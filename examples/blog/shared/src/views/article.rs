//! Source article reading surface, raw HTML, tag selection and attached comment cards.
use crate::{Page, helpers::*};
use leptos::prelude::*;
use stackpress_framework_schema::SchemaResult;

/// Render exactly the article/comment data supplied by the native page operation.
pub(super) fn body(page: &Page) -> SchemaResult<AnyView> {
    let article = &page.data;
    let title = page.phrase(article["title"].as_str().unwrap_or_default());
    let date = format_article_date(article["published"].as_str())?;
    let author = article["profile"]["name"]
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| page.phrase("Stackpress"));
    let contents = article["contents"].as_str().unwrap_or_default().to_owned();
    let reading = get_reading_time_label(Some(&contents));
    let tags=article["tags"].as_array().into_iter().flatten().take(3)
        .chain(article["keywords"].as_array().into_iter().flatten().take(2)).take(4)
        .map(|tag|view!{<span class="article-tag">{tag.as_str().unwrap_or_default().to_owned()}</span>}).collect_view();
    let banner=article["banner"].as_str().filter(|value|!value.is_empty()).map(|url|view!{
        <div class="article-banner knowledge-panel"><img src=url.to_owned() alt=title.clone()/></div>
    });
    let rows = article["comments"].as_array().cloned().unwrap_or_default();
    let comments=rows.iter().map(|row|{
        let name=row["profile"]["name"].as_str().filter(|value|!value.is_empty()).map(str::to_owned).unwrap_or_else(||page.phrase("Reader"));
        let date=format_article_date(row["created"].as_str())?;
        let text=row["comment"].as_str().filter(|value|!value.is_empty()).map(str::to_owned).unwrap_or_else(||page.phrase("No comment text provided."));
        Ok(view!{<article class="article-comment-card"><div class="article-comment-meta"><strong>{name}</strong><span>{date}</span></div><p class="article-comment-body">{text}</p></article>}.into_any())
    }).collect::<SchemaResult<Vec<_>>>()?;
    let discussion = if rows.is_empty() {
        view!{<div class="article-comments-empty"><p>{page.phrase("No discussion has been attached to this article yet.")}</p></div>}.into_any()
    } else {
        view! {<div class="article-comment-list">{comments.collect_view()}</div>}.into_any()
    };
    Ok(view!{<main id="main-content" class="knowledge-app article-detail" tabindex="-1"><div class="knowledge-shell knowledge-shell-reading">
        <a class="article-backlink" href="/">{page.phrase("Back to knowledge console")}</a>
        <header class="article-hero knowledge-panel"><p class="knowledge-section-label">{page.phrase("Publication Entry")}</p>
            <h1 class="article-title">{title}</h1><div class="article-meta"><span>{date}</span><span>{author}</span><span>{reading}</span></div><div class="article-tags">{tags}</div></header>
        {banner}<section class="article-reading-surface knowledge-panel"><div class="article-prose" inner_html=contents/></section>
        <section class="article-comments knowledge-panel"><div class="article-comments-head"><div><p class="knowledge-section-label">{page.phrase("Discussion")}</p>
            <h2 class="article-comments-title">{page.phrase(&get_comment_count_label(rows.len()))}</h2></div>
            <p class="article-comments-copy">{page.phrase("Reader responses attached to this entry appear here as part of the article record.")}</p></div>{discussion}</section>
    </div></main>}.into_any())
}
