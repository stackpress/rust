//! Ordinary input/display decision tables and native semantic markup.
use super::*;
use leptos::prelude::*;

/// Typed values reject invalid-to-zero/CSS/URL coercions while preserving empty and zero.
#[test]
fn checked_display_boundaries() {
    assert!(ProgressValue::determinate(0.0, 1.0).is_ok());
    for (value, max) in [
        (f64::NAN, 1.0),
        (f64::INFINITY, 1.0),
        (0.0, 0.0),
        (-1.0, 1.0),
        (2.0, 1.0),
    ] {
        assert!(ProgressValue::determinate(value, max).is_err());
    }
    for (value, max) in [(f64::NAN, 5), (-1.0, 5), (6.0, 5), (1.0, 0), (1.0, 101)] {
        assert!(RatingValue::new(value, max).is_err());
    }
    let rating = RatingValue::new(2.5, 5).unwrap();
    assert_eq!(rating.stars(RatingRound::Nearest, true), "★★★☆☆");
    assert_eq!(rating.stars(RatingRound::Ceil, false), "★★★");
    assert_eq!(rating.stars(RatingRound::Floor, true), "★★☆☆☆");
    for value in ["#000", "#aBcD09"] {
        assert_eq!(HexColor::new(value).unwrap().as_str(), value);
    }
    for value in [
        "",
        "red",
        "#ab",
        "#12345678",
        "#fff;background:url(x)",
        "#雪",
    ] {
        assert!(HexColor::new(value).is_err());
    }
    assert_eq!(
        PhoneNumber::new("+12025550123").unwrap().href(),
        "tel:+12025550123"
    );
    for value in [
        "",
        "+",
        "+0",
        "+0123",
        "123",
        "+12;ext=4",
        "+1234567890123456",
        "+雪",
    ] {
        assert!(PhoneNumber::new(value).is_err());
    }
    assert_eq!(
        EmailAddress::new("a+%0d@example.invalid").unwrap().href(),
        "mailto:a%2B%250d@example.invalid"
    );
    for value in [
        "",
        "a",
        ".a@b",
        "a..b@c",
        "a@",
        "a@b@c",
        "a@-b",
        "a@b..c",
        "a@b?subject=x",
        "a\r\n@b",
        "雪@b",
    ] {
        assert!(EmailAddress::new(value).is_err());
    }
    assert!(EmailAddress::new(format!("{}@b", "a".repeat(65))).is_err());
    assert!(ImageSource::new("/assets/a.png", 1, 1, 4096).is_ok());
    for (path, bytes, width, height) in [
        ("https://evil.invalid/a.png", 1, 1, 1),
        ("//evil.invalid/a.png", 1, 1, 1),
        ("/a.svg", 1, 1, 1),
        ("/a.png?x=.png", 1, 1, 1),
        ("/a.png", 0, 1, 1),
        ("/a.png", 1024 * 1024 + 1, 1, 1),
        ("/a.png", 1, 0, 1),
        ("/a.png", 1, 1, 4097),
    ] {
        assert!(ImageSource::new(path, bytes, width, height).is_err());
    }
    let error = ImageSource::new("//evil.invalid/a.png", 1, 1, 1).unwrap_err();
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(error.to_string(), "Invalid display value");
}

/// UTF-8 and explicit zero/word/ellipsis policy are independent of ambient locale.
#[test]
fn text_and_collection_boundaries() {
    for (text, limit, ellipsis, expected) in [
        ("雪😀abc", TextLimit::Characters(2), true, "雪😀…"),
        ("a b", TextLimit::Characters(2), false, "a "),
        ("abc", TextLimit::Characters(0), true, "…"),
        ("", TextLimit::Characters(0), true, ""),
        (" a  雪 b ", TextLimit::Words(2), true, " a  雪…"),
        ("  ", TextLimit::Words(0), false, ""),
        ("a b", TextLimit::Words(3), true, "a b"),
    ] {
        assert_eq!(overflow_text(text, limit, ellipsis), expected);
    }
    assert_eq!(
        DisplayValues::new(vec!["".into(), "0".into()])
            .unwrap()
            .entries(),
        ["", "0"]
    );
    assert!(DisplayValues::new(vec!["x".into(); 101]).is_err());
    assert!(DisplayValues::new(vec!["x".repeat(4097)]).is_err());
    assert!(MetadataValues::new(vec![("a".into(), "0".into())]).is_ok());
    for values in [
        vec![("".into(), "0".into())],
        vec![("a".into(), "0".into()), ("a".into(), "1".into())],
        vec![("a".into(), "x".repeat(4097))],
    ] {
        assert!(MetadataValues::new(values).is_err());
    }
    assert!(DisplayTable::new(vec!["A".into()], vec![]).is_ok());
    for (columns, rows) in [
        (vec![], vec![]),
        (vec!["A".into(), "A".into()], vec![]),
        (
            vec!["A".into()],
            vec![DisplayRow {
                key: "1".into(),
                cells: vec![],
            }],
        ),
        (
            vec!["A".into()],
            vec![
                DisplayRow {
                    key: "1".into(),
                    cells: vec!["0".into()]
                };
                2
            ],
        ),
    ] {
        assert!(DisplayTable::new(columns, rows).is_err());
    }
}

/// Native selection admits known enabled options and distinguishes None/Some(empty).
#[test]
fn select_admission_and_state() {
    let option = |value: &str, disabled| SelectOption {
        value: value.into(),
        label: value.into(),
        disabled,
    };
    assert!(SelectOptions::new(vec![option("a", false), option("a", false)]).is_err());
    assert!(SelectOptions::new(vec![option("__stackpress_unselected__", false)]).is_err());
    assert!(SelectOptions::new(vec![option(&"x".repeat(1025), false)]).is_err());
    assert!(
        SelectOptions::new(
            (0..101)
                .map(|index| option(&index.to_string(), false))
                .collect()
        )
        .is_err()
    );
    let options = SelectOptions::new(vec![
        option("", false),
        option("0", false),
        option("blocked", true),
    ])
    .unwrap();
    assert_eq!(options.entries().len(), 3);
    assert_eq!(
        options.admit(&[
            "unknown".into(),
            "blocked".into(),
            "0".into(),
            "".into(),
            "0".into()
        ]),
        ["", "0"]
    );
    let owner = Owner::new();
    owner.with(|| {
        let value = RwSignal::new(Some(String::new()));
        let single = SelectValue::Single(value);
        single.update(&options, &["0".into()]);
        assert_eq!(value.get().as_deref(), Some("0"));
        single.update(&options, &["blocked".into()]);
        assert_eq!(value.get(), None);
        single.update(&options, &[String::new()]);
        assert_eq!(value.get().as_deref(), Some(""));
        let values = RwSignal::new(vec![]);
        SelectValue::Multiple(values).update(&options, &["0".into(), String::new()]);
        assert_eq!(values.get(), ["", "0"]);
    });
    owner.cleanup();
}

/// Real native SSR structures cover every ordinary control and safe display family.
#[test]
fn ordinary_semantic_markup() {
    let owner = Owner::new();
    owner.with(|| {
        let text=RwSignal::new("\n</textarea><script>globalThis.pwned=1</script>&\"'\u{2028}".to_owned()); let selected=RwSignal::new(None::<String>);let checked=RwSignal::new(false);
        let html=view! {
            <Input id="text" name="text" value=text required=true read_only=true/>
            <Textarea id="notes" name="notes" value=text rows=0 required=true/>
            <PasswordInput id="pw" name="pw" value=text/>
            <Checkbox id="c" name="c" checked/><Switch id="s" name="s" checked/>
            <Radio id="r" name="group" value="0" selected/>
            <Select id="single" name="single" value=SelectValue::Single(selected) options=SelectOptions::new(vec![]).unwrap()/>
            <Select id="multi" name="multi" value=SelectValue::Multiple(RwSignal::new(vec![])) options=SelectOptions::new(vec![]).unwrap()/>
            <Card label="Example"><CardHead><CardTitle>"Heading"</CardTitle><CardDescription>"Description"</CardDescription></CardHead><CardBody><Badge>"0"</Badge></CardBody><CardFoot><ActionGroup label="Actions"><button>"Go"</button></ActionGroup></CardFoot></Card>
            <Progress id="known" label="Known" value=ProgressValue::determinate(0.0,1.0).unwrap()/>
            <Progress id="unknown" label="Unknown" value=ProgressValue::Indeterminate/>
            <ColorView value=HexColor::new("#abc").unwrap()/><ColorView value=HexColor::new("#000").unwrap() swatch=false/>
            <EmailLink value=EmailAddress::new("a+b@c.invalid").unwrap()/><PhoneLink value=PhoneNumber::new("+123").unwrap()/>
            <Image source=ImageSource::new("/a.png",68,1,1).unwrap() alt="Example"/>
            <RatingView value=RatingValue::new(0.0,5).unwrap()/><YesNo value=false yes="Oui".to_owned() no="Non".to_owned()/><YesNo value=true/>
            <TextOverflow value="雪😀abc" limit=TextLimit::Characters(2)/>
            <TextTransform value="<b>safe</b>" format=TextCase::Uppercase/>
            <TextTransform value="X" format=TextCase::Lowercase/><TextTransform value="x" format=TextCase::Capitalize/><TextTransform value="x"/>
            <MetadataView value=MetadataValues::new(vec![("Key".into(),"0".into())]).unwrap()/>
            <Spread value=DisplayValues::new(vec!["0".into(),"<safe>".into()]).unwrap() separator=SpreadSeparator::Inline(" | ".into())/>
            <Spread value=DisplayValues::new(vec!["x".into()]).unwrap() separator=SpreadSeparator::Lines/>
            <Tags value=DisplayValues::new(vec!["<safe>".into()]).unwrap()/>
            <Tabular caption="Empty" value=DisplayTable::new(vec!["Key".into()],vec![]).unwrap()/>
            <Tabular caption="Data" value=DisplayTable::new(vec!["Key".into()],vec![DisplayRow {key:"row".into(),cells:vec!["0".into()]}]).unwrap()/>
        }.to_html();
        for needle in ["readonly","required","rows=\"1\"","type=\"password\"","aria-controls=\"pw\"","role=\"switch\"","type=\"radio\"","multiple","__stackpress_unselected__","class=\"sp-card\"","<progress id=\"known\" max=\"1\" value=\"0\"","<progress id=\"unknown\" max=\"1\"","mailto:a%2Bb@c.invalid","tel:+123","alt=\"Example\"","0 / 5","Non","Yes","雪😀…","<dl","colspan=\"1\"","data-row-id=\"row\""] { assert!(html.contains(needle),"missing {needle}: {html}"); }
        assert!(!html.contains("<script>"));assert!(html.contains("&lt;/textarea&gt;&lt;script&gt;globalThis.pwned=1&lt;/script&gt;&amp;\"'\u{2028}"));
    });
    owner.cleanup();
}

/// Characterize pinned raw-text behavior without ever serving the unsafe output.
#[test]
fn pinned_leptos_raw_text_characterization() {
    let payload =
        "</textarea></style></script></title><script>globalThis.pwned=1</script>&\"'\u{2028}";
    let raw=view! { <textarea>{payload}</textarea><style>{payload}</style><script type="application/json">{payload}</script> }.to_html();
    // This expected counterexample proves why generic text escaping is insufficient here.
    assert_eq!(raw.matches(payload).count(), 3);
    let title = view! { <title>{payload}</title> }.to_html();
    assert!(!title.contains("<script>globalThis.pwned"));
    assert!(title.contains("&lt;/title&gt;"));
}
