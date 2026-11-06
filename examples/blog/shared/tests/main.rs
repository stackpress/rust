//! Source helper and portable seed boundary proofs, independent of SQLite/native services.
use blog_shared::{helpers::*, root::Root};
use serde_json::json;

#[test]
fn strips_tags_and_collapses_source_whitespace() {
    assert_eq!(
        strip_html(" <p>Hello</p>\n<strong>world</strong> "),
        "Hello world"
    );
    assert_eq!(strip_html("<> &amp; <unfinished"), "<> &amp; <unfinished");
    assert_eq!(strip_html("\u{feff}a\u{0085}b\u{feff}"), "a\u{0085}b");
}
#[test]
fn summaries_keep_fallback_limit_and_utf16_boundaries() {
    assert_eq!(
        get_article_summary(None, 140),
        "Open the article to read the full entry."
    );
    assert_eq!(
        get_article_summary(Some("<p>hello world</p>"), 5),
        "hello..."
    );
    assert_eq!(get_article_summary(Some("a😀b"), 3), "a😀...");
    assert_eq!(get_article_summary(Some("hello"), 5), "hello");
}
#[test]
fn reading_time_uses_180_words_and_one_minimum() {
    assert_eq!(get_reading_time_label(None), "1 min read");
    assert_eq!(
        get_reading_time_label(Some(&"word ".repeat(180))),
        "1 min read"
    );
    assert_eq!(
        get_reading_time_label(Some(&"word ".repeat(181))),
        "2 min read"
    );
    assert_eq!(
        get_reading_time_label(Some(&"one\u{0085}word ".repeat(100))),
        "1 min read"
    );
}
#[test]
fn dates_keep_english_medium_fallback_and_errors() {
    assert_eq!(format_article_date(None).unwrap(), "Unscheduled");
    assert_eq!(
        format_article_date(Some("2026-02-27T00:00:00.000Z")).unwrap(),
        "Feb 27, 2026"
    );
    assert!(format_article_date(Some("invalid")).is_err());
}
#[test]
fn featured_keeps_query_order_without_reordering() {
    assert_eq!(get_featured_article(&[3, 2, 1]), (Some(&3), &[2, 1][..]));
    assert_eq!(get_featured_article::<u8>(&[]), (None, &[][..]));
}
#[test]
fn metrics_topics_and_comment_labels_match_source() {
    let rows = vec![
        json!({"published":"2026-02-27T00:00:00Z","banner":"image"}),
        json!({"published":"2026-01-01T00:00:00Z"}),
    ];
    assert_eq!(
        get_content_signals(&rows).unwrap(),
        vec![
            ("Published", "2".into()),
            ("With Visuals", "1".into()),
            ("Latest", "Feb 27, 2026".into())
        ]
    );
    assert_eq!(get_topic_highlights().len(), 4);
    assert_eq!(get_comment_count_label(0), "No comments yet");
    assert_eq!(get_comment_count_label(1), "1 comment");
    assert_eq!(get_comment_count_label(2), "2 comments");
}
#[test]
fn seeds_reject_build_drift_and_relation_choices_keep_title_labels() {
    assert!(Root::decode("missing", "{}").is_err());
    assert!(Root::decode("stackpress-page", r#"{"build":"drift"}"#).is_err());
    let choices = blog_shared::root::choices(
        json!({"results":[{"id":"p","name":"Profile"},{"id":"a","title":"Article"}]}),
    );
    assert_eq!(choices[0].label, "Profile");
    assert_eq!(choices[1].label, "Article");
    assert_eq!(choices[1].row["title"], "Article");
}

/// Native form transport retains nesting, repeated values and source scalar truthiness.
#[test]
fn form_fields_keep_nested_arrays_objects_scalars_and_empty_drafts() {
    let draft = json!({
        "name": "a & b", "enabled": false, "count": 0, "nullable": null,
        "tags": ["rust", "sqlite"], "filter": {"title": "a", "flags": [true, null]},
        "empty_array": [], "empty_object": {}, "rows": [{"id": "p"}]
    });
    assert_eq!(
        blog_shared::form::fields(&draft),
        vec![
            ("name", "a & b"),
            ("enabled", "false"),
            ("count", "0"),
            ("nullable", "null"),
            ("tags[]", "rust"),
            ("tags[]", "sqlite"),
            ("filter[title]", "a"),
            ("filter[flags][]", "true"),
            ("filter[flags][]", "null"),
            ("rows[][id]", "p")
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect::<Vec<_>>()
    );
    for value in [
        json!({}),
        json!([]),
        json!(null),
        json!("text"),
        json!(false),
    ] {
        assert!(blog_shared::form::fields(&value).is_empty());
    }
}

/// Existing checked snapshots remain the seed decoder, including first-present precedence.
#[test]
fn browser_seed_selection_preserves_priority_missing_and_empty_text_errors() {
    use stackpress_framework_view::{PageId, PageSnapshot, PublicStatus};
    let snapshot = PageSnapshot {
        revision: 1,
        build: "blog-v1".into(),
        page: PageId::try_from("blog-home".to_owned()).unwrap(),
        location: "/".into(),
        locale: "en-US".into(),
        theme: None,
        status: PublicStatus::new(200, None).unwrap(),
        data: blog_shared::Page::default(),
    };
    let text = serde_json::to_string(&snapshot).unwrap();
    assert!(matches!(
        blog_shared::browser::decode_seed([
            Some(Some(text.clone())),
            Some(Some("bad".into())),
            None
        ]),
        Ok(Root::Blog(_))
    ));
    assert_eq!(
        blog_shared::browser::decode_seed([None, None, None])
            .err()
            .unwrap(),
        "page seed missing"
    );
    // Empty/null text on a present first element never falls through to the next seed.
    for text in [None, Some(String::new()), Some("not JSON".into())] {
        assert!(blog_shared::browser::decode_seed([Some(text), None, None]).is_err());
    }
    // Later existing seed contracts are selected by their real element ID, not by guessed JSON.
    assert!(blog_shared::browser::decode_seed([None, Some(Some("{}".into())), None]).is_err());
    assert!(blog_shared::browser::decode_seed([None, None, Some(Some("{}".into()))]).is_err());
}

/// Non-success replies retain choices, including redirects and transport/type/read errors.
#[test]
fn browser_relation_status_and_error_mapping_keep_source_failure_policy() {
    use blog_shared::browser::{RelationError, response_status};
    for status in [200, 204, 299] {
        assert_eq!(response_status(status), Ok(()));
    }
    for status in [0, 199, 300, 401, 404, 500] {
        assert_eq!(response_status(status), Err(RelationError::Http(status)));
    }
    for (error, message) in [
        (RelationError::Fetch, "relation fetch unavailable"),
        (RelationError::Response, "relation response unavailable"),
        (
            RelationError::ReadBody,
            "relation response text unavailable",
        ),
        (RelationError::Http(401), "relation response status 401"),
        (
            RelationError::Json("invalid".into()),
            "relation response JSON: invalid",
        ),
        (RelationError::Inactive, "relation request inactive"),
    ] {
        assert_eq!(error.to_string(), message);
    }
}

/// Joined labels, empty replies, parse precedence and stale completions use plain native data.
#[test]
fn browser_relation_json_and_cancellation_preserve_joined_rows_and_precedence() {
    use blog_shared::browser::{RelationError, relation_response};
    for text in [
        r#"{"results":[{"id":"p","name":"Profile"},{"id":"a","title":"Article"},{"id":2,"name":"skip"},{"id":"skip"}]}"#,
        r#"[{"id":"p","name":"Profile"},{"id":"a","title":"Article"}]"#,
    ] {
        let choices = relation_response(Some(text), true, false).unwrap();
        assert_eq!(
            choices
                .iter()
                .map(|row| row.id.as_str())
                .collect::<Vec<_>>(),
            ["p", "a"]
        );
        assert_eq!(choices[1].label, "Article");
        assert_eq!(choices[1].row["title"], "Article");
    }
    for text in ["[]", "{}", "null", r#"{"results":null}"#] {
        assert!(
            relation_response(Some(text), true, false)
                .unwrap()
                .is_empty()
        );
    }
    for (alive, aborted) in [(false, false), (true, true), (false, true)] {
        assert_eq!(
            relation_response(Some("[]"), alive, aborted).err(),
            Some(RelationError::Inactive)
        );
        assert!(matches!(
            relation_response(Some("bad"), alive, aborted),
            Err(RelationError::Json(_))
        ));
    }
    assert!(matches!(
        relation_response(None, true, false),
        Err(RelationError::Json(_))
    ));
}
