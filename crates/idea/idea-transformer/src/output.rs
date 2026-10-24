//! Cooperative single-writer generation; staged installs have explicit recovery, not atomicity.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::registry::valid_path;
use crate::{OutputPlan, Result, TransformError, read_source};

/// Reserved marker and ownership manifest; it never claims its own digest.
const MANIFEST: &str = ".stackpress-generated.json";

/// One installed output's cooperative drift-detection data.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    /// Plugin or aggregate owner recorded when planning.
    owner: String,
    /// SHA-256 of the last installed UTF-8 file bytes.
    sha256: String,
}

/// Versioned owned-root marker; it is not proof of authenticity against malicious editors.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    /// Only this exact version is accepted for managed cleanup.
    version: u32,
    /// Every owned relative path and its last installed content digest.
    files: BTreeMap<String, Entry>,
}

/// Result of a no-write output comparison.
#[derive(Debug, Serialize)]
pub struct CheckReport {
    /// Paths added, changed or removed, in deterministic order.
    pub differences: Vec<String>,
}

impl CheckReport {
    /// Whether every planned file and the exact ownership set match disk.
    pub fn is_current(&self) -> bool {
        self.differences.is_empty()
    }
}

/// Native generated root requiring exclusive/cooperative ownership by all writers.
///
/// No operation follows output symlinks. A lock prevents competing managed writers, but
/// uncooperative external edits can still race checks. No crash-atomic/durability claim is made.
pub struct OutputWorkspace {
    /// Validated destination, independent of process cwd changes.
    root: PathBuf,
    /// Stable same-parent staging path, retained when installation needs recovery.
    stage: PathBuf,
    /// Prior validated tree preserved while replacing the destination.
    backup: PathBuf,
    /// Intent marker survives interrupted installs until explicit recovery.
    journal: PathBuf,
    /// Cooperative create-new writer lock outside the replaced tree.
    lock: PathBuf,
}

/// Open create-new lock; ordinary returns/panics release it, process death leaves it explicit.
struct WriterLock(PathBuf);

impl Drop for WriterLock {
    /// Release only this invocation's successfully acquired lock.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

impl OutputWorkspace {
    /// Bind a dedicated destination whose parent already exists and has no symlink components.
    ///
    /// # Errors
    /// Rejects roots without a portable name, missing parents and symlink path components.
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let name = root
            .file_name()
            .and_then(|part| part.to_str())
            .ok_or_else(|| TransformError::new("generated root needs a named directory"))?;
        valid_path(name)?;
        let parent = root
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or_else(|| TransformError::new("generated root needs an explicit parent"))?;
        no_symlinks(parent)?;
        let parent = parent.canonicalize()?;
        let sibling = |suffix: &str| parent.join(format!(".{name}.stackpress-{suffix}"));
        let workspace = Self {
            root: parent.join(name),
            stage: sibling("stage"),
            backup: sibling("backup"),
            journal: sibling("journal"),
            lock: sibling("lock"),
        };
        no_symlinks(&workspace.root)?;
        Ok(workspace)
    }

    /// Acquire a cooperative lock immediately, without waits or automatic stale-PID reclamation.
    fn acquire(&self) -> Result<WriterLock> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.lock)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    TransformError::new("generation writer lock exists; inspect its owner")
                } else {
                    error.into()
                }
            })?;
        let guard = WriterLock(self.lock.clone());
        // PID is informational for explicit operator inspection, never automatic reclamation.
        writeln!(file, "writer pid {}", std::process::id())?;
        Ok(guard)
    }

    /// Refuse normal work while a prior install has retained recovery state.
    fn ready(&self) -> Result<()> {
        for path in [&self.journal, &self.stage, &self.backup] {
            if path.try_exists()? {
                return Err(TransformError::new(
                    "generation recovery required before another run",
                ));
            }
        }
        Ok(())
    }

    /// Compare fresh planned output without modifying the root, manifest or file contents.
    ///
    /// # Errors
    /// Returns formatting, lock, recovery, manual-drift or filesystem failures.
    pub fn check(&self, plan: &OutputPlan) -> Result<CheckReport> {
        let files = formatted(plan)?;
        let _guard = self.acquire()?;
        self.ready()?;
        let previous = inspect(&self.root)?;
        Ok(compare(&previous, &files))
    }

    /// Stage a complete tree, retain the previous tree, then install with recoverable intent.
    ///
    /// # Errors
    /// Rejects manual files/edits, conflicting writers and invalid prior state. After intent
    /// publication, IO failure preserves stage/backup/journal for [`Self::recover`].
    pub fn install(&self, plan: OutputPlan) -> Result<CheckReport> {
        self.install_with(plan, || Ok(()))
    }

    /// Run the actual state machine with one narrow interruption seam after retaining old output.
    fn install_with(
        &self,
        plan: OutputPlan,
        after_backup: impl FnOnce() -> Result<()>,
    ) -> Result<CheckReport> {
        let files = formatted(&plan)?;
        let _guard = self.acquire()?;
        self.ready()?;
        let previous = inspect(&self.root)?;
        let report = compare(&previous, &files);
        if report.is_current() && self.root.try_exists()? {
            return Ok(report);
        }
        // TempDir cleans staging-write failures. Moving it to stage retains recovery data.
        let temporary = tempfile::tempdir_in(
            self.root
                .parent()
                .ok_or_else(|| TransformError::new("generated root lost its parent"))?,
        )?;
        let manifest = manifest(&files);
        for (path, (_, contents)) in &files {
            let destination = temporary.path().join(path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(destination, contents)?;
        }
        let marker = serde_json::to_vec_pretty(&manifest)
            .map_err(|_| TransformError::new("manifest serialization failed"))?;
        fs::write(temporary.path().join(MANIFEST), marker)?;
        fs::rename(temporary.path(), &self.stage)?;
        // Every recovery path is a fixed validated sibling; journal data never supplies paths.
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.journal)?
            .write_all(b"stackpress-install-v1\n")?;
        if self.root.try_exists()? {
            fs::rename(&self.root, &self.backup)?;
        }
        after_backup()?;
        fs::rename(&self.stage, &self.root)?;
        self.finish_recovery()?;
        Ok(report)
    }

    /// Recover a complete current tree or restore its retained predecessor, then clear intent.
    ///
    /// Recovery prefers an installed valid root; if absent it restores backup, or installs the
    /// complete stage for a first install. A process-death lock must be removed explicitly by
    /// an operator after establishing no writer remains; recovery never guesses from a PID.
    ///
    /// # Errors
    /// Returns lock, ownership, unexpected marker or IO failures; it never purges manual files.
    pub fn recover(&self) -> Result<()> {
        let _guard = self.acquire()?;
        if !self.journal.try_exists()? {
            return Err(TransformError::new(
                "no generation install intent to recover",
            ));
        }
        if read_source(&self.journal, 64)? != "stackpress-install-v1\n" {
            return Err(TransformError::new("unrecognized generation journal"));
        }
        // Inspect before any rename/removal; ownership drift blocks rather than destroys data.
        if !self.root.try_exists()? {
            let source = if self.backup.try_exists()? {
                &self.backup
            } else {
                &self.stage
            };
            if !source.try_exists()? {
                return Err(TransformError::new("recovery tree is missing"));
            }
            inspect(source)?;
            fs::rename(source, &self.root)?;
        }
        inspect(&self.root)?;
        self.finish_recovery()
    }

    /// Remove only validated retained generated trees, leaving the intent marker until last.
    fn finish_recovery(&self) -> Result<()> {
        for path in [&self.backup, &self.stage] {
            if path.try_exists()? {
                inspect(path)?;
                fs::remove_dir_all(path)?;
            }
        }
        fs::remove_file(&self.journal)?;
        Ok(())
    }
}

/// Inspect every existing path component without following output symlinks.
fn no_symlinks(path: &Path) -> Result<()> {
    let mut prefix = PathBuf::new();
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(TransformError::new(
                "output path cannot contain parent traversal",
            ));
        }
        prefix.push(component);
        match fs::symlink_metadata(&prefix) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(TransformError::new(
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
fn manifest(files: &BTreeMap<String, (String, String)>) -> Manifest {
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
                    },
                )
            })
            .collect(),
    }
}

/// Format Rust through stdin/stdout, avoiding formatter writes outside the staged plan.
fn formatted(plan: &OutputPlan) -> Result<BTreeMap<String, (String, String)>> {
    let mut files = plan.files.clone();
    for (path, (_, text)) in &mut files {
        if path.ends_with(".rs") {
            let mut child = Command::new("rustfmt")
                .args([
                    "--edition",
                    "2024",
                    "--config",
                    "skip_children=true",
                    "--emit",
                    "stdout",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let written = match child.stdin.take() {
                Some(mut input) => input.write_all(text.as_bytes()),
                None => Err(std::io::Error::other("formatter stdin missing")),
            };
            // A broken pipe must not leave a live child behind on an early error return.
            if let Err(error) = written {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
            let output = child.wait_with_output()?;
            if !output.status.success() {
                return Err(TransformError::new("Rust formatting failed"));
            }
            *text = String::from_utf8(output.stdout)
                .map_err(|_| TransformError::new("formatter output is not UTF-8"))?;
        }
    }
    if files.values().map(|(_, text)| text.len()).sum::<usize>() > 1024 * 1024 {
        return Err(TransformError::new("formatted output byte budget exceeded"));
    }
    Ok(files)
}

/// Verify marker, exact file set and every last-installed digest before granting cleanup rights.
fn inspect(root: &Path) -> Result<BTreeMap<String, (String, String)>> {
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
    let manifest: Manifest = serde_json::from_str(&read_source(&marker, 1024 * 1024)?)
        .map_err(|_| TransformError::new("root is not a recognized generated directory"))?;
    if manifest.version != 1 || manifest.files.len() > 1024 {
        return Err(TransformError::new("unsupported generated manifest"));
    }
    let mut files = BTreeMap::new();
    let mut remaining = 1024 * 1024usize;
    for (path, entry) in manifest.files {
        valid_path(&path)?;
        let absolute = root.join(&path);
        no_symlinks(&absolute)?;
        let text = read_source(&absolute, 1024 * 1024)?;
        remaining = remaining
            .checked_sub(text.len())
            .ok_or_else(|| TransformError::new("installed output byte budget exceeded"))?;
        if digest(&text) != entry.sha256 {
            return Err(TransformError::new(
                "manually edited generated file; restore it explicitly",
            ));
        }
        files.insert(path, (entry.owner, text));
    }
    inspect_paths(root, root, &files, 0)?;
    Ok(files)
}

/// Reject unknown files, special files, symlinks and excessive directory depth before cleanup.
fn inspect_paths(
    root: &Path,
    path: &Path,
    files: &BTreeMap<String, (String, String)>,
    depth: usize,
) -> Result<()> {
    if depth > 16 {
        return Err(TransformError::new("generated directory depth exceeded"));
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            let relative = path
                .strip_prefix(root)
                .ok()
                .and_then(Path::to_str)
                .ok_or_else(|| TransformError::new("invalid generated directory"))?;
            if !files
                .keys()
                .any(|file| file.starts_with(&format!("{relative}/")))
            {
                return Err(TransformError::new(
                    "manual directory inside generated root",
                ));
            }
            inspect_paths(root, &path, files, depth + 1)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .ok()
                .and_then(Path::to_str)
                .ok_or_else(|| TransformError::new("invalid generated path"))?;
            if !kind.is_file() || (relative != MANIFEST && !files.contains_key(relative)) {
                return Err(TransformError::new(
                    "manual or special file inside generated root",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

/// Compare exact bytes and ownership, retaining removals in the deterministic path set.
fn compare(
    previous: &BTreeMap<String, (String, String)>,
    files: &BTreeMap<String, (String, String)>,
) -> CheckReport {
    let mut differences: Vec<_> = previous
        .keys()
        .chain(files.keys())
        .filter(|path| previous.get(*path) != files.get(*path))
        .cloned()
        .collect();
    differences.sort();
    differences.dedup();
    CheckReport { differences }
}
