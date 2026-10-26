//! Startup-only integrity checks and immutable allowlisted assets, before any socket bind.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stackpress_example_profile_ui_contracts::{BUILD, REVISION};
use stackpress_framework_view::BasePath;
use stackpress_lib::{Body, Flow, handler};
use stackpress_server::ServerBuilder;
use std::{
    collections::BTreeMap,
    io::{self, Read},
    path::{Component, Path},
    sync::Arc,
};

/// Largest single static file accepted by this buffered example.
pub const MAX_ASSET_BYTES: usize = 64 * 1024 * 1024;

/// Checked output identity from the pinned Python client build.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    /// Manifest grammar revision, independent of application wire revision.
    format: u32,
    /// Native and browser DTO wire revision must agree.
    revision: u32,
    /// Compiled application asset build identity.
    build: String,
    /// The web gateway admits hydrate outputs only.
    mode: String,
    /// Exact approved CLI/binding version.
    wasm_bindgen: String,
    /// Fingerprint of the shared source files compiled into both targets.
    contract_sha256: String,
    /// Aggregate digest used as the public immutable resource directory.
    content_id: String,
    /// Exact allowlist; caller paths never become filesystem lookups.
    files: BTreeMap<String, FileRecord>,
}

/// Each path has a byte count, SHA-256 identity and explicit allowed MIME.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileRecord {
    /// Logical byte ceiling checked before and during startup reads.
    bytes: usize,
    /// Expected content digest, checked before retaining any asset.
    sha256: String,
    /// Explicit type checked against the generated filename kind.
    mime: String,
}

/// Owned immutable buffers; runtime requests never reopen paths supplied by callers.
pub struct AssetBundle {
    /// Validated build and exact filename inventory.
    manifest: Manifest,
    /// Verified immutable bytes shared by route closures until host disposal.
    bytes: BTreeMap<String, Arc<Vec<u8>>>,
}

impl AssetBundle {
    /// Load the hydration inventory and fail closed on any source/version/hash drift.
    ///
    /// # Errors
    /// Names the offending asset path for missing, symlinked, oversized or mismatched files.
    /// IO/JSON failures retain concrete sources under distinct read/decode context.
    /// Failure publishes no bundle or routes and never changes the files.
    pub fn load(folder: &Path) -> io::Result<Self> {
        if std::fs::symlink_metadata(folder)
            .map_err(|error| at(folder, error))?
            .file_type()
            .is_symlink()
        {
            return Err(invalid(folder, "asset root symlink refused"));
        }
        let path = folder.join("manifest.json");
        if std::fs::symlink_metadata(&path)
            .map_err(|error| at(&path, error))?
            .len()
            > 64 * 1024
        {
            return Err(invalid(&path, "manifest too large"));
        }
        let manifest: Manifest =
            serde_json::from_slice(&read_bounded(&path, 64 * 1024)?).map_err(|cause| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    AssetReadError {
                        path: path.clone(),
                        operation: "asset manifest could not be decoded",
                        cause: Box::new(cause),
                    },
                )
            })?;
        if manifest.format != 1
            || manifest.revision != REVISION
            || manifest.build != BUILD
            || manifest.mode != "hydrate"
            || manifest.wasm_bindgen != "0.2.129"
            || manifest.contract_sha256 != contract_hash()
        {
            return Err(invalid(
                &path,
                "asset/seed contract mismatch; rebuild hydrate",
            ));
        }
        if manifest.files.len() > 128
            || manifest
                .files
                .values()
                .try_fold(0_usize, |sum, entry| sum.checked_add(entry.bytes))
                .unwrap_or(usize::MAX)
                > 128 * 1024 * 1024
        {
            return Err(invalid(&path, "asset inventory exceeds finite limit"));
        }
        let mut bytes = BTreeMap::new();
        for (name, entry) in &manifest.files {
            let file = folder.join(name);
            if !safe_name(name) || entry.bytes > MAX_ASSET_BYTES || !allowed_mime(name, &entry.mime)
            {
                return Err(invalid(&file, "invalid asset metadata"));
            }
            // Check every component, refusing symlinks before retaining its bytes.
            let mut checked = folder.to_path_buf();
            for component in Path::new(name).components() {
                checked.push(component);
                if std::fs::symlink_metadata(&checked)
                    .map_err(|error| at(&checked, error))?
                    .file_type()
                    .is_symlink()
                {
                    return Err(invalid(&file, "asset symlink refused"));
                }
            }
            let metadata = std::fs::metadata(&file).map_err(|error| at(&file, error))?;
            if !metadata.is_file() || metadata.len() != entry.bytes as u64 {
                return Err(invalid(&file, "asset size mismatch"));
            }
            let data = read_bounded(&file, entry.bytes)?;
            if data.len() != entry.bytes || hex(&Sha256::digest(&data)) != entry.sha256 {
                return Err(invalid(&file, "asset hash mismatch"));
            }
            bytes.insert(name.clone(), Arc::new(data));
        }
        for required in ["start.js", "profile_ui.js", "profile_ui_bg.wasm"] {
            if !bytes.contains_key(required) {
                return Err(invalid(&folder.join(required), "required asset missing"));
            }
        }
        if identity(&manifest) != manifest.content_id {
            return Err(invalid(&path, "asset content identity mismatch"));
        }
        Ok(Self { manifest, bytes })
    }

    /// Application-relative content-addressed bootstrap module, with unchanged relative imports.
    pub fn module(&self) -> String {
        format!("/assets/{}/start.js", self.manifest.content_id)
    }

    /// Register exact static paths through the existing HttpAdapter and response authority.
    ///
    /// # Errors
    /// Returns native route registration or mount validation errors.
    pub fn register(&self, app: &mut ServerBuilder, base: &BasePath) -> stackpress_lib::Result<()> {
        for (name, data) in &self.bytes {
            let path = base
                .join(&format!("/assets/{}/{name}", self.manifest.content_id))
                .map_err(stackpress_lib::Error::from_source)?;
            let data = Arc::clone(data);
            let mime = self.manifest.files[name].mime.clone();
            app.get(
                path.as_str(),
                handler(move |_, res, _| {
                    let data = Arc::clone(&data);
                    let mime = mime.clone();
                    Box::pin(async move {
                        res.set(&mime, Body::Bytes(data.as_ref().clone()))?;
                        res.headers_mut()?
                            .set("cache-control", "public, max-age=31536000, immutable")?;
                        res.headers_mut()?
                            .set("x-content-type-options", "nosniff")?;
                        Ok(Flow::Continue)
                    })
                }),
            )?;
        }
        Ok(())
    }
}

/// Bound relative asset names to unambiguous compiled output paths.
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 1024
        && Path::new(name)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        && name.split('/').all(|part| {
            !matches!(part, "" | "." | "..")
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
        })
}
/// Keep MIME selected from the file kind rather than caller headers.
fn allowed_mime(name: &str, mime: &str) -> bool {
    (name.ends_with(".js") && mime == "text/javascript; charset=utf-8")
        || (name.ends_with(".wasm") && mime == "application/wasm")
}
/// Preserve a clear startup path without using it as an HTTP error body.
fn invalid(path: &Path, reason: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{reason}: {}", path.display()),
    )
}
/// Format a standard SHA-256 digest as lowercase hexadecimal.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
/// Fingerprint the same explicit seed-contract source inventory as the build tool.
fn contract_hash() -> String {
    let mut hash = Sha256::new();
    for (name, bytes) in [
        (
            "examples/profile-ui/contracts/src/lib.rs",
            include_bytes!("../../contracts/src/lib.rs").as_slice(),
        ),
        (
            "examples/profile-ui/contracts/src/reply.rs",
            include_bytes!("../../contracts/src/reply.rs").as_slice(),
        ),
        (
            "examples/profile-ui/route-contract.rs",
            include_bytes!("../../route-contract.rs").as_slice(),
        ),
        (
            "examples/generated-profile/app/src/generated/contracts/profile.rs",
            include_bytes!("../../../generated-profile/app/src/generated/contracts/profile.rs")
                .as_slice(),
        ),
        (
            "crates/framework/framework-view/src/lib.rs",
            include_bytes!("../../../../crates/framework/framework-view/src/lib.rs").as_slice(),
        ),
        (
            "crates/framework/framework-view/src/policy.rs",
            include_bytes!("../../../../crates/framework/framework-view/src/policy.rs").as_slice(),
        ),
        (
            "crates/framework/framework-view/src/routes.rs",
            include_bytes!("../../../../crates/framework/framework-view/src/routes.rs").as_slice(),
        ),
    ] {
        hash.update(name);
        hash.update([0]);
        hash.update(bytes);
        hash.update([0]);
    }
    hex(&hash.finalize())
}
/// Hash explicit delimited fields, independent of JSON map/serializer ordering.
fn identity(manifest: &Manifest) -> String {
    let mut hash = Sha256::new();
    let mut field = |text: &str| {
        hash.update(text);
        hash.update([0]);
    };
    for text in [
        "profile-ui-assets-v1",
        &manifest.revision.to_string(),
        &manifest.build,
        &manifest.mode,
        &manifest.wasm_bindgen,
        &manifest.contract_sha256,
    ] {
        field(text);
    }
    for (name, entry) in &manifest.files {
        for text in [name, &entry.mime, &entry.bytes.to_string(), &entry.sha256] {
            field(text);
        }
    }
    hex(&hash.finalize())
}

/// Bound actual read allocation even when a local file changes after its metadata check.
fn read_bounded(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| at(path, error))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(invalid(path, "asset is not a regular file"));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|error| at(path, error))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| at(path, error))?;
    if bytes.len() > limit {
        return Err(invalid(path, "asset exceeds read limit"));
    }
    Ok(bytes)
}
#[cfg(test)]
#[path = "assets/tests.rs"]
pub(crate) mod tests;

/// Preserve the original native I/O cause while naming the exact startup asset path.
fn at(path: &Path, cause: io::Error) -> io::Error {
    io::Error::new(
        cause.kind(),
        AssetReadError {
            path: path.to_path_buf(),
            operation: "asset could not be read",
            cause: Box::new(cause),
        },
    )
}

/// Native diagnostic context only; never sent to an HTTP client.
#[derive(Debug)]
struct AssetReadError {
    /// Exact trusted startup path being checked/read.
    path: std::path::PathBuf,
    /// Original IO/JSON failure for explicit source-chain inspection.
    cause: Box<dyn std::error::Error + Send + Sync>,
    /// Distinct startup operation; the cause owns its underlying explanation.
    operation: &'static str,
}
impl std::fmt::Display for AssetReadError {
    /// Name the startup operation and path; the native explanation stays in source().
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.operation, self.path.display())
    }
}
impl std::error::Error for AssetReadError {
    /// Native diagnostics keep the actual I/O cause instead of a string surrogate.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.cause.as_ref())
    }
}
