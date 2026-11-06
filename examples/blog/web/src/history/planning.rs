//! Source Migrations planning over retained generated definitions and existing SQL diffs.
//!
//! This layer detects source rename/destructive metadata. It never refuses a warning: migrate
//! still writes its SQL, and only the framework upgrade/push execution path interprets force.

use stackpress_framework::commands::sql::Migration;
use stackpress_lib::{Error, Result};
use stackpress_sql::{DialectId, Field, ForeignKey, Render, TableCommand, TableDefinition};

/// One source ambiguity group. Names are sorted only inside the diagnostic group.
struct Ambiguity {
    model: String,
    from: Vec<String>,
    to: Vec<String>,
}

/// Removed physical pieces, after equivalent rename replacements have been consumed.
struct Removed {
    table: String,
    fields: Vec<String>,
    primary: Vec<String>,
    unique: Vec<String>,
    keys: Vec<String>,
    foreign: Vec<String>,
}

/// Match native field semantics while ignoring the removed/added name itself.
fn same_column(
    before: &TableDefinition,
    old: &Field,
    after: &TableDefinition,
    new: &Field,
) -> bool {
    let mut old_shape = old.clone();
    old_shape.name = new.name.clone();
    let count = |table: &TableDefinition, name: &str, unique| {
        table
            .indexes
            .iter()
            .filter(|index| {
                index.unique == unique && index.columns.iter().any(|column| column == name)
            })
            .count()
    };
    let foreign = |table: &TableDefinition, name: &str| {
        table
            .foreign_keys
            .iter()
            .filter_map(|key| {
                key.columns
                    .iter()
                    .position(|column| column == name)
                    .map(|position| {
                        (
                            key.table.clone(),
                            key.foreign_columns[position].clone(),
                            key.on_delete,
                            key.on_update,
                        )
                    })
            })
            .collect::<Vec<_>>()
    };
    old_shape == *new
        && before.primary.contains(&old.name) == after.primary.contains(&new.name)
        && count(before, &old.name, true) == count(after, &new.name, true)
        && count(before, &old.name, false) == count(after, &new.name, false)
        && foreign(before, &old.name) == foreign(after, &new.name)
}

/// Reconcile exactly the original one-to-one groups; any ambiguous group suppresses table guesses.
fn renames(
    before: &TableDefinition,
    after: &TableDefinition,
    model: &str,
) -> (Vec<(String, String)>, Option<Ambiguity>) {
    let removed = before
        .fields
        .iter()
        .filter(|old| !after.fields.iter().any(|new| new.name == old.name));
    let added = after
        .fields
        .iter()
        .filter(|new| !before.fields.iter().any(|old| old.name == new.name))
        .collect::<Vec<_>>();
    let matches = removed
        .filter_map(|old| {
            let candidates = added
                .iter()
                .filter(|new| same_column(before, old, after, new))
                .map(|new| new.name.clone())
                .collect::<Vec<_>>();
            (!candidates.is_empty()).then(|| (old.name.clone(), candidates))
        })
        .collect::<Vec<_>>();
    let mut ambiguous_from = Vec::new();
    let mut ambiguous_to = Vec::new();
    for (old, candidates) in &matches {
        // A many-target source or a target shared by single-candidate sources is ambiguous.
        let shared = candidates.len() == 1
            && matches
                .iter()
                .filter(|(_, values)| values.len() == 1 && values[0] == candidates[0])
                .count()
                > 1;
        if candidates.len() > 1 || shared {
            ambiguous_from.push(old.clone());
            ambiguous_to.extend(candidates.iter().cloned());
        }
    }
    if !ambiguous_from.is_empty() {
        ambiguous_from.sort();
        ambiguous_from.dedup();
        ambiguous_to.sort();
        ambiguous_to.dedup();
        return (
            Vec::new(),
            Some(Ambiguity {
                model: model.into(),
                from: ambiguous_from,
                to: ambiguous_to,
            }),
        );
    }
    (
        matches
            .into_iter()
            .map(|(old, mut values)| (old, values.remove(0)))
            .collect(),
        None,
    )
}

/// Translate local names only after an accepted rename; this is not a guessed table rename.
fn mapped(name: &str, renames: &[(String, String)]) -> String {
    renames
        .iter()
        .find(|(old, _)| old == name)
        .map_or_else(|| name.into(), |(_, new)| new.clone())
}

/// Consume equivalent index replacements one-for-one, separately for unique and ordinary keys.
fn removed_indexes(
    before: &TableDefinition,
    after: &TableDefinition,
    renames: &[(String, String)],
    unique: bool,
) -> Vec<String> {
    let mut replacements = after
        .indexes
        .iter()
        .filter(|key| key.unique == unique)
        .map(|key| key.columns.clone())
        .collect::<Vec<_>>();
    before
        .indexes
        .iter()
        .filter(|key| key.unique == unique && !after.indexes.contains(key))
        .filter_map(|key| {
            let columns = key
                .columns
                .iter()
                .map(|column| mapped(column, renames))
                .collect::<Vec<_>>();
            if let Some(index) = replacements
                .iter()
                .position(|replacement| *replacement == columns)
            {
                replacements.remove(index);
                None
            } else {
                Some(key.name.clone())
            }
        })
        .collect()
}

/// Source foreign equivalence ignores constraint names but preserves target columns and actions.
fn equivalent_foreign(old: &ForeignKey, new: &ForeignKey, renames: &[(String, String)]) -> bool {
    old.columns
        .iter()
        .map(|column| mapped(column, renames))
        .collect::<Vec<_>>()
        == new.columns
        && old.table == new.table
        && old.foreign_columns == new.foreign_columns
        && old.on_delete == new.on_delete
        && old.on_update == new.on_update
}

/// Inspect removals after reconciled renames; changed names alone do not invent data-loss warnings.
fn removed(
    before: &TableDefinition,
    after: &TableDefinition,
    renames: &[(String, String)],
) -> Removed {
    let mut replacements = after.foreign_keys.iter().collect::<Vec<_>>();
    let foreign = before
        .foreign_keys
        .iter()
        .filter(|key| !after.foreign_keys.contains(key))
        .filter_map(|key| {
            if let Some(index) = replacements
                .iter()
                .position(|new| equivalent_foreign(key, new, renames))
            {
                replacements.remove(index);
                None
            } else {
                Some(key.name.clone())
            }
        })
        .collect();
    Removed {
        table: after.name.clone(),
        fields: before
            .fields
            .iter()
            .filter(|old| {
                !after.fields.iter().any(|new| new.name == old.name)
                    && !renames.iter().any(|(from, _)| *from == old.name)
            })
            .map(|field| field.name.clone())
            .collect(),
        primary: before
            .primary
            .iter()
            .filter(|column| !after.primary.contains(&mapped(column, renames)))
            .cloned()
            .collect(),
        unique: removed_indexes(before, after, renames, true),
        keys: removed_indexes(before, after, renames, false),
        foreign,
    }
}

/// Reproduce source child-first drop order; the fixed blog graph is acyclic.
pub(super) fn drop_order(tables: &[TableDefinition]) -> Result<Vec<&TableDefinition>> {
    let mut remaining = tables.iter().collect::<Vec<_>>();
    let mut order = Vec::new();
    while !remaining.is_empty() {
        let leaves = remaining
            .iter()
            .copied()
            .filter(|parent| {
                !remaining.iter().any(|child| {
                    child
                        .foreign_keys
                        .iter()
                        .any(|key| key.table == parent.name)
                })
            })
            .collect::<Vec<_>>();
        if leaves.is_empty() {
            return Err(Error::new(
                "source table ordering requires an acyclic catalog",
            ));
        }
        remaining.retain(|table| !leaves.iter().any(|leaf| leaf.name == table.name));
        order.extend(leaves);
    }
    Ok(order)
}

/// Format source warnings; no command execution/refusal happens at this boundary.
fn warning(ambiguous: &[Ambiguity], removed: &[Removed], drops: &[String]) -> Option<String> {
    let mut warnings = Vec::new();
    if !ambiguous.is_empty() {
        let mut lines = vec![
            "Ambiguous field rename detected.".into(),
            "Stackpress found multiple same-shape rename candidates and refused to guess.".into(),
            "Review the pending schema change before executing it against a database.".into(),
        ];
        for group in ambiguous {
            lines.push(format!(
                "- {}: {} -> {}",
                group.model,
                group.from.join(", "),
                group.to.join(", ")
            ));
        }
        warnings.push(lines.join("\n"));
    }
    if !removed.is_empty() || !drops.is_empty() {
        let mut lines = vec![
            "Destructive schema changes detected.".into(),
            "This migration removes existing schema pieces and may drop data.".into(),
        ];
        for table in removed {
            lines.extend([String::new(), format!("Table \"{}\":", table.table)]);
            for (label, values) in [
                ("fields", &table.fields),
                ("primary keys", &table.primary),
                ("unique keys", &table.unique),
                ("indexes", &table.keys),
                ("foreign keys", &table.foreign),
            ] {
                if !values.is_empty() {
                    lines.push(format!("- Removed {label}: {}", values.join(", ")));
                }
            }
        }
        if !drops.is_empty() {
            lines.extend([
                String::new(),
                format!("Dropped tables: {}", drops.join(", ")),
            ]);
        }
        warnings.push(lines.join("\n"));
    }
    if warnings.is_empty() {
        return None;
    }
    warnings.push(
        "Re-run with `--force` only if you intentionally want to execute these changes.".into(),
    );
    Some(warnings.join("\n\n"))
}

/// Plan a baseline or adjacent source revision using the native SQL owner's exact diff/render APIs.
pub(super) fn plan(
    before: Option<&[TableDefinition]>,
    after: &[TableDefinition],
    epoch: i64,
    dialect: DialectId,
    source: &serde_json::Value,
) -> Result<Migration> {
    let mut queries = Vec::new();
    let mut destructive = Vec::new();
    let mut ambiguous = Vec::new();
    let mut drops = Vec::new();
    if let Some(before) = before {
        // Preserve catalog order for adjacent changes, applying only unambiguous field renames.
        for table in after {
            if let Some(previous) = before.iter().find(|previous| previous.name == table.name) {
                let model = source["tables"]
                    .as_array()
                    .and_then(|tables| {
                        tables
                            .iter()
                            .find(|identity| identity["name"] == table.name)
                    })
                    .and_then(|identity| identity["model"].as_str())
                    .unwrap_or(&table.name);
                let (renames, ambiguity) = renames(previous, table, model);
                ambiguous.extend(ambiguity);
                let mut diff = previous.diff(table).map_err(Error::from_source)?;
                for (from, to) in &renames {
                    diff = diff.rename_field(from, to).map_err(Error::from_source)?;
                }
                let removal = removed(previous, table, &renames);
                if [
                    &removal.fields,
                    &removal.primary,
                    &removal.unique,
                    &removal.keys,
                    &removal.foreign,
                ]
                .iter()
                .any(|values| !values.is_empty())
                {
                    destructive.push(removal);
                }
                if let Some(plan) = diff.render(dialect).map_err(Error::from_source)? {
                    queries.extend_from_slice(plan.queries());
                }
            } else {
                queries.extend_from_slice(
                    table
                        .create()
                        .render(dialect)
                        .map_err(Error::from_source)?
                        .queries(),
                );
            }
        }
        // Removed tables follow the original catalog rather than a new migration policy.
        for table in before
            .iter()
            .filter(|old| !after.iter().any(|new| new.name == old.name))
        {
            drops.push(table.name.clone());
            queries.push(
                TableCommand::Drop {
                    table: table.name.clone(),
                    if_exists: true,
                    cascade: false,
                }
                .render(dialect)
                .map_err(Error::from_source)?,
            );
        }
    } else {
        // The source baseline reinstalls: children drop first, then parents create first.
        let order = drop_order(after)?;
        for table in &order {
            drops.push(table.name.clone());
            queries.push(
                TableCommand::Drop {
                    table: table.name.clone(),
                    if_exists: true,
                    cascade: false,
                }
                .render(dialect)
                .map_err(Error::from_source)?,
            );
        }
        for table in order.into_iter().rev() {
            queries.extend_from_slice(
                table
                    .create()
                    .render(dialect)
                    .map_err(Error::from_source)?
                    .queries(),
            );
        }
    }
    Ok(Migration {
        epoch,
        queries,
        warning: warning(&ambiguous, &destructive, &drops),
    })
}
