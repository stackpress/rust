//! Explicit compiled package sources layered over an ordinary application loader.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{LoadError, LoadedSource, SourceId, SourceKind, SourceLoader, paths};

/// Reserved virtual namespace; fallback loaders must never publish identities below it.
const ROOT: &str = "/__stackpress_package_sources__";

/// One compiled immutable source; ownership stays with the selected Cargo package.
#[derive(Debug, Clone)]
struct Source {
    /// Exact bare import name, also used in diagnostics rather than a host filesystem path.
    name: String,
    /// Syntax chosen once when the source is registered.
    kind: SourceKind,
    /// Static package data, bounded before producing an owned acquisition snapshot.
    text: &'static str,
}

/// Opt-in compiled package imports over an existing [`SourceLoader`].
///
/// Exact registered bare names win over local paths, independently of app nesting.
/// Explicit relative, `@/`, absolute and unregistered app imports keep the fallback's
/// rules. Relative imports *from an embedded source* stay in the catalog namespace;
/// a missing package source never probes a synthetic host path. No global state or IO
/// is introduced by construction or registration.
///
/// # Examples
/// ```
/// use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader, PackageLoader};
/// let mut app = MemoryLoader::new("/app")?;
/// app.insert("nested/schema.idea", "use \"package/schema.idea\"");
/// let loader = PackageLoader::new(app)
///     .with_source("package/schema.idea", "model Profile { id String }")?;
/// let schema = Composer::new(loader, CompositionLimits::default())
///     .compose("nested/schema.idea")?;
/// assert!(schema.config.model.contains_key("Profile"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct PackageLoader<L: SourceLoader> {
    /// App-owned filesystem, memory or custom source acquisition.
    fallback: L,
    /// Canonical virtual paths map to a single snapshot source; there are no aliases.
    sources: BTreeMap<PathBuf, Source>,
}

impl<L: SourceLoader> PackageLoader<L> {
    /// Wrap an app loader without activating any package imports or reading input.
    pub fn new(fallback: L) -> Self {
        Self {
            fallback,
            sources: BTreeMap::new(),
        }
    }

    /// Register one exact bare import name and static package text.
    ///
    /// Names use slash-separated normal path components, without `@/`, leading dots,
    /// absolute paths or traversal. Use the fallback for runtime-owned source strings.
    ///
    /// # Errors
    /// Rejects invalid/duplicate names and unsupported source extensions. Registration
    /// consumes no composition budget; acquisition checks the caller's bound before cloning.
    pub fn with_source(mut self, name: &str, text: &'static str) -> Result<Self, LoadError> {
        // Normalize neither spelling nor aliases: the compiled name is the import contract.
        if !bare(name) {
            return Err(LoadError::new("package source requires a normal bare name"));
        }
        let path = Path::new(ROOT).join(name);
        let kind = paths::kind(&path)?;
        if self.sources.contains_key(&path) {
            return Err(LoadError::new("duplicate package source name"));
        }
        self.sources.insert(
            path,
            Source {
                name: name.into(),
                kind,
                text,
            },
        );
        Ok(self)
    }

    /// Acquire one known catalog snapshot, checking bounds before any source text clone.
    fn acquire(&self, path: &Path, max_bytes: usize) -> Option<Result<LoadedSource, LoadError>> {
        self.sources.get(path).map(|source| {
            if source.text.len() > max_bytes {
                return Err(LoadError::classified(
                    crate::TransformErrorKind::Limit,
                    "source byte budget exceeded",
                ));
            }
            Ok(LoadedSource {
                id: SourceId::new(path),
                name: source.name.clone(),
                kind: source.kind,
                text: source.text.into(),
            })
        })
    }
}

impl<L: SourceLoader> SourceLoader for PackageLoader<L> {
    /// Resolve exact package names first, then embedded-relative or ordinary app sources.
    fn load(
        &self,
        importer: Option<&SourceId>,
        specifier: &str,
        max_bytes: usize,
    ) -> Result<LoadedSource, LoadError> {
        // Only literal bare registrations shadow the fallback; ./ and @/ remain explicit.
        if bare(specifier)
            && let Some(source) = self.acquire(&Path::new(ROOT).join(specifier), max_bytes)
        {
            return source;
        }
        let embedded = importer.is_some_and(|id| id.as_path().starts_with(ROOT));
        if embedded && !specifier.starts_with("@/") && !Path::new(specifier).is_absolute() {
            let base = paths::base_path(Path::new(ROOT), importer, specifier);
            // Package-relative traversal may reach another registered package, never the host.
            for candidate in paths::candidates(&base) {
                let path = paths::normalize(&candidate);
                if path.starts_with(ROOT)
                    && let Some(source) = self.acquire(&path, max_bytes)
                {
                    return source;
                }
            }
            return Err(LoadError::with_cause(
                "package source not found",
                std::io::Error::from(std::io::ErrorKind::NotFound),
            ));
        }
        // Root/absolute app requests do not need a synthetic embedded importer identity.
        let source =
            self.fallback
                .load(if embedded { None } else { importer }, specifier, max_bytes)?;
        if source.id.as_path().starts_with(ROOT) {
            return Err(LoadError::new(
                "fallback source collides with package namespace",
            ));
        }
        Ok(source)
    }
}

/// Distinguish literal package names from explicit app-relative/root/absolute requests.
fn bare(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && !name.starts_with("@/")
        && !Path::new(name).is_absolute()
        && !name.contains('\\')
        && name
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}
