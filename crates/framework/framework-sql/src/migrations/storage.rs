//! Strict framework storage DTOs; SQL-core types and arbitrary SQL are never deserialized.

use super::{DataError, DataResult, unsupported};
use serde::{Deserialize, Serialize};
use stackpress_sql::{
    DefaultValue, Field, ForeignKey, Index, Length, ReferentialAction, SqlType, SqlValue,
    TableDefinition,
};

/// A checked literal used for defaults, backfills and database-only seeds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Literal {
    /// SQL NULL, distinct from a missing default.
    Null,
    /// Boolean without text coercion.
    Bool(bool),
    /// Exact signed integer.
    Integer(i64),
    /// Finite real number.
    Real(f64),
    /// Bound UTF-8 text.
    Text(String),
    /// Bound bytes.
    Blob(Vec<u8>),
}

impl Literal {
    /// Convert one validated portable literal to a detached SQL binding.
    pub fn value(&self) -> DataResult<SqlValue> {
        // Check borrowed payloads before cloning a literal into a retained SQL binding.
        if matches!(self,Self::Real(value) if !value.is_finite()) {
            return Err(DataError::new("nonfinite migration literal"));
        }
        if self.bytes() > 1024 * 1024 {
            return Err(DataError::new("migration literal budget exceeded"));
        }
        let value = match self {
            Self::Null => SqlValue::Null,
            Self::Bool(value) => SqlValue::Bool(*value),
            Self::Integer(value) => SqlValue::Integer(*value),
            Self::Real(value) => SqlValue::Real(*value),
            Self::Text(value) => SqlValue::Text(value.clone()),
            Self::Blob(value) => SqlValue::Blob(value.clone()),
        };
        Ok(value)
    }
    /// Account for retained value bytes without allocating another copy.
    pub(super) fn bytes(&self) -> usize {
        match self {
            Self::Text(value) => value.len(),
            Self::Blob(value) => value.len(),
            _ => 8,
        }
    }
    /// Retain the exact variant; dynamic expressions have no portable literal representation.
    pub(super) fn from_value(value: &SqlValue) -> Self {
        match value {
            SqlValue::Null => Self::Null,
            SqlValue::Bool(v) => Self::Bool(*v),
            SqlValue::Integer(v) => Self::Integer(*v),
            SqlValue::Real(v) => Self::Real(*v),
            SqlValue::Text(v) => Self::Text(v.clone()),
            SqlValue::Blob(v) => Self::Blob(v.clone()),
        }
    }
}

/// Private storage declaration, independent of SQL core's serialization policy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TableDto {
    /// Format-3 generated target policy; absent in the frozen format-2 codec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mysql_storage: Option<MysqlDto>,
    /// Exact managed SQL table identity.
    name: String,
    /// Complete logical field definitions, with physical order carried separately by plans.
    fields: Vec<FieldDto>,
    /// Ordered compound identity columns.
    primary: Vec<String>,
    /// Owned secondary indexes, including unique constraints.
    indexes: Vec<IndexDto>,
    /// Owned outgoing references and native actions.
    foreign_keys: Vec<ForeignDto>,
}

/// Closed policy vocabulary; engine/row format/charset are fixed by the selected variant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum MysqlDto {
    Required,
    InnoDbDynamicUtf8mb4 { collation: String, page_bytes: u32 },
}

/// Complete portable field metadata; unknown types and native expressions are refused.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FieldDto {
    /// Exact stored column identity.
    name: String,
    /// Closed portable type vocabulary; unknown strings refuse conversion.
    kind: String,
    /// Whether SQL NULL may be stored.
    nullable: bool,
    /// Size or precision/scale; None means no explicit length.
    length: Option<(i32, Option<i32>)>,
    /// Missing differs from explicit NULL/false/zero/empty literals.
    default: Option<Literal>,
    /// True requests SQLite AUTOINCREMENT semantics on a supported integer key.
    automatic: bool,
    /// Backend-specific metadata, refused by incompatible renderers.
    unsigned: bool,
    /// Optional checked storage annotation, never raw plugin configuration.
    comment: Option<String>,
}

/// Owned named index with exact compound order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexDto {
    /// Owned index identity, not inferred from live SQL.
    name: String,
    /// Ordered exact key columns.
    columns: Vec<String>,
    /// Database-enforced uniqueness, independent of validation hints.
    unique: bool,
}

/// Owned foreign constraint with exact actions and compound mapping.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ForeignDto {
    /// Owned constraint identity.
    name: String,
    /// Ordered local columns.
    columns: Vec<String>,
    /// Exact managed parent table.
    table: String,
    /// Ordered referenced columns with matching arity.
    foreign_columns: Vec<String>,
    /// Closed serialized ON DELETE action.
    delete: String,
    /// Closed serialized ON UPDATE action.
    update: String,
}

impl TableDto {
    /// Capture all supported metadata without serializing SQL text as execution authority.
    pub fn capture(table: &TableDefinition) -> DataResult<Self> {
        let fields = table
            .fields
            .iter()
            .map(|f| {
                let kind = match f.data_type {
                    SqlType::Integer => "integer",
                    SqlType::Real => "real",
                    SqlType::Boolean => "boolean",
                    SqlType::Text => "text",
                    SqlType::Varchar => "varchar",
                    SqlType::Blob => "blob",
                    SqlType::Json => "json",
                    SqlType::DateTime => "datetime",
                    SqlType::Decimal => "decimal",
                    SqlType::Trusted(..) => {
                        return Err(unsupported(
                            "trusted storage types cannot enter revision history",
                        ));
                    }
                };
                let default = match &f.default {
                    None => None,
                    Some(DefaultValue::Value(v)) => Some(Literal::from_value(v)),
                    _ => {
                        return Err(unsupported(
                            "dynamic storage defaults cannot enter revision history",
                        ));
                    }
                };
                Ok(FieldDto {
                    name: f.name.clone(),
                    kind: kind.into(),
                    nullable: f.nullable,
                    length: f.length.map(|l| match l {
                        Length::Size(n) => (n, None),
                        Length::Precision { precision, scale } => (precision, Some(scale)),
                    }),
                    default,
                    automatic: f.auto_increment,
                    unsigned: f.unsigned,
                    comment: f.comment.clone(),
                })
            })
            .collect::<DataResult<_>>()?;
        Ok(Self {
            mysql_storage: table
                .mysql_storage
                .as_ref()
                .map(|policy| match policy.facts() {
                    None => MysqlDto::Required,
                    Some((collation, page_bytes)) => MysqlDto::InnoDbDynamicUtf8mb4 {
                        collation: collation.into(),
                        page_bytes,
                    },
                }),
            name: table.name.clone(),
            fields,
            primary: table.primary.clone(),
            indexes: table
                .indexes
                .iter()
                .map(|i| IndexDto {
                    name: i.name.clone(),
                    columns: i.columns.clone(),
                    unique: i.unique,
                })
                .collect(),
            foreign_keys: table
                .foreign_keys
                .iter()
                .map(|f| ForeignDto {
                    name: f.name.clone(),
                    columns: f.columns.clone(),
                    table: f.table.clone(),
                    foreign_columns: f.foreign_columns.clone(),
                    delete: action(f.on_delete).into(),
                    update: action(f.on_update).into(),
                })
                .collect(),
        })
    }

    /// Reconstruct through checked core normalization, never by parsing companion SQL files.
    pub fn checked(&self) -> DataResult<TableDefinition> {
        if self.fields.len() > 128 {
            return Err(DataError::new("snapshot column budget exceeded"));
        }
        let fields = self
            .fields
            .iter()
            .map(|f| {
                let kind = match f.kind.as_str() {
                    "integer" => SqlType::Integer,
                    "real" => SqlType::Real,
                    "boolean" => SqlType::Boolean,
                    "text" => SqlType::Text,
                    "varchar" => SqlType::Varchar,
                    "blob" => SqlType::Blob,
                    "json" => SqlType::Json,
                    "datetime" => SqlType::DateTime,
                    "decimal" => SqlType::Decimal,
                    _ => return Err(unsupported("unknown revision storage type")),
                };
                let mut field = Field::new(&f.name, kind);
                field.nullable = f.nullable;
                field.length = f.length.map(|(n, s)| match s {
                    None => Length::Size(n),
                    Some(scale) => Length::Precision {
                        precision: n,
                        scale,
                    },
                });
                field.default = f
                    .default
                    .as_ref()
                    .map(|v| v.value().map(DefaultValue::Value))
                    .transpose()?;
                field.auto_increment = f.automatic;
                field.unsigned = f.unsigned;
                field.comment = f.comment.clone();
                Ok(field)
            })
            .collect::<DataResult<_>>()?;
        let table = TableDefinition {
            mysql_storage: None,
            name: self.name.clone(),
            fields,
            primary: self.primary.clone(),
            indexes: self
                .indexes
                .iter()
                .map(|i| Index {
                    name: i.name.clone(),
                    columns: i.columns.clone(),
                    unique: i.unique,
                })
                .collect(),
            foreign_keys: self
                .foreign_keys
                .iter()
                .map(|f| {
                    Ok(ForeignKey {
                        name: f.name.clone(),
                        columns: f.columns.clone(),
                        table: f.table.clone(),
                        foreign_columns: f.foreign_columns.clone(),
                        on_delete: parse_action(&f.delete)?,
                        on_update: parse_action(&f.update)?,
                    })
                })
                .collect::<DataResult<_>>()?,
        }
        .normalized()
        .map_err(DataError::from)?;
        let create = match &self.mysql_storage {
            None => table.create(),
            Some(MysqlDto::Required) => table.create().require_mysql_target(),
            Some(MysqlDto::InnoDbDynamicUtf8mb4 {
                collation,
                page_bytes,
            }) => table
                .create()
                .mysql_utf8mb4_dynamic(collation, *page_bytes)?,
        };
        create.definition().map_err(Into::into)
    }
}

/// Stable portable action spelling, independent of backend SQL text.
fn action(value: ReferentialAction) -> &'static str {
    match value {
        ReferentialAction::NoAction => "no_action",
        ReferentialAction::Restrict => "restrict",
        ReferentialAction::Cascade => "cascade",
        ReferentialAction::SetNull => "set_null",
        ReferentialAction::SetDefault => "set_default",
    }
}

/// Reject a future action rather than adopting different constraint behavior.
fn parse_action(value: &str) -> DataResult<ReferentialAction> {
    Ok(match value {
        "no_action" => ReferentialAction::NoAction,
        "restrict" => ReferentialAction::Restrict,
        "cascade" => ReferentialAction::Cascade,
        "set_null" => ReferentialAction::SetNull,
        "set_default" => ReferentialAction::SetDefault,
        _ => return Err(DataError::new("unknown revision foreign-key action")),
    })
}
