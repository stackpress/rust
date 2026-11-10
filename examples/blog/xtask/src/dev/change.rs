//! Poll authored files by content and debounce changes before selecting build work.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The smallest safe work for a settled batch. Schema changes imply all three builds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Work {
    /// Run the app generator before either consumer compiles.
    pub generate: bool,
    /// Rebuild both independent browser renderers.
    pub browser: bool,
    /// Compile a replacement server while the previous executable keeps running.
    pub native: bool,
    /// Restart for configuration/browser generations even when native compilation is unnecessary.
    pub restart: bool,
    /// Stylesheet URLs understood by Leptos's CSS swap message.
    pub css: BTreeSet<String>,
}

/// Classify relative blog paths; mixed CSS/source batches always take the build path.
pub(super) fn classify(paths: &BTreeSet<PathBuf>) -> Work {
    let mut work = Work::default();
    for path in paths {
        let text = path.to_string_lossy().replace('\\', "/");
        if text == "schema.idea" {
            work.generate = true;
            work.browser = true;
            work.native = true;
        } else if text == "Cargo.toml" || text == "Cargo.lock" {
            work.native = true;
            work.browser = true;
        } else if text.starts_with("config/") {
            work.restart = true;
        } else if path.parent() == Some(Path::new("plugins/assets/assets"))
            && path.extension().is_some_and(|e| e == "css")
        {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                work.css.insert(format!("styles/{name}"));
            }
        } else if text.starts_with("client/") {
            work.browser = true;
        } else {
            // App UI is shared by SSR and both browser modes. Native plugin callbacks
            // have their own directories and do not require a WASM rebuild.
            work.native = true;
            if text.starts_with("plugins/app/")
                && !text.starts_with("plugins/app/src/pages/")
                && text != "plugins/app/src/server.rs"
            {
                work.browser = true;
            }
        }
    }
    // A browser-only build reuses the executable but advances its HTML build ID. This
    // catches tabs that loaded old bundles while compilation was still in progress.
    work.restart |= work.native || work.generate || work.browser;
    work
}

/// Content snapshots avoid timestamp granularity and detect creations and deletions.
#[derive(Default)]
pub(super) struct Snapshot(pub BTreeMap<PathBuf, Vec<u8>>);

impl Snapshot {
    /// Read only authored inputs. Build output, generated source and the database stay outside.
    pub fn read(root: &Path) -> std::io::Result<Self> {
        let mut files = BTreeMap::new();
        for path in [
            "plugins",
            "web",
            "client",
            "config",
            "schema.idea",
            "Cargo.toml",
            "Cargo.lock",
        ] {
            collect(root, &root.join(path), &mut files)?;
        }
        Ok(Self(files))
    }

    /// Compare bytes and the union of keys, so removing a file schedules the same work.
    pub fn changes(&self, next: &Self) -> BTreeSet<PathBuf> {
        self.0
            .keys()
            .chain(next.0.keys())
            .filter(|p| self.0.get(*p) != next.0.get(*p))
            .cloned()
            .collect()
    }
}

/// Recursively collect ordinary files without following symlinked directories or build trees.
fn collect(
    root: &Path,
    path: &Path,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> std::io::Result<()> {
    // Editors may leave temporary files visible across polls during an atomic save.
    // They must not turn a markup-only edit into an unrelated native build.
    if let Some(name) = path.file_name().and_then(|name| name.to_str())
        && (name.ends_with('~')
            || [".swp", ".swo", ".tmp"]
                .iter()
                .any(|suffix| name.ends_with(suffix))
            || name == ".DS_Store")
    {
        return Ok(());
    }
    if !path.exists() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if ["target", ".build", ".git"]
                .iter()
                .any(|s| entry.file_name() == *s)
            {
                continue;
            }
            collect(root, &entry.path(), files)?;
        }
    } else if metadata.is_file() {
        files.insert(
            path.strip_prefix(root)
                .map_err(std::io::Error::other)?
                .to_path_buf(),
            std::fs::read(path)?,
        );
    }
    Ok(())
}

/// A burst settles only after the last observed edit, never after the first one.
#[derive(Default)]
pub(super) struct Debounce {
    /// Union of paths since the previous dispatched batch.
    paths: BTreeSet<PathBuf>,
    /// Most recent content change in that batch.
    last: Option<Instant>,
}

impl Debounce {
    /// Add a poll's changes; an empty poll does not extend the debounce window.
    pub fn observe(&mut self, paths: BTreeSet<PathBuf>, now: Instant) {
        if !paths.is_empty() {
            self.paths.extend(paths);
            self.last = Some(now);
        }
    }

    /// Drain once at the inclusive quiet-window boundary.
    pub fn ready(&mut self, now: Instant, delay: Duration) -> Option<BTreeSet<PathBuf>> {
        if self
            .last
            .is_some_and(|last| now.saturating_duration_since(last) >= delay)
        {
            self.last = None;
            return Some(std::mem::take(&mut self.paths));
        }
        None
    }
}
