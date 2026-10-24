//! Adapter contracts against owned memory snapshots and isolated native temporary projects.

use std::error::Error;
use std::fs;

use stackpress_idea_transformer::{
    MemoryLoader, ProjectLoader, SourceId, SourceKind, SourceLoader,
};

#[test]
/// Both adapters require explicit absolute roots; native construction preserves IO failures.
fn roots_are_explicit_and_checked() {
    assert!(MemoryLoader::new("relative").is_err());
    assert!(ProjectLoader::new("relative").is_err());
    let directory = tempfile::tempdir().unwrap();
    let error = ProjectLoader::new(directory.path().join("missing")).unwrap_err();
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::NotFound
    );
    fs::write(directory.path().join("file"), "").unwrap();
    assert!(ProjectLoader::new(directory.path().join("file")).is_err());
}

#[test]
/// Exact, Idea, JSON and directory-index candidates have identical deterministic precedence.
fn native_and_memory_candidate_order() {
    let directory = tempfile::tempdir().unwrap();
    let mut memory = MemoryLoader::new(directory.path()).unwrap();
    let native = ProjectLoader::new(directory.path()).unwrap();
    let fixtures = [
        ("exact", "exact"),
        ("exact.idea", "shadowed"),
        ("extension.idea", "idea"),
        ("extension.json", "shadowed"),
        ("json.json", "{}"),
        ("folder/index.idea", "index idea"),
        ("folder/index.json", "shadowed"),
        ("jsonfolder/index.json", "{}"),
    ];
    // Every native fixture has the same relative path/text as its in-memory counterpart.
    for (path, text) in fixtures {
        let absolute = directory.path().join(path);
        fs::create_dir_all(absolute.parent().unwrap()).unwrap();
        fs::write(&absolute, text).unwrap();
        memory.insert(path, text);
    }
    for (specifier, expected, kind) in [
        ("exact", "exact", SourceKind::Idea),
        ("extension", "idea", SourceKind::Idea),
        ("json", "{}", SourceKind::Json),
        ("folder", "index idea", SourceKind::Idea),
        ("jsonfolder", "{}", SourceKind::Json),
    ] {
        let disk = native.load(None, specifier, 100).unwrap();
        let virtual_file = memory.load(None, specifier, 100).unwrap();
        assert_eq!(disk.text, expected);
        assert_eq!(disk.kind, kind);
        assert_eq!(disk.name, virtual_file.name);
        assert_eq!(disk.text, virtual_file.text);
        assert_eq!(disk.kind, virtual_file.kind);
    }
}

#[test]
/// Importer-relative, parent, root-alias and absolute resolution ignore the process cwd.
fn resolution_is_relative_to_importer() {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir_all(directory.path().join("nested")).unwrap();
    fs::write(directory.path().join("nested/root.idea"), "nested").unwrap();
    fs::write(directory.path().join("base.idea"), "base").unwrap();
    let native = ProjectLoader::new(directory.path()).unwrap();
    let importer = native.load(None, "nested/root", 100).unwrap();
    for specifier in [
        "../base",
        "@/base",
        directory.path().join("base.idea").to_str().unwrap(),
    ] {
        let source = native.load(Some(&importer.id), specifier, 100).unwrap();
        assert_eq!(source.text, "base");
        assert_eq!(source.id.as_path(), native.root().join("base.idea"));
    }
    let mut memory = MemoryLoader::new("/project").unwrap();
    memory.insert("base.idea", "base");
    let importer = SourceId::new("/project/nested/root.idea");
    for specifier in ["../base", "@/base", "/project/base.idea"] {
        assert_eq!(
            memory.load(Some(&importer), specifier, 100).unwrap().text,
            "base"
        );
    }
}

#[test]
/// Limits are inclusive and checked before native read or memory clone; invalid UTF-8 survives.
fn byte_limits_and_encoding_causes() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("tiny.idea"), "1234").unwrap();
    let native = ProjectLoader::new(directory.path()).unwrap();
    let mut memory = MemoryLoader::new(directory.path()).unwrap();
    memory.insert("tiny.idea", "1234");
    for loader in [&native as &dyn SourceLoader, &memory] {
        assert_eq!(loader.load(None, "tiny", 4).unwrap().text, "1234");
        assert!(loader.load(None, "tiny", 3).is_err());
        let missing = loader.load(None, "missing", 4).unwrap_err();
        assert_eq!(
            missing
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::NotFound
        );
    }
    fs::write(directory.path().join("bad.idea"), [0xff]).unwrap();
    let encoding = native.load(None, "bad.idea", 4).unwrap_err();
    assert!(
        encoding
            .source()
            .unwrap()
            .downcast_ref::<std::string::FromUtf8Error>()
            .is_some()
    );
    // Sparse files prove size preflight without allocating or reading their declared size.
    let sparse = fs::File::create(directory.path().join("huge.idea")).unwrap();
    sparse.set_len(1024 * 1024 * 1024).unwrap();
    assert!(native.load(None, "huge.idea", 16).is_err());
}

#[test]
/// Data adapters never load executable modules or silently reinterpret explicit extensions.
fn unsupported_extensions_do_not_fall_through() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("plugin.js"), "secret script").unwrap();
    let native = ProjectLoader::new(directory.path()).unwrap();
    let mut memory = MemoryLoader::new(directory.path()).unwrap();
    memory.insert("plugin.js", "secret script");
    for loader in [&native as &dyn SourceLoader, &memory] {
        let error = loader.load(None, "plugin.js", 100).unwrap_err();
        assert_eq!(error.to_string(), "unsupported schema source extension");
        assert!(!error.to_string().contains("secret script"));
    }
}

#[test]
/// Virtual aliases canonicalize identity, clones own bytes and explicit insert replaces aliases.
fn memory_aliases_and_clones_are_independent() {
    let mut memory = MemoryLoader::new("/project/./nested/..").unwrap();
    memory.insert("base.idea", "original");
    memory.alias("alias.idea", "base.idea").unwrap();
    assert!(memory.alias("bad.idea", "missing.idea").is_err());
    assert!(memory.alias("base.idea", "base.idea").is_err());
    let original = memory.load(None, "alias", 100).unwrap();
    assert_eq!(original.id, SourceId::new("/project/base.idea"));
    let mut cloned = memory.clone();
    cloned.insert("base.idea", "changed");
    assert_eq!(memory.load(None, "base", 100).unwrap().text, "original");
    cloned.insert("alias.idea", "own alias bytes");
    assert_eq!(
        cloned.load(None, "alias", 100).unwrap().text,
        "own alias bytes"
    );
}

#[cfg(unix)]
#[test]
/// Native aliases preserve identity, permission failures remain causes and devices are rejected.
fn native_symlinks_permissions_and_special_files() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("base.idea"), "original").unwrap();
    symlink(
        directory.path().join("base.idea"),
        directory.path().join("alias.idea"),
    )
    .unwrap();
    let native = ProjectLoader::new(directory.path()).unwrap();
    assert_eq!(
        native.load(None, "base", 100).unwrap().id,
        native.load(None, "alias", 100).unwrap().id
    );
    symlink("/dev/null", directory.path().join("device.idea")).unwrap();
    assert!(native.load(None, "device.idea", 100).is_err());
    let restricted = directory.path().join("restricted.idea");
    fs::write(&restricted, "restricted").unwrap();
    fs::set_permissions(&restricted, fs::Permissions::from_mode(0o000)).unwrap();
    let result = native.load(None, "restricted.idea", 100);
    fs::set_permissions(&restricted, fs::Permissions::from_mode(0o600)).unwrap();
    match result {
        Err(error) => assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        ),
        Ok(_) => eprintln!("permission proof unavailable: host bypassed mode 000"),
    }
}

#[test]
/// Directory-only input and a non-directory parent retain distinct native IO categories.
fn native_directory_failures_are_not_missing_files() {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir(directory.path().join("empty")).unwrap();
    fs::write(directory.path().join("regular.idea"), "").unwrap();
    let loader = ProjectLoader::new(directory.path()).unwrap();
    for (specifier, expected) in [
        ("empty", std::io::ErrorKind::IsADirectory),
        ("regular.idea/child", std::io::ErrorKind::NotADirectory),
    ] {
        let error = loader.load(None, specifier, 100).unwrap_err();
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            expected
        );
    }
}
