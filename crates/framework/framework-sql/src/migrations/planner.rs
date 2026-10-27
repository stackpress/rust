//! Pure planning, aggregate findings and checked backend rendering; no database acquisition.

use super::steps::{literal_for, rebuild, seed_query};
use super::*;
use stackpress_sql::{Compare, Delete, Expr, Render, SqlType, SqlValue, TableCommand, Update};

/// Pure checked planner. Backend support is rendering capability, never an application claim.
pub struct MigrationPlanner;
impl MigrationPlanner {
    /// Plan installation without uninstalling or adopting existing managed objects.
    pub fn baseline(target: &RevisionSnapshot, dialect: DialectId) -> DataResult<MigrationPlan> {
        let source = RevisionSnapshot::new(&target.app, "empty", Vec::new())?;
        Self::between(&source, target, RenameMap::new(), dialect)
    }
    /// Plan exact old/new storage, retaining old physical slots and aggregating every loss finding.
    ///
    /// # Errors
    /// Invalid/stale mappings and unresolved same-shaped matches refuse planning. Unsupported
    /// conversions remain visible blocked plans; data-loss approval cannot authorize coercion.
    pub fn between(
        from: &RevisionSnapshot,
        to: &RevisionSnapshot,
        renames: RenameMap,
        dialect: DialectId,
    ) -> DataResult<MigrationPlan> {
        let to = to.physical_target(from, &renames)?;
        from_recipe(PlanRecipe {
            format: 2,
            dialect: dialect_name(dialect).into(),
            from: from.body.clone(),
            to: to.body,
            renames,
            kind: PlanKind::Schema,
            reverse: false,
            source_order: storage_order(from),
        })
    }
    /// List candidates without accepting any match, including a single same-shaped pair.
    pub fn suggestions(
        from: &RevisionSnapshot,
        to: &RevisionSnapshot,
    ) -> DataResult<Vec<RenameSuggestion>> {
        let mut result = Vec::new();
        for old in &from.tables {
            if let Some(new) = to.tables.iter().find(|t| t.name == old.name) {
                for removed in old
                    .fields
                    .iter()
                    .filter(|f| !new.fields.iter().any(|v| v.name == f.name))
                {
                    for added in new
                        .fields
                        .iter()
                        .filter(|f| !old.fields.iter().any(|v| v.name == f.name))
                    {
                        let mut renamed = removed.clone();
                        renamed.name = added.name.clone();
                        if renamed == *added {
                            if result.len() == 4096 {
                                return Err(DataError::new("rename suggestion budget exceeded"));
                            }
                            result.push(RenameSuggestion {
                                table: old.name.clone(),
                                from: removed.name.clone(),
                                to: added.name.clone(),
                            });
                        }
                    }
                }
            }
        }
        Ok(result)
    }
    /// Validate mapping authority before reporting only unresolved same-shaped candidates.
    pub fn unresolved_suggestions(
        from: &RevisionSnapshot,
        to: &RevisionSnapshot,
        renames: &RenameMap,
    ) -> DataResult<Vec<RenameSuggestion>> {
        validate_mappings(from, to, renames)?;
        Ok(Self::suggestions(from, to)?
            .into_iter()
            .filter(|s| {
                !renames
                    .fields
                    .iter()
                    .any(|r| r.table == s.table && r.from == s.from)
                    && !renames.drops.contains(&(s.table.clone(), s.from.clone()))
            })
            .collect())
    }
    /// Aggregate forward loss/allocation findings even when execution planning will refuse a precondition.
    ///
    /// # Errors
    /// Invalid mapping authority or mismatched app namespaces refuse review. No SQL is rendered,
    /// no history is published and no database is opened by this inspection operation.
    pub fn review(
        from: &RevisionSnapshot,
        to: &RevisionSnapshot,
        renames: &RenameMap,
        backend: DialectId,
    ) -> DataResult<Vec<RiskFinding>> {
        if from.app != to.app {
            return Err(DataError::new("migration application namespaces differ"));
        }
        validate_mappings(from, to, renames)?;
        // This pass deliberately accumulates losses without rendering or requiring backfills.
        // Construction below may stop at an unrepresentable step; review must still show later losses.
        let mut findings = Vec::new();
        for source in &from.tables {
            let name = renames
                .tables
                .iter()
                .find(|r| r.from == source.name)
                .map_or(source.name.as_str(), |r| r.to.as_str());
            let Some(target) = to.tables.iter().find(|t| t.name == name) else {
                findings.push(risk("drop_table", &source.name, None, true));
                continue;
            };
            for old in &source.fields {
                let name = renames
                    .fields
                    .iter()
                    .find(|r| r.table == source.name && r.from == old.name)
                    .map_or(old.name.as_str(), |r| r.to.as_str());
                match target.fields.iter().find(|f| f.name == name) {
                    None => findings.push(risk("drop_column", &source.name, Some(&old.name), true)),
                    Some(new) => {
                        if old.data_type != new.data_type
                            || old.length != new.length
                            || old.unsigned != new.unsigned
                            || old.auto_increment != new.auto_increment
                        {
                            findings.push(risk(
                                "retype_or_narrow",
                                &source.name,
                                Some(&old.name),
                                true,
                            ));
                        }
                        if old.nullable && !new.nullable {
                            findings.push(risk(
                                "nullable_to_required",
                                &source.name,
                                Some(&old.name),
                                true,
                            ));
                        }
                    }
                }
            }
            for new in &target.fields {
                let name = renames
                    .fields
                    .iter()
                    .find(|r| r.table == source.name && r.to == new.name)
                    .map_or(new.name.as_str(), |r| r.from.as_str());
                if !source.fields.iter().any(|f| f.name == name)
                    && !new.nullable
                    && new.default.is_none()
                    && !renames
                        .backfills
                        .iter()
                        .any(|b| b.table == target.name && b.field == new.name)
                {
                    findings.push(risk(
                        "required_addition_without_backfill",
                        &target.name,
                        Some(&new.name),
                        true,
                    ));
                }
            }
            if constraints_weakened(source, target, renames) {
                findings.push(risk(
                    "key_unique_or_foreign_key_replacement",
                    &source.name,
                    None,
                    true,
                ));
            }
        }
        if backend == DialectId::Sqlite {
            for table in &to.tables {
                if table.fields.iter().any(|f| {
                    f.data_type == SqlType::Integer
                        && table.primary == [f.name.clone()]
                        && !f.auto_increment
                }) {
                    findings.push(risk(
                        "plain_integer_primary_key_may_reuse_deleted_ids",
                        &table.name,
                        None,
                        false,
                    ));
                }
            }
        }
        Ok(findings)
    }
    /// Enumerate a destructive managed-data purge; schema/head remain intact.
    pub fn purge(snapshot: &RevisionSnapshot, dialect: DialectId) -> DataResult<MigrationPlan> {
        operator(snapshot, PlanKind::Purge, dialect)
    }
    /// Enumerate owned table removal while preserving namespaced ledger history.
    pub fn uninstall(snapshot: &RevisionSnapshot, dialect: DialectId) -> DataResult<MigrationPlan> {
        operator(snapshot, PlanKind::Uninstall, dialect)
    }
    /// Plan a named bounded database-only seed, with explicit create/upsert modes.
    pub fn populate(
        snapshot: &RevisionSnapshot,
        seed: &str,
        rows: Vec<SeedRow>,
        dialect: DialectId,
    ) -> DataResult<MigrationPlan> {
        identity(seed)?;
        seed_budget(&rows)?;
        operator(
            snapshot,
            PlanKind::Populate {
                seed: seed.into(),
                rows,
            },
            dialect,
        )
    }
}
/// Construct an explicit operator recipe without inventing another revision.
fn operator(
    snapshot: &RevisionSnapshot,
    kind: PlanKind,
    dialect: DialectId,
) -> DataResult<MigrationPlan> {
    from_recipe(PlanRecipe {
        format: 2,
        dialect: dialect_name(dialect).into(),
        from: snapshot.body.clone(),
        to: snapshot.body.clone(),
        renames: RenameMap::new(),
        kind,
        reverse: false,
        source_order: storage_order(snapshot),
    })
}

/// Re-render decoded authority; no statement from a SQL companion is consumed.
pub(super) fn from_recipe(recipe: PlanRecipe) -> DataResult<MigrationPlan> {
    if recipe.format != 2 {
        return Err(unsupported("unknown migration plan format"));
    }
    let backend = dialect(&recipe.dialect)?;
    let mut from = RevisionSnapshot::from_body(recipe.from.clone())?;
    apply_order(&mut from, &recipe.source_order)?;
    let to =
        RevisionSnapshot::from_body(recipe.to.clone())?.physical_target(&from, &recipe.renames)?;
    if from.app != to.app {
        return Err(DataError::new("migration application namespaces differ"));
    }
    validate_mappings(&from, &to, &recipe.renames)?;
    let mut build = Build {
        steps: Vec::new(),
        findings: Vec::new(),
        blocked: Vec::new(),
        discardable: Vec::new(),
    };
    match &recipe.kind {
        PlanKind::Schema => schema(&from, &to, &recipe, backend, &mut build)?,
        PlanKind::Purge | PlanKind::Uninstall => {
            if from.hash != to.hash {
                return Err(DataError::new("operator plan cannot change revision"));
            }
            // FK enforcement is disabled only by the SQLite maintenance owner; final integrity
            // guards the complete managed group. Other dialects receive inspection plans only.
            for table in dependency_order(&from.tables)?.into_iter().rev() {
                build
                    .findings
                    .push(risk("purge_or_uninstall", &table.name, None, true));
                let query = if matches!(recipe.kind, PlanKind::Purge) {
                    Delete::new(table.name.as_str())
                        .all_rows()
                        .render(backend)?
                } else {
                    TableCommand::drop_table(&table.name).render(backend)?
                };
                build.steps.push(MigrationStep::Native(vec![query]));
            }
        }
        PlanKind::Populate { seed, rows } => {
            identity(seed)?;
            if from.hash != to.hash || rows.is_empty() || rows.len() > 128 {
                return Err(DataError::new("invalid bounded seed plan"));
            }
            seed_budget(rows)?;
            for row in rows {
                build.steps.push(MigrationStep::Native(vec![seed_query(
                    &from, row, backend,
                )?]));
            }
        }
    }
    let irreversible = irreversible(&from, &to, &recipe, &build);
    let reverse = if recipe.reverse || irreversible {
        ReversePlan::Refused("data cannot be recovered automatically; use a reviewed custom reverse or backup restoration".into())
    } else {
        let mut mappings = RenameMap::new();
        for mapping in &recipe.renames.tables {
            mappings = mappings.table(&mapping.to, &mapping.from)?;
        }
        for mapping in &recipe.renames.fields {
            let table = recipe
                .renames
                .tables
                .iter()
                .find(|r| r.from == mapping.table)
                .map_or(mapping.table.as_str(), |r| r.to.as_str());
            mappings = mappings.field(table, &mapping.to, &mapping.from)?;
        }
        ReversePlan::Available(Box::new(from_recipe(PlanRecipe {
            format: 2,
            dialect: recipe.dialect.clone(),
            from: to.body.clone(),
            to: from.body.clone(),
            renames: mappings,
            kind: PlanKind::Schema,
            reverse: true,
            source_order: storage_order(&to),
        })?))
    };
    finish(from, to, recipe, build, reverse)
}
/// Mutable pure construction state; never stores native resources or caller SQL.
pub(super) struct Build {
    pub(super) steps: Vec<MigrationStep>,
    pub(super) findings: Vec<RiskFinding>,
    pub(super) blocked: Vec<String>,
    pub(super) discardable: Vec<(String, String)>,
}

/// Validate the entire mapping authority, including stale unused entries, before rendering.
fn validate_mappings(
    from: &RevisionSnapshot,
    to: &RevisionSnapshot,
    map: &RenameMap,
) -> DataResult<()> {
    if map.fields.len() + map.tables.len() + map.drops.len() + map.backfills.len() > 256 {
        return Err(DataError::new("migration mapping budget exceeded"));
    }
    let mut checked = RenameMap::new();
    for r in &map.tables {
        checked = checked.table(&r.from, &r.to)?;
        if !from.tables.iter().any(|t| t.name == r.from)
            || !to.tables.iter().any(|t| t.name == r.to)
            || from.tables.iter().any(|t| t.name == r.to)
            || to.tables.iter().any(|t| t.name == r.from)
        {
            return Err(DataError::new("stale table rename mapping"));
        }
    }
    for r in &map.fields {
        checked = checked.field(&r.table, &r.from, &r.to)?;
        let old = from
            .tables
            .iter()
            .find(|t| t.name == r.table)
            .ok_or_else(|| DataError::new("rename source table missing"))?;
        let target = map
            .tables
            .iter()
            .find(|t| t.from == r.table)
            .map_or(r.table.as_str(), |r| r.to.as_str());
        let new = to
            .tables
            .iter()
            .find(|t| t.name == target)
            .ok_or_else(|| DataError::new("rename target table missing"))?;
        if !old.fields.iter().any(|f| f.name == r.from)
            || old.fields.iter().any(|f| f.name == r.to)
            || !new.fields.iter().any(|f| f.name == r.to)
            || new.fields.iter().any(|f| f.name == r.from)
        {
            return Err(DataError::new("stale field rename mapping"));
        }
    }
    for (table, field) in &map.drops {
        checked = checked.drop_field(table, field)?;
        if !from
            .tables
            .iter()
            .any(|t| t.name == *table && t.fields.iter().any(|f| f.name == *field))
            || map
                .fields
                .iter()
                .any(|r| r.table == *table && r.from == *field)
        {
            return Err(DataError::new("invalid deliberate removal"));
        }
        let target = map
            .tables
            .iter()
            .find(|r| r.from == *table)
            .map_or(table.as_str(), |r| r.to.as_str());
        if to
            .tables
            .iter()
            .any(|t| t.name == target && t.fields.iter().any(|f| f.name == *field))
        {
            return Err(DataError::new("deliberate removal target still exists"));
        }
    }
    for b in &map.backfills {
        checked = checked.backfill(&b.table, &b.field, b.value.clone())?;
        let target = to
            .tables
            .iter()
            .find(|t| t.name == b.table)
            .and_then(|t| t.fields.iter().find(|f| f.name == b.field))
            .ok_or_else(|| DataError::new("backfill target missing"))?;
        literal_for(target, &b.value)?;
        let source_table = map
            .tables
            .iter()
            .find(|r| r.to == b.table)
            .map_or(b.table.as_str(), |r| r.from.as_str());
        let source_field = map
            .fields
            .iter()
            .find(|r| r.table == source_table && r.to == b.field)
            .map_or(b.field.as_str(), |r| r.from.as_str());
        if from
            .tables
            .iter()
            .find(|t| t.name == source_table)
            .and_then(|t| t.fields.iter().find(|f| f.name == source_field))
            .is_some_and(|f| !f.nullable || target.nullable)
        {
            return Err(DataError::new(
                "backfill does not resolve an addition or required transition",
            ));
        }
    }
    Ok(())
}

/// Plan removals first and retained/additional tables in deterministic dependency order.
fn schema(
    from: &RevisionSnapshot,
    to: &RevisionSnapshot,
    recipe: &PlanRecipe,
    backend: DialectId,
    build: &mut Build,
) -> DataResult<()> {
    for old in dependency_order(&from.tables)?.into_iter().rev() {
        let name = recipe
            .renames
            .tables
            .iter()
            .find(|r| r.from == old.name)
            .map_or(old.name.as_str(), |r| r.to.as_str());
        if !to.tables.iter().any(|t| t.name == name) {
            build
                .findings
                .push(risk("drop_table", &old.name, None, true));
            build.steps.push(MigrationStep::Native(vec![
                TableCommand::drop_table(&old.name).render(backend)?,
            ]));
        }
    }
    for target in dependency_order(&to.tables)? {
        if backend == DialectId::Sqlite
            && target.fields.iter().any(|f| {
                f.data_type == SqlType::Integer
                    && target.primary == [f.name.clone()]
                    && !f.auto_increment
            })
        {
            build.findings.push(risk(
                "plain_integer_primary_key_may_reuse_deleted_ids",
                &target.name,
                None,
                false,
            ));
        }
        let name = recipe
            .renames
            .tables
            .iter()
            .find(|r| r.to == target.name)
            .map_or(target.name.as_str(), |r| r.from.as_str());
        match from.tables.iter().find(|t| t.name == name) {
            None => build.steps.push(MigrationStep::Native(
                target.create().render(backend)?.queries().to_vec(),
            )),
            Some(source) if source == target => {}
            Some(source) => change_table(source, target, recipe, backend, build)?,
        }
    }
    Ok(())
}

/// Resolve explicit renames and aggregate loss, conversion and backfill policy for one table.
fn change_table(
    source: &TableDefinition,
    target: &TableDefinition,
    recipe: &PlanRecipe,
    backend: DialectId,
    build: &mut Build,
) -> DataResult<()> {
    // Retain source-local mapping identities even when the table itself receives a new name.
    let mappings = recipe
        .renames
        .fields
        .iter()
        .filter(|r| r.table == source.name)
        .collect::<Vec<_>>();
    // Required additions must have a complete value source before any executable step is retained.
    for new in &target.fields {
        let old = mappings
            .iter()
            .find(|r| r.to == new.name)
            .map_or(new.name.as_str(), |r| r.from.as_str());
        if !source.fields.iter().any(|f| f.name == old)
            && !new.nullable
            && new.default.is_none()
            && !recipe
                .renames
                .backfills
                .iter()
                .any(|b| b.table == target.name && b.field == new.name)
        {
            return Err(DataError::new(
                "required addition needs a checked literal backfill",
            ));
        }
    }
    // A retained field is copied exactly; removals and conversions carry separate review authority.
    for old in &source.fields {
        let name = mappings
            .iter()
            .find(|r| r.from == old.name)
            .map_or(old.name.as_str(), |r| r.to.as_str());
        match target.fields.iter().find(|f| f.name == name) {
            None => {
                let candidates = target
                    .fields
                    .iter()
                    .filter(|f| !source.fields.iter().any(|o| o.name == f.name))
                    .any(|f| {
                        let mut o = old.clone();
                        o.name = f.name.clone();
                        o == *f
                    });
                if candidates
                    && !recipe.reverse
                    && !recipe
                        .renames
                        .drops
                        .contains(&(source.name.clone(), old.name.clone()))
                {
                    return Err(DataError::new(
                        "likely rename requires explicit mapping or deliberate removal",
                    ));
                }
                if recipe.reverse && old.nullable {
                    build
                        .discardable
                        .push((source.name.clone(), old.name.clone()));
                } else {
                    build
                        .findings
                        .push(risk("drop_column", &source.name, Some(&old.name), true));
                }
            }
            Some(new) => {
                if old.data_type != new.data_type
                    || old.length != new.length
                    || old.unsigned != new.unsigned
                    || old.auto_increment != new.auto_increment
                {
                    build.findings.push(risk(
                        "retype_or_narrow",
                        &source.name,
                        Some(&old.name),
                        true,
                    ));
                    build.blocked.push("storage conversion requires a separately reviewed conversion implementation".into());
                }
                if old.nullable && !new.nullable {
                    build.findings.push(risk(
                        "nullable_to_required",
                        &source.name,
                        Some(&old.name),
                        true,
                    ));
                    // A supplied literal fills only NULLs; existing non-NULL values survive.
                    if let Some(b) = recipe
                        .renames
                        .backfills
                        .iter()
                        .find(|b| b.table == target.name && b.field == new.name)
                    {
                        build.steps.push(MigrationStep::Native(vec![
                            Update::new(source.name.as_str())
                                .set(old.name.as_str(), b.value.value()?)
                                .and_where(
                                    Expr::column(old.name.as_str())
                                        .compare(Compare::Eq, Expr::value(SqlValue::Null)),
                                )
                                .render(backend)?,
                        ]));
                    } else {
                        return Err(DataError::new(
                            "required transition needs explicit complete literal backfill",
                        ));
                    }
                }
            }
        }
    }
    if constraints_weakened(source, target, &recipe.renames) {
        // Rename-only constraint spelling is checked by the core; conservative findings
        // still require review when key/constraint identity changes.
        build.findings.push(risk(
            "key_unique_or_foreign_key_replacement",
            &source.name,
            None,
            true,
        ));
    }
    let mut mapped_source = source.clone();
    mapped_source.name = target.name.clone();
    let mut diff = mapped_source.diff(target)?;
    for mapping in &mappings {
        diff = diff.rename_field(&mapping.from, &mapping.to)?;
    }
    if backend != DialectId::Sqlite {
        if source.name != target.name {
            build.steps.push(MigrationStep::Native(vec![rename_table(
                &source.name,
                &target.name,
                backend,
            )?]));
        }
        if let Some(plan) = diff.render(backend)? {
            build
                .steps
                .push(MigrationStep::Native(plan.queries().to_vec()));
        }
        return Ok(());
    }
    // Native ALTER is used only where the core can realize the complete physical target.
    if source.name == target.name && recipe.renames.backfills.is_empty() {
        if let Ok(Some(plan)) = diff.render(backend) {
            build
                .steps
                .push(MigrationStep::Native(plan.queries().to_vec()));
            return Ok(());
        }
        if matches!(diff, stackpress_sql::SchemaDiff::NoChanges) {
            return Ok(());
        }
    }
    rebuild(source, target, &mappings, recipe, build)
}

/// Deterministic parent-first order; managed cycles retain sorted order and are checked together.
fn dependency_order(tables: &[TableDefinition]) -> DataResult<Vec<&TableDefinition>> {
    let names = tables
        .iter()
        .map(|t| t.name.as_str())
        .collect::<BTreeSet<_>>();
    if tables.iter().any(|t| {
        t.foreign_keys
            .iter()
            .any(|f| !names.contains(f.table.as_str()))
    }) {
        return Err(DataError::new(
            "foreign key refers outside the managed group",
        ));
    }
    let mut result = Vec::new();
    let mut remaining = tables.iter().collect::<Vec<_>>();
    while !remaining.is_empty() {
        let ready = remaining
            .iter()
            .position(|t| {
                t.foreign_keys.iter().all(|f| {
                    f.table == t.name || result.iter().any(|v: &&TableDefinition| v.name == f.table)
                })
            })
            .unwrap_or(0);
        result.push(remaining.remove(ready));
    }
    Ok(result)
}
/// Final aggregate preflight and exact approval hash, including reverse/capability metadata.
fn finish(
    from: RevisionSnapshot,
    to: RevisionSnapshot,
    recipe: PlanRecipe,
    build: Build,
    reverse: ReversePlan,
) -> DataResult<MigrationPlan> {
    let queries = build
        .steps
        .iter()
        .flat_map(|s| match s {
            MigrationStep::Native(q) | MigrationStep::Rebuild { queries: q, .. } => {
                q.iter().cloned()
            }
        })
        .collect::<Vec<_>>();
    if queries.len() > 128
        || queries.iter().map(|q| q.sql().len()).sum::<usize>() > 1024 * 1024
        || queries.iter().map(|q| q.binds().len()).sum::<usize>() > 10000
    {
        return Err(DataError::new("migration aggregate budget exceeded"));
    }
    let mut bytes = 0_usize;
    for query in &queries {
        query.validate(QueryLimits::default())?;
        bytes = bytes.saturating_add(query.sql().len());
        for value in query.binds() {
            bytes = bytes.saturating_add(match value {
                SqlValue::Text(v) => v.len(),
                SqlValue::Blob(v) => v.len(),
                _ => 8,
            });
        }
        if bytes > 1024 * 1024 {
            return Err(DataError::new(
                "migration aggregate retained byte budget exceeded",
            ));
        }
    }
    let statements = queries
        .iter()
        .map(|q| {
            (
                q.sql(),
                q.binds()
                    .iter()
                    .map(Literal::from_value)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let reverse_identity = match &reverse {
        ReversePlan::Available(p) => p.hash.0.as_str(),
        ReversePlan::Refused(reason) => reason.as_str(),
    };
    let hash = PlanHash(digest(&(
        &recipe,
        &statements,
        &build.findings,
        &build.blocked,
        &build.discardable,
        reverse_identity,
    ))?);
    Ok(MigrationPlan {
        from,
        to,
        queries,
        steps: build.steps,
        discardable: build.discardable,
        reverse,
        hash,
        recipe,
        findings: build.findings,
        blocked: build.blocked,
    })
}
/// Construct one review finding without rejected values or private connection information.
fn risk(kind: &str, table: &str, field: Option<&str>, destructive: bool) -> RiskFinding {
    RiskFinding {
        kind: kind.into(),
        table: table.into(),
        field: field.map(Into::into),
        destructive,
    }
}
/// Compose only fixed table-rename grammar, never user-authored fragments.
fn rename_table(from: &str, to: &str, backend: DialectId) -> DataResult<Query> {
    Ok(TableCommand::rename(from, to).render(backend)?)
}
/// Determine recoverability separately from SQL construction; renames retain field identity.
fn irreversible(
    from: &RevisionSnapshot,
    to: &RevisionSnapshot,
    recipe: &PlanRecipe,
    build: &Build,
) -> bool {
    from.tables.is_empty()
        || !matches!(recipe.kind, PlanKind::Schema)
        || !recipe.renames.backfills.is_empty()
        || build.findings.iter().any(|r| r.destructive)
        || to.tables.iter().any(|t| {
            t.fields.iter().any(|f| {
                let table = recipe
                    .renames
                    .tables
                    .iter()
                    .find(|r| r.to == t.name)
                    .map_or(t.name.as_str(), |r| r.from.as_str());
                let field = recipe
                    .renames
                    .fields
                    .iter()
                    .find(|r| r.table == table && r.to == f.name)
                    .map_or(f.name.as_str(), |r| r.from.as_str());
                !f.nullable
                    && !from
                        .tables
                        .iter()
                        .any(|o| o.name == table && o.fields.iter().any(|v| v.name == field))
            })
        })
}

/// Bound the whole seed before statement builders retain copies of supplied literals.
fn seed_budget(rows: &[SeedRow]) -> DataResult<()> {
    if rows.is_empty() || rows.len() > 128 {
        return Err(DataError::new("seed row budget exceeded"));
    }
    let mut bytes = 0_usize;
    for row in rows {
        if row.values.len() > 128 {
            return Err(DataError::new("seed column budget exceeded"));
        }
        bytes = bytes.saturating_add(row.table.len());
        for (name, value) in &row.values {
            bytes = bytes
                .saturating_add(name.len())
                .saturating_add(value.bytes());
        }
        if bytes > 1024 * 1024 {
            return Err(DataError::new("seed aggregate byte budget exceeded"));
        }
    }
    Ok(())
}

/// Compare constraint identity after applying exact local and referenced rename mappings.
fn constraints_weakened(
    source: &TableDefinition,
    target: &TableDefinition,
    renames: &RenameMap,
) -> bool {
    let mappings = renames
        .fields
        .iter()
        .filter(|r| r.table == source.name)
        .collect::<Vec<_>>();
    let mut constraint_source = source.clone();
    let rename_local = |name: &str| {
        mappings
            .iter()
            .find(|r| r.from == name)
            .map_or_else(|| name.to_owned(), |r| r.to.clone())
    };
    constraint_source.primary = constraint_source
        .primary
        .iter()
        .map(|n| rename_local(n))
        .collect();
    for index in &mut constraint_source.indexes {
        index.columns = index.columns.iter().map(|n| rename_local(n)).collect();
    }
    for key in &mut constraint_source.foreign_keys {
        key.columns = key.columns.iter().map(|n| rename_local(n)).collect();
        let old_table = key.table.clone();
        key.foreign_columns = key
            .foreign_columns
            .iter()
            .map(|name| {
                renames
                    .fields
                    .iter()
                    .find(|r| r.table == old_table && r.from == *name)
                    .map_or_else(|| name.clone(), |r| r.to.clone())
            })
            .collect();
        key.table = renames
            .tables
            .iter()
            .find(|r| r.from == old_table)
            .map_or(old_table, |r| r.to.clone());
    }
    constraint_source.primary != target.primary
        || constraint_source
            .indexes
            .iter()
            .any(|i| i.unique && !target.indexes.contains(i))
        || constraint_source
            .foreign_keys
            .iter()
            .any(|f| !target.foreign_keys.contains(f))
}
