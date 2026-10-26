//! Validate complete or partially purged owned trees before the installer mutates them.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::registry::valid_path;
use crate::{Result, TransformError, read_source};

/// Reserved marker and ownership manifest; it never claims its own digest.
pub(super) const MANIFEST: &str = ".stackpress-generated.json";

/// One installed output's cooperative drift-detection data.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    /// Plugin or aggregate owner recorded when planning.
    owner: String,
    /// SHA-256 of the last installed UTF-8 file bytes.
    sha256: String,
    /// Exact installed byte count; older M0 manifests omitted this optional field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bytes: Option<u64>,
}

/// Versioned owned-root marker; it is not proof of authenticity against malicious editors.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    /// Only this exact version is accepted for managed cleanup.
    version: u32,
    /// Every owned relative path and its last installed content digest.
    #[serde(deserialize_with = "unique_entries")]
    files: BTreeMap<String, Entry>,
}

/// Reject duplicate ownership keys during deserialization rather than silently taking the last.
fn unique_entries<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, Entry>, D::Error> {
    /// Bounded manifest-map visitor; the outer source reader also bounds total marker bytes.
    struct Entries;
    impl<'de> serde::de::Visitor<'de> for Entries {
        type Value = BTreeMap<String, Entry>;

        /// Describe the expected data without disclosing marker contents.
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a unique bounded ownership map")
        }

        /// Consume each ownership claim once, rejecting duplicates before replacement.
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> std::result::Result<Self::Value, M::Error> {
            let mut entries = BTreeMap::new();
            while let Some((path, entry)) = map.next_entry::<String, Entry>()? {
                if entries.len() >= 1024 || entries.insert(path, entry).is_some() {
                    return Err(serde::de::Error::custom(
                        "duplicate ownership or entry budget",
                    ));
                }
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_map(Entries)
}

/// Inspect every existing path component without following output symlinks.
pub(super) fn no_symlinks(path: &Path) -> Result<()> {
    let mut prefix = PathBuf::new();
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(TransformError::classified(
                crate::TransformErrorKind::Ownership,
                "output path cannot contain parent traversal",
            ));
        }
        prefix.push(component);
        match fs::symlink_metadata(&prefix) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(TransformError::classified(
                    crate::TransformErrorKind::Ownership,
                    "generated output cannot follow symlinks",
                ));
            }
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error.into()),
            _ => {}
        }
    }
    Ok(())
}

/// Hash bounded UTF-8 bytes for cooperative drift detection, not authentication.
fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

/// Describe exactly the newly formatted owned files.
pub(super) fn manifest(files: &BTreeMap<String, (String, String)>) -> Manifest {
    Manifest {
        version: 1,
        files: files
            .iter()
            .map(|(path, (owner, text))| {
                (
                    path.clone(),
                    Entry {
                        owner: owner.clone(),
                        sha256: digest(text),
                        bytes: Some(text.len() as u64),
                    },
                )
            })
            .collect(),
    }
}

/// Verify marker, exact file set and every last-installed digest before granting cleanup rights.
pub(super) fn inspect(root: &Path) -> Result<BTreeMap<String, (String, String)>> {
    inspect_mode(root, false)
}

/// Validate existing bytes; cleanup alone may tolerate already removed promised files.
fn inspect_mode(root: &Path, allow_missing: bool) -> Result<BTreeMap<String, (String, String)>> {
    no_symlinks(root)?;
    if !root.try_exists()? {
        return Ok(BTreeMap::new());
    }
    // An empty dedicated directory contains no manual data and can be adopted on first install.
    if fs::read_dir(root)?.next().is_none() {
        return Ok(BTreeMap::new());
    }
    let marker = root.join(MANIFEST);
    no_symlinks(&marker)?;
    let manifest: Manifest =
        serde_json::from_str(&read_regular(&marker, 1024 * 1024).map_err(|error| {
            // Retain native NotFound while identifying a missing ownership marker semantically.
            let missing = std::error::Error::source(&error)
                .and_then(|cause| cause.downcast_ref::<std::io::Error>())
                .is_some_and(|cause| cause.kind() == std::io::ErrorKind::NotFound);
            if missing {
                TransformError::caused(
                    crate::TransformErrorKind::Ownership,
                    "generated manifest is missing",
                    error,
                )
            } else {
                error
            }
        })?)
        .map_err(|cause| {
            TransformError::caused(
                crate::TransformErrorKind::Ownership,
                "root is not a recognized generated directory",
                cause,
            )
        })?;
    if manifest.version != 1 || manifest.files.len() > 1024 {
        return Err(TransformError::classified(
            if manifest.version != 1 {
                crate::TransformErrorKind::Ownership
            } else {
                crate::TransformErrorKind::Limit
            },
            "unsupported generated manifest",
        ));
    }
    let mut files: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut remaining = 1024 * 1024usize;
    for (path, entry) in manifest.files {
        // JSON keys grant no native path authority until they pass the same rules as plans.
        valid_path(&path)?;
        if entry.owner.is_empty()
            || entry.owner.len() > 128
            || entry.sha256.len() != 64
            || !entry.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            || files
                .keys()
                .any(|prior| crate::plan::paths_collide(prior, &path))
        {
            return Err(TransformError::classified(
                crate::TransformErrorKind::Ownership,
                "invalid generated manifest ownership",
            ));
        }
        let absolute = root.join(&path);
        no_symlinks(&absolute)?;
        if allow_missing && !exists(&absolute)? {
            // Keep the promised path for exact directory/unknown-file validation below.
            files.insert(path, (entry.owner, String::new()));
            continue;
        }
        let text = read_regular(&absolute, 1024 * 1024)?;
        // Bound the entire installed tree, rather than allowing 1024 individually large files.
        remaining = remaining.checked_sub(text.len()).ok_or_else(|| {
            TransformError::classified(
                crate::TransformErrorKind::Limit,
                "installed output byte budget exceeded",
            )
        })?;
        // Legacy lengths may be absent, but the installed bytes must always match SHA-256.
        if entry.bytes.is_some_and(|bytes| bytes != text.len() as u64)
            || digest(&text) != entry.sha256
        {
            return Err(TransformError::classified(
                crate::TransformErrorKind::Ownership,
                "manually edited generated file; restore it explicitly",
            ));
        }
        files.insert(path, (entry.owner, text));
    }
    inspect_paths(root, root, &files, 0)?;
    Ok(files)
}

/// Reject symlinks/special files before opening a path that could block a bounded text reader.
pub(super) fn read_regular(path: &Path, max_bytes: usize) -> Result<String> {
    no_symlinks(path)?;
    // Byte limits do not bound open/read latency for FIFOs, devices or sockets.
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Ownership,
            "generated input must be a regular file",
        ));
    }
    read_source(path, max_bytes)
}

/// Inspect existence without hiding dangling symlinks as absent paths.
pub(super) fn exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

/// Require a real ownership marker before removing a retained stage or backup tree.
pub(super) fn inspect_owned(root: &Path) -> Result<BTreeMap<String, (String, String)>> {
    no_symlinks(root)?;
    no_symlinks(&root.join(MANIFEST))?;
    if !exists(&root.join(MANIFEST))? {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Ownership,
            "recovery tree lacks generated ownership marker",
        ));
    }
    inspect(root)
}

/// Cleanup may resume after individual files or the final marker have already been removed.
pub(super) fn inspect_retained(root: &Path, cleaning: bool) -> Result<()> {
    no_symlinks(root)?;
    if cleaning && fs::read_dir(root)?.next().is_none() {
        // An empty directory contains no manual bytes and can be removed after cleanup intent.
        return Ok(());
    }
    no_symlinks(&root.join(MANIFEST))?;
    if !exists(&root.join(MANIFEST))? {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Ownership,
            "recovery tree lacks generated ownership marker",
        ));
    }
    inspect_mode(root, cleaning)?;
    Ok(())
}

/// Reject unknown files, special files, symlinks and excessive directory depth before cleanup.
fn inspect_paths(
    root: &Path,
    path: &Path,
    files: &BTreeMap<String, (String, String)>,
    depth: usize,
) -> Result<()> {
    if depth > 16 {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Limit,
            "generated directory depth exceeded",
        ));
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        // Native names are checked losslessly; an undecodable directory is never purged.
        if kind.is_dir() {
            let relative = path
                .strip_prefix(root)
                .ok()
                .and_then(Path::to_str)
                .ok_or_else(|| {
                    TransformError::classified(
                        crate::TransformErrorKind::Ownership,
                        "invalid generated directory",
                    )
                })?;
            if !files
                .keys()
                .any(|file| file.starts_with(&format!("{relative}/")))
            {
                return Err(TransformError::classified(
                    crate::TransformErrorKind::Ownership,
                    "manual directory inside generated root",
                ));
            }
            inspect_paths(root, &path, files, depth + 1)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .ok()
                .and_then(Path::to_str)
                .ok_or_else(|| {
                    TransformError::classified(
                        crate::TransformErrorKind::Ownership,
                        "invalid generated path",
                    )
                })?;
            if !kind.is_file() || (relative != MANIFEST && !files.contains_key(relative)) {
                return Err(TransformError::classified(
                    crate::TransformErrorKind::Ownership,
                    "manual or special file inside generated root",
                ));
            }
        }
    }
    Ok(())
}
