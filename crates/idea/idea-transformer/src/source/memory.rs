//! Owned in-memory project snapshots with the native adapter's resolution order.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{LoadError, LoadedSource, SourceId, SourceLoader, paths};

/// Deterministic in-memory loader; clones copy all source bytes independently.
#[derive(Debug, Clone)]
pub struct MemoryLoader {
    /// Explicit absolute namespace root, never inferred from the process directory.
    root: PathBuf,
    /// Canonical virtual paths and their UTF-8 text.
    files: BTreeMap<PathBuf, String>,
    /// Alias paths mapped directly to canonical source paths.
    aliases: BTreeMap<PathBuf, PathBuf>,
}

impl MemoryLoader {
    /// Create a virtual project with an explicit absolute root.
    ///
    /// # Errors
    /// Rejects relative roots rather than consulting the current directory.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_transformer::{MemoryLoader, SourceLoader};
    /// let mut loader = MemoryLoader::new("/project")?;
    /// loader.insert("schema.idea", "model User { id String }");
    /// let file = loader.load(None, "schema", 1024)?;
    /// assert_eq!(file.name, "schema.idea");
    /// # Ok::<(), stackpress_idea_transformer::LoadError>(())
    /// ```
    pub fn new(root: impl AsRef<Path>) -> Result<Self, LoadError> {
        // Requiring an absolute namespace makes fixtures independent of the test runner cwd.
        if !root.as_ref().is_absolute() {
            return Err(LoadError::new("source root must be absolute"));
        }
        Ok(Self {
            root: paths::normalize(root.as_ref()),
            files: BTreeMap::new(),
            aliases: BTreeMap::new(),
        })
    }

    /// Insert or replace a root-relative virtual source; no disk IO occurs.
    pub fn insert(&mut self, path: impl AsRef<Path>, text: impl Into<String>) {
        let path = paths::normalize(&self.root.join(path));
        // Explicit insertion replaces a prior alias as well as a prior source at this path.
        self.aliases.remove(&path);
        self.files.insert(path, text.into());
    }

    /// Register a fixture alias to an existing canonical file, without chained alias lookup.
    ///
    /// # Errors
    /// Rejects a missing target or an alias that would hide a canonical source.
    pub fn alias(
        &mut self,
        alias: impl AsRef<Path>,
        target: impl AsRef<Path>,
    ) -> Result<(), LoadError> {
        let alias = paths::normalize(&self.root.join(alias));
        let target = paths::normalize(&self.root.join(target));
        // Aliases cannot shadow physical virtual files; every identity stays unambiguous.
        if !self.files.contains_key(&target) || self.files.contains_key(&alias) {
            return Err(LoadError::new(
                "alias requires an existing target and unused path",
            ));
        }
        self.aliases.insert(alias, target);
        Ok(())
    }
}

impl SourceLoader for MemoryLoader {
    /// Resolve deterministically, checking byte limits before cloning stored text.
    fn load(
        &self,
        importer: Option<&SourceId>,
        specifier: &str,
        max_bytes: usize,
    ) -> Result<LoadedSource, LoadError> {
        let base = paths::base_path(&self.root, importer, specifier);
        for candidate in paths::candidates(&base) {
            let candidate = paths::normalize(&candidate);
            let canonical = self.aliases.get(&candidate).unwrap_or(&candidate);
            // Only canonical files exist; absent candidates fall through in declared order.
            if let Some(text) = self.files.get(canonical) {
                if text.len() > max_bytes {
                    return Err(LoadError::classified(
                        crate::TransformErrorKind::Limit,
                        "source byte budget exceeded",
                    ));
                }
                return Ok(LoadedSource {
                    id: SourceId::new(canonical.clone()),
                    name: paths::logical_name(&self.root, canonical),
                    kind: paths::kind(canonical)?,
                    text: text.clone(),
                });
            }
        }
        Err(LoadError::with_cause(
            "schema source not found",
            std::io::Error::from(std::io::ErrorKind::NotFound),
        ))
    }
}
