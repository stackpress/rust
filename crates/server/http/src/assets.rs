//! Immutable, startup-verified asset allowlists registered as ordinary exact application routes.

use crate::{external, startup::read_regular};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result, event::handler};
use stackpress_server::{
    ServerBuilder,
    http::stream::{HttpResponseExt, StreamLimits},
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Finite startup manifest/entry/representation budgets, independent of request paths.
#[derive(Clone, Debug)]
pub struct AssetLimits {
    /// Serialized JSON bytes, checked before parsing.
    pub manifest_bytes: usize,
    /// Maximum declared entries before opening any asset.
    pub entries: usize,
    /// Largest single preloaded representation.
    pub file_bytes: usize,
    /// Aggregate retained representation bytes.
    pub total_bytes: usize,
}
impl Default for AssetLimits {
    /// Choose the accepted 64KiB/1024/64MiB/128MiB bundle defaults.
    fn default() -> Self {
        Self {
            manifest_bytes: 64 * 1024,
            entries: 1024,
            file_bytes: 64 * 1024 * 1024,
            total_bytes: 128 * 1024 * 1024,
        }
    }
}
/// Explicit cache policy; immutable requires the full content hash in the URL basename.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetCache {
    /// Public bytes must be revalidated on each reuse.
    Revalidate,
    /// A hash-addressed route can be cached for a year.
    Immutable,
}
/// Strict JSON entry; unknown fields are errors rather than ignored deployment typos.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetEntry {
    /// Exact ASCII pathname; no patterns, escapes, query, traversal or fallback.
    pub route: String,
    /// Relative ASCII file path within the operator-owned bundle tree.
    pub file: String,
    /// Exact byte count checked against the opened representation.
    pub size: usize,
    /// Lowercase SHA-256 hex digest of those exact bytes.
    pub sha256: String,
    /// Explicit allowlisted MIME, never inferred from a request path.
    pub mime: String,
    /// Deliberate shared-cache semantics.
    pub cache: AssetCache,
}
/// Strict manifest root, bounded before deserialization.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    /// Count is checked before any asset open; the serialized allocation is independently capped.
    entries: Vec<AssetEntry>,
}
/// One checked representation; request handling never reopens its source path.
struct Asset {
    /// Validated route/MIME/cache identity, never changed after preload.
    entry: AssetEntry,
    /// Immutable representation shared with independent one-shot response descriptors.
    bytes: Arc<[u8]>,
    /// Strong validator derived from the verified representation hash.
    etag: String,
}
/// Immutable verified storage. Load and register before building the application and binding IO.
#[derive(Clone)]
pub struct AssetBundle(Arc<Vec<Arc<Asset>>>);
impl AssetBundle {
    /// Verify a bounded JSON manifest and preload every regular file exactly once.
    ///
    /// Paths must remain operator-owned and immutable during loading. Symlink components,
    /// special files, unknown JSON fields, duplicate routes and unsupported MIME/cache are rejected.
    /// No asset route is registered when loading fails. Startup reads are synchronous and bounded.
    /// # Errors
    /// Returns contextual original IO/JSON causes or a sanitized policy diagnostic.
    ///
    /// ```no_run
    /// use stackpress_server::ServerBuilder;
    /// use stackpress_server_http::{AssetBundle, AssetLimits};
    /// let bundle = AssetBundle::load("dist", "dist/manifest.json", AssetLimits::default())?;
    /// let mut app = ServerBuilder::new();
    /// bundle.register(&mut app)?;
    /// // Build app services/routes before preparing or binding the deployment host.
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn load(
        root: impl AsRef<Path>,
        manifest: impl AsRef<Path>,
        limits: AssetLimits,
    ) -> Result<Self> {
        if [
            limits.manifest_bytes,
            limits.entries,
            limits.file_bytes,
            limits.total_bytes,
        ]
        .contains(&0)
            || limits.manifest_bytes == usize::MAX
            || limits.file_bytes == usize::MAX
        {
            return Err(Error::invalid("asset limits must be positive and finite"));
        }
        // Serialized bytes are bounded before serde allocates/deserializes any entry structure.
        let json = read_regular(manifest.as_ref(), limits.manifest_bytes)?;
        let manifest: Manifest = serde_json::from_slice(&json)
            .map_err(|cause| external("asset manifest decoding failed", cause))?;
        if manifest.entries.len() > limits.entries {
            return Err(Error::limit("asset entry limit exceeded"));
        }
        let mut routes = BTreeSet::new();
        let mut total = 0usize;
        // Validate all descriptors/budgets before allocating representation storage or opening assets.
        for entry in &manifest.entries {
            validate_entry(entry)?;
            if !routes.insert(entry.route.clone()) {
                return Err(Error::invalid("duplicate asset route"));
            }
            total = total
                .checked_add(entry.size)
                .ok_or_else(|| Error::limit("asset total overflow"))?;
            if entry.size > limits.file_bytes || total > limits.total_bytes {
                return Err(Error::limit("asset byte limit exceeded"));
            }
        }
        let mut assets = Vec::with_capacity(manifest.entries.len());
        for entry in manifest.entries {
            // The disk path comes only from the validated manifest, never a request target.
            let bytes = read_regular(
                &root.as_ref().join(PathBuf::from(&entry.file)),
                entry.size.max(1),
            )?;
            if bytes.len() != entry.size || hex(&Sha256::digest(&bytes)) != entry.sha256 {
                return Err(Error::invalid("asset size/hash mismatch"));
            }
            let etag = format!("\"{}\"", entry.sha256);
            // Retain only verified bytes; later filesystem changes cannot change this identity.
            assets.push(Arc::new(Asset {
                entry,
                bytes: bytes.into(),
                etag,
            }));
        }
        Ok(Self(Arc::new(assets)))
    }
    /// Register GET and HEAD as ordinary exact routes, retaining the existing lifecycle/hooks.
    ///
    /// Registration can fail under the builder's normal route budgets; retain/discard that builder
    /// as with other plugin registration errors. No request-derived filesystem path is ever opened.
    /// # Errors
    /// Returns the original checked route-registration failure.
    pub fn register(&self, builder: &mut ServerBuilder) -> Result<()> {
        for asset in self.0.iter() {
            // HEAD is explicitly registered; it does not dispatch a substituted GET operation.
            for method in [
                stackpress_lib::MethodPattern::GET,
                stackpress_lib::MethodPattern::HEAD,
            ] {
                let owned = Arc::clone(asset);
                builder.route(
                    method,
                    &asset.entry.route,
                    handler(
                        move |req: &mut Request, res: &mut Response, _: Invocation<'_>| {
                            let asset = Arc::clone(&owned);
                            Box::pin(async move {
                                asset.respond(req, res)?;
                                Ok(Flow::Continue)
                            })
                        },
                    ),
                )?;
            }
        }
        Ok(())
    }
    /// Number of exact asset identities; GET and HEAD share each representation.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// An empty manifest deliberately registers no routes.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl Asset {
    /// Add checked caching headers before the ordinary response stage/publication authority.
    fn respond(&self, req: &Request, res: &mut Response) -> Result<()> {
        // Weak comparison is valid for GET/HEAD validators; no request field selects a disk path.
        let cached = req
            .headers
            .get_all("if-none-match")
            .unwrap_or_default()
            .iter()
            .any(|line| {
                line.split(',').any(|value| {
                    let value = value.trim();
                    value == "*" || value.strip_prefix("W/").unwrap_or(value) == self.etag
                })
            });
        res.immutable_bytes(
            &self.entry.mime,
            Arc::clone(&self.bytes),
            StreamLimits {
                max_bytes: self.bytes.len().max(1),
                ..Default::default()
            },
        )?;
        // The normal response stage can still edit/suppress this output or set independent cookies.
        if cached {
            res.status_code(304, None)?;
        }
        let headers = res.headers_mut()?;
        headers.set("etag", &self.etag)?;
        headers.set("x-content-type-options", "nosniff")?;
        headers.set(
            "cache-control",
            match self.entry.cache {
                AssetCache::Revalidate => "public, max-age=0, must-revalidate",
                AssetCache::Immutable => "public, max-age=31536000, immutable",
            },
        )?;
        Ok(())
    }
}
/// Restrict manifest routes and disk names to literal safe ASCII components.
fn safe_path(path: &str, absolute: bool) -> bool {
    let path = if absolute {
        let Some(path) = path.strip_prefix('/') else {
            return false;
        };
        path
    } else {
        path
    };
    !path.is_empty()
        && path.split('/').all(|part| {
            !part.is_empty()
                && !matches!(part, "." | "..")
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        })
}
/// Reject unsafe descriptors before any file opens, including route-pattern metacharacters.
fn validate_entry(entry: &AssetEntry) -> Result<()> {
    const MIMES: &[&str] = &[
        "text/plain; charset=utf-8",
        "text/html; charset=utf-8",
        "text/css; charset=utf-8",
        "text/javascript; charset=utf-8",
        "application/javascript",
        "application/json",
        "application/wasm",
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/webp",
        "image/svg+xml",
        "image/x-icon",
        "font/woff2",
    ];
    if !safe_path(&entry.route, true)
        || !safe_path(&entry.file, false)
        || entry.sha256.len() != 64
        || !entry
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !MIMES.contains(&entry.mime.as_str())
    {
        return Err(Error::invalid("asset descriptor invalid"));
    }
    if matches!(entry.cache, AssetCache::Immutable)
        && !entry
            .route
            .rsplit('/')
            .next()
            .is_some_and(|name| name.contains(&entry.sha256))
    {
        return Err(Error::invalid(
            "immutable asset route must contain content hash",
        ));
    }
    Ok(())
}
/// Stable lowercase hex without adding a second encoding dependency.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
