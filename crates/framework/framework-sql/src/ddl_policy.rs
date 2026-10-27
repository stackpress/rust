//! Explicit generated MySQL target selection and complete-catalog checks, without IO.
use crate::{DataError, DataErrorKind, DataResult};
use stackpress_sql::{DialectId, SqlType, TableDefinition};

/// Apply app-owned InnoDB DYNAMIC/utf8mb4 expectations and validate every managed FK.
///
/// Collation and expected page size are required, never inferred from server defaults.
/// The renderer admits a conservative all-inline record layout; unknown/off-page layouts
/// refuse. Plain SQL-core VARCHAR builders keep their existing behavior.
/// This constructs render-only MySQL definitions; it does not enable migration application.
///
/// # Errors
/// Refuses unknown target facts, oversized rows/keys, external FK targets, and text family,
/// declared length (including absence), or target-policy disagreement across FK endpoints.
///
/// # Examples
/// ```
/// use stackpress_framework_sql::ddl_policy::mysql_definitions;
/// use stackpress_sql::{Create, DialectId, Field, Length, SqlType};
/// let note = Create::new("note")
///     .add_field(Field::new("id", SqlType::Varchar).length(Length::Size(191)).not_null())
///     .primary_key(["id"])
///     .require_mysql_target()
///     .definition()?;
/// let tables = mysql_definitions(vec![note], "utf8mb4_bin", 16384)?;
/// let ddl = tables[0].create().render(DialectId::Mysql)?;
/// assert!(ddl.queries()[0].sql().contains("ROW_FORMAT=DYNAMIC"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn mysql_definitions(
    tables: Vec<TableDefinition>,
    collation: &str,
    page_bytes: u32,
) -> DataResult<Vec<TableDefinition>> {
    // Select facts even for an empty catalog: callers can preflight options before source IO.
    let _ = stackpress_sql::Create::new("_stackpress_target")
        .mysql_utf8mb4_dynamic(collation, page_bytes)
        .map_err(crate::write_plan_error)?;
    if tables.len() > 128 {
        return Err(DataError::new("MySQL catalog table budget exceeded"));
    }
    catalog_budget(&tables)?;
    let tables = tables
        .into_iter()
        .map(|table| {
            if table
                .mysql_storage
                .as_ref()
                .and_then(|policy| policy.facts())
                .is_some_and(|facts| facts != (collation, page_bytes))
            {
                return Err(refusal(
                    "explicit MySQL target facts disagree with catalog intent",
                ));
            }
            table
                .create()
                .mysql_utf8mb4_dynamic(collation, page_bytes)
                .map_err(crate::write_plan_error)?
                .definition()
                .map_err(crate::write_plan_error)
        })
        .collect::<DataResult<Vec<_>>>()?;
    check_catalog(&tables)?;
    for table in &tables {
        table
            .create()
            .render(DialectId::Mysql)
            .map_err(crate::write_plan_error)?;
    }
    Ok(tables)
}

/// Check already-selected facts without silently changing any table's declared intent.
pub(crate) fn check_catalog(tables: &[TableDefinition]) -> DataResult<()> {
    check_widths(tables)?;
    for table in tables {
        for key in &table.foreign_keys {
            let target = tables
                .iter()
                .find(|target| target.name == key.table)
                .ok_or_else(|| {
                    refusal("generated MySQL FK target is outside the managed catalog")
                })?;
            if table.mysql_storage != target.mysql_storage {
                return Err(refusal("generated MySQL FK target facts disagree"));
            }
        }
    }
    Ok(())
}

/// Every generated dialect retains explicit FK widths; a missing annotation is never inherited.
pub(crate) fn check_widths(tables: &[TableDefinition]) -> DataResult<()> {
    for (index, table) in tables.iter().enumerate() {
        if tables[..index]
            .iter()
            .any(|other| other.name.eq_ignore_ascii_case(&table.name))
        {
            return Err(DataError::new("duplicate MySQL catalog table"));
        }
        for key in &table.foreign_keys {
            let target = tables
                .iter()
                .find(|target| target.name == key.table)
                .ok_or_else(|| {
                    refusal("generated MySQL FK target is outside the managed catalog")
                })?;
            for (local, foreign) in key.columns.iter().zip(&key.foreign_columns) {
                let local = table
                    .fields
                    .iter()
                    .find(|field| field.name == *local)
                    .ok_or_else(|| DataError::new("missing MySQL FK source field"))?;
                let foreign = target
                    .fields
                    .iter()
                    .find(|field| field.name == *foreign)
                    .ok_or_else(|| DataError::new("missing MySQL FK target field"))?;
                let text = matches!(local.data_type, SqlType::Text | SqlType::Varchar)
                    || matches!(foreign.data_type, SqlType::Text | SqlType::Varchar);
                if text && (local.data_type != foreign.data_type || local.length != foreign.length)
                {
                    return Err(refusal(
                        "generated MySQL FK text family or declared width disagrees",
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Keep unsupported-policy context distinct from private native render causes.
fn refusal(context: &'static str) -> DataError {
    DataError::new(context).with_kind(DataErrorKind::Unsupported)
}

/// Bound borrowed metadata before core normalization or DTO capture clones any large values.
pub(crate) fn catalog_budget(tables: &[TableDefinition]) -> DataResult<()> {
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
