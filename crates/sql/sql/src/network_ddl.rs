//! PostgreSQL/MySQL DDL rendering. No network connection or driver is owned here.

use crate::expr::join_sql;
use crate::identifier::{invalid, unsupported};
use crate::{
    AlterData, AlterOperation, CreateData, DefaultValue, DialectId, Field, ForeignKey, Index,
    Length, Query, QueryPlan, ReferentialAction, Result, SqlType, SqlValue, TrustedSql,
};

/// Render validated table fields/constraints first, then ordered indexes/comments.
pub(crate) fn create(data: &CreateData, dialect: DialectId) -> Result<QueryPlan> {
    let table = dialect.quote(&data.table)?;
    validate_columns(&data.primary, &data.fields, dialect, true)?;
    for index in &data.indexes {
        validate_columns(&index.columns, &data.fields, dialect, true)?;
    }
    // Render the table body incrementally; stop before retaining an oversized next clause.
    let primary = (!data.primary.is_empty()).then(|| {
        let name = if dialect == DialectId::Postgres {
            format!("CONSTRAINT {} ", dialect.quote(&primary_name(&data.table))?)
        } else {
            String::new()
        };
        Ok(format!(
            "{name}PRIMARY KEY ({})",
            columns(&data.primary, dialect)?
        ))
    });
    let inline_indexes = data
        .indexes
        .iter()
        .filter(|_| dialect == DialectId::Mysql)
        .map(|index| {
            Ok(format!(
                "{}INDEX {} ({})",
                if index.unique { "UNIQUE " } else { "" },
                dialect.quote(&index.name)?,
                columns(&index.columns, dialect)?
            ))
        });
    let parts = join_sql(
        data.fields
            .iter()
            .map(|field| definition(field, &data.primary, dialect))
            .chain(primary)
            .chain(
                data.foreign_keys
                    .iter()
                    .map(|key| foreign(key, &data.fields, dialect)),
            )
            .chain(inline_indexes),
        ", ",
    )?;
    let mut queries = vec![Query::bound(
        dialect,
        format!(
            "CREATE TABLE {}{table} ({})",
            if data.if_not_exists {
                "IF NOT EXISTS "
            } else {
                ""
            },
            parts
        ),
        Vec::new(),
    )?];
    if dialect == DialectId::Postgres {
        for index in &data.indexes {
            queries.push(Query::bound(
                dialect,
                index_sql(index, &data.table, data.if_not_exists, dialect)?,
                Vec::new(),
            )?);
        }
        for field in &data.fields {
            if let Some(comment) = &field.comment {
                queries.push(Query::bound(
                    dialect,
                    format!(
                        "COMMENT ON COLUMN {table}.{} IS {}",
                        dialect.quote(&field.name)?,
                        text_literal(comment, dialect)?
                    ),
                    Vec::new(),
                )?);
            }
        }
    }
    QueryPlan::new(queries)
}

/// Render all operations before exposing a plan, carrying field state through renames.
pub(crate) fn alter(data: &AlterData, dialect: DialectId) -> Result<QueryPlan> {
    let table = dialect.quote(&data.table)?;
    let mut fields = data.source_fields.clone();
    let mut statements = Vec::new();
    let mut queries = Vec::new();
    for operation in &data.operations {
        match operation {
            AlterOperation::AddField(field) => {
                if field.auto_increment {
                    return Err(unsupported(
                        "adding identity fields requires a coordinated key definition",
                    ));
                }
                statements.push(format!(
                    "ALTER TABLE {table} ADD COLUMN {}",
                    definition(field, &[], dialect)?
                ));
                fields.push(field.clone());
            }
            AlterOperation::RemoveField(name) => {
                statements.push(format!(
                    "ALTER TABLE {table} DROP COLUMN {}",
                    dialect.quote(name)?
                ));
                fields.retain(|field| !native_name_matches(&field.name, name, dialect));
            }
            AlterOperation::RenameField { from, to } => {
                statements.push(format!(
                    "ALTER TABLE {table} RENAME COLUMN {} TO {}",
                    dialect.quote(from)?,
                    dialect.quote(to)?
                ));
                let source = fields
                    .iter_mut()
                    .find(|field| native_name_matches(&field.name, from, dialect))
                    .ok_or_else(|| invalid("missing rename source field"))?;
                source.name = to.clone();
            }
            AlterOperation::ChangeField(field) => {
                // Validate all attributes even when the storage type itself is unchanged.
                definition(
                    field,
                    if field.auto_increment {
                        std::slice::from_ref(&field.name)
                    } else {
                        &[]
                    },
                    dialect,
                )?;
                let previous = fields
                    .iter_mut()
                    .find(|previous| native_name_matches(&previous.name, &field.name, dialect))
                    .ok_or_else(|| invalid("missing changed field source"))?;
                if previous.auto_increment != field.auto_increment {
                    return Err(unsupported(
                        "changing identity policy requires an explicit migration",
                    ));
                }
                if dialect == DialectId::Mysql {
                    statements.push(format!(
                        "ALTER TABLE {table} MODIFY COLUMN {}",
                        definition(
                            field,
                            if field.auto_increment {
                                std::slice::from_ref(&field.name)
                            } else {
                                &[]
                            },
                            dialect
                        )?
                    ));
                } else {
                    // Drop an old default before changing its type; restore exactly the requested default.
                    let name = dialect.quote(&field.name)?;
                    if previous.default.is_some() {
                        statements.push(format!(
                            "ALTER TABLE {table} ALTER COLUMN {name} DROP DEFAULT"
                        ));
                    }
                    if previous.data_type != field.data_type || previous.length != field.length {
                        statements.push(format!(
                            "ALTER TABLE {table} ALTER COLUMN {name} TYPE {}",
                            type_sql(field, dialect)?
                        ));
                    }
                    statements.push(format!(
                        "ALTER TABLE {table} ALTER COLUMN {name} {} NOT NULL",
                        if field.nullable { "DROP" } else { "SET" }
                    ));
                    if let Some(default) = &field.default {
                        statements.push(format!(
                            "ALTER TABLE {table} ALTER COLUMN {name} SET DEFAULT {}",
                            default_sql(default, &field.data_type, dialect)?
                        ));
                    }
                    // COMMENT NULL removes an old comment; absence is not silently ignored.
                    if previous.comment != field.comment {
                        statements.push(format!(
                            "COMMENT ON COLUMN {table}.{name} IS {}",
                            field
                                .comment
                                .as_ref()
                                .map(|comment| text_literal(comment, dialect))
                                .transpose()?
                                .unwrap_or_else(|| "NULL".into())
                        ));
                    }
                }
                *previous = field.clone();
            }
            AlterOperation::AddIndex(index) => {
                if !fields.is_empty() {
                    validate_columns(&index.columns, &fields, dialect, true)?;
                }
                statements.push(index_sql(index, &data.table, false, dialect)?)
            }
            AlterOperation::RemoveIndex(name) => statements.push(format!(
                "DROP INDEX {}{}",
                dialect.quote(name)?,
                if dialect == DialectId::Mysql {
                    format!(" ON {table}")
                } else {
                    String::new()
                }
            )),
            AlterOperation::AddPrimary(names) => {
                if !fields.is_empty() {
                    validate_columns(names, &fields, dialect, true)?;
                }
                statements.push(format!(
                    "ALTER TABLE {table} ADD {}PRIMARY KEY ({})",
                    if dialect == DialectId::Postgres {
                        format!("CONSTRAINT {} ", dialect.quote(&primary_name(&data.table))?)
                    } else {
                        String::new()
                    },
                    columns(names, dialect)?
                ))
            }
            AlterOperation::RemovePrimary => statements.push(if dialect == DialectId::Mysql {
                format!("ALTER TABLE {table} DROP PRIMARY KEY")
            } else {
                format!(
                    "ALTER TABLE {table} DROP CONSTRAINT {}",
                    dialect.quote(&primary_name(&data.table))?
                )
            }),
            AlterOperation::AddForeignKey(key) => statements.push(format!(
                "ALTER TABLE {table} ADD {}",
                foreign(key, &fields, dialect)?
            )),
            AlterOperation::RemoveForeignKey(name) => statements.push(format!(
                "ALTER TABLE {table} DROP {} {}",
                if dialect == DialectId::Mysql {
                    "FOREIGN KEY"
                } else {
                    "CONSTRAINT"
                },
                dialect.quote(name)?
            )),
        }
        // Freeze one operation before generating the next; an invalid oversized statement
        // cannot accumulate behind later operations. Draining preserves exact source order.
        for sql in statements.drain(..) {
            queries.push(Query::bound(dialect, sql, Vec::new())?);
        }
    }
    QueryPlan::new(queries)
}

/// Known field syntax, with native support checks instead of silently discarded attributes.
fn definition(field: &Field, primary: &[String], dialect: DialectId) -> Result<String> {
    field.validate()?;
    let mut sql = format!(
        "{} {}",
        dialect.quote(&field.name)?,
        type_sql(field, dialect)?
    );
    if field.auto_increment {
        if field.data_type != SqlType::Integer
            || primary != [field.name.clone()]
            || field.default.is_some()
        {
            return Err(invalid(
                "identity requires a sole integer primary without default",
            ));
        }
        sql.push_str(if dialect == DialectId::Postgres {
            " GENERATED BY DEFAULT AS IDENTITY"
        } else {
            " AUTO_INCREMENT"
        });
    }
    if !field.nullable {
        sql.push_str(" NOT NULL");
    }
    if let Some(default) = &field.default {
        sql.push_str(&format!(
            " DEFAULT {}",
            default_sql(default, &field.data_type, dialect)?
        ));
    }
    if dialect == DialectId::Mysql
        && let Some(comment) = &field.comment
    {
        if comment.contains('\\') || comment.contains('\0') {
            return Err(unsupported(
                "MySQL comments with backslash/NUL are outside the renderer subset",
            ));
        }
        sql.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
    }
    Ok(sql)
}

/// Render native storage type and enforce portable field-length/attribute limits.
fn type_sql(field: &Field, dialect: DialectId) -> Result<String> {
    field.validate()?;
    if field.unsigned && (dialect == DialectId::Postgres || field.data_type != SqlType::Integer) {
        return Err(unsupported(
            "unsigned storage is supported only for MySQL integer fields",
        ));
    }
    let sql = match &field.data_type {
        SqlType::Integer => "BIGINT".into(),
        SqlType::Real => "DOUBLE PRECISION".into(),
        SqlType::Boolean => "BOOLEAN".into(),
        SqlType::Text => "TEXT".into(),
        SqlType::DateTime => "VARCHAR(30)".into(),
        SqlType::Json => if dialect == DialectId::Postgres {
            "JSONB"
        } else {
            "JSON"
        }
        .into(),
        SqlType::Blob => if dialect == DialectId::Postgres {
            "BYTEA"
        } else {
            "LONGBLOB"
        }
        .into(),
        SqlType::Varchar => match field.length {
            Some(Length::Size(size)) if dialect != DialectId::Mysql || size <= 16383 => {
                format!("VARCHAR({size})")
            }
            None if dialect == DialectId::Postgres => "VARCHAR".into(),
            _ => {
                return Err(unsupported(
                    "MySQL VARCHAR requires an explicit supported positive length",
                ));
            }
        },
        SqlType::Decimal => match field.length {
            Some(Length::Precision { precision, scale }) => {
                if dialect == DialectId::Mysql && (precision > 65 || scale > 30) {
                    return Err(invalid(
                        "MySQL decimal precision/scale exceeds native limits",
                    ));
                }
                if dialect == DialectId::Postgres && precision > 1000 {
                    return Err(invalid("PostgreSQL decimal precision exceeds native limit"));
                }
                format!("DECIMAL({precision}, {scale})")
            }
            _ => "DECIMAL".into(),
        },
        SqlType::Trusted(identity, text) => fragment(*identity, text, dialect)?,
    };
    Ok(format!(
        "{sql}{}",
        if field.unsigned { " UNSIGNED" } else { "" }
    ))
}

/// Render typed scalar defaults without depending on a server's string escape mode.
fn default_sql(default: &DefaultValue, data_type: &SqlType, dialect: DialectId) -> Result<String> {
    let expression = match default {
        DefaultValue::CurrentTimestamp => "CURRENT_TIMESTAMP".into(),
        DefaultValue::Trusted(identity, text) => {
            format!("({})", fragment(*identity, text, dialect)?)
        }
        DefaultValue::Value(value) => {
            value.validate()?;
            match value {
                SqlValue::Null => "NULL".into(),
                SqlValue::Bool(value) => if *value { "TRUE" } else { "FALSE" }.into(),
                SqlValue::Integer(value) => value.to_string(),
                SqlValue::Real(value) => value.to_string(),
                SqlValue::Text(value) => text_literal(value, dialect)?,
                SqlValue::Blob(value) => {
                    if value.len() > 512 * 1024 {
                        return Err(invalid("SQL blob default exceeds byte limit"));
                    }
                    let hex = value
                        .iter()
                        .map(|byte| format!("{byte:02X}"))
                        .collect::<String>();
                    if dialect == DialectId::Postgres {
                        format!("decode('{hex}', 'hex')")
                    } else {
                        format!("X'{hex}'")
                    }
                }
            }
        }
    };
    // MySQL expression defaults on TEXT/BLOB/JSON require parentheses, even for literals.
    if dialect == DialectId::Mysql
        && matches!(
            data_type,
            SqlType::Text | SqlType::Blob | SqlType::Json | SqlType::DateTime
        )
        && expression != "NULL"
    {
        Ok(format!("({expression})"))
    } else {
        Ok(expression)
    }
}

/// Quote text without a dependency on MySQL NO_BACKSLASH_ESCAPES or PostgreSQL settings.
fn text_literal(value: &str, dialect: DialectId) -> Result<String> {
    if value.is_empty() {
        return Ok("''".into());
    }
    if value.contains('\0') || value.len() > 1024 * 1024 {
        return Err(invalid("invalid SQL default/comment text"));
    }
    if dialect == DialectId::Postgres {
        Ok(format!(
            "E'{}'",
            value.replace('\\', "\\\\").replace('\'', "''")
        ))
    } else {
        // Hex plus CONVERT preserves UTF-8 bytes under either MySQL quote-escape mode.
        let hex = value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<String>();
        Ok(format!("(CONVERT(X'{hex}' USING utf8mb4))"))
    }
}

/// Preserve explicit trust and native identity; a trusted fragment is never inferred.
fn fragment(identity: DialectId, text: &TrustedSql, dialect: DialectId) -> Result<String> {
    if identity != dialect {
        return Err(unsupported("trusted DDL fragment dialect mismatch"));
    }
    let text = text.text();
    if text.trim().is_empty()
        || text.len() > 16 * 1024
        || text.contains([';', '\0'])
        || text.contains("--")
        || text.contains("/*")
    {
        return Err(invalid("invalid trusted DDL fragment"));
    }
    Ok(text.into())
}

/// Render a checked native foreign key, including unsupported SET DEFAULT detection.
fn foreign(key: &ForeignKey, fields: &[Field], dialect: DialectId) -> Result<String> {
    key.sqlite(fields)?;
    validate_columns(&key.columns, fields, dialect, true)?;
    if dialect == DialectId::Mysql
        && [key.on_delete, key.on_update].contains(&ReferentialAction::SetDefault)
    {
        return Err(unsupported(
            "MySQL InnoDB does not support SET DEFAULT foreign actions",
        ));
    }
    Ok(format!(
        "CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {} ON UPDATE {}",
        dialect.quote(&key.name)?,
        columns(&key.columns, dialect)?,
        dialect.quote(&key.table)?,
        columns(&key.foreign_columns, dialect)?,
        key.on_delete.sql(),
        key.on_update.sql()
    ))
}

/// Render non-idempotent ALTER indexes and explicit CREATE idempotence for PostgreSQL.
fn index_sql(index: &Index, table: &str, idempotent: bool, dialect: DialectId) -> Result<String> {
    dialect.require(
        !idempotent || dialect.capabilities().idempotent_indexes,
        "dialect lacks idempotent standalone indexes",
    )?;
    Ok(format!(
        "CREATE {}INDEX {}{} ON {} ({})",
        if index.unique { "UNIQUE " } else { "" },
        if idempotent { "IF NOT EXISTS " } else { "" },
        dialect.quote(&index.name)?,
        dialect.quote(table)?,
        columns(&index.columns, dialect)?
    ))
}

/// Quote ordered compound columns after common duplicate/empty validation.
fn columns(names: &[String], dialect: DialectId) -> Result<String> {
    crate::schema::column_list(names)?;
    names
        .iter()
        .map(|name| dialect.quote(name))
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.join(", "))
}

/// Use an explicit PostgreSQL primary name for later pure diff removal.
fn primary_name(table: &str) -> String {
    format!("{table}_pkey")
}

/// Validate known native local operands, including PG quoted case and MySQL prefix needs.
fn validate_columns(
    names: &[String],
    fields: &[Field],
    dialect: DialectId,
    index: bool,
) -> Result<()> {
    for name in names {
        let field = fields
            .iter()
            .find(|field| native_name_matches(&field.name, name, dialect))
            .ok_or_else(|| invalid("native key references an absent quoted field"))?;
        if dialect == DialectId::Mysql
            && index
            && matches!(
                field.data_type,
                SqlType::Text | SqlType::Blob | SqlType::Json
            )
        {
            return Err(unsupported(
                "MySQL TEXT/BLOB/JSON indexes require an explicit prefix/expression design",
            ));
        }
    }
    Ok(())
}

/// Preserve quoted PostgreSQL case while matching MySQL's case-insensitive column names.
fn native_name_matches(defined: &str, referenced: &str, dialect: DialectId) -> bool {
    if dialect == DialectId::Postgres {
        defined == referenced
    } else {
        defined.eq_ignore_ascii_case(referenced)
    }
}
