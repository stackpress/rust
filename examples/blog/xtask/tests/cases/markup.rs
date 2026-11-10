//! Source safety and cache transitions use real syn/Leptos parsers and owned files.
use super::*;

/// Markup can change; Rust expressions, props, macro locations and types cannot.
#[test]
fn separates_static_markup_from_rust_and_props() {
    let before = r#"fn page() { view! { <p class="before">"Before"</p> }; }"#;
    assert!(pure(
        before.as_bytes(),
        before.replace("Before", "After").as_bytes()
    ));
    assert!(pure(
        before.as_bytes(),
        r#"fn page() { view! { <div><span>"After"</span></div> }; }"#.as_bytes()
    ));
    assert!(pure(
        before.as_bytes(),
        r#"fn page() { view! { <><p>"One"</p><p>"Two"</p></> }; }"#.as_bytes()
    ));
    let mixed = r#"fn page() { view! { <p title=value>"Before"{count}</p> }; }"#;
    assert!(pure(
        mixed.as_bytes(),
        mixed.replace("Before", "After").as_bytes()
    ));
    for after in [
        mixed.replace("count", "other"),
        mixed.replace("value", "other"),
        mixed
            .replace("p title", "div title")
            .replace("/p>", "/div>"),
        format!("\n{mixed}"),
        format!("{mixed}\nfn logic() {{}}"),
        "invalid Rust".into(),
    ] {
        assert!(!pure(mixed.as_bytes(), after.as_bytes()), "{after}");
    }
    assert!(!pure(before.as_bytes(), before.as_bytes()));
    assert!(!pure(b"\xff", before.as_bytes()));
    assert!(!pure(before.as_bytes(), b"fn page() { view! { < }; }"));
    assert!(!pure(b"fn page(){}", b"fn page(){}"));
    let props = r#"fn page() { view! { <p>"Before"<Widget label="same"/></p> }; }"#;
    assert!(!pure(
        props.as_bytes(),
        props
            .replace("Before", "After")
            .replace("same", "changed")
            .as_bytes()
    ));
    let multiple = r#"fn page() { view! { <p>"Before"</p> }; view! { <p title=value/> }; }"#;
    assert!(!pure(
        multiple.as_bytes(),
        multiple
            .replace("Before", "After")
            .replace("value", "other")
            .as_bytes()
    ));
    assert!(!pure(
        before.as_bytes(),
        format!("{before} fn other() {{view! {{<p/>}};}}").as_bytes()
    ));
    let collision = r#"fn page(){view!{<p title=__static_string__>"Before"{n}</p>};}"#;
    assert!(!pure(
        collision.as_bytes(),
        collision
            .replace("title=__static_string__", "title=\"__static_string__\"")
            .replace("Before", "After")
            .as_bytes()
    ));
}

/// Consecutive patches advance the baseline and a later full reload compiles accepted markup.
#[test]
fn advances_only_successful_patch_baselines() {
    let root = tempfile::tempdir().unwrap();
    let relative = PathBuf::from("plugins/app/src/views/page.rs");
    let path = root.path().join(&relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let first = r#"fn page(){view!{<p>"First"</p>};}"#;
    std::fs::write(&path, first).unwrap();
    let snapshot = Snapshot::read(root.path()).unwrap();
    let mut markup = Markup::new(root.path(), &snapshot);
    let paths = BTreeSet::from([relative.clone()]);
    std::fs::write(&path, first.replace("First", "Second")).unwrap();
    let next = Snapshot::read(root.path()).unwrap();
    let json = markup.patch(&paths, &next).unwrap();
    assert!(json.contains("Second"));
    assert!(json.contains("plugins-app-src-views-page.rs-1"));
    assert!(!json.contains(root.path().to_str().unwrap()));
    let css = Work {
        css: BTreeSet::from(["styles/global.css".into()]),
        ..Default::default()
    };
    assert_eq!(markup.rebuild_work(css.clone()), css);
    let work = markup.rebuild_work(Work {
        restart: true,
        ..Default::default()
    });
    assert!(work.native && work.browser && work.restart);
    std::fs::write(&path, first.replace("First", "Third")).unwrap();
    assert!(
        markup
            .patch(&paths, &Snapshot::read(root.path()).unwrap())
            .unwrap()
            .contains("Third")
    );
    assert!(markup.patch(&BTreeSet::new(), &next).is_none());
    assert!(
        markup
            .patch(
                &BTreeSet::from([PathBuf::from("config/develop.json")]),
                &next
            )
            .is_none()
    );
    // A live file changed after the snapshot; do not patch bytes that were never validated.
    assert!(markup.patch(&paths, &next).is_none());
}

/// Parser/cache failures and removed source take the rebuild path until a successful reset.
#[test]
fn failed_or_unknown_views_rebuild_safely() {
    let root = tempfile::tempdir().unwrap();
    let relative = PathBuf::from("plugins/app/src/views/page.rs");
    let path = root.path().join(&relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let first = r#"fn page(){view!{<p>"First"</p>};}"#;
    std::fs::write(&path, first).unwrap();
    let snapshot = Snapshot::read(root.path()).unwrap();
    let mut markup = Markup::new(root.path(), &snapshot);
    markup.views.update_from_paths::<PathBuf>(&[]).unwrap();
    std::fs::write(&path, first.replace("First", "Second")).unwrap();
    let next = Snapshot::read(root.path()).unwrap();
    let paths = BTreeSet::from([relative]);
    assert!(markup.patch(&paths, &next).is_none());
    assert!(!markup.valid);
    assert!(markup.patch(&paths, &next).is_none());
    let mut reset = Markup::new(root.path(), &next);
    assert!(reset.valid);
    std::fs::remove_file(&path).unwrap();
    assert!(
        reset
            .patch(&paths, &Snapshot::read(root.path()).unwrap())
            .is_none()
    );
    assert!(!Markup::new(root.path(), &next).valid);
    std::fs::write(&path, "bad Rust").unwrap();
    assert!(!Markup::new(root.path(), &Snapshot::read(root.path()).unwrap()).valid);
}
