//! Explicit immutable history, checked migration plans and opt-in SQLite maintenance.
//!
//! Source generation never records revisions or opens a database. Plans retain typed
//! storage metadata and exact approvals; companion SQL is inspection material only.
//! PostgreSQL/MySQL are render-only. SQLite AUTOINCREMENT sequences are preserved;
//! plain INTEGER PRIMARY KEY allocation may reuse deleted IDs by SQLite design.
//!
//! A plan is inspectable without opening a database. A rename needs an explicit mapping;
//! adding a nullable column permits a reverse only while that column remains NULL:
//!
//! ```
//! use stackpress_framework_sql::migrations::{
//!     DialectId, MigrationPlanner, RenameMap, RevisionSnapshot,
//! };
//! use stackpress_sql::{Create, Field, SqlType};
//! let original = Create::new("note")
//!     .add_field(Field::new("id", SqlType::Text).not_null())
//!     .add_field(Field::new("title", SqlType::Text).not_null())
//!     .primary_key(["id"])
//!     .definition()?;
//! let mut changed = original.clone();
//! changed.fields[1].name = "heading".into();
//! changed.fields.push(Field::new("summary", SqlType::Text));
//! let before = RevisionSnapshot::new("notes", "001", vec![original])?;
//! let after = RevisionSnapshot::new("notes", "002", vec![changed])?;
//! let candidates = MigrationPlanner::suggestions(&before, &after)?;
//! assert_eq!(candidates[0].to, "heading");
//! let mapping = RenameMap::new().field("note", "title", "heading")?;
//! let forward = MigrationPlanner::between(&before, &after, mapping, DialectId::Sqlite)?;
//! let reverse = forward.reverse().expect("rename and nullable addition are reversible");
//! assert_eq!(reverse.discardable_columns(), &[("note".into(), "summary".into())]);
//! assert_ne!(forward.hash(), reverse.hash());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{DataError, DataErrorKind, DataResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub use stackpress_sql::DialectId;
use stackpress_sql::{Query, QueryLimits, TableDefinition};
#[cfg(feature = "sqlite-maintenance")]
pub use stackpress_sql_sqlite::{SqliteOptions as DatabaseOptions, maintenance::SqliteMaintenance};
use std::collections::{BTreeMap, BTreeSet};

mod history;
mod planner;
mod steps;
mod storage;
pub use history::{RevisionOutcome, RevisionStore};
pub use planner::MigrationPlanner;
pub use storage::Literal;
use storage::TableDto;
#[cfg(feature = "sqlite-maintenance")]
mod sqlite;
#[cfg(feature = "sqlite-maintenance")]
pub use sqlite::{DatabaseStatus, LossApproval, MigrationRunner};

/// Explicit immutable revision identity; time and environment never determine its value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RevisionId(String);
impl RevisionId {
    /// Validate one portable revision directory name.
    pub fn parse(value: &str) -> DataResult<Self> {
        identity(value)?;
        Ok(Self(value.into()))
    }
    /// Borrow its exact explicit spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Immutable checked storage snapshot, independent of browser contracts.
#[derive(Clone, Debug)]
pub struct RevisionSnapshot {
    /// Logical app namespace, shared by all artifacts and ledger entries.
    app: String,
    /// Explicit history identity, independent of generation time.
    revision: String,
    /// Checked physical definitions used by this plan; logical bodies remain immutable.
    tables: Vec<TableDefinition>,
    /// Private logical snapshot envelope used for the revision hash.
    body: SnapshotBody,
    /// Canonical immutable revision identity, independent of a plan layout.
    hash: String,
}

/// Versioned framework DTO envelope, with no credentials or raw plugin config.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotBody {
    /// Persisted DTO codec; unknown versions refuse before execution.
    format: u32,
    /// Logical namespace, never connection configuration.
    app: String,
    /// Explicit revision directory identity.
    revision: String,
    /// Proposed history parent, distinct from live applied state.
    parent: Option<String>,
    /// Pinned normalized semantics without raw plugin/environment data.
    source_fingerprint: String,
    /// Pinned interpretation of generated storage definitions.
    generator_revision: u32,
    /// App-owned selected definition/provider revision.
    definition_revision: String,
    /// Complete portable storage semantics; no arbitrary SQL is decoded.
    tables: Vec<TableDto>,
}

impl RevisionSnapshot {
    /// Freeze normalized storage metadata without IO or arbitrary SQL expressions.
    ///
    /// # Errors
    /// Refuses invalid/reserved/case-colliding identities, metadata and aggregate budgets.
    pub fn new(app: &str, revision: &str, tables: Vec<TableDefinition>) -> DataResult<Self> {
        identity(app)?;
        identity(revision)?;
        if tables.len() > 128 {
            return Err(DataError::new("snapshot table budget exceeded"));
        }
        snapshot_budget(&tables)?;
        let mut tables = tables
            .into_iter()
            .map(|t| t.normalized().map_err(Into::into))
            .collect::<DataResult<Vec<_>>>()?;
        tables.sort_by(|a, b| a.name.cmp(&b.name));
        let mut names = BTreeSet::new();
        for table in &tables {
            identity(&table.name)?;
            if table.name.to_ascii_lowercase().starts_with("_stackpress_")
                || table.name.to_ascii_lowercase().starts_with("sqlite_")
                || !names.insert(table.name.to_ascii_lowercase())
            {
                return Err(DataError::new("reserved or duplicate storage name"));
            }
            if table.fields.len() > 128 {
                return Err(DataError::new("snapshot column budget exceeded"));
            }
        }
        for table in &tables {
            for field in &table.fields {
                if let Some(stackpress_sql::DefaultValue::Value(value)) = &field.default {
                    steps::literal_for(field, &Literal::from_value(value))?;
                }
            }
        }
        let dto = tables
            .iter()
            .map(TableDto::capture)
            .collect::<DataResult<Vec<_>>>()?;
        let source_fingerprint = digest(&dto)?;
        let body = SnapshotBody {
            format: 2,
            app: app.into(),
            revision: revision.into(),
            parent: None,
            source_fingerprint,
            generator_revision: 1,
            definition_revision: "storage-1".into(),
            tables: dto,
        };
        Self::from_body(body)
    }
    /// Pin a semantic source and selected app definition revision, never environment config.
    pub fn with_provenance(mut self, source: &str, definitions: &str) -> DataResult<Self> {
        if source.len() != 64 || !source.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(DataError::new("invalid source fingerprint"));
        }
        identity(definitions)?;
        self.body.source_fingerprint = source.into();
        self.body.definition_revision = definitions.into();
        self.hash = digest(&self.body)?;
        Ok(self)
    }
    /// Inspect the complete private artifact identity.
    pub fn hash(&self) -> &str {
        &self.hash
    }
    /// Inspect the explicit revision name.
    pub fn revision(&self) -> &str {
        &self.revision
    }
    /// Inspect the declared parent, distinct from the live database head.
    pub fn parent(&self) -> Option<&str> {
        self.body.parent.as_deref()
    }
    /// Borrow checked physical storage definitions.
    pub fn tables(&self) -> &[TableDefinition] {
        &self.tables
    }
    /// Rebuild a checked snapshot from bounded decoded framework DTOs.
    fn from_body(body: SnapshotBody) -> DataResult<Self> {
        if body.format != 2 || body.generator_revision != 1 {
            return Err(unsupported("unknown revision format or generator"));
        }
        identity(&body.app)?;
        identity(&body.revision)?;
        identity(&body.definition_revision)?;
        if let Some(parent) = &body.parent {
            identity(parent)?;
            if parent == &body.revision {
                return Err(DataError::new("revision is its own parent"));
            }
        }
        if body.tables.len() > 128 {
            return Err(DataError::new("snapshot table budget exceeded"));
        }
        let tables = body
            .tables
            .iter()
            .map(TableDto::checked)
            .collect::<DataResult<Vec<_>>>()?;
        let mut names = BTreeSet::new();
        for table in &tables {
            identity(&table.name)?;
            if table.fields.len() > 128
                || table.name.to_ascii_lowercase().starts_with("_stackpress_")
                || table.name.to_ascii_lowercase().starts_with("sqlite_")
                || !names.insert(table.name.to_ascii_lowercase())
            {
                return Err(DataError::new(
                    "reserved, duplicate or oversized revision storage",
                ));
            }
        }
        if body.source_fingerprint.len() != 64
            || !body
                .source_fingerprint
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err(DataError::new("invalid revision source fingerprint"));
        }
        for table in &tables {
            for field in &table.fields {
                if let Some(stackpress_sql::DefaultValue::Value(value)) = &field.default {
                    steps::literal_for(field, &Literal::from_value(value))?;
                }
            }
        }
        let hash = digest(&body)?;
        Ok(Self {
            app: body.app.clone(),
            revision: body.revision.clone(),
            tables,
            body,
            hash,
        })
    }
    /// Keep retained physical slots while appending new columns; UI order alone cannot reorder SQL.
    fn physical_target(&self, source: &Self, renames: &RenameMap) -> DataResult<Self> {
        let mut tables = self.tables.clone();
        for target in &mut tables {
            let old_name = renames
                .tables
                .iter()
                .find(|r| r.to == target.name)
                .map_or(target.name.as_str(), |r| r.from.as_str());
            if let Some(old) = source.tables.iter().find(|t| t.name == old_name) {
                let mut fields = Vec::new();
                for field in &old.fields {
                    let name = renames
                        .fields
                        .iter()
                        .find(|r| r.table == old.name && r.from == field.name)
                        .map_or(field.name.as_str(), |r| r.to.as_str());
                    if let Some(field) = target.fields.iter().find(|f| f.name == name) {
                        fields.push(field.clone());
                    }
                }
                fields.extend(
                    target
                        .fields
                        .iter()
                        .filter(|f| !fields.iter().any(|v| v.name == f.name))
                        .cloned()
                        .collect::<Vec<_>>(),
                );
                target.fields = fields;
            }
        }
        let mut result = Self::new(&self.app, &self.revision, tables)?;
        // Physical planning must not rewrite or rehash the immutable logical revision artifact.
        result.body = self.body.clone();
        result.hash = self.hash.clone();
        Ok(result)
    }
}

/// An explicit one-to-one field rename, never an inferred match.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldRename {
    /// Managed source table.
    pub table: String,
    /// Existing stored column.
    pub from: String,
    /// Intended target column.
    pub to: String,
}
/// An explicit logical table identity mapping.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TableRename {
    from: String,
    to: String,
}
/// A checked literal backfill selected explicitly by the application tool.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Backfill {
    table: String,
    field: String,
    value: Literal,
}
/// All explicit rename, deliberate removal and literal backfill authority.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenameMap {
    /// Selected one-to-one field identities.
    fields: Vec<FieldRename>,
    /// Selected one-to-one table identities.
    tables: Vec<TableRename>,
    /// Explicit deliberate removals that resolve suggestions without granting loss approval.
    drops: Vec<(String, String)>,
    /// Checked literal replacements for additions or NULL-to-required transitions.
    backfills: Vec<Backfill>,
}
impl RenameMap {
    /// Start without rename or loss authority.
    pub fn new() -> Self {
        Self::default()
    }
    /// Select one exact field pair, refusing duplicate endpoints.
    pub fn field(mut self, table: &str, from: &str, to: &str) -> DataResult<Self> {
        identity(table)?;
        identity(from)?;
        identity(to)?;
        if from == to
            || self.fields.len() >= 128
            || self
                .fields
                .iter()
                .any(|r| r.table == table && (r.from == from || r.to == to))
        {
            return Err(DataError::new("rename mapping is not one-to-one"));
        }
        self.fields.push(FieldRename {
            table: table.into(),
            from: from.into(),
            to: to.into(),
        });
        Ok(self)
    }
    /// Select a table identity; omitted mappings remain destructive drop/create changes.
    pub fn table(mut self, from: &str, to: &str) -> DataResult<Self> {
        identity(from)?;
        identity(to)?;
        if from == to
            || self.tables.len() >= 128
            || self.tables.iter().any(|r| r.from == from || r.to == to)
        {
            return Err(DataError::new("table rename mapping is not one-to-one"));
        }
        self.tables.push(TableRename {
            from: from.into(),
            to: to.into(),
        });
        Ok(self)
    }
    /// Resolve a suggested match as a deliberate removal, still requiring exact loss approval.
    pub fn drop_field(mut self, table: &str, field: &str) -> DataResult<Self> {
        identity(table)?;
        identity(field)?;
        if self.drops.len() >= 128 || self.drops.contains(&(table.into(), field.into())) {
            return Err(DataError::new("duplicate deliberate field removal"));
        }
        self.drops.push((table.into(), field.into()));
        Ok(self)
    }
    /// Select a bounded literal backfill; arbitrary expressions/conversion callbacks are excluded.
    pub fn backfill(mut self, table: &str, field: &str, value: Literal) -> DataResult<Self> {
        identity(table)?;
        identity(field)?;
        if self
            .backfills
            .iter()
            .map(|b| b.value.bytes())
            .sum::<usize>()
            .saturating_add(value.bytes())
            > 1024 * 1024
        {
            return Err(DataError::new("backfill aggregate byte budget exceeded"));
        }
        value.value()?;
        if self.backfills.len() >= 128
            || self
                .backfills
                .iter()
                .any(|v| v.table == table && v.field == field)
        {
            return Err(DataError::new("duplicate backfill"));
        }
        self.backfills.push(Backfill {
            table: table.into(),
            field: field.into(),
            value,
        });
        Ok(self)
    }
}

/// Reviewable same-shaped candidate; it carries no execution authority.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenameSuggestion {
    /// Source table.
    pub table: String,
    /// Removed field.
    pub from: String,
    /// Potential added field.
    pub to: String,
}
/// One aggregate loss or allocation finding, included in the plan hash.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskFinding {
    /// Stable machine-readable category.
    pub kind: String,
    /// Managed table, never a database URL.
    pub table: String,
    /// Exact field when applicable.
    pub field: Option<String>,
    /// Whether exact target-bound data-loss authorization is required.
    pub destructive: bool,
}
/// An explicit data-preserving reverse or a refusal with a recovery path.
#[derive(Clone, Debug)]
pub enum ReversePlan {
    /// Checked statements plus runtime data preconditions.
    Available(Box<MigrationPlan>),
    /// Schema recreation cannot recover discarded information; restore a backup/custom plan.
    Refused(String),
}
/// Reviewed hash, independent of generic CLI force flags.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlanHash(String);
impl PlanHash {
    /// Borrow the canonical approval spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Decode one lower-case SHA-256 hash without granting data-loss authority.
    pub fn parse(value: &str) -> DataResult<Self> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(DataError::new("invalid plan approval hash"));
        }
        Ok(Self(value.into()))
    }
}
/// A database-only typed seed row, with explicit create or primary-key upsert semantics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedRow {
    /// Managed table.
    pub table: String,
    /// Explicit checked column/literal pairs.
    pub values: BTreeMap<String, Literal>,
    /// True selects an atomic primary-key-targeted upsert; false is create-only.
    pub upsert: bool,
}
/// Durable operation recipe; checked conversions reconstruct every executable statement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum PlanKind {
    Schema,
    Purge,
    Uninstall,
    Populate { seed: String, rows: Vec<SeedRow> },
}
/// Versioned plan source, never arbitrary statement text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanRecipe {
    /// Versioned structured authority format.
    format: u32,
    /// Explicit backend capability; no environment-dependent choice.
    dialect: String,
    /// Complete immutable predecessor body.
    from: SnapshotBody,
    /// Complete intended logical target body.
    to: SnapshotBody,
    /// Explicit one-to-one identities, loss resolutions and literal backfills.
    renames: RenameMap,
    /// Selected operator capability, including bounded typed seed data.
    kind: PlanKind,
    /// Marks a separately reviewed reverse rather than inferring direction at runtime.
    reverse: bool,
    /// Exact source physical slots, hashed independently of logical/UI order.
    source_order: BTreeMap<String, Vec<String>>,
}
/// One pre-rendered maintenance phase; reconstruction boundaries retain row/sequence checks.
#[derive(Clone, Debug)]
pub enum MigrationStep {
    /// Ordinary checked native statements.
    Native(Vec<Query>),
    /// Replacement table with explicit copy and post-swap checks.
    Rebuild {
        /// Existing source table.
        source: String,
        /// Final destination identity.
        target: String,
        /// Ordered create/copy/drop/rename/index statements.
        queries: Vec<Query>,
        /// Both tables retain AUTOINCREMENT, so capture/restore historical allocation.
        preserve_sequence: bool,
    },
}
/// Complete immutable plan with typed execution and canonical review artifacts.
#[derive(Clone, Debug)]
pub struct MigrationPlan {
    /// Checked predecessor and its explicit physical layout.
    from: RevisionSnapshot,
    /// Checked target with retained/appended physical slots.
    to: RevisionSnapshot,
    /// Complete ordered detached statements, also exposed for inspection.
    queries: Vec<Query>,
    /// Native/reconstruction boundaries for runtime integrity checks.
    steps: Vec<MigrationStep>,
    /// Columns that may be reversed only when every value remains NULL.
    discardable: Vec<(String, String)>,
    /// A checked reverse or an explicit recovery refusal.
    reverse: ReversePlan,
    /// Exact recipe/statements/binds/findings/reverse approval identity.
    hash: PlanHash,
    /// Structured authority used to re-render artifacts on load.
    recipe: PlanRecipe,
    /// All destructive and allocation-policy findings for review.
    findings: Vec<RiskFinding>,
    /// Capabilities/conversions that loss approval cannot waive.
    blocked: Vec<String>,
}
impl MigrationPlan {
    /// Inspect the approved predecessor.
    pub fn source_snapshot(&self) -> &RevisionSnapshot {
        &self.from
    }
    /// Inspect the intended target.
    pub fn target_snapshot(&self) -> &RevisionSnapshot {
        &self.to
    }
    /// Inspect exact NULL-only reverse-drop preconditions.
    pub fn discardable_columns(&self) -> &[(String, String)] {
        &self.discardable
    }
    /// Borrow the exact plan identity.
    pub fn hash(&self) -> &PlanHash {
        &self.hash
    }
    /// Inspect all native statements and binds in effect order.
    pub fn queries(&self) -> &[Query] {
        &self.queries
    }
    /// Inspect reconstruction/native phase ownership.
    pub fn steps(&self) -> &[MigrationStep] {
        &self.steps
    }
    /// Borrow an available reverse; use reverse_plan to inspect refusal reasons.
    pub fn reverse(&self) -> Option<&MigrationPlan> {
        match &self.reverse {
            ReversePlan::Available(p) => Some(p),
            ReversePlan::Refused(_) => None,
        }
    }
    /// Inspect explicit recoverability instead of assuming every schema change can be undone.
    pub fn reverse_plan(&self) -> &ReversePlan {
        &self.reverse
    }
    /// Inspect every aggregated loss/allocation finding before approval.
    pub fn findings(&self) -> &[RiskFinding] {
        &self.findings
    }
    /// Inspect capabilities/conversions that data-loss approval cannot waive.
    pub fn blocked_reasons(&self) -> &[String] {
        &self.blocked
    }
    /// Whether this plan is an explicit checked reverse recipe.
    pub fn is_reverse(&self) -> bool {
        self.recipe.reverse
    }
    /// Inspect the selected operator capability without decoding private artifact JSON.
    pub fn operation(&self) -> PlanOperation {
        match self.recipe.kind {
            PlanKind::Schema => PlanOperation::Schema,
            PlanKind::Purge => PlanOperation::Purge,
            PlanKind::Uninstall => PlanOperation::Uninstall,
            PlanKind::Populate { .. } => PlanOperation::Populate,
        }
    }
    /// Inspect the pinned seed identity only for a population plan.
    pub fn seed_id(&self) -> Option<&str> {
        match &self.recipe.kind {
            PlanKind::Populate { seed, .. } => Some(seed),
            _ => None,
        }
    }
    /// Inspect the backend without opening a database.
    pub fn dialect(&self) -> DialectId {
        dialect(&self.recipe.dialect).expect("planner validated dialect")
    }
    /// Serialize checked execution authority with a hash over re-rendered operations and binds.
    pub fn artifact(&self) -> DataResult<String> {
        json(&PlanArtifact {
            hash: self.hash.0.clone(),
            recipe: self.recipe.clone(),
        })
    }
    /// Reconstruct through the planner and verify the exact canonical hash.
    pub fn from_artifact(value: &str) -> DataResult<Self> {
        let artifact: PlanArtifact = decode(value)?;
        let plan = planner::from_recipe(artifact.recipe)?;
        if plan.hash.0 != artifact.hash {
            return Err(stale("migration artifact hash mismatch"));
        }
        Ok(plan)
    }
}
/// Durable plan envelope; its recipe is the only executable authority.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanArtifact {
    hash: String,
    recipe: PlanRecipe,
}
/// Acknowledged database application versus exact-ledger reconciliation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// Effects and ledger committed together.
    Applied,
    /// The exact operation was already acknowledged in the authoritative ledger.
    AlreadyApplied,
}
/// One authoritative ledger entry, independent of proposed history directories.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppliedRevision {
    /// Monotonic database application sequence, including downgrade/operator events.
    pub sequence: i64,
    /// Logical app namespace.
    pub app: String,
    /// Revision at this operation's target.
    pub revision: String,
    /// Prior applied revision.
    pub parent: String,
    /// Complete target snapshot hash.
    pub revision_hash: String,
    /// Exact rendered plan identity.
    pub plan_hash: String,
    /// Backend capability identity.
    pub dialect: String,
    /// Applied or uninstalled.
    pub state: String,
    /// Database execution time, excluded from deterministic artifact bytes.
    pub executed_at: String,
}
/// Check the portable identity vocabulary without echoing private inputs.
fn identity(value: &str) -> DataResult<()> {
    if value.is_empty()
        || value.len() > 63
        || value.starts_with('.')
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(DataError::new("invalid migration identity"));
    }
    Ok(())
}
/// Encode bounded deterministic DTOs; BTree maps retain canonical key order.
fn json(value: &impl Serialize) -> DataResult<String> {
    let bytes = serde_json::to_string_pretty(value).map_err(crate::codec_error)?;
    if bytes.len() > 1024 * 1024 {
        return Err(DataError::new("migration artifact byte budget exceeded"));
    }
    Ok(bytes)
}
/// Decode a bounded envelope before allocating nested artifact structures.
fn decode<T: serde::de::DeserializeOwned>(value: &str) -> DataResult<T> {
    if value.len() > 1024 * 1024 {
        return Err(DataError::new("migration artifact byte budget exceeded"));
    }
    serde_json::from_str(value).map_err(crate::codec_error)
}
/// Hash the full canonical body, excluding clock/environment values.
fn digest(value: &impl Serialize) -> DataResult<String> {
    Ok(format!("{:x}", Sha256::digest(json(value)?.as_bytes())))
}
/// Refuse unsupported capabilities distinctly from schema drift.
fn unsupported(context: &'static str) -> DataError {
    DataError::new(context).with_kind(DataErrorKind::Unsupported)
}
/// Refuse stale artifacts/ledger without automatic repair or replay.
fn stale(context: &'static str) -> DataError {
    DataError::new(context).with_kind(DataErrorKind::Stale)
}
/// Strict persisted backend vocabulary.
fn dialect(value: &str) -> DataResult<DialectId> {
    match value {
        "sqlite" => Ok(DialectId::Sqlite),
        "postgres" => Ok(DialectId::Postgres),
        "mysql" => Ok(DialectId::Mysql),
        _ => Err(unsupported("unknown migration dialect")),
    }
}
/// Stable backend spelling for deterministic plans.
fn dialect_name(value: DialectId) -> &'static str {
    match value {
        DialectId::Sqlite => "sqlite",
        DialectId::Postgres => "postgres",
        DialectId::Mysql => "mysql",
    }
}

/// Retain explicit physical column slots independently of logical/UI snapshot order.
fn storage_order(snapshot: &RevisionSnapshot) -> BTreeMap<String, Vec<String>> {
    snapshot
        .tables
        .iter()
        .map(|t| {
            (
                t.name.clone(),
                t.fields.iter().map(|f| f.name.clone()).collect(),
            )
        })
        .collect()
}
/// Reapply only a checked permutation, never allowing plan metadata to alter a field definition.
fn apply_order(
    snapshot: &mut RevisionSnapshot,
    order: &BTreeMap<String, Vec<String>>,
) -> DataResult<()> {
    if order.len() != snapshot.tables.len() {
        return Err(DataError::new("plan storage-order table set differs"));
    }
    for table in &mut snapshot.tables {
        let names = order
            .get(&table.name)
            .ok_or_else(|| DataError::new("plan storage-order table missing"))?;
        if names.len() != table.fields.len()
            || names.iter().collect::<BTreeSet<_>>().len() != names.len()
        {
            return Err(DataError::new("invalid plan storage-order mapping"));
        }
        table.fields = names
            .iter()
            .map(|name| {
                table
                    .fields
                    .iter()
                    .find(|f| f.name == *name)
                    .cloned()
                    .ok_or_else(|| DataError::new("plan storage-order column missing"))
            })
            .collect::<DataResult<_>>()?;
    }
    Ok(())
}

/// Bound borrowed metadata before core normalization or DTO capture clones any large values.
fn snapshot_budget(tables: &[TableDefinition]) -> DataResult<()> {
    let mut bytes = 0_usize;
    for table in tables {
        if table.fields.len() > 128 {
            return Err(DataError::new("snapshot column budget exceeded"));
        }
        bytes = bytes.saturating_add(table.name.len());
        for field in &table.fields {
            bytes = bytes
                .saturating_add(field.name.len() + field.comment.as_ref().map_or(0, String::len));
            if let Some(stackpress_sql::DefaultValue::Value(value)) = &field.default {
                bytes = bytes.saturating_add(match value {
                    stackpress_sql::SqlValue::Text(v) => v.len(),
                    stackpress_sql::SqlValue::Blob(v) => v.len(),
                    _ => 16,
                });
            }
        }
        for index in &table.indexes {
            bytes = bytes.saturating_add(
                index.name.len() + index.columns.iter().map(String::len).sum::<usize>(),
            );
        }
        for key in &table.foreign_keys {
            bytes = bytes.saturating_add(
                key.name.len()
                    + key.table.len()
                    + key
                        .columns
                        .iter()
                        .chain(&key.foreign_columns)
                        .map(String::len)
                        .sum::<usize>(),
            );
        }
        if bytes > 1024 * 1024 {
            return Err(DataError::new("snapshot aggregate byte budget exceeded"));
        }
    }
    Ok(())
}

/// Explicit operator purpose; the app tool cannot reinterpret one approved plan as another command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanOperation {
    /// Baseline, upgrade or checked reverse schema change.
    Schema,
    /// Enumerated managed-data deletion retaining schema/head.
    Purge,
    /// Enumerated owned-object removal retaining ledger history.
    Uninstall,
    /// Named bounded database-only seed steps.
    Populate,
}
