//! Finite development decisions; none of these tests opens a TCP listener.
use super::*;
use change::*;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// CLI defaults and invalid input fail before any process or socket is opened.
#[test]
fn validates_ports() {
    let parse =
        |s: &str| Options::parse(&s.split_whitespace().map(str::to_owned).collect::<Vec<_>>());
    assert_eq!(
        parse("").unwrap(),
        Options {
            port: 3000,
            reload_port: 3001
        }
    );
    assert_eq!(parse("--port 4000").unwrap().reload_port, 4001);
    assert_eq!(
        parse("--reload-port 4500 --port 65535")
            .unwrap()
            .reload_port,
        4500
    );
    for invalid in [
        "--port",
        "--port 0",
        "--wat 2",
        "--port x",
        "--port 65535",
        "--reload-port 3000",
    ] {
        assert!(parse(invalid).is_err(), "{invalid}");
    }
}

/// Every input family selects its minimum safe build, including mixed CSS and source edits.
#[test]
fn classifies_build_work() {
    let plan = |paths: &[&str]| classify(&paths.iter().map(PathBuf::from).collect());
    let schema = plan(&["schema.idea"]);
    assert!(schema.generate && schema.browser && schema.native && schema.restart);
    let web = plan(&["web/src/render.rs"]);
    assert!(web.native && !web.browser);
    let ui = plan(&["plugins/app/src/views/home.rs"]);
    assert!(ui.native && ui.browser);
    let native = plan(&["plugins/app/src/pages/home.rs"]);
    assert!(native.native && !native.browser);
    assert!(!plan(&["plugins/app/src/server.rs"]).browser);
    assert!(plan(&["Cargo.toml"]).browser);
    assert!(plan(&["Cargo.lock"]).native);
    let client = plan(&["client/src/lib.rs"]);
    assert!(client.browser && !client.native && client.restart);
    let config = plan(&["config/develop.json"]);
    assert!(config.restart && !config.native);
    let css = plan(&["plugins/assets/assets/global.css"]);
    assert_eq!(css.css, BTreeSet::from(["styles/global.css".into()]));
    assert!(!css.native && !css.browser && !css.restart);
    let mixed = plan(&[
        "plugins/assets/assets/global.css",
        "plugins/store/src/connect.rs",
    ]);
    assert!(mixed.native && mixed.restart);
    assert_eq!(plan(&[]), Work::default());
}

/// Content polling catches edits, additions and removals while ignoring output and symlinks.
#[test]
fn polls_authored_content() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("plugins/app/target")).unwrap();
    std::fs::write(root.join("plugins/app/input.rs"), "old").unwrap();
    std::fs::write(root.join("plugins/app/input.rs.tmp"), "editor temporary").unwrap();
    std::fs::write(root.join("plugins/app/.input.rs.swp"), "editor swap").unwrap();
    std::fs::write(root.join("plugins/app/target/noise"), "ignore").unwrap();
    let old = Snapshot::read(root).unwrap();
    assert_eq!(old.0.len(), 1);
    std::fs::write(root.join("plugins/app/input.rs"), "new").unwrap();
    std::fs::write(root.join("schema.idea"), "model").unwrap();
    let next = Snapshot::read(root).unwrap();
    assert_eq!(old.changes(&next).len(), 2);
    std::fs::remove_file(root.join("plugins/app/input.rs")).unwrap();
    assert_eq!(next.changes(&Snapshot::read(root).unwrap()).len(), 1);
    assert!(next.changes(&next).is_empty());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("schema.idea"), root.join("web")).unwrap();
        assert!(
            !Snapshot::read(root)
                .unwrap()
                .0
                .contains_key(&PathBuf::from("web"))
        );
    }
}

/// A quiet poll does not postpone work; bursts union paths and dispatch exactly once.
#[test]
fn debounces_at_the_last_edit() {
    let now = Instant::now();
    let delay = Duration::from_millis(100);
    let mut debounce = Debounce::default();
    assert!(debounce.ready(now, delay).is_none());
    debounce.observe(BTreeSet::from([PathBuf::from("a")]), now);
    debounce.observe(BTreeSet::new(), now + delay);
    assert!(debounce.ready(now + delay / 2, delay).is_none());
    debounce.observe(BTreeSet::from([PathBuf::from("b")]), now + delay / 2);
    assert!(debounce.ready(now + delay, delay).is_none());
    assert_eq!(
        debounce
            .ready(now + delay + delay / 2, delay)
            .unwrap()
            .len(),
        2
    );
    assert!(debounce.ready(now + delay * 2, delay).is_none());
}

/// Parsing/diffing still works; rendering/hydration compatibility is a separate prototype.
#[test]
fn official_view_macros_still_produce_static_markup_patches() {
    use quote::ToTokens;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("view.rs");
    let before = "fn page() { view! { <p class=\"before\">\"Before\"</p> }; }";
    let after = "fn page() { view! { <p class=\"after\">\"After\"</p> }; }";
    // Use syn's real AST to demonstrate this fixture changes only its macro tokens.
    let old = syn::parse_file(before)
        .unwrap()
        .into_token_stream()
        .to_string();
    let new = syn::parse_file(after)
        .unwrap()
        .into_token_stream()
        .to_string();
    assert_eq!(
        old.replace("before", "after").replace("Before", "After"),
        new
    );
    std::fs::write(&path, before).unwrap();
    let views = leptos_hot_reload::ViewMacros::new();
    views.update_from_paths(&[&path]).unwrap();
    std::fs::write(&path, after).unwrap();
    let patches = views.patch(&path.try_into().unwrap()).unwrap().unwrap();
    let json = serde_json::to_string(&patches).unwrap();
    assert!(json.contains("After"));
    assert!(json.contains("SetAttribute"));
}

/// Optional real-app patch fixture for the finite browser prototype; authored source stays intact.
#[test]
fn real_home_markup_patch_prototype() {
    let Some(output) = std::env::var_os("BLOG_VIEW_PATCH_OUTPUT") else {
        return;
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned();
    let before = std::fs::read_to_string(root.join("plugins/app/src/views/home.rs")).unwrap();
    let after = before
        .replace("knowledge-topbar-copy", "knowledge-topbar-copy-patched")
        .replace("\"The Blog\"", "\"The Blog (patched)\"");
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("home.rs");
    std::fs::write(&path, &before).unwrap();
    let views = leptos_hot_reload::ViewMacros::new();
    views.update_from_paths(&[&path]).unwrap();
    std::fs::write(&path, after).unwrap();
    let mut patches = views.patch(&path.try_into().unwrap()).unwrap().unwrap();
    for (id, _) in &mut patches.0 {
        let line = id.rsplit('-').next().unwrap();
        *id = format!("plugins-app-src-views-home.rs-{line}");
    }
    std::fs::write(&output, serde_json::to_string(&patches).unwrap()).unwrap();
    std::fs::write(
        PathBuf::from(output).with_extension("js"),
        leptos_hot_reload::HOT_RELOAD_JS,
    )
    .unwrap();
}

/// Signal registration/removal does not need a listener or a process-wide test signal.
#[test]
fn owns_its_signal_registrations() {
    let flag = Arc::new(AtomicBool::new(false));
    let signals = Signals::register(flag.clone()).unwrap();
    assert_eq!(signals.0.len(), 2);
    drop(signals);
    assert!(!flag.load(Ordering::Relaxed));
}

/// Build errors leave a real child alive; replacement and Drop reap its entire group.
#[cfg(unix)]
#[test]
fn retains_last_good_child_and_reaps_replacements() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    std::fs::write(&first, "owned executable").unwrap();
    std::fs::write(&second, "owned executable").unwrap();
    let mut session = Session::default();
    session
        .replace(
            Command::new("/bin/sh").args(["-c", "sleep 60 & wait"]),
            first.clone(),
        )
        .unwrap();
    let first_id = session.child.as_ref().unwrap().child.id();
    let mut builder = Builder::new(root.path(), None).unwrap();
    assert!(
        session
            .update(
                root.path(),
                &Options::parse(&[]).unwrap(),
                &mut builder,
                &Work {
                    native: true,
                    restart: true,
                    ..Default::default()
                },
                |_, _| Err("compiler failure".into())
            )
            .is_err()
    );
    assert_eq!(session.child.as_ref().unwrap().child.id(), first_id);
    assert!(
        session
            .child
            .as_mut()
            .unwrap()
            .child
            .try_wait()
            .unwrap()
            .is_none()
    );
    let notices = session
        .update(
            root.path(),
            &Options::parse(&[]).unwrap(),
            &mut builder,
            &classify(&BTreeSet::from([PathBuf::from(
                "plugins/assets/assets/global.css",
            )])),
            |_, _| panic!("CSS compiled"),
        )
        .unwrap();
    assert_eq!(notices, vec![Notice::Css("styles/global.css".into())]);
    session
        .replace(
            Command::new("/bin/sh").args(["-c", "exit 0"]),
            second.clone(),
        )
        .unwrap();
    assert!(!first.exists());
    let deadline = Instant::now() + Duration::from_secs(3);
    while session.child.is_some() && Instant::now() < deadline {
        session.check_exit().unwrap();
        std::thread::yield_now();
    }
    assert!(session.child.is_none());
    drop(session);
    assert!(!second.exists());
    let mut empty = Session::default();
    assert!(
        empty
            .update(
                root.path(),
                &Options::parse(&[]).unwrap(),
                &mut builder,
                &Work {
                    restart: true,
                    ..Default::default()
                },
                |_, _| Ok(())
            )
            .is_err()
    );
    assert!(
        empty
            .replace(
                &mut Command::new("/no-such-dev-command"),
                root.path().join("missing")
            )
            .is_err()
    );
}
