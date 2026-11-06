//! JSON storage of actual generated SQL definitions, without a second Idea/DDL compiler.
//!
//! The blog uses native generated storage kinds and defaults. SQL builders still validate and
//! render them. This codec deliberately does not use the stricter typed migration ledger, which
//! cannot retain the source's CURRENT_TIMESTAMP defaults or warning-only planning policy.

use serde::{Deserialize, Serialize};
use stackpress_framework_sql::migrations::Literal;
use stackpress_lib::{Error, Result};
use stackpress_sql::{
    DefaultValue, Field, ForeignKey, Index, Length, ReferentialAction, SqlType, SqlValue,
    TableDefinition,
};

/// Native generated scalar kinds; audited custom SQL is outside this SQLite example.
#[derive(Serialize, Deserialize)]
#[serde(remote = "SqlType")]
enum Kind {
    Integer,
    Real,
    Boolean,
    Text,
    Varchar,
    Blob,
    Json,
    DateTime,
    Decimal,
    #[serde(skip)]
    Trusted(stackpress_sql::DialectId, stackpress_sql::TrustedSql),
}

/// Exact foreign-key actions, without choosing a new cascading policy.
#[derive(Serialize, Deserialize)]
#[serde(remote = "ReferentialAction")]
enum ForeignRule {
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}

/// Source defaults distinguish an explicit NULL, a literal, and native current timestamp.
#[derive(Serialize, Deserialize)]
enum Default {
    Literal(Literal),
    Now,
}

/// Plain serialization of one generated column; no storage/type inference occurs here.
#[derive(Serialize, Deserialize)]
struct Column {
    name: String,
    #[serde(with = "Kind")]
    kind: SqlType,
    nullable: bool,
    length: Option<(i32, Option<i32>)>,
    default: Option<Default>,
    automatic: bool,
    unsigned: bool,
    comment: Option<String>,
}

/// Ordered generated index, retaining names as well as physical columns.
#[derive(Serialize, Deserialize)]
struct Key {
    name: String,
    columns: Vec<String>,
    unique: bool,
}

/// Ordered generated foreign key, including both action rules.
#[derive(Serialize, Deserialize)]
struct Foreign {
    name: String,
    columns: Vec<String>,
    table: String,
    foreign_columns: Vec<String>,
    #[serde(with = "ForeignRule")]
    on_delete: ReferentialAction,
    #[serde(with = "ForeignRule")]
    on_update: ReferentialAction,
}

/// Private native storage description persisted next to the complete source metadata.
#[derive(Serialize, Deserialize)]
pub(super) struct Table {
    name: String,
    fields: Vec<Column>,
    primary: Vec<String>,
    indexes: Vec<Key>,
    foreign_keys: Vec<Foreign>,
}

/// Retain scalar identity, including Bool versus Integer and Text versus Blob.
fn literal(value: &SqlValue) -> Literal {
    match value {
        SqlValue::Null => Literal::Null,
        SqlValue::Bool(v) => Literal::Bool(*v),
        SqlValue::Integer(v) => Literal::Integer(*v),
        SqlValue::Real(v) => Literal::Real(*v),
        SqlValue::Text(v) => Literal::Text(v.clone()),
        SqlValue::Blob(v) => Literal::Blob(v.clone()),
    }
}

impl Table {
    /// Capture definitions from the generated stores, preserving every native field property.
    pub(super) fn capture(table: &TableDefinition) -> Result<Self> {
        if table.mysql_storage.is_some() {
            return Err(Error::new("blog history uses SQLite storage definitions"));
        }
        let fields = table
            .fields
            .iter()
            .map(|field| {
                let default = match &field.default {
                    None => None,
                    Some(DefaultValue::Value(value)) => Some(Default::Literal(literal(value))),
                    Some(DefaultValue::CurrentTimestamp) => Some(Default::Now),
                    Some(DefaultValue::Trusted(..)) => {
                        return Err(Error::new(
                            "custom SQL defaults need a history codec adapter",
                        ));
                    }
                };
                Ok(Column {
                    name: field.name.clone(),
                    kind: field.data_type.clone(),
                    nullable: field.nullable,
                    length: field.length.map(|length| match length {
                        Length::Size(size) => (size, None),
                        Length::Precision { precision, scale } => (precision, Some(scale)),
                    }),
                    default,
                    automatic: field.auto_increment,
                    unsigned: field.unsigned,
                    comment: field.comment.clone(),
                })
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            name: table.name.clone(),
            fields,
            primary: table.primary.clone(),
            indexes: table
                .indexes
                .iter()
                .map(|key| Key {
                    name: key.name.clone(),
                    columns: key.columns.clone(),
                    unique: key.unique,
                })
                .collect(),
            foreign_keys: table
                .foreign_keys
                .iter()
                .map(|key| Foreign {
                    name: key.name.clone(),
                    columns: key.columns.clone(),
                    table: key.table.clone(),
                    foreign_columns: key.foreign_columns.clone(),
                    on_delete: key.on_delete,
                    on_update: key.on_update,
                })
                .collect(),
        })
    }

    /// Restore builder inputs; the existing SQL owner handles validation/diff/rendering.
    pub(super) fn definition(self) -> Result<TableDefinition> {
        let fields = self
            .fields
            .into_iter()
            .map(|field| {
                Ok(Field {
                    name: field.name,
                    data_type: field.kind,
                    nullable: field.nullable,
                    length: field.length.map(|(precision, scale)| {
                        scale.map_or(Length::Size(precision), |scale| Length::Precision {
                            precision,
                            scale,
                        })
                    }),
                    default: match field.default {
                        None => None,
                        Some(Default::Now) => Some(DefaultValue::CurrentTimestamp),
                        Some(Default::Literal(value)) => Some(DefaultValue::Value(
                            value.value().map_err(Error::from_source)?,
                        )),
                    },
                    auto_increment: field.automatic,
                    unsigned: field.unsigned,
                    comment: field.comment,
                })
            })
            .collect::<Result<_>>()?;
        let table = TableDefinition {
            mysql_storage: None,
            name: self.name,
            fields,
            primary: self.primary,
            indexes: self
                .indexes
                .into_iter()
                .map(|key| Index {
                    name: key.name,
                    columns: key.columns,
                    unique: key.unique,
                })
                .collect(),
            foreign_keys: self
                .foreign_keys
                .into_iter()
                .map(|key| ForeignKey {
                    name: key.name,
                    columns: key.columns,
                    table: key.table,
                    foreign_columns: key.foreign_columns,
                    on_delete: key.on_delete,
                    on_update: key.on_update,
                })
                .collect(),
        };
        table.create().definition().map_err(Error::from_source)
    }
}
