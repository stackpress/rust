//! Typed field, index and foreign-key metadata for detached DDL builders.
//!
//! These are explicit builder inputs, not schema introspection or migration history.
//! TableDefinition and structural diff belong to the following milestone.

use crate::identifier::{invalid, quote, unsupported};
use crate::{DialectId, Result, SqlValue, TrustedSql};

/// Known storage types; arbitrary SQL strings require an explicit trusted type.
#[derive(Clone, Debug, PartialEq)]
pub enum SqlType {
    /// Exact signed integer storage; SQLite primary auto-increment requires this type.
    Integer,
    /// Finite real storage.
    Real,
    /// Boolean affinity; native decoding is still strict 0/1.
    Boolean,
    /// Unbounded UTF-8 text.
    Text,
    /// Text affinity with optional declared length; SQLite does not enforce that length.
    Varchar,
    /// Owned bytes.
    Blob,
    /// Explicit JSON stored as text in SQLite; document validity is checked by callers.
    Json,
    /// Explicit canonical UTC date text in SQLite.
    DateTime,
    /// Exact decimal metadata; SQLite exact-decimal storage is unsupported.
    Decimal,
    /// Audited backend-specific type syntax; never inferred from ordinary text.
    Trusted(DialectId, TrustedSql),
}

/// Explicit type-length metadata. Invalid nonpositive lengths are rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Length {
    /// A positive character/storage declaration length.
    Size(i32),
    /// Decimal precision and scale, with 0 <= scale <= precision.
    Precision {
        /// Positive total number of decimal digits.
        precision: i32,
        /// Nonnegative fractional digit count.
        scale: i32,
    },
}

/// Absence of this enum means no explicit default; NULL/false/zero/empty remain values.
#[derive(Clone, Debug, PartialEq)]
pub enum DefaultValue {
    /// A validated scalar literal; DDL cannot use bind parameters for defaults.
    Value(SqlValue),
    /// The explicit native current timestamp expression.
    CurrentTimestamp,
    /// Audited native expression, carrying its dialect identity.
    Trusted(DialectId, TrustedSql),
}

impl<T: Into<SqlValue>> From<T> for DefaultValue {
    /// Preserve the actual scalar rather than testing truthiness.
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

/// One explicit field definition, compared structurally rather than by identity.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    /// One literal column name.
    pub name: String,
    /// Known or explicitly audited native type.
    pub data_type: SqlType,
    /// Whether SQL NULL is allowed.
    pub nullable: bool,
    /// Optional declared size/precision, validated by type.
    pub length: Option<Length>,
    /// No explicit default differs from an explicit NULL default.
    pub default: Option<DefaultValue>,
    /// Requires the sole integer primary key in SQLite.
    pub auto_increment: bool,
    /// Unsigned enforcement is unsupported by SQLite and never silently ignored.
    pub unsigned: bool,
    /// Native comment metadata; unsupported by SQLite rather than silently dropped.
    pub comment: Option<String>,
}

impl Field {
    /// Define a nullable field with no default or backend-specific annotations.
    pub fn new(name: impl Into<String>, data_type: SqlType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
            length: None,
            default: None,
            auto_increment: false,
            unsigned: false,
            comment: None,
        }
    }

    /// Disallow SQL NULL; this does not invent a default value.
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Preserve an explicit default, including false/zero/empty/NULL.
    pub fn default(mut self, value: impl Into<DefaultValue>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Request auto-increment on the sole integer primary key.
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    /// Declare a checked length/precision. SQLite Varchar length is affinity metadata only.
    pub fn length(mut self, length: Length) -> Self {
        self.length = Some(length);
        self
    }

    /// Validate dialect-independent field shape, without inferring SQL expressions.
    pub(crate) fn validate(&self) -> Result<()> {
        quote(&self.name)?;
        match (self.length, &self.data_type) {
            (None, _) => {}
            (Some(Length::Size(value)), SqlType::Varchar) if value > 0 => {}
            (Some(Length::Precision { precision, scale }), SqlType::Decimal)
                if precision > 0 && scale >= 0 && scale <= precision => {}
            _ => return Err(invalid("invalid SQL type length")),
        }
        if let Some(DefaultValue::Value(value)) = &self.default {
            value.validate()?;
            if matches!(value, SqlValue::Null) && !self.nullable {
                return Err(invalid("NULL default requires nullable field"));
            }
            // Explicit defaults must match known storage semantics; trusted types own their codec.
            let compatible = matches!(
                (value, &self.data_type),
                (SqlValue::Null, _)
                    | (_, SqlType::Trusted(_, _))
                    | (
                        SqlValue::Integer(_),
                        SqlType::Integer | SqlType::Real | SqlType::Decimal
                    )
                    | (SqlValue::Real(_), SqlType::Real | SqlType::Decimal)
                    | (SqlValue::Bool(_), SqlType::Boolean)
                    | (
                        SqlValue::Text(_),
                        SqlType::Text | SqlType::Varchar | SqlType::Json | SqlType::DateTime
                    )
                    | (SqlValue::Blob(_), SqlType::Blob)
            );
            if !compatible {
                return Err(invalid("SQL default type mismatch"));
            }
            if let (SqlValue::Text(value), SqlType::DateTime) = (value, &self.data_type) {
                SqlValue::utc_text(value)?;
            }
        }
        if matches!(self.default, Some(DefaultValue::CurrentTimestamp))
            && !matches!(self.data_type, SqlType::DateTime | SqlType::Text)
        {
            return Err(invalid("timestamp default requires text or datetime"));
        }
        Ok(())
    }

    /// Render one SQLite field, rejecting every unsupported annotation before effects.
    pub(crate) fn sqlite(&self, primary: &[String]) -> Result<String> {
        self.validate()?;
        if self.unsigned || self.comment.is_some() {
            return Err(unsupported(
                "SQLite cannot enforce unsigned fields or field comments",
            ));
        }
        let data_type = match &self.data_type {
            SqlType::Integer => "INTEGER".into(),
            SqlType::Real => "REAL".into(),
            SqlType::Boolean => "BOOLEAN".into(),
            SqlType::Text | SqlType::Json | SqlType::DateTime => "TEXT".into(),
            SqlType::Blob => "BLOB".into(),
            SqlType::Varchar => match self.length {
                Some(Length::Size(size)) => format!("VARCHAR({size})"),
                _ => "VARCHAR".into(),
            },
            SqlType::Decimal => {
                return Err(unsupported("SQLite exact decimal storage is unsupported"));
            }
            SqlType::Trusted(dialect, text) => trusted_fragment(*dialect, text)?,
        };
        let mut sql = format!("{} {data_type}", quote(&self.name)?);
        if self.auto_increment {
            // SQLite AUTOINCREMENT is legal only on an INTEGER PRIMARY KEY, never a compound key.
            if self.data_type != SqlType::Integer
                || primary != [self.name.clone()]
                || self.default.is_some()
            {
                return Err(invalid(
                    "SQLite auto-increment requires a sole integer primary without default",
                ));
            }
            sql.push_str(" PRIMARY KEY AUTOINCREMENT");
        }
        if !self.nullable {
            sql.push_str(" NOT NULL");
        }
        if let Some(default) = &self.default {
            sql.push_str(&format!(" DEFAULT {}", sqlite_default(default)?));
        }
        Ok(sql)
    }
}

/// One named ordered index; a unique definition wins only over identical fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
    /// Literal index name.
    pub name: String,
    /// Ordered literal local columns.
    pub columns: Vec<String>,
    /// Whether duplicate key tuples are forbidden.
    pub unique: bool,
}

impl Index {
    /// Define a regular index; validation runs at build/render time.
    pub fn new(
        name: impl Into<String>,
        columns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            columns: columns.into_iter().map(Into::into).collect(),
            unique: false,
        }
    }

    /// Require uniqueness while preserving name and compound-column order.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Validate one standalone index and render optional idempotence explicitly.
    pub(crate) fn sqlite(&self, table: &str, if_not_exists: bool) -> Result<String> {
        let columns = column_list(&self.columns)?;
        Ok(format!(
            "CREATE {}INDEX {}{} ON {} ({columns})",
            if self.unique { "UNIQUE " } else { "" },
            if if_not_exists { "IF NOT EXISTS " } else { "" },
            quote(&self.name)?,
            quote(table)?
        ))
    }
}

/// Typed foreign-key update/delete policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReferentialAction {
    /// No automatic action; the native constraint policy decides failure timing.
    #[default]
    NoAction,
    /// Reject conflicting parent changes.
    Restrict,
    /// Apply the parent change to children.
    Cascade,
    /// Set local columns to SQL NULL (all must be nullable).
    SetNull,
    /// Set local columns to their declared defaults.
    SetDefault,
}

impl ReferentialAction {
    /// Render only a finite native action keyword.
    fn sql(self) -> &'static str {
        match self {
            Self::NoAction => "NO ACTION",
            Self::Restrict => "RESTRICT",
            Self::Cascade => "CASCADE",
            Self::SetNull => "SET NULL",
            Self::SetDefault => "SET DEFAULT",
        }
    }
}

/// An explicitly named compound foreign key with ordered local/remote columns.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForeignKey {
    /// Literal constraint name.
    pub name: String,
    /// Ordered local columns.
    pub columns: Vec<String>,
    /// Literal target table (never a SQL fragment).
    pub table: String,
    /// Ordered target columns, with equal arity.
    pub foreign_columns: Vec<String>,
    /// Parent delete policy.
    pub on_delete: ReferentialAction,
    /// Parent update policy.
    pub on_update: ReferentialAction,
}

impl ForeignKey {
    /// Construct a compound relationship with no automatic parent-change action.
    pub fn new(
        name: impl Into<String>,
        columns: impl IntoIterator<Item = impl Into<String>>,
        table: impl Into<String>,
        foreign_columns: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            columns: columns.into_iter().map(Into::into).collect(),
            table: table.into(),
            foreign_columns: foreign_columns.into_iter().map(Into::into).collect(),
            on_delete: ReferentialAction::NoAction,
            on_update: ReferentialAction::NoAction,
        }
    }

    /// Validate local nullability/default constraints before rendering the relationship.
    pub(crate) fn sqlite(&self, fields: &[Field]) -> Result<String> {
        if self.columns.len() != self.foreign_columns.len() {
            return Err(invalid("foreign key arity mismatch"));
        }
        let local = column_list(&self.columns)?;
        let remote = column_list(&self.foreign_columns)?;
        for column in &self.columns {
            let field = find_field(fields, column)?;
            if [self.on_delete, self.on_update].contains(&ReferentialAction::SetNull)
                && !field.nullable
            {
                return Err(invalid("SET NULL requires nullable local fields"));
            }
            if [self.on_delete, self.on_update].contains(&ReferentialAction::SetDefault)
                && field.default.is_none()
            {
                return Err(invalid("SET DEFAULT requires explicit local defaults"));
            }
        }
        Ok(format!(
            "CONSTRAINT {} FOREIGN KEY ({local}) REFERENCES {} ({remote}) ON DELETE {} ON UPDATE {}",
            quote(&self.name)?,
            quote(&self.table)?,
            self.on_delete.sql(),
            self.on_update.sql()
        ))
    }
}

/// Validate a bounded, nonempty, ASCII-case-insensitively unique column list.
pub(crate) fn column_list(columns: &[String]) -> Result<String> {
    if columns.is_empty() || columns.len() > 1000 {
        return Err(invalid("invalid SQL key column list"));
    }
    let mut seen = std::collections::HashSet::new();
    let mut quoted = Vec::new();
    for column in columns {
        if !seen.insert(column.to_ascii_lowercase()) {
            return Err(invalid("duplicate SQL key column"));
        }
        quoted.push(quote(column)?);
    }
    Ok(quoted.join(", "))
}

/// Resolve a known local field using SQLite's identifier comparison semantics.
pub(crate) fn find_field<'a>(fields: &'a [Field], name: &str) -> Result<&'a Field> {
    fields
        .iter()
        .find(|field| field.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| invalid("key references an absent local field"))
}

/// Normalize index precedence without weakening a unique key or hiding conflicts.
pub(crate) fn normalize_indexes(indexes: &[Index]) -> Result<Vec<Index>> {
    if indexes.len() > 1000 {
        return Err(invalid("too many SQL indexes"));
    }
    let mut normalized: Vec<Index> = Vec::new();
    for index in indexes {
        quote(&index.name)?;
        column_list(&index.columns)?;
        if let Some(previous) = normalized
            .iter_mut()
            .find(|previous| previous.name.eq_ignore_ascii_case(&index.name))
        {
            // Ordered field identity matters; same-name incompatible definitions are an error.
            if previous.columns != index.columns {
                return Err(invalid("conflicting same-name SQL index"));
            }
            previous.unique |= index.unique;
        } else {
            normalized.push(index.clone());
        }
    }
    Ok(normalized)
}

/// Render typed defaults as correctly escaped SQLite literals or explicit trusted syntax.
fn sqlite_default(default: &DefaultValue) -> Result<String> {
    match default {
        DefaultValue::CurrentTimestamp => Ok("CURRENT_TIMESTAMP".into()),
        DefaultValue::Trusted(dialect, text) => {
            Ok(format!("({})", trusted_fragment(*dialect, text)?))
        }
        DefaultValue::Value(value) => {
            value.validate()?;
            match value {
                SqlValue::Null => Ok("NULL".into()),
                SqlValue::Bool(value) => Ok(if *value { "1" } else { "0" }.into()),
                SqlValue::Integer(value) => Ok(value.to_string()),
                SqlValue::Real(value) => Ok(value.to_string()),
                SqlValue::Text(value) => {
                    // DDL text cannot be bound; reject NUL rather than truncating at SQLite's C boundary.
                    if value.contains('\0') {
                        return Err(invalid("NUL in SQL default text"));
                    }
                    if value.len() > 1024 * 1024 {
                        return Err(crate::SqlError::new(
                            crate::ErrorKind::Limit,
                            "SQL default byte limit exceeded",
                        ));
                    }
                    Ok(format!("'{}'", value.replace('\'', "''")))
                }
                SqlValue::Blob(value) => {
                    if value.len() > 512 * 1024 {
                        return Err(crate::SqlError::new(
                            crate::ErrorKind::Limit,
                            "SQL default byte limit exceeded",
                        ));
                    }
                    let hex = value
                        .iter()
                        .map(|byte| format!("{byte:02X}"))
                        .collect::<String>();
                    Ok(format!("X'{hex}'"))
                }
            }
        }
    }
}

/// Keep trusted type/default syntax bounded, single-fragment and dialect-specific.
fn trusted_fragment(dialect: DialectId, text: &TrustedSql) -> Result<String> {
    if dialect != DialectId::Sqlite {
        return Err(unsupported("trusted DDL fragment dialect mismatch"));
    }
    let text = text.text();
    if text.is_empty() || text.contains([';', '\0']) || text.contains("--") || text.contains("/*") {
        return Err(invalid("invalid trusted DDL fragment"));
    }
    if text.len() > 16 * 1024 {
        return Err(crate::SqlError::new(
            crate::ErrorKind::Limit,
            "SQL DDL fragment limit exceeded",
        ));
    }
    Ok(text.into())
}
