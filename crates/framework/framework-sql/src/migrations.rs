//! Immutable, typed M0 migration plans and explicit SQLite ledger execution.
//!
//! M0 supports baseline install, explicit safe field rename and nullable native additions.
//! Reverse of an addition checks that its values are all NULL. Durable history publication,
//! general reconstruction planning and destructive plans belong to later milestones.
//! The runner never executes an edited SQL file or infers an applied revision from a directory.

use std::collections::BTreeSet;

use serde::Serialize;
use sha2::{Digest, Sha256};
use stackpress_sql::{DialectId, Query, QueryLimits, TableDefinition};

use crate::{DataError, DataErrorKind, DataResult};

/// Immutable checked storage snapshot, built from actual SQL-core definitions.
#[derive(Clone, Debug)]
pub struct RevisionSnapshot {
    /// App identity, never a connection URL or path.
    app: String,
    /// Explicit revision name, independent of the wall clock.
    revision: String,
    /// Normalized private typed definitions; text exports never become execution authority.
    tables: Vec<TableDefinition>,
    /// Canonical framework-owned inspection representation.
    body: SnapshotBody,
    /// Hash of the complete app/revision/storage description.
    hash: String,
}

/// Framework-owned deterministic snapshot envelope used for hashing and inspection.
#[derive(Clone, Debug, Serialize)]
struct SnapshotBody {
    /// Codec identity rejects future interpretation drift.
    format: u32,
    /// Logical application namespace.
    app: String,
    /// Explicit history identity.
    revision: String,
    /// Core-rendered canonical table declarations; never parsed back into executable SQL.
    tables: Vec<TableSnapshot>,
}

/// Canonical display-only storage declaration, always derived from checked typed definitions.
#[derive(Clone, Debug, Serialize)]
struct TableSnapshot {
    /// Checked table identity.
    name: String,
    /// Ordered inspection SQL; the runner uses retained Query values instead.
    declarations: Vec<String>,
}

impl RevisionSnapshot {
    /// Freeze normalized tables and their deterministic inspection bytes, without any IO.
    ///
    /// # Errors
    /// Refuses unsafe/reserved names, unsupported identity/default/type semantics and budgets.
    pub fn new(app: &str, revision: &str, tables: Vec<TableDefinition>) -> DataResult<Self> {
        identity(app)?;
        identity(revision)?;
        if tables.len() > 128 {
            return Err(DataError::new("snapshot table budget exceeded"));
        }
        let mut normalized = Vec::with_capacity(tables.len());
        let mut names = BTreeSet::new();
        let mut snapshot = Vec::with_capacity(tables.len());
        for table in tables {
            let table = table.normalized()?;
            identity(&table.name)?;
            if table.name.starts_with("_stackpress_") || !names.insert(table.name.clone()) {
                return Err(DataError::new("reserved or duplicate storage name"));
            }
            // Automatic identity needs separate sequence preservation proofs. M0 refuses it.
            if table.fields.iter().any(|field| field.auto_increment) {
                return Err(unsupported(
                    "M0 automatic identity migrations are unavailable",
                ));
            }
            let plan = table.create().render(DialectId::Sqlite)?;
            snapshot.push(TableSnapshot {
                name: table.name.clone(),
                declarations: plan
                    .queries()
                    .iter()
                    .map(|query| query.sql().into())
                    .collect(),
            });
            normalized.push(table);
        }
        let body = SnapshotBody {
            format: 1,
            app: app.into(),
            revision: revision.into(),
            tables: snapshot,
        };
        let hash = digest(&body)?;
        Ok(Self {
            app: app.into(),
            revision: revision.into(),
            tables: normalized,
            body,
            hash,
        })
    }

    /// Inspect the canonical app/revision/storage hash.
    pub fn hash(&self) -> &str {
        &self.hash
    }
    /// Inspect explicit revision identity without reading a filesystem or database.
    pub fn revision(&self) -> &str {
        &self.revision
    }
    /// Borrow checked typed tables for explicit installation or pure planning.
    pub fn tables(&self) -> &[TableDefinition] {
        &self.tables
    }
}

/// One explicit field rename. No shape-based inference authorizes a mapping.
#[derive(Clone, Debug, Serialize)]
pub struct FieldRename {
    /// Managed table identity.
    pub table: String,
    /// Exact old stored name.
    pub from: String,
    /// Exact new stored name.
    pub to: String,
}

/// Explicit one-to-one rename selection supplied during planning.
#[derive(Clone, Debug, Default, Serialize)]
pub struct RenameMap {
    /// Source-ordered mappings, validated as a whole before effects.
    fields: Vec<FieldRename>,
}

impl RenameMap {
    /// Begin with no rename authority; matching shapes alone never imply intent.
    pub fn new() -> Self {
        Self::default()
    }
    /// Add one explicit old/new mapping, refusing duplicate endpoints before mutation.
    pub fn field(mut self, table: &str, from: &str, to: &str) -> DataResult<Self> {
        identity(table)?;
        identity(from)?;
        identity(to)?;
        if from == to
            || self.fields.len() >= 128
            || self
                .fields
                .iter()
                .any(|item| item.table == table && (item.from == from || item.to == to))
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
}

/// Exact approval identity; generic force flags cannot construct approval for another plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanHash(String);

impl PlanHash {
    /// Inspect the reviewed canonical hash for an operator command or receipt.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Decode an operator-supplied SHA-256 approval without any implicit force semantics.
    pub fn parse(value: &str) -> DataResult<Self> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(DataError::new("invalid plan approval hash"));
        }
        Ok(Self(value.into()))
    }
}

/// Frozen pre-rendered plan, with all statements and reverse preconditions checked before IO.
#[derive(Clone, Debug)]
pub struct MigrationPlan {
    /// Source revision, empty only for explicit baseline installation.
    from: RevisionSnapshot,
    /// Intended applied state.
    to: RevisionSnapshot,
    /// Ordered typed component statements, detached from native resources.
    queries: Vec<Query>,
    /// Checked reverse requirements for dropping newly added columns.
    discardable: Vec<(String, String)>,
    /// Safe reverse plan, absent for baseline installation.
    reverse: Option<Box<MigrationPlan>>,
    /// Canonical exact plan authority.
    hash: PlanHash,
}

impl MigrationPlan {
    /// Inspect the approved predecessor without acquiring a database connection.
    pub fn source_snapshot(&self) -> &RevisionSnapshot {
        &self.from
    }
    /// Inspect the intended target before approving this plan's hash.
    pub fn target_snapshot(&self) -> &RevisionSnapshot {
        &self.to
    }
    /// Inspect reverse-drop columns whose values must all be NULL at application time.
    pub fn discardable_columns(&self) -> &[(String, String)] {
        &self.discardable
    }

    /// Borrow the immutable reviewed plan hash.
    pub fn hash(&self) -> &PlanHash {
        &self.hash
    }
    /// Inspect typed statements and detached binds without applying anything.
    pub fn queries(&self) -> &[Query] {
        &self.queries
    }
    /// Borrow the checked reverse; nullable-add reverse still has runtime data preconditions.
    pub fn reverse(&self) -> Option<&MigrationPlan> {
        self.reverse.as_deref()
    }
}

/// Pure migration planner; construction and rendering never open a database.
pub struct MigrationPlanner;

impl MigrationPlanner {
    /// Plan explicit installation into an empty managed namespace, with no automatic uninstall.
    pub fn baseline(target: &RevisionSnapshot, dialect: DialectId) -> DataResult<MigrationPlan> {
        sqlite_only(dialect)?;
        let source = RevisionSnapshot::new(&target.app, "empty", Vec::new())?;
        let mut queries = Vec::new();
        for table in &target.tables {
            queries.extend(table.create().render(dialect)?.queries().iter().cloned());
        }
        finish_plan(source, target.clone(), queries, Vec::new(), None)
    }

    /// Plan one explicit safe rename/addition subset and its checked reverse.
    ///
    /// # Errors
    /// Ambiguous remove/add, stale mappings, changed keys/types/constraints and required
    /// additions without a supported complete backfill are refused before database effects.
    pub fn between(
        from: &RevisionSnapshot,
        to: &RevisionSnapshot,
        renames: RenameMap,
        dialect: DialectId,
    ) -> DataResult<MigrationPlan> {
        sqlite_only(dialect)?;
        if from.app != to.app || from.tables.len() != to.tables.len() {
            return Err(unsupported("M0 requires the same managed tables"));
        }
        let mut queries = Vec::new();
        let mut reverse_queries = Vec::new();
        let mut discardable = Vec::new();
        let mut used = 0_usize;
        for source in &from.tables {
            let target = to
                .tables
                .iter()
                .find(|table| table.name == source.name)
                .ok_or_else(|| unsupported("M0 table rename/removal is unavailable"))?;
            let mappings: Vec<_> = renames
                .fields
                .iter()
                .filter(|mapping| mapping.table == source.name)
                .collect();
            let mut expected = source.clone();
            let mut diff = source.diff(target)?;
            for mapping in &mappings {
                // The SQL core's checked mapping retains its type/key/index/FK semantics.
                diff = diff.rename_field(&mapping.from, &mapping.to)?;
                let field = expected
                    .fields
                    .iter_mut()
                    .find(|field| field.name == mapping.from)
                    .ok_or_else(|| DataError::new("rename source does not exist"))?;
                field.name = mapping.to.clone();
                if expected.primary.contains(&mapping.from) {
                    return Err(unsupported("M0 key renames are unavailable"));
                }
                used += 1;
            }
            // Any removal without explicit authority remains a rejection, even for one candidate.
            if expected
                .fields
                .iter()
                .any(|field| !target.fields.iter().any(|other| other.name == field.name))
            {
                return Err(DataError::new(
                    "removed fields require explicit rename mapping",
                ));
            }
            let additions: Vec<_> = target
                .fields
                .iter()
                .filter(|field| !expected.fields.iter().any(|old| old.name == field.name))
                .collect();
            for field in &additions {
                if !field.nullable || field.default.is_some() {
                    return Err(DataError::new(
                        "M0 addition requires nullable field without backfill",
                    ));
                }
                expected.fields.push((*field).clone());
                discardable.push((source.name.clone(), field.name.clone()));
            }
            if expected != *target {
                return Err(unsupported("M0 changed storage semantics are unavailable"));
            }
            if let Some(plan) = diff.render(dialect)? {
                queries.extend(plan.queries().iter().cloned());
            }
            // Reverse native additions in reverse declaration order, then invert safe renames.
            let mut reverse = stackpress_sql::Alter::new(&target.name);
            // The core validates rename sources from explicit metadata, never live guesses.
            for field in &target.fields {
                reverse = reverse.source_field(field.clone());
            }
            for field in additions.iter().rev() {
                reverse = reverse.remove_field(&field.name);
            }
            for mapping in mappings.iter().rev() {
                reverse = reverse.rename_field(&mapping.to, &mapping.from);
            }
            if !additions.is_empty() || !mappings.is_empty() {
                reverse_queries.extend(reverse.render(dialect)?.queries().iter().cloned());
            }
        }
        if used != renames.fields.len() {
            return Err(DataError::new("rename table does not exist"));
        }
        let reverse = finish_plan(to.clone(), from.clone(), reverse_queries, discardable, None)?;
        finish_plan(
            from.clone(),
            to.clone(),
            queries,
            Vec::new(),
            Some(Box::new(reverse)),
        )
    }
}

/// Hash a complete immutable plan body and reject aggregate budgets before acquisition.
fn finish_plan(
    from: RevisionSnapshot,
    to: RevisionSnapshot,
    queries: Vec<Query>,
    discardable: Vec<(String, String)>,
    reverse: Option<Box<MigrationPlan>>,
) -> DataResult<MigrationPlan> {
    if queries.len() > 128
        || queries.iter().map(|query| query.sql().len()).sum::<usize>() > 1024 * 1024
    {
        return Err(DataError::new("migration plan budget exceeded"));
    }
    for query in &queries {
        query.validate(QueryLimits::default())?;
    }
    // M0 DDL is fully typed and literal defaults are renderer-escaped, so no query bind is omitted.
    if queries.iter().any(|query| !query.binds().is_empty()) {
        return Err(unsupported("M0 bound DDL artifacts are unavailable"));
    }
    let body = PlanBody {
        format: 1,
        dialect: "sqlite",
        from: &from.body,
        to: &to.body,
        queries: queries.iter().map(|query| query.sql()).collect(),
        discardable: &discardable,
        reverse: reverse.as_ref().map(|plan| plan.hash.as_str()),
    };
    let hash = PlanHash(digest(&body)?);
    Ok(MigrationPlan {
        from,
        to,
        queries,
        discardable,
        reverse,
        hash,
    })
}

/// A struct fixes canonical field order independently of serde_json's optional map features.
#[derive(Serialize)]
struct PlanBody<'a> {
    /// Plan codec revision.
    format: u32,
    /// Explicit executor capability identity.
    dialect: &'a str,
    /// Complete approved predecessor.
    from: &'a SnapshotBody,
    /// Complete intended target.
    to: &'a SnapshotBody,
    /// Core-rendered inspection statements, never edited SQL execution authority.
    queries: Vec<&'a str>,
    /// Data-preservation preconditions included in exact hash approval.
    discardable: &'a [(String, String)],
    /// Checked reverse identity, when supplied.
    reverse: Option<&'a str>,
}

/// Applied-state result; reconciliation never replays an identical committed plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// Schema and ledger committed once during this call.
    Applied,
    /// Authoritative state already matches this exact target and plan hash.
    AlreadyApplied,
}

/// SQLite operator runtime, available only with the explicit maintenance feature.
#[cfg(feature = "sqlite-maintenance")]
mod sqlite;
#[cfg(feature = "sqlite-maintenance")]
pub use sqlite::MigrationRunner;

/// Check the small explicit identity vocabulary without echoing private input.
fn identity(value: &str) -> DataResult<()> {
    if value.is_empty()
        || value.len() > 63
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(DataError::new("invalid migration identity"));
    }
    Ok(())
}
/// State M0's actual backend boundary before any effect.
fn sqlite_only(dialect: DialectId) -> DataResult<()> {
    if dialect != DialectId::Sqlite {
        return Err(unsupported("M0 migration execution is SQLite only"));
    }
    Ok(())
}
/// Hash a framework-owned canonical representation, without time or environment values.
fn digest(value: &impl Serialize) -> DataResult<String> {
    let bytes = serde_json::to_vec(value).map_err(crate::codec_error)?;
    if bytes.len() > 1024 * 1024 {
        return Err(DataError::new("migration artifact byte budget exceeded"));
    }
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
/// Construct an explicit scope refusal, rather than weakening the selected contract.
fn unsupported(context: &'static str) -> DataError {
    DataError::new(context).with_kind(DataErrorKind::Unsupported)
}
