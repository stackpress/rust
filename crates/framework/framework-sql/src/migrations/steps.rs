//! Checked pure statement compilation for reconstruction, literal data and bounded seeds.
use super::planner::Build;
use super::*;
use stackpress_sql::{
    Expr, Insert, InsertConflict, Render, Select, Selector, Sql, SqlType, SqlValue, TableCommand,
    TrustedSql,
};
/// Prepare a complete replacement group before acquisition; never rename the old table first.
pub(super) fn rebuild(
    source: &TableDefinition,
    target: &TableDefinition,
    mappings: &[&FieldRename],
    recipe: &PlanRecipe,
    build: &mut Build,
) -> DataResult<()> {
    let suffix = digest(&(source.name.as_str(), target.name.as_str(), &recipe.to))?;
    let temporary = format!("_sp_rebuild_{}", &suffix[..16]);
    let mut replacement = target.clone();
    replacement.name = temporary.clone();
    replacement.indexes.clear();
    let mut queries = replacement
        .create()
        .render(DialectId::Sqlite)?
        .queries()
        .to_vec();
    // Copy every target column explicitly: retained values stay columns; additions use only checked literals.
    let mut selectors = Vec::new();
    for new in &target.fields {
        let old = mappings
            .iter()
            .find(|r| r.to == new.name)
            .map_or(new.name.as_str(), |r| r.from.as_str());
        let expression = if source.fields.iter().any(|f| f.name == old) {
            Expr::column(old)
        } else {
            let literal = recipe
                .renames
                .backfills
                .iter()
                .find(|b| b.table == target.name && b.field == new.name)
                .map(|b| b.value.clone())
                .or_else(|| match &new.default {
                    Some(stackpress_sql::DefaultValue::Value(v)) => Some(Literal::from_value(v)),
                    _ => None,
                });
            let literal = match literal {
                Some(v) => v,
                None if new.nullable => Literal::Null,
                None => {
                    return Err(DataError::new(
                        "required addition needs a checked literal backfill",
                    ));
                }
            };
            literal_for(new, &literal)?;
            Expr::value(literal.value()?)
        };
        selectors.push(Selector::expression(expression));
    }
    let selection = Select::new(selectors)
        .from(source.name.as_str())
        .render(DialectId::Sqlite)?;
    // Fixed INSERT-SELECT grammar: typed SELECT owns expressions/binds; only checked names
    // are composed here. No caller expression, cast, filter or row omission is accepted.
    let columns = target
        .fields
        .iter()
        .map(|f| quoted(&f.name))
        .collect::<DataResult<Vec<_>>>()?
        .join(", ");
    let copy = format!(
        "INSERT INTO {} ({columns}) {}",
        quoted(&temporary)?,
        selection.sql()
    );
    queries.push(bound_sql(DialectId::Sqlite, copy, selection.binds())?);
    queries.push(TableCommand::drop_table(&source.name).render(DialectId::Sqlite)?);
    queries.push(TableCommand::rename(&temporary, &target.name).render(DialectId::Sqlite)?);
    queries.extend(
        target
            .create()
            .render(DialectId::Sqlite)?
            .queries()
            .iter()
            .skip(1)
            .cloned(),
    );
    let preserve_sequence = source.fields.iter().any(|f| f.auto_increment)
        && target.fields.iter().any(|f| f.auto_increment);
    build.steps.push(MigrationStep::Rebuild {
        source: source.name.clone(),
        target: target.name.clone(),
        queries,
        preserve_sequence,
    });
    Ok(())
}

/// Validate complete supplied seed values before the first write, leaving DB uniqueness authoritative.
pub(super) fn seed_query(
    snapshot: &RevisionSnapshot,
    row: &SeedRow,
    backend: DialectId,
) -> DataResult<Query> {
    let table = snapshot
        .tables
        .iter()
        .find(|t| t.name == row.table)
        .ok_or_else(|| DataError::new("seed table is unmanaged"))?;
    if row.values.is_empty() || row.values.len() > 128 {
        return Err(DataError::new("invalid seed column count"));
    }
    for (name, value) in &row.values {
        let field = table
            .fields
            .iter()
            .find(|f| f.name == *name)
            .ok_or_else(|| DataError::new("seed field is unknown"))?;
        literal_for(field, value)?;
    }
    if table.fields.iter().any(|f| {
        !f.nullable && f.default.is_none() && !f.auto_increment && !row.values.contains_key(&f.name)
    }) {
        return Err(DataError::new("seed required field is absent"));
    }
    // All rows are prepared before acquisition; database constraints remain authoritative at execution.
    let mut insert = Insert::new(row.table.as_str())
        .columns(row.values.keys().map(String::as_str))
        .values(
            row.values
                .values()
                .map(Literal::value)
                .collect::<DataResult<Vec<_>>>()?,
        );
    // A named primary-key target cannot accidentally update a different unique conflict.
    if row.upsert {
        if table.primary.is_empty() || table.primary.iter().any(|k| !row.values.contains_key(k)) {
            return Err(DataError::new("seed upsert needs a complete primary key"));
        }
        let assignments = row
            .values
            .iter()
            .filter(|(k, _)| !table.primary.contains(k))
            .map(|(k, v)| Ok((k.clone(), v.value()?.into())))
            .collect::<DataResult<Vec<_>>>()?;
        let conflict = if assignments.is_empty() {
            InsertConflict::DoNothing {
                target: table.primary.clone(),
            }
        } else {
            InsertConflict::DoUpdate {
                target: table.primary.clone(),
                assignments,
            }
        };
        insert = insert.on_conflict(conflict);
    }
    Ok(insert.render(backend)?)
}
/// Reject silent type conversions for literal data before any database effect.
pub(super) fn literal_for(field: &stackpress_sql::Field, value: &Literal) -> DataResult<()> {
    value.value()?;
    // Variant matching rejects SQLite affinity coercion before SQL can silently accept the value.
    let valid = match (value, &field.data_type) {
        (Literal::Null, _) => field.nullable,
        (Literal::Integer(_), SqlType::Integer)
        | (Literal::Real(_), SqlType::Real)
        | (Literal::Bool(_), SqlType::Boolean)
        | (
            Literal::Text(_),
            SqlType::Text | SqlType::Varchar | SqlType::Json | SqlType::DateTime,
        )
        | (Literal::Blob(_), SqlType::Blob) => true,
        _ => false,
    };
    if !valid {
        return Err(DataError::new(
            "literal backfill or seed has incompatible storage type",
        ));
    }
    if let Literal::Text(text) = value {
        if let Some(stackpress_sql::Length::Size(length)) = field.length
            && text.chars().count() > length as usize
        {
            return Err(DataError::new("literal exceeds storage length"));
        }
        if field.data_type == SqlType::Json {
            let _: serde_json::Value = decode(text)?;
        }
        if field.data_type == SqlType::DateTime {
            SqlValue::utc_text(text)?;
        }
    }
    Ok(())
}
/// Quote a deliberately conservative portable identifier at the audited fixed-grammar seam.
fn quoted(value: &str) -> DataResult<String> {
    identity(value)?;
    Ok(format!("\"{value}\""))
}
/// Apply detached binds to audited generated grammar and retain ordinary core query validation.
fn bound_sql(backend: DialectId, text: String, values: &[SqlValue]) -> DataResult<Query> {
    let mut sql = Sql::trusted(backend, TrustedSql::audited(text));
    for value in values {
        sql = sql.bind(value.clone());
    }
    Ok(sql.render(backend)?)
}
