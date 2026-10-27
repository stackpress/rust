//! Cooperative immutable publication with retained staging on interrupted writes.

use super::*;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// App-owned history root, separate from disposable generated output.
pub struct RevisionStore {
    root: PathBuf,
}
/// Recording identical semantics does not append another revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RevisionOutcome {
    /// A complete new immutable directory was published.
    Recorded(RevisionId),
    /// Current semantics already match this head.
    NoChanges(RevisionId),
}
/// Versioned snapshot with an independently checked canonical content hash.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotArtifact {
    hash: String,
    snapshot: SnapshotBody,
}
/// Cooperative writer ownership; retained staging, never recursive cleanup, is recovery evidence.
struct WriterLock {
    path: PathBuf,
    _file: File,
}
impl Drop for WriterLock {
    /// Release only the lock this guard created; history/staging are never deleted.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
impl RevisionStore {
    /// Select an explicit root without IO or implicit repairs.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    /// Validate complete immutable artifacts, hashes and a single unbranched parent chain.
    ///
    /// # Errors
    /// Refuses symlinks/special files, interrupted publications, unknown formats, missing
    /// parents, branches and edited companions. No repair or baseline adoption occurs.
    pub fn check(&self) -> DataResult<Vec<RevisionSnapshot>> {
        safe_path(&self.root)?;
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        for entry in entries(&self.root)? {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !matches!(name.as_str(), "revisions" | "migrations" | ".writer.lock") {
                return Err(stale(
                    "history contains interrupted or unknown publication state",
                ));
            }
        }
        let mut snapshots = BTreeMap::new();
        for path in directories(&self.root.join("revisions"))? {
            complete_files(&path, &["schema.json"])?;
            let text = read(&path.join("schema.json"))?;
            let artifact: SnapshotArtifact = decode(&text)?;
            let snapshot = RevisionSnapshot::from_body(artifact.snapshot)?;
            if text
                != json(&SnapshotArtifact {
                    hash: snapshot.hash.clone(),
                    snapshot: snapshot.body.clone(),
                })?
            {
                return Err(stale("immutable revision artifact bytes were modified"));
            }
            if artifact.hash != snapshot.hash
                || path.file_name().and_then(|s| s.to_str()) != Some(snapshot.revision.as_str())
            {
                return Err(stale("revision artifact was modified"));
            }
            if snapshots
                .insert(snapshot.revision.clone(), snapshot)
                .is_some()
            {
                return Err(stale("duplicate revision identity"));
            }
        }
        // Follow parent links rather than directory names or timestamps; a second child is a branch.
        let mut ordered = Vec::new();
        let mut parent = None;
        while !snapshots.is_empty() {
            let matches = snapshots
                .values()
                .filter(|s| s.body.parent == parent)
                .map(|s| s.revision.clone())
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(stale("history has missing parents or unresolved branches"));
            }
            let snapshot = snapshots
                .remove(&matches[0])
                .ok_or_else(|| stale("history parent vanished"))?;
            if ordered
                .first()
                .is_some_and(|s: &RevisionSnapshot| s.app != snapshot.app)
            {
                return Err(stale("history app namespaces disagree"));
            }
            parent = Some(snapshot.revision.clone());
            ordered.push(snapshot);
        }
        // Every proposed plan must still point at the exact immutable revision bodies.
        for path in directories(&self.root.join("migrations"))? {
            let plan = self.read_plan_dir(&path)?;
            if plan
                .reverse()
                .is_some_and(|r| !self.root.join("migrations").join(r.hash.as_str()).is_dir())
            {
                return Err(stale("reverse artifact publication is incomplete"));
            }
            if !ordered.iter().any(|s| s.hash == plan.to.hash)
                || (!plan.from.tables.is_empty()
                    && !ordered.iter().any(|s| s.hash == plan.from.hash))
            {
                return Err(stale("migration references missing or modified revision"));
            }
            if matches!(plan.recipe.kind, PlanKind::Schema)
                && !plan.recipe.reverse
                && !plan.from.tables.is_empty()
                && plan.to.parent() != Some(plan.from.revision())
            {
                return Err(stale("migration does not follow its declared parent"));
            }
        }
        Ok(ordered)
    }
    /// Record current semantics explicitly; generation never calls this method.
    ///
    /// # Errors
    /// A busy writer, stale explicit parent, duplicate revision or retained staging refuses
    /// publication. Files are create-new and synced before directory publication. Filesystem
    /// rename is not a portable crash-atomic transaction; `check` detects partial state.
    pub fn record(
        &self,
        snapshot: &RevisionSnapshot,
        parent: Option<&RevisionId>,
    ) -> DataResult<RevisionOutcome> {
        // One cooperative writer owns parent validation through publication; readers never repair.
        let _lock = self.lock()?;
        let history = self.check()?;
        let head = history.last();
        if parent.map(RevisionId::as_str) != head.map(RevisionSnapshot::revision) {
            return Err(stale("explicit history parent is stale"));
        }
        if head.is_some_and(|head| head.app != snapshot.app) {
            return Err(stale("revision app differs from history namespace"));
        }
        let mut snapshot = snapshot.clone();
        snapshot.body.parent = head.map(|s| s.revision.clone());
        snapshot.hash = digest(&snapshot.body)?;
        if let Some(head) = head
            && head.app == snapshot.app
            && head.body.tables == snapshot.body.tables
            && head.body.source_fingerprint == snapshot.body.source_fingerprint
            && head.body.definition_revision == snapshot.body.definition_revision
        {
            return Ok(RevisionOutcome::NoChanges(RevisionId::parse(
                head.revision(),
            )?));
        }
        if history.iter().any(|s| s.revision == snapshot.revision) {
            return Err(stale("immutable revision already exists"));
        }
        let bytes = json(&SnapshotArtifact {
            hash: snapshot.hash.clone(),
            snapshot: snapshot.body.clone(),
        })?;
        self.publish(
            "revisions",
            &snapshot.revision,
            &[("schema.json".into(), bytes)],
        )?;
        Ok(RevisionOutcome::Recorded(RevisionId::parse(
            &snapshot.revision,
        )?))
    }
    /// Borrow one checked history snapshot by explicit identity.
    pub fn revision(&self, id: &RevisionId) -> DataResult<RevisionSnapshot> {
        let mut snapshot = self
            .check()?
            .into_iter()
            .find(|s| s.revision == id.0)
            .ok_or_else(|| stale("revision is missing"))?;
        let mut layout = None;
        for directory in directories(&self.root.join("migrations"))? {
            let plan = self.read_plan_dir(&directory)?;
            if plan.to.hash == snapshot.hash {
                let order = storage_order(&plan.to);
                if layout.as_ref().is_some_and(|previous| previous != &order) {
                    return Err(stale("revision has ambiguous planned physical layouts"));
                }
                layout = Some(order);
            }
        }
        if let Some(order) = layout {
            apply_order(&mut snapshot, &order)?;
        }
        Ok(snapshot)
    }
    /// Publish immutable plan and re-rendered SQL companions after complete history checks.
    pub fn write_plan(&self, plan: &MigrationPlan) -> DataResult<PlanHash> {
        // One cooperative writer owns parent validation through publication; readers never repair.
        let _lock = self.lock()?;
        let history = self.check()?;
        if !history.iter().any(|s| s.hash == plan.to.hash)
            || (!plan.from.tables.is_empty() && !history.iter().any(|s| s.hash == plan.from.hash))
        {
            return Err(stale("plan snapshots are not recorded history"));
        }
        if matches!(plan.recipe.kind, PlanKind::Schema)
            && !plan.recipe.reverse
            && !plan.from.tables.is_empty()
            && plan.to.parent() != Some(plan.from.revision())
        {
            return Err(stale("plan does not follow the explicit history parent"));
        }
        let files = plan_files(plan)?;
        let directory = self.root.join("migrations").join(plan.hash.as_str());
        if directory.exists() {
            let old = self.read_plan_dir(&directory)?;
            if old.hash == plan.hash {
                return Ok(plan.hash.clone());
            }
            return Err(stale("immutable migration already exists"));
        }
        self.publish("migrations", plan.hash.as_str(), &files)?;
        if let Some(reverse) = plan.reverse() {
            self.publish("migrations", reverse.hash.as_str(), &plan_files(reverse)?)?;
        }
        Ok(plan.hash.clone())
    }
    /// Load checked structured authority and verify every companion before DB acquisition.
    pub fn plan(&self, hash: &PlanHash) -> DataResult<MigrationPlan> {
        // Keep path validation local even if another constructor is added in a future codec.
        PlanHash::parse(hash.as_str())?;
        self.check()?;
        self.read_plan_dir(&self.root.join("migrations").join(hash.as_str()))
    }
    /// Decode one bounded plan directory and compare every deterministic artifact byte.
    fn read_plan_dir(&self, path: &Path) -> DataResult<MigrationPlan> {
        let plan = MigrationPlan::from_artifact(&read(&path.join("plan.json"))?)?;
        let files = plan_files(&plan)?;
        complete_files(
            path,
            &files.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
        )?;
        if path.file_name().and_then(|s| s.to_str()) != Some(plan.hash.as_str()) {
            return Err(stale("migration directory hash disagrees"));
        }
        for (name, expected) in files {
            if read(&path.join(name))? != expected {
                return Err(stale("migration companion was modified"));
            }
        }
        Ok(plan)
    }
    /// Create the cooperative writer lock only after refusing symlinked path components.
    fn lock(&self) -> DataResult<WriterLock> {
        safe_path(&self.root)?;
        fs::create_dir_all(&self.root).map_err(io_error)?;
        let path = self.root.join(".writer.lock");
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(io_error)?;
        Ok(WriterLock { path, _file: file })
    }
    /// Stage create-new files and publish a complete directory without deleting recovery state.
    fn publish(&self, category: &str, name: &str, files: &[(String, String)]) -> DataResult<()> {
        if category == "migrations" {
            PlanHash::parse(name)?;
        } else {
            identity(name)?;
        }
        // A failed write retains this staging directory so the next check cannot mistake it for success.
        let stage = self.root.join(format!(".pending-{name}"));
        fs::create_dir(&stage).map_err(io_error)?;
        for (name, content) in files {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(stage.join(name))
                .map_err(io_error)?;
            file.write_all(content.as_bytes()).map_err(io_error)?;
            file.sync_all().map_err(io_error)?;
        }
        File::open(&stage)
            .and_then(|f| f.sync_all())
            .map_err(io_error)?;
        let parent = self.root.join(category);
        fs::create_dir_all(&parent).map_err(io_error)?;
        let destination = parent.join(name);
        if destination.exists() {
            return Err(stale("immutable history destination exists"));
        }
        // All create-new files are complete before this cooperative publication boundary.
        fs::rename(&stage, &destination).map_err(io_error)?;
        File::open(&parent)
            .and_then(|f| f.sync_all())
            .map_err(io_error)?;
        Ok(())
    }
}
/// Construct deterministic exports; SQL contains bindings as comments for inspection only.
fn plan_files(plan: &MigrationPlan) -> DataResult<Vec<(String, String)>> {
    let mut files = vec![
        ("plan.json".into(), plan.artifact()?),
        (format!("{}-up.sql", plan.recipe.dialect), sql_export(plan)?),
    ];
    if let Some(reverse) = plan.reverse() {
        files.push((
            format!("{}-down.sql", plan.recipe.dialect),
            sql_export(reverse)?,
        ));
    }
    Ok(files)
}
/// Render each checked statement and detached values without pretending placeholders are authority.
fn sql_export(plan: &MigrationPlan) -> DataResult<String> {
    let mut text = String::from("-- Inspection only. Execute the checked plan.json recipe.\n");
    for query in plan.queries() {
        text.push_str(query.sql());
        text.push_str(";\n-- binds: ");
        text.push_str(
            &serde_json::to_string(
                &query
                    .binds()
                    .iter()
                    .map(Literal::from_value)
                    .collect::<Vec<_>>(),
            )
            .map_err(crate::codec_error)?,
        );
        text.push('\n');
    }
    Ok(text)
}
/// Reject symlink components before opening managed paths; cooperative external writers are required.
fn safe_path(path: &Path) -> DataResult<()> {
    let absolute = std::path::absolute(path).map_err(io_error)?;
    let mut walked = PathBuf::new();
    for component in absolute.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(stale("history path contains traversal"));
        }
        walked.push(component);
        match fs::symlink_metadata(&walked) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(stale("history path contains symlink"));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(io_error(e)),
        }
    }
    Ok(())
}
/// Read regular files only; opening a FIFO must not bypass the artifact byte bound.
fn read(path: &Path) -> DataResult<String> {
    safe_path(path)?;
    let meta = fs::symlink_metadata(path).map_err(io_error)?;
    if !meta.is_file() || meta.len() > 1024 * 1024 {
        return Err(stale("history artifact is not a bounded regular file"));
    }
    let mut bytes = String::new();
    File::open(path)
        .map_err(io_error)?
        .take(1024 * 1024 + 1)
        .read_to_string(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() > 1024 * 1024 {
        return Err(stale("history artifact byte budget exceeded"));
    }
    Ok(bytes)
}
/// Bound directory enumeration, never silently truncating proposed history.
fn entries(path: &Path) -> DataResult<Vec<fs::DirEntry>> {
    safe_path(path)?;
    let mut entries = Vec::new();
    for entry in fs::read_dir(path).map_err(io_error)? {
        entries.push(entry.map_err(io_error)?);
        if entries.len() > 256 {
            return Err(stale("history directory budget exceeded"));
        }
    }
    Ok(entries)
}
/// Require ordinary complete directories; retained pending state fails `check`.
fn directories(path: &Path) -> DataResult<Vec<PathBuf>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in entries(path)? {
        if !entry.file_type().map_err(io_error)?.is_dir() {
            return Err(stale("history entry is not an immutable directory"));
        }
        paths.push(entry.path());
    }
    Ok(paths)
}
/// Reject missing or extra files, including stale SQL reverse companions.
fn complete_files(path: &Path, names: &[&str]) -> DataResult<()> {
    let actual = entries(path)?
        .iter()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect::<BTreeSet<_>>();
    if actual != names.iter().map(|s| s.to_string()).collect() {
        return Err(stale(
            "history publication is incomplete or contains unknown files",
        ));
    }
    Ok(())
}
/// Preserve the concrete filesystem cause without placing its path/Display in public context.
fn io_error(cause: std::io::Error) -> DataError {
    DataError::new("revision history filesystem operation failed")
        .with_kind(DataErrorKind::Stale)
        .with_cause(cause)
}
