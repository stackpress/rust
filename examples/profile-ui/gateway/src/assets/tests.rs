//! Startup integrity, source contract and immutable exact-path registration tests.
use super::*;

/// Tiny actual files exercise integrity code; real WASM assets run in browser proofs.
pub(crate) fn fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    let files = [
        (
            "start.js",
            b"export {};".as_slice(),
            "text/javascript; charset=utf-8",
        ),
        (
            "profile_ui.js",
            b"export {};".as_slice(),
            "text/javascript; charset=utf-8",
        ),
        (
            "profile_ui_bg.wasm",
            b"\0asm".as_slice(),
            "application/wasm",
        ),
    ];
    let mut records = BTreeMap::new();
    for (name, data, mime) in files {
        std::fs::write(directory.path().join(name), data).unwrap();
        records.insert(
            name.to_owned(),
            FileRecord {
                bytes: data.len(),
                sha256: hex(&Sha256::digest(data)),
                mime: mime.into(),
            },
        );
    }
    let mut manifest = Manifest {
        format: 1,
        revision: REVISION,
        build: BUILD.into(),
        mode: "hydrate".into(),
        wasm_bindgen: "0.2.129".into(),
        contract_sha256: contract_hash(),
        content_id: String::new(),
        files: records,
    };
    manifest.content_id = identity(&manifest);
    std::fs::write(
        directory.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    directory
}

/// Missing/tampered files and stale seed/build metadata fail before host construction.
#[test]
fn fail_closed_with_asset_path() {
    for mode in [
        "hash",
        "missing",
        "size",
        "build",
        "source",
        "identity",
        "path",
        "mime",
        "count",
        "overflow",
        "manifest-size",
        "manifest-link",
        "asset-link",
        "directory",
    ] {
        let directory = fixture();
        let root = directory.path();
        let path = root.join("manifest.json");
        let mut json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        match mode {
            "hash" => std::fs::write(root.join("start.js"), b"mismatch!!").unwrap(),
            "missing" => std::fs::remove_file(root.join("start.js")).unwrap(),
            "size" => {
                json["files"]["start.js"]["bytes"] = 0.into();
            }
            "build" => {
                json["build"] = "old".into();
            }
            "source" => {
                json["contract_sha256"] = "old".into();
            }
            "identity" => {
                json["content_id"] = "old".into();
            }
            "path" => {
                json["files"]["../escaped.js"] = json["files"]["start.js"].clone();
            }
            "mime" => {
                json["files"]["start.js"]["mime"] = "text/html".into();
            }
            "count" => {
                for i in 0..129 {
                    json["files"][format!("x{i}.js")] = json["files"]["start.js"].clone();
                }
            }
            "overflow" => {
                json["files"]["start.js"]["bytes"] = u64::MAX.into();
            }
            "manifest-size" => {
                std::fs::write(&path, vec![b' '; 65537]).unwrap();
            }
            "manifest-link" => {
                std::fs::rename(&path, root.join("real.json")).unwrap();
                std::os::unix::fs::symlink(root.join("real.json"), &path).unwrap();
            }
            "asset-link" => {
                std::fs::remove_file(root.join("start.js")).unwrap();
                std::os::unix::fs::symlink(root.join("profile_ui.js"), root.join("start.js"))
                    .unwrap();
            }
            "directory" => {
                std::fs::remove_file(root.join("start.js")).unwrap();
                std::fs::create_dir(root.join("start.js")).unwrap();
            }
            _ => unreachable!(),
        }
        if !matches!(mode, "manifest-size" | "manifest-link") {
            std::fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
        }
        let error = AssetBundle::load(root).err().unwrap().to_string();
        assert!(error.contains(root.to_str().unwrap()), "{mode}: {error}");
        if mode == "hash" {
            assert!(
                error.contains("hash mismatch") && error.contains("start.js"),
                "{error}"
            );
        }
    }
    let directory = fixture();
    let bundle = AssetBundle::load(directory.path()).unwrap();
    assert!(bundle.module().starts_with("/assets/"));
    // Owned buffers remain stable if the source directory is later removed.
    drop(directory);
    assert_eq!(bundle.bytes["start.js"].as_slice(), b"export {};");
    for bad in ["../a.js", "/a.js", "a//b.js", "a\\b.js", "a%2fb.js", ""] {
        assert!(!safe_name(bad));
    }
    assert!(read_bounded(Path::new("/does-not-exist"), 10).is_err());
}
