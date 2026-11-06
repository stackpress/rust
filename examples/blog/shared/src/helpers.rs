//! The original blog's pure reading helpers, over canonical source JSON date strings.
use serde_json::Value;
use stackpress_framework_schema::{SchemaError, SchemaErrorKind, SchemaResult};
use stackpress_leptos::format::{CalendarDate, DisplayLocale};

/// ECMAScript whitespace differs from Rust on BOM and the NEXT LINE control.
fn space(value: char) -> bool {
    (value.is_whitespace() && value != '\u{0085}') || value == '\u{feff}'
}

/// Match `<[^>]+>` replacement and whitespace collapsing, without interpreting HTML/entities.
pub fn strip_html(html: &str) -> String {
    let mut text = String::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        let Some(end) = rest[start + 1..].find('>').map(|end| start + 1 + end) else {
            break;
        };
        if end == start + 1 {
            text.push_str(&rest[..end + 1]);
            rest = &rest[end + 1..];
            continue;
        }
        text.push_str(&rest[..start]);
        text.push(' ');
        rest = &rest[end + 1..];
    }
    text.push_str(rest);
    text.split(space)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format canonical UTC source dates with the same pinned English medium date as the template.
/// # Errors
/// Invalid nonempty dates remain errors; no replacement date or local timezone is invented.
pub fn format_article_date(value: Option<&str>) -> SchemaResult<String> {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return Ok("Unscheduled".into());
    };
    let date = value.get(..10).unwrap_or(value);
    CalendarDate::parse(date)
        .and_then(|date| date.display(DisplayLocale::English))
        .map_err(|error| {
            SchemaError::new(SchemaErrorKind::Codec, "invalid article date").with_cause(error)
        })
}

/// The first supplied article is featured; the remaining order is unchanged.
pub fn get_featured_article<T>(articles: &[T]) -> (Option<&T>, &[T]) {
    (articles.first(), articles.get(1..).unwrap_or(&[]))
}

/// Preserve the source fallback and JavaScript UTF-16 slice semantics at the requested limit.
pub fn get_article_summary(contents: Option<&str>, limit: usize) -> String {
    let text = strip_html(contents.unwrap_or_default());
    if text.is_empty() {
        return "Open the article to read the full entry.".into();
    }
    let units = text.encode_utf16().collect::<Vec<_>>();
    if units.len() <= limit {
        return text;
    }
    format!(
        "{}...",
        String::from_utf16_lossy(&units[..limit]).trim_end_matches(space)
    )
}

/// Use 180 words per minute, rounded upward, with the original one-minute minimum.
pub fn get_reading_time_label(contents: Option<&str>) -> String {
    let count = strip_html(contents.unwrap_or_default())
        // strip_html already collapses exactly ECMAScript whitespace to ASCII spaces.
        // Rust split_whitespace would additionally split U+0085 and overcount source words.
        .split(' ')
        .filter(|word| !word.is_empty())
        .count();
    format!("{} min read", count.div_ceil(180).max(1))
}

/// Stable source labels for the homepage metrics.
pub fn get_content_signals(articles: &[Value]) -> SchemaResult<Vec<(&'static str, String)>> {
    // Generated native Date values cross the page boundary as canonical UTC ISO strings.
    let latest = articles
        .iter()
        .filter_map(|row| row["published"].as_str())
        .filter(|value| format_article_date(Some(value)).is_ok())
        .max();
    Ok(vec![
        ("Published", articles.len().to_string()),
        (
            "With Visuals",
            articles
                .iter()
                .filter(|row| {
                    row["banner"]
                        .as_str()
                        .is_some_and(|value| !value.is_empty())
                })
                .count()
                .to_string(),
        ),
        ("Latest", format_article_date(latest)?),
    ])
}

/// Original four orientation statements, in source order.
pub fn get_topic_highlights() -> [&'static str; 4] {
    [
        "Systems and platform notes",
        "Design-aware implementation details",
        "Operational thinking for product builders",
        "Patterns that scale from content to application UI",
    ]
}

/// Original zero, singular and plural comment labels.
pub fn get_comment_count_label(count: usize) -> String {
    match count {
        0 => "No comments yet".into(),
        1 => "1 comment".into(),
        count => format!("{count} comments"),
    }
}
