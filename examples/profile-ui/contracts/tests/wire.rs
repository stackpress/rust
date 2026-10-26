//! Strict input/reply codecs and generated public source reuse.
use stackpress_example_profile_ui_contracts::*;

/// Every admitted public row must have an unambiguous key for rendering and detail links.
#[test]
fn replies_reject_ambiguous_row_identity() {
    for ids in [vec![""], vec!["abc", "abc"]] {
        let rows = ids
            .into_iter()
            .map(|id| Profile {
                id: id.into(),
                name: "Public fixture".into(),
            })
            .collect::<Vec<_>>();
        let reply = SearchReply {
            revision: REVISION,
            total: rows.len() as u64,
            rows,
        };
        assert!(
            reply.validate().is_err(),
            "ambiguous identity must not enter the keyed UI"
        );
        let body = serde_json::to_string(&reply).unwrap();
        assert!(decode_search_reply(&body, None).is_err());
        assert!(
            SearchReply::from_envelope(
                &serde_json::json!({"results":reply.rows,"total":reply.total})
            )
            .is_err()
        );
    }
}

/// Public boundaries reject drift, malformed envelopes and unbounded fields.
#[test]
fn strict_input_and_public_projection() {
    assert!(SearchInput::new(String::new()).is_err());
    assert!(SearchInput::new("x".repeat(1025)).is_err());
    assert!(SearchInput::new("é".repeat(512)).is_ok());
    assert!(
        SearchInput {
            revision: 9,
            id: "abc".into()
        }
        .validate()
        .is_err()
    );
    assert!(
        serde_json::from_str::<SearchInput>(r#"{"revision":1,"id":"abc","event":"shell"}"#)
            .is_err()
    );
    let empty = SearchReply::from_envelope(
        &serde_json::json!({"results":[],"total":0,"private":"sentinel"}),
    )
    .unwrap();
    assert_eq!(empty.total, 0);
    assert!(!serde_json::to_string(&empty).unwrap().contains("sentinel"));
    for value in [
        serde_json::json!({}),
        serde_json::json!({"results":{},"total":1}),
        serde_json::json!({"results":[]}),
        serde_json::json!({"results":[],"total":1}),
    ] {
        assert!(SearchReply::from_envelope(&value).is_err());
    }
    let mut reply = SearchReply {
        revision: REVISION,
        rows: vec![Profile {
            id: "abc".into(),
            name: "Ada".into(),
        }],
        total: 1,
    };
    reply.revision = 99;
    assert!(reply.validate().is_err());
    reply.revision = REVISION;
    reply.rows[0].name = "x".repeat(4097);
    assert!(reply.validate().is_err());
    reply.rows[0].name = "Ada".into();
    let page = snapshot(SearchPage {
        detail: None,
        input: SearchInput::new("abc".into()).unwrap(),
        reply,
    })
    .unwrap();
    page.validate(REVISION, BUILD).unwrap();
    assert_eq!(page.location, "/ui/profiles?id=abc");
    let unicode = snapshot(SearchPage {
        detail: None,
        input: SearchInput::new("é &".into()).unwrap(),
        reply: empty,
    })
    .unwrap();
    assert_eq!(unicode.location, "/ui/profiles?id=%C3%A9%20%26");
}
