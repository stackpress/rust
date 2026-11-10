//! Opt-in finite save-to-notification measurements with real compilers and no listeners.
use super::*;
use std::path::PathBuf;
use std::sync::mpsc;

/// Measure an owned copied app; ordinary tests return without filesystem/build side effects.
#[test]
fn finite_save_to_notification() {
    let Some(root) = std::env::var_os("BLOG_DEV_TIMING_ROOT").map(PathBuf::from) else {
        return;
    };
    let native = root.join("target/timing/native");
    let mut builder = Builder::new(&root, Some(&native)).unwrap();
    let initial = Work {
        generate: true,
        browser: true,
        native: true,
        restart: true,
        ..Default::default()
    };
    builder
        .prepare(&initial, execute)
        .unwrap()
        .publish(&root)
        .unwrap();
    csr(&builder, &root);
    let cases = [
        (
            "css",
            "plugins/assets/assets/global.css",
            "",
            "\n/* timing edit */\n",
        ),
        (
            "markup",
            "plugins/app/src/views/home.rs",
            "\"The Blog\"",
            "\"The Blog timing\"",
        ),
        (
            "view_logic",
            "plugins/app/src/views/home.rs",
            "cloned().unwrap_or_default()",
            "cloned().unwrap_or_else(Vec::new)",
        ),
        (
            "handler",
            "plugins/app/src/pages/home.rs",
            "{\"published\":\"desc\"}",
            "{\"title\":\"asc\"}",
        ),
    ];
    let mut results = Vec::new();
    for (name, file, before, after) in cases {
        if let Ok(selected) = std::env::var("BLOG_DEV_TIMING_EDIT")
            && selected != name
        {
            continue;
        }
        let path = root.join(file);
        let original = std::fs::read_to_string(&path).unwrap();
        for iteration in 0..3 {
            // Distinct matching saves avoid measuring a repeated local-crate sccache hit.
            let replacement = match name {
                "css" => format!("\n/* distinct timing edit {iteration} */\n"),
                "markup" => format!("\"The Blog timing variant {iteration}\""),
                "view_logic" => [
                    "cloned().unwrap_or_else(|| Vec::new())",
                    "cloned().unwrap_or_else(|| Vec::default())",
                    "cloned().unwrap_or_else(|| vec![])",
                ][iteration]
                    .into(),
                "handler" => [
                    "{\"title\":\"asc\"}",
                    "{\"title\":\"desc\"}",
                    "{\"id\":\"asc\"}",
                ][iteration]
                    .into(),
                _ => after.into(),
            };
            let edited = if before.is_empty() {
                format!("{original}{replacement}")
            } else {
                assert!(original.contains(before));
                original.replacen(before, &replacement, 1)
            };
            let snapshot = Snapshot::read(&root).unwrap();
            let mut markup = Markup::new(&root, &snapshot);
            let started = Instant::now();
            std::fs::write(&path, &edited).unwrap();
            let next = Snapshot::read(&root).unwrap();
            let mut debounce = Debounce::default();
            debounce.observe(snapshot.changes(&next), started);
            std::thread::sleep(Duration::from_millis(250));
            let paths = debounce
                .ready(Instant::now(), Duration::from_millis(250))
                .unwrap();
            let mut deferred = false;
            let notice = if let Some(patches) = markup.patch(&paths, &next) {
                Notice::View(patches)
            } else {
                let work = markup.rebuild_work(classify(&paths));
                builder
                    .prepare(&work, execute)
                    .unwrap()
                    .publish(&root)
                    .unwrap();
                deferred = work.browser;
                if work.browser {
                    Notice::Hydrate
                } else if work.restart {
                    Notice::All
                } else {
                    Notice::Css(work.css.into_iter().next().unwrap())
                }
            };
            let (sender, receiver) = mpsc::channel();
            sender.send(notice.text()).unwrap();
            let message: serde_json::Value =
                serde_json::from_str(&receiver.recv_timeout(Duration::from_secs(1)).unwrap())
                    .unwrap();
            let seconds = started.elapsed().as_secs_f64();
            let csr_seconds = if deferred {
                csr(&builder, &root);
                sender.send(Notice::Csr.text()).unwrap();
                assert!(
                    receiver
                        .recv_timeout(Duration::from_secs(1))
                        .unwrap()
                        .contains("csr")
                );
                Some(started.elapsed().as_secs_f64())
            } else {
                None
            };
            results.push(serde_json::json!({"edit":name,"iteration":iteration,"seconds":seconds,"csr_seconds":csr_seconds,"message":message}));
            std::fs::write(&path, &original).unwrap();
            // Restore the same compiled baseline outside the measured interval.
            let work = classify(&std::collections::BTreeSet::from([PathBuf::from(file)]));
            builder
                .prepare(&work, execute)
                .unwrap()
                .publish(&root)
                .unwrap();
            if work.browser {
                csr(&builder, &root);
            }
        }
    }
    std::fs::write(
        std::env::var_os("BLOG_DEV_TIMING_OUTPUT").unwrap(),
        serde_json::to_vec_pretty(&results).unwrap(),
    )
    .unwrap();
}

/// The real xtask executable handles Cargo's compiler-wrapper invocations in this test.
fn execute(command: &mut Command, timeout: Duration) -> BuildResult {
    if command
        .get_envs()
        .any(|(key, _)| key == "BLOG_DEV_COMPILER_WRAPPER")
    {
        command.env(
            "RUSTC_WRAPPER",
            std::env::var_os("BLOG_DEV_TIMING_WRAPPER").expect("build blog-xtask first"),
        );
    }
    crate::bounded(command, timeout)
}

/// Join a real deferred build finitely; never start the app or reload listener.
fn csr(builder: &Builder, root: &Path) {
    let mut worker = build::Csr::start_with(builder.clone(), |command, timeout, _| {
        execute(command, timeout)
    });
    worker.finish().unwrap().publish(root).unwrap();
}
