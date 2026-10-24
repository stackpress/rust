//! Native bounded file acquisition, canonical identities and detectable snapshot drift.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use super::{LoadError, LoadedSource, SourceId, SourceLoader, paths};

/// Native project loader rooted explicitly, without a process cwd mutation or global cache.
#[derive(Debug, Clone)]
pub struct ProjectLoader {
    /// Canonical directory supplying entry and `@/` resolution.
    root: PathBuf,
}

impl ProjectLoader {
    /// Open an explicit absolute project root; this is a resolution base, not a sandbox.
    ///
    /// # Errors
    /// Rejects relative/non-directory roots and preserves canonicalization IO causes.
    ///
    /// # Examples
    /// ```no_run
    /// use stackpress_idea_transformer::{ProjectLoader, SourceLoader};
    /// let loader = ProjectLoader::new("/workspace/project")?;
    /// let source = loader.load(None, "schema.idea", 1024 * 1024)?;
    /// assert!(!source.name.is_empty());
    /// # Ok::<(), stackpress_idea_transformer::LoadError>(())
    /// ```
    pub fn new(root: impl AsRef<Path>) -> Result<Self, LoadError> {
        // Avoid making caller intent depend on the process directory at construction time.
        if !root.as_ref().is_absolute() {
            return Err(LoadError::new("source root must be absolute"));
        }
        let root = fs::canonicalize(root)
            .map_err(|cause| LoadError::with_cause("source root resolution failed", cause))?;
        if !root.is_dir() {
            return Err(LoadError::new("source root is not a directory"));
        }
        Ok(Self { root })
    }

    /// Borrow the canonical resolution root without reading the process directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Canonicalize, open and read at most one detection byte past the caller's bound.
    fn read(&self, path: &Path, max_bytes: usize) -> Result<LoadedSource, LoadError> {
        let canonical = fs::canonicalize(path)
            .map_err(|cause| LoadError::with_cause("source canonicalization failed", cause))?;
        let kind = paths::kind(&canonical)?;
        let mut file = File::open(&canonical)
            .map_err(|cause| LoadError::with_cause("source open failed", cause))?;
        let before = file
            .metadata()
            .map_err(|cause| LoadError::with_cause("source metadata failed", cause))?;
        // Preflight large/special files before allocating, then still bound reads after races.
        if !before.is_file() {
            return Err(LoadError::new("schema source is not a regular file"));
        }
        if before.len() > max_bytes as u64 {
            return Err(LoadError::new("source byte budget exceeded"));
        }
        let mut bytes = Vec::new();
        (&mut file)
            .take((max_bytes as u64).saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|cause| LoadError::with_cause("source read failed", cause))?;
        if bytes.len() > max_bytes {
            return Err(LoadError::new("source byte budget exceeded"));
        }
        let after = file
            .metadata()
            .map_err(|cause| LoadError::with_cause("source metadata failed", cause))?;
        // Size/mtime checks detect common cooperative edits, not arbitrary concurrent writers.
        if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
            return Err(LoadError::new("source changed during acquisition"));
        }
        let text = String::from_utf8(bytes)
            .map_err(|cause| LoadError::with_cause("source is not UTF-8", cause))?;
        Ok(LoadedSource {
            name: paths::logical_name(&self.root, &canonical),
            id: SourceId::new(canonical),
            kind,
            text,
        })
    }
}

impl SourceLoader for ProjectLoader {
    /// Resolve exact/extension/index candidates, preserving non-missing IO failures.
    fn load(
        &self,
        importer: Option<&SourceId>,
        specifier: &str,
        max_bytes: usize,
    ) -> Result<LoadedSource, LoadError> {
        let base = paths::base_path(&self.root, importer, specifier);
        let mut directory_without_index = false;
        for candidate in paths::candidates(&base) {
            match fs::metadata(&candidate) {
                Ok(metadata) if metadata.is_dir() => {
                    directory_without_index = true;
                    continue;
                }
                Ok(metadata) if metadata.is_file() => return self.read(&candidate, max_bytes),
                // Avoid opening devices/FIFOs, whose open/read could block before metadata checks.
                Ok(_) => return Err(LoadError::new("schema source is not a regular file")),
                // Missing candidates may fall through; permission and other IO errors may not.
                Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => {
                    continue;
                }
                Err(cause) => return Err(LoadError::with_cause("source resolution failed", cause)),
            }
        }
        if directory_without_index {
            return Err(LoadError::with_cause(
                "schema directory has no index source",
                std::io::Error::from(std::io::ErrorKind::IsADirectory),
            ));
        }
        Err(LoadError::with_cause(
            "schema source not found",
            std::io::Error::from(std::io::ErrorKind::NotFound),
        ))
    }
}
