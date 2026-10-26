//! One canonical route/query convention for native, hydration and installed clients.
use stackpress_example_profile_ui_contracts::routes::{self, Destination};
use stackpress_framework_view::BasePath;

/// Reserved characters and Unicode roundtrip once; malformed aliases cannot select a target.
#[test]
fn canonical_return_query_and_targets() {
    for mount in ["", "/demo/v1"] {
        let base = BasePath::new(mount).unwrap();
        for prefix in ["abc", "雪/+&=%", "0", "é &", ".", ".."] {
            let url = routes::detail_location(&base, "abc456", prefix).unwrap();
            assert_eq!(
                routes::resolve(&base, url.as_str()).unwrap(),
                Destination::Detail {
                    id: "abc456".into(),
                    search_prefix: prefix.into()
                }
            );
            let query = url.as_str().split_once('?').unwrap().1;
            assert_eq!(
                routes::decode_return_prefix(query).unwrap().as_deref(),
                Some(prefix)
            );
            let search = routes::search_location(&base, prefix).unwrap();
            assert_eq!(
                routes::resolve(&base, search.as_str()).unwrap(),
                Destination::Search(prefix.into())
            );
        }
        assert!(routes::resolve(&base, "/foreign/ui/profiles?id=abc").is_err());
        assert!(routes::resolve(&base, &format!("{mount}/ui/profiles/a/b?return=abc")).is_err());
    }
    assert_eq!(routes::decode_return_prefix("").unwrap(), None);
    for bad in [
        "return=",
        "return=%",
        "return=%GG",
        "return=%FF",
        "return=abc+def",
        "return=abc&return=def",
        "id=abc",
        "return=a&other=b",
        "return=a=b",
    ] {
        assert!(routes::decode_return_prefix(bad).is_err(), "{bad}");
    }
}
