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
    /// Exact installed byte count; older M0 manifests omitted this optional field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bytes: Option<u64>,
}

/// Versioned owned-root marker; it is not proof of authenticity against malicious editors.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
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

/// Result of a no-write output comparison.
#[derive(Debug, Serialize)]
pub struct CheckReport {
    /// Paths added, changed or removed, in deterministic order.
    pub differences: Vec<String>,
}

/// Deterministic install receipt containing logical names, relative paths and selected IDs.
///
/// No timestamp, native root, environment value or source contents enter this report.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct GenerationReport {
    /// Consumed auxiliary input names, sorted and deduplicated.
    pub inputs: Vec<String>,
    /// Actual transform execution order.
    pub selected: Vec<String>,
    /// Added or changed planned paths, sorted lexically.
    pub generated: Vec<String>,
    /// Byte- and owner-identical planned paths, sorted lexically.
    pub unchanged: Vec<String>,
    /// Previously owned paths omitted from the new plan, sorted lexically.
    pub removed: Vec<String>,
    /// Writer receipt/ownership contract revision, independent of app runtime contracts.
    pub contract_version: u32,
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
    ///
    /// ```no_run
    /// use stackpress_idea_transformer::OutputWorkspace;
    /// let workspace = OutputWorkspace::new("./generated")?;
    /// # Ok::<(), stackpress_idea_transformer::TransformError>(())
    /// ```
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
            if exists(path)? {
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
    ///
    /// ```no_run
    /// use stackpress_idea_transformer::{OutputWorkspace, Registry};
    /// let plan = Registry::new().plan(&Default::default(), vec![])?;
    /// let report = OutputWorkspace::new("./generated")?.check(&plan)?;
    /// assert!(report.is_current(), "regenerate the owned output");
    /// # Ok::<(), stackpress_idea_transformer::TransformError>(())
    /// ```
    pub fn check(&self, plan: &OutputPlan) -> Result<CheckReport> {
        let files = formatted(plan)?;
        let _guard = self.acquire()?;
        self.ready()?;
        let previous = inspect(&self.root)?;
        self.comparison(&previous, &files)
    }

    /// Include a missing owned-root marker even for an otherwise empty aggregate-only plan.
    fn comparison(
        &self,
        previous: &BTreeMap<String, (String, String)>,
        files: &BTreeMap<String, (String, String)>,
    ) -> Result<CheckReport> {
        let mut report = compare(previous, files);
        if !exists(&self.root.join(MANIFEST))? {
            report.differences.push(MANIFEST.into());
            report.differences.sort();
        }
        Ok(report)
    }

    /// Stage a complete tree, retain the previous tree, then install with recoverable intent.
    ///
    /// # Errors
    /// Rejects manual files/edits, conflicting writers and invalid prior state. After intent
    /// publication, IO failure preserves stage/backup/journal for [`Self::recover`].
    /// A complete stage retained before intent publication requires explicit operator
    /// inspection; recovery never guesses whether an unrecognized install was authorized.
    ///
    /// ```no_run
    /// use stackpress_idea_transformer::{OutputWorkspace, Registry};
    /// let plan = Registry::new().plan(&Default::default(), vec![])?;
    /// OutputWorkspace::new("./generated")?.install(plan)?;
    /// # Ok::<(), stackpress_idea_transformer::TransformError>(())
    /// ```
    pub fn install(&self, plan: OutputPlan) -> Result<CheckReport> {
        self.install_at(plan, |_| Ok(()))
    }

    /// Install and return a deterministic summary while retaining the M0 `install` return type.
    ///
    /// # Errors
    /// Has the same preflight, ownership, formatter, lock and recovery failures as
    /// [`Self::install`].
    ///
    /// ```no_run
    /// use stackpress_idea_transformer::{OutputWorkspace, TransformRegistry};
    /// let registry = TransformRegistry::new();
    /// let plan = registry.plan(&Default::default(), vec![])?;
    /// let report = OutputWorkspace::new("./generated")?.install_report(plan)?;
    /// assert!(report.selected.is_empty());
    /// # Ok::<(), stackpress_idea_transformer::TransformError>(())
    /// ```
    pub fn install_report(&self, plan: OutputPlan) -> Result<GenerationReport> {
        let paths: Vec<_> = plan.files.keys().cloned().collect();
        let inputs = plan.inputs.clone();
        let selected = plan.executed().to_vec();
        let report = self.install(plan)?;
        let (generated, unchanged) = paths
            .iter()
            .cloned()
            .partition(|path| report.differences.binary_search(path).is_ok());
        let removed = report
            .differences
            .into_iter()
            .filter(|path| path != MANIFEST && paths.binary_search(path).is_err())
            .collect();
        Ok(GenerationReport {
            inputs,
            selected,
            generated,
            unchanged,
            removed,
            contract_version: 1,
        })
    }

    /// Run real native operations with a test seam at every installation boundary.
    fn install_at(
        &self,
        plan: OutputPlan,
        mut fault: impl FnMut(Phase) -> Result<()>,
    ) -> Result<CheckReport> {
        let files = formatted(&plan)?;
        let _guard = self.acquire()?;
        self.ready()?;
        let previous = inspect(&self.root)?;
        let report = self.comparison(&previous, &files)?;
        if report.is_current() && exists(&self.root.join(MANIFEST))? {
            return Ok(report);
        }
        // Only installation errors get a phase wrapper; preflight/lock errors stay actionable.
        let mut phase = Phase::StageWrite;
        self.stage_and_install(&files, |next| {
            phase = next;
            fault(next)
        })
        .map_err(|error| TransformError {
            message: format!("generation install failed during {phase:?}; inspect recovery state"),
            cause: Some(Box::new(error)),
        })?;
        Ok(report)
    }

    /// Publish a complete staged tree while the caller retains exclusive writer ownership.
    fn stage_and_install(
        &self,
        files: &BTreeMap<String, (String, String)>,
        mut fault: impl FnMut(Phase) -> Result<()>,
    ) -> Result<()> {
        // TempDir cleans staging-write failures. Moving it to stage retains recovery data.
        let temporary = tempfile::tempdir_in(
            self.root
                .parent()
                .ok_or_else(|| TransformError::new("generated root lost its parent"))?,
        )?;
        let manifest = manifest(files);
        fault(Phase::StageWrite)?;
        for (path, (_, contents)) in files {
            let destination = temporary.path().join(path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(destination, contents)?;
        }
        let marker = serde_json::to_vec_pretty(&manifest)
            .map_err(|_| TransformError::new("manifest serialization failed"))?;
        fs::write(temporary.path().join(MANIFEST), marker)?;
        fault(Phase::StagePublish)?;
        fs::rename(temporary.path(), &self.stage)?;
        // Every recovery path is a fixed validated sibling; journal data never supplies paths.
        fault(Phase::IntentWrite)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.journal)?
            .write_all(b"stackpress-install-v1\n")?;
        fault(Phase::IntentPublished)?;
        fault(Phase::BackupRename)?;
        if exists(&self.root)? {
            // An initial empty directory has no owned predecessor to preserve as a backup.
            if !exists(&self.root.join(MANIFEST))? {
                fs::remove_dir(&self.root)?;
            } else {
                fs::rename(&self.root, &self.backup)?;
            }
        }
        fault(Phase::BackupRetained)?;
        fault(Phase::InstallRename)?;
        fs::rename(&self.stage, &self.root)?;
        fault(Phase::Installed)?;
        self.finish_at(&mut fault)?;
        Ok(())
    }

    /// Recover a complete current tree or restore its retained predecessor, then clear intent.
    ///
    /// Recovery prefers an installed valid root; if absent it restores backup, or installs the
    /// complete stage for a first install. A process-death lock must be removed explicitly by
    /// an operator after establishing no writer remains; recovery never guesses from a PID.
    /// Recognized cleanup intent requires a valid installed root and permits missing promised
    /// backup files, so interrupted deletion can resume. It never restores a partial backup.
    ///
    /// # Errors
    /// Returns lock, ownership, unexpected marker or IO failures; it never purges manual files.
    ///
    /// ```no_run
    /// use stackpress_idea_transformer::OutputWorkspace;
    /// // First inspect/remove a stale process-death lock after confirming no writer remains.
    /// OutputWorkspace::new("./generated")?.recover()?;
    /// # Ok::<(), stackpress_idea_transformer::TransformError>(())
    /// ```
    pub fn recover(&self) -> Result<()> {
        let _guard = self.acquire()?;
        no_symlinks(&self.journal)?;
        if !exists(&self.journal)? {
            return Err(TransformError::new(
                "no generation install intent to recover",
            ));
        }
        let intent = read_source(&self.journal, 64)?;
        let cleaning = intent == "stackpress-cleanup-v1\n";
        if intent != "stackpress-install-v1\n" && !cleaning {
            return Err(TransformError::new("unrecognized generation journal"));
        }
        // Inspect before any rename/removal; ownership drift blocks rather than destroys data.
        // Validate every retained tree before moving or deleting any of them.
        if cleaning {
            // A partially purged predecessor is never eligible to restore the destination.
            inspect_owned(&self.root)?;
        }
        for path in [&self.stage, &self.backup] {
            if exists(path)? {
                inspect_retained(path, cleaning)?;
            }
        }
        if exists(&self.root)? {
            inspect(&self.root)?;
            if !exists(&self.root.join(MANIFEST))? {
                // Only the documented initial empty directory can be adopted here.
                if !exists(&self.stage)? || exists(&self.backup)? {
                    return Err(TransformError::new("unmarked recovery root"));
                }
                fs::remove_dir(&self.root)?;
            }
        }
        if !exists(&self.root)? {
            let source = if exists(&self.backup)? {
                &self.backup
            } else {
                &self.stage
            };
            if !exists(source)? {
                return Err(TransformError::new("recovery tree is missing"));
            }
            inspect_owned(source)?;
            fs::rename(source, &self.root)?;
        }
        inspect_owned(&self.root)?;
        self.finish_recovery()
    }

    /// Remove only validated retained generated trees, leaving the intent marker until last.
    fn finish_recovery(&self) -> Result<()> {
        self.finish_at(&mut |_| Ok(()))
    }

    /// Validate all cleanup rights first, then clear intent last after every retained tree.
    fn finish_at(&self, fault: &mut impl FnMut(Phase) -> Result<()>) -> Result<()> {
        inspect_owned(&self.root)?;
        no_symlinks(&self.journal)?;
        let cleaning = read_source(&self.journal, 64)? == "stackpress-cleanup-v1\n";
        for path in [&self.backup, &self.stage] {
            if exists(path)? {
                inspect_retained(path, cleaning)?;
            }
        }
        // Publish cleanup intent by rename, so interrupted recursive deletion is recognizable.
        let mut marker = tempfile::NamedTempFile::new_in(self.root.parent().unwrap())?;
        marker.write_all(b"stackpress-cleanup-v1\n")?;
        marker
            .persist(&self.journal)
            .map_err(|error| TransformError::from(error.error))?;
        for path in [&self.backup, &self.stage] {
            if exists(path)? {
                fault(Phase::CleanupTree)?;
                fs::remove_dir_all(path)?;
            }
        }
        fault(Phase::ClearIntent)?;
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
                        bytes: Some(text.len() as u64),
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
    let manifest: Manifest = serde_json::from_str(&read_source(&marker, 1024 * 1024)?)
        .map_err(|_| TransformError::new("root is not a recognized generated directory"))?;
    if manifest.version != 1 || manifest.files.len() > 1024 {
        return Err(TransformError::new("unsupported generated manifest"));
    }
    let mut files: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut remaining = 1024 * 1024usize;
    for (path, entry) in manifest.files {
        valid_path(&path)?;
        if entry.owner.is_empty()
            || entry.owner.len() > 128
            || entry.sha256.len() != 64
            || !entry.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            || files
                .keys()
                .any(|prior| crate::plan::paths_collide(prior, &path))
        {
            return Err(TransformError::new("invalid generated manifest ownership"));
        }
        let absolute = root.join(&path);
        no_symlinks(&absolute)?;
        if allow_missing && !exists(&absolute)? {
            // Keep the promised path for exact directory/unknown-file validation below.
            files.insert(path, (entry.owner, String::new()));
            continue;
        }
        let text = read_source(&absolute, 1024 * 1024)?;
        remaining = remaining
            .checked_sub(text.len())
            .ok_or_else(|| TransformError::new("installed output byte budget exceeded"))?;
        if entry.bytes.is_some_and(|bytes| bytes != text.len() as u64)
            || digest(&text) != entry.sha256
        {
            return Err(TransformError::new(
                "manually edited generated file; restore it explicitly",
            ));
        }
        files.insert(path, (entry.owner, text));
    }
    inspect_paths(root, root, &files, 0)?;
    Ok(files)
}

/// Testable installation boundaries; faults exercise the production state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    StageWrite,
    StagePublish,
    IntentWrite,
    IntentPublished,
    BackupRename,
    BackupRetained,
    InstallRename,
    Installed,
    CleanupTree,
    ClearIntent,
}

/// Inspect existence without hiding dangling symlinks as absent paths.
fn exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

/// Require a real ownership marker before removing a retained stage or backup tree.
fn inspect_owned(root: &Path) -> Result<BTreeMap<String, (String, String)>> {
    no_symlinks(root)?;
    no_symlinks(&root.join(MANIFEST))?;
    if !exists(&root.join(MANIFEST))? {
        return Err(TransformError::new(
            "recovery tree lacks generated ownership marker",
        ));
    }
    inspect(root)
}

/// Cleanup may resume after individual files or the final marker have already been removed.
fn inspect_retained(root: &Path, cleaning: bool) -> Result<()> {
    no_symlinks(root)?;
    if cleaning && fs::read_dir(root)?.next().is_none() {
        // An empty directory contains no manual bytes and can be removed after cleanup intent.
        return Ok(());
    }
    no_symlinks(&root.join(MANIFEST))?;
    if !exists(&root.join(MANIFEST))? {
        return Err(TransformError::new(
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
