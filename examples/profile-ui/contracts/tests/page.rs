//! Checked page-root selection, independent origin, and native missing-detail status.
use stackpress_example_profile_ui_contracts::*;

/// Search/detail identity and status cannot be substituted by arbitrary wire data.
#[test]
fn page_identity_and_detail_status_agree() {
    let mut data = SearchPage {
        input: SearchInput::new("abc456".into()).unwrap(),
        reply: SearchReply {
            revision: REVISION,
            rows: vec![Profile {
                id: "abc456".into(),
                name: "Grace Hopper".into(),
            }],
            total: 1,
        },
        detail: None,
    };
    assert!(validate_page("profiles", &PublicStatus::ok(), &data).is_ok());
    assert!(validate_page("unknown", &PublicStatus::ok(), &data).is_err());
    assert!(validate_page("profile-detail", &PublicStatus::ok(), &data).is_err());
    assert!(validate_page("profiles", &PublicStatus::new(404, None).unwrap(), &data).is_err());
    data.detail = Some(DetailContext {
        id: "abc456".into(),
        search_prefix: "abc".into(),
    });
    let found = snapshot(data).unwrap();
    assert_eq!(found.page.as_str(), "profile-detail");
    assert_eq!(found.location, "/ui/profiles/abc456?return=abc");
    assert!(validate_page(found.page.as_str(), &found.status, &found.data).is_ok());
    let mut data = found.data;
    assert!(validate_page("profiles", &PublicStatus::ok(), &data).is_err());
    assert!(
        validate_page(
            "profile-detail",
            &PublicStatus::new(404, None).unwrap(),
            &data
        )
        .is_err()
    );
    data.reply.rows.clear();
    data.reply.total = 0;
    let missing = snapshot(data).unwrap();
    assert_eq!(missing.status.code(), 404);
    assert!(validate_page("profile-detail", &missing.status, &missing.data).is_ok());
    let mut data = missing.data;
    data.detail.as_mut().unwrap().id = "different".into();
    assert!(validate_page("profile-detail", &missing.status, &data).is_err());
    data.detail.as_mut().unwrap().id = data.input.id.clone();
    data.detail.as_mut().unwrap().search_prefix.clear();
    assert!(validate_page("profile-detail", &missing.status, &data).is_err());
}
