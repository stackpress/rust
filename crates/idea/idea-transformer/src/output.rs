//! Cooperative single-writer generation; staged installs have explicit recovery, not atomicity.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::registry::valid_path;
use crate::{OutputPlan, Result, TransformError};

mod format;
mod ownership;
mod report;

use format::formatted;
use ownership::read_regular;
use ownership::{
    MANIFEST, exists, inspect, inspect_owned, inspect_retained, manifest, no_symlinks,
};
use report::compare;
pub use report::{CheckReport, GenerationReport};

/// Recognized pre-cleanup intent; paths are always fixed workspace siblings, never journal data.
const INSTALL_INTENT: &str = "stackpress-install-v1\n";
/// Recognized cleanup intent, which permits missing promised files only in retained trees.
const CLEANUP_INTENT: &str = "stackpress-cleanup-v1\n";

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
        // A portable final component keeps every fixed recovery sibling in the chosen parent.
        let name = root
            .file_name()
            .and_then(|part| part.to_str())
            .ok_or_else(|| {
                TransformError::classified(
                    crate::TransformErrorKind::Ownership,
                    "generated root needs a named directory",
                )
            })?;
        valid_path(name)?;
        let parent = root
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or_else(|| {
                TransformError::classified(
                    crate::TransformErrorKind::Ownership,
                    "generated root needs an explicit parent",
                )
            })?;
        // Reject aliases before canonicalizing; later cwd changes cannot redirect this workspace.
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
        // The create-new operation arbitrates ownership without a check-then-create window.
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.lock)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    TransformError::caused(
                        crate::TransformErrorKind::LockExists,
                        "generation writer lock exists; inspect its owner",
                        error,
                    )
                } else {
                    error.into()
                }
            })?;
        // Establish RAII ownership before writing diagnostics, so a write error releases the lock.
        let guard = WriterLock(self.lock.clone());
        // PID is informational for explicit operator inspection, never automatic reclamation.
        writeln!(file, "writer pid {}", std::process::id())?;
        Ok(guard)
    }

    /// Refuse normal work while a prior install has retained recovery state.
    fn ready(&self) -> Result<()> {
        for path in [&self.journal, &self.stage, &self.backup] {
            if exists(path)? {
                return Err(TransformError::classified(
                    crate::TransformErrorKind::RecoveryRequired,
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
        // Formatting has no output-root authority and fails before acquiring a managed lock.
        let files = formatted(plan)?;
        // Keep the guard alive through inspection and reporting; no generated bytes are written.
        let _guard = self.acquire()?;
        self.compare_plan(&files)
    }

    /// Share readiness, drift inspection and marker-aware differences between check and install.
    fn compare_plan(&self, files: &BTreeMap<String, (String, String)>) -> Result<CheckReport> {
        // Both callers already hold the writer lock; checking is never a second lock attempt.
        self.ready()?;
        let previous = inspect(&self.root)?;
        let mut report = compare(&previous, files);
        // Even an empty plan is stale when its dedicated root lacks an ownership marker.
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
    #[expect(
        clippy::needless_pass_by_value,
        reason = "the compatible installation API intentionally consumes completed plans"
    )]
    pub fn install(&self, plan: OutputPlan) -> Result<CheckReport> {
        self.install_at(&plan, |_| Ok(()))
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
        plan: &OutputPlan,
        mut fault: impl FnMut(Phase) -> Result<()>,
    ) -> Result<CheckReport> {
        let files = formatted(plan)?;
        let _guard = self.acquire()?;
        let report = self.compare_plan(&files)?;
        if report.is_current() && exists(&self.root.join(MANIFEST))? {
            // Repeats preserve exact bytes and avoid a needless visibility gap during renames.
            return Ok(report);
        }
        // Only installation errors get a phase wrapper; preflight/lock errors stay actionable.
        let mut phase = Phase::StageWrite;
        self.stage_and_install(&files, |next| {
            phase = next;
            fault(next)
        })
        .map_err(|error| TransformError {
            kind: error.kind(),
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
        let temporary = tempfile::tempdir_in(self.root.parent().ok_or_else(|| {
            TransformError::classified(
                crate::TransformErrorKind::Ownership,
                "generated root lost its parent",
            )
        })?)?;
        let manifest = manifest(files);
        fault(Phase::StageWrite)?;
        for (path, (_, contents)) in files {
            let destination = temporary.path().join(path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(destination, contents)?;
        }
        let marker = serde_json::to_vec_pretty(&manifest).map_err(|cause| {
            TransformError::caused(
                crate::TransformErrorKind::Parse,
                "manifest serialization failed",
                cause,
            )
        })?;
        fs::write(temporary.path().join(MANIFEST), marker)?;
        fault(Phase::StagePublish)?;
        fs::rename(temporary.path(), &self.stage)?;
        // Every recovery path is a fixed validated sibling; journal data never supplies paths.
        fault(Phase::IntentWrite)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.journal)?
            .write_all(INSTALL_INTENT.as_bytes())?;
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
            return Err(TransformError::classified(
                crate::TransformErrorKind::Ownership,
                "no generation install intent to recover",
            ));
        }
        let intent = read_regular(&self.journal, 64)?;
        let cleaning = intent == CLEANUP_INTENT;
        if intent != INSTALL_INTENT && !cleaning {
            return Err(TransformError::classified(
                crate::TransformErrorKind::Ownership,
                "unrecognized generation journal",
            ));
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
        // Every candidate has passed validation before the first destination mutation.
        self.restore_root()?;
        self.finish_recovery()
    }

    /// Keep a complete root, or restore the validated predecessor/stage if it is absent.
    fn restore_root(&self) -> Result<()> {
        if exists(&self.root)? {
            inspect(&self.root)?;
            if !exists(&self.root.join(MANIFEST))? {
                // Only the documented initial empty directory can be adopted here.
                if !exists(&self.stage)? || exists(&self.backup)? {
                    return Err(TransformError::classified(
                        crate::TransformErrorKind::Ownership,
                        "unmarked recovery root",
                    ));
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
                return Err(TransformError::classified(
                    crate::TransformErrorKind::Ownership,
                    "recovery tree is missing",
                ));
            }
            inspect_owned(source)?;
            fs::rename(source, &self.root)?;
        }
        inspect_owned(&self.root)?;
        Ok(())
    }

    /// Remove only validated retained generated trees, leaving the intent marker until last.
    fn finish_recovery(&self) -> Result<()> {
        self.finish_at(&mut |_| Ok(()))
    }

    /// Validate all cleanup rights first, then clear intent last after every retained tree.
    fn finish_at(&self, fault: &mut impl FnMut(Phase) -> Result<()>) -> Result<()> {
        inspect_owned(&self.root)?;
        no_symlinks(&self.journal)?;
        let cleaning = read_regular(&self.journal, 64)? == CLEANUP_INTENT;
        // Validate every retained tree before purging any, preserving manual data on conflicts.
        for path in [&self.backup, &self.stage] {
            if exists(path)? {
                inspect_retained(path, cleaning)?;
            }
        }
        // Publish cleanup intent by rename, so interrupted recursive deletion is recognizable.
        // Construction fixes root to canonical_parent.join(name), so it always has a parent.
        let mut marker = tempfile::NamedTempFile::new_in(self.root.parent().unwrap())?;
        marker.write_all(CLEANUP_INTENT.as_bytes())?;
        marker
            .persist(&self.journal)
            .map_err(|error| TransformError::from(error.error))?;
        for path in [&self.backup, &self.stage] {
            if exists(path)? {
                fault(Phase::CleanupTree)?;
                purge_retained(path, fault)?;
            }
        }
        fault(Phase::ClearIntent)?;
        fs::remove_file(&self.journal)?;
        Ok(())
    }
}

/// Remove a validated retained tree while preserving its ownership marker through partial errors.
fn purge_retained(root: &Path, fault: &mut impl FnMut(Phase) -> Result<()>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        // Never delegate the root marker to recursive deletion's unspecified entry order.
        if entry.file_name() == MANIFEST {
            continue;
        }
        fault(Phase::CleanupEntry)?;
        if entry.file_type()?.is_dir() {
            // Nested deletion may be partial, but the root marker still lists every promise.
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    let marker = root.join(MANIFEST);
    if exists(&marker)? {
        // Only an otherwise empty retained root can lose its marker. Cleanup can resume empty.
        fault(Phase::CleanupEntry)?;
        fs::remove_file(marker)?;
    }
    fault(Phase::CleanupEntry)?;
    fs::remove_dir(root)?;
    Ok(())
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
    CleanupEntry,
    ClearIntent,
}

#[cfg(test)]
mod tests;
