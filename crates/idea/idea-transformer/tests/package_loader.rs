//! Package source resolution reuses existing composition and preserves app-loader policy.

use stackpress_idea_transformer::{
    Composer, CompositionLimits, LoadError, LoadedSource, MemoryLoader, PackageLoader,
    ProjectLoader, SourceId, SourceKind, SourceLoader, TransformErrorKind,
};
use std::{error::Error, path::Path, process::Command, time::Duration};

#[path = "support/watchdog.rs"]
mod watchdog;

/// One catalog is reusable across nested applications without process-global registration.
fn catalog<L: SourceLoader>(fallback: L) -> PackageLoader<L> {
    PackageLoader::new(fallback)
        .with_source(
            "pkg/schema.idea",
            "use \"./child\"\nmodel Parent { id String }",
        )
        .unwrap()
        .with_source("pkg/child.idea", "model Child { id String }")
        .unwrap()
}

#[test]
fn registered_bare_names_win_and_explicit_app_paths_keep_their_meaning() {
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("nested");
    std::fs::create_dir_all(nested.join("pkg")).unwrap();
    std::fs::create_dir_all(root.path().join("pkg")).unwrap();
    std::fs::write(nested.join("schema.idea"), "use \"pkg/schema.idea\"").unwrap();
    std::fs::write(nested.join("pkg/schema.idea"), "local nested").unwrap();
    std::fs::write(root.path().join("pkg/schema.idea"), "local root").unwrap();
    let loader = catalog(ProjectLoader::new(root.path()).unwrap());
    let parent = loader.load(None, "nested/schema.idea", 1000).unwrap();
    let package = loader
        .load(Some(&parent.id), "pkg/schema.idea", 1000)
        .unwrap();
    assert_eq!(package.name, "pkg/schema.idea");
    for (name, expected) in [
        ("./pkg/schema.idea", "local nested"),
        ("pkg/./schema.idea", "local nested"),
        ("@/pkg/schema.idea", "local root"),
        (
            root.path().join("pkg/schema.idea").to_str().unwrap(),
            "local root",
        ),
    ] {
        assert_eq!(
            loader.load(Some(&parent.id), name, 1000).unwrap().text,
            expected
        );
    }
    let composed = Composer::new(loader, CompositionLimits::default())
        .compose("nested/schema.idea")
        .unwrap();
    assert_eq!(
        composed
            .config
            .model
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["Parent", "Child"]
    );
    assert_eq!(composed.imports.len(), 2);
    assert_eq!(composed.imports[0].specifier, "pkg/schema.idea");
    assert_eq!(composed.sources[1].id, package.id);
    // An independent undecorated loader still sees its own ordinary local file.
    assert_eq!(
        ProjectLoader::new(root.path())
            .unwrap()
            .load(Some(&parent.id), "pkg/schema.idea", 100)
            .unwrap()
            .text,
        "local nested"
    );
}

#[test]
fn embedded_relative_candidates_stay_in_the_catalog_and_root_imports_return_to_app() {
    let mut app = MemoryLoader::new("/project").unwrap();
    app.insert("local.idea", "app");
    app.insert(
        "/__stackpress_package_sources__/pkg/missing.idea",
        "must not be probed",
    );
    let loader = catalog(app)
        .with_source("pkg/folder/index.json", "{}")
        .unwrap();
    let source = loader.load(None, "pkg/schema.idea", 1000).unwrap();
    assert_eq!(
        loader.load(Some(&source.id), "./child", 1000).unwrap().name,
        "pkg/child.idea"
    );
    assert_eq!(
        loader.load(Some(&source.id), "folder", 1000).unwrap().kind,
        SourceKind::Json
    );
    for specifier in ["./missing", "../../outside"] {
        let error = loader.load(Some(&source.id), specifier, 1000).unwrap_err();
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::NotFound
        );
    }
    for specifier in ["@/local", "/project/local.idea"] {
        assert_eq!(
            loader.load(Some(&source.id), specifier, 1000).unwrap().text,
            "app"
        );
    }
}

#[test]
fn registration_limits_and_fallback_errors_are_explicit() {
    for name in [
        "",
        "./a.idea",
        "../a.idea",
        "@/a.idea",
        "/a.idea",
        "pkg/../a.idea",
        "pkg/./a.idea",
        "pkg//a.idea",
        "pkg\\a.idea",
    ] {
        assert!(
            PackageLoader::new(MemoryLoader::new("/app").unwrap())
                .with_source(name, "")
                .is_err(),
            "{name}"
        );
    }
    assert!(
        PackageLoader::new(MemoryLoader::new("/app").unwrap())
            .with_source("pkg/file.ts", "")
            .is_err()
    );
    let loader = PackageLoader::new(MemoryLoader::new("/app").unwrap())
        .with_source("pkg/file.idea", "1234")
        .unwrap();
    assert_eq!(loader.load(None, "pkg/file.idea", 4).unwrap().text, "1234");
    assert!(loader.load(None, "pkg/file.idea", 3).is_err());
    assert!(loader.with_source("pkg/file.idea", "other").is_err());
    let error = PackageLoader::new(MemoryLoader::new("/app").unwrap())
        .load(None, "missing", 100)
        .unwrap_err();
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::NotFound
    );
    let loader = catalog(MemoryLoader::new("/app").unwrap());
    let error = Composer::new(
        loader,
        CompositionLimits {
            max_source_bytes: 3,
            ..Default::default()
        },
    )
    .compose("pkg/schema.idea")
    .unwrap_err();
    assert_eq!(error.kind(), TransformErrorKind::Limit);
}

/// A custom fallback must not impersonate a package identity, even if its text matches.
struct Collision;
impl SourceLoader for Collision {
    fn load(&self, _: Option<&SourceId>, _: &str, _: usize) -> Result<LoadedSource, LoadError> {
        Ok(LoadedSource {
            id: SourceId::new("/__stackpress_package_sources__/pkg/schema.idea"),
            name: "collision".into(),
            kind: SourceKind::Idea,
            text: String::new(),
        })
    }
}

#[test]
fn namespace_collision_is_not_accepted_as_an_alias() {
    assert!(
        catalog(Collision)
            .load(None, "local", 100)
            .unwrap_err()
            .to_string()
            .contains("collides")
    );
}

#[test]
fn merge_references_diamonds_and_independent_catalogs_keep_composer_contracts() {
    let mut app = MemoryLoader::new("/app").unwrap();
    app.insert("root.idea", "use \"pkg/left.idea\"\nuse \"pkg/right.idea\"");
    let loader = PackageLoader::new(app)
        .with_source(
            "pkg/left.idea",
            "use \"./shared\"\nmodel Item { id String }",
        )
        .unwrap()
        .with_source("pkg/right.idea", "use \"./shared.idea\"")
        .unwrap()
        .with_source(
            "pkg/shared.idea",
            "prop Label { name \"Child\" }\nmodel Item @value(Label) { child String }",
        )
        .unwrap();
    let composed = Composer::new(loader.clone(), CompositionLimits::default())
        .compose("root.idea")
        .unwrap();
    assert_eq!(composed.sources.len(), 4);
    assert_eq!(composed.imports.len(), 4);
    assert_eq!(composed.imports[1].imported, composed.imports[3].imported);
    assert_eq!(composed.config.model["Item"].columns[0].name, "child");
    assert!(!composed.references.is_empty());
    assert_eq!(
        Composer::new(loader, CompositionLimits::default())
            .compose("root.idea")
            .unwrap()
            .config,
        composed.config
    );
    let empty = PackageLoader::new(MemoryLoader::new("/app").unwrap());
    assert!(empty.load(None, "pkg/shared.idea", 1000).is_err());
}

#[test]
fn package_cycles_and_depth_limits_have_external_watchdog() {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", "package_cycle_child", "--ignored", "--nocapture"]);
    let result = watchdog::run(&mut command, "package cycles", Duration::from_secs(10)).unwrap();
    assert!(
        !result.timed_out && result.status.success(),
        "{}",
        result.output
    );
}

#[test]
fn watchdog_forced_kill_negative_control() {
    let mut command = Command::new("python3");
    command.args([
        "-c",
        "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(60)",
    ]);
    let result = watchdog::run(&mut command, "forced kill", Duration::from_millis(300)).unwrap();
    assert!(result.timed_out && !result.status.success());
}

#[test]
#[ignore = "executed only by the external parent watchdog"]
fn package_cycle_child() {
    let loader = PackageLoader::new(MemoryLoader::new("/app").unwrap())
        .with_source("pkg/a.idea", "use \"./b.idea\"")
        .unwrap()
        .with_source("pkg/b.idea", "use \"./a.idea\"")
        .unwrap();
    assert!(
        Composer::new(loader, CompositionLimits::default())
            .compose("pkg/a.idea")
            .is_err()
    );
    let loader = catalog(MemoryLoader::new("/app").unwrap());
    for limits in [
        CompositionLimits {
            max_depth: 1,
            ..Default::default()
        },
        CompositionLimits {
            max_total_bytes: 50,
            ..Default::default()
        },
        CompositionLimits {
            max_files: 1,
            ..Default::default()
        },
        CompositionLimits {
            max_imports: 0,
            ..Default::default()
        },
        CompositionLimits {
            max_expanded_bytes: 1,
            ..Default::default()
        },
    ] {
        assert!(
            Composer::new(&loader, limits)
                .compose("pkg/schema.idea")
                .is_err()
        );
    }
    assert!(Path::new("/__stackpress_package_sources__").is_absolute());
}
