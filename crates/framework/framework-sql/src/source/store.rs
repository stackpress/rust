//! Metadata-driven native stores. No framework model names or plugin dependencies live here.
use super::{SourceResult as Result, SourceStatement, StoreQuery};
use stackpress_framework_schema::{
    Model, ValueType,
    source::{Record, SourceRecordSchema, SourceSchema, Value},
};
use stackpress_sql::{
    Alter, Create, DefaultValue, Delete, DialectId, Field, ForeignKey, Index, Insert, Length,
    ReferentialAction, SchemaDiff, SqlType, SqlValue, Update,
};
use std::sync::Arc;

/// Detached store over one emitted schema; siblings are immutable catalog data, never a global registry.
#[derive(Clone)]
pub struct NativeStore<S: SourceSchema<Value = Value, Record = Record>> {
    /// Application-generated schema including all extensions and its configured providers.
    pub schema: S,
    /// The same source descriptors supply dynamic joins and storage names.
    pub(crate) record: SourceRecordSchema,
    pub(crate) model: Model,
    pub(crate) catalog: Arc<stackpress_framework_schema::SourceCatalog>,
}
impl<S: SourceSchema<Value = Value, Record = Record>> NativeStore<S> {
    /// Construction resolves metadata only; no defaults or database work occur.
    pub fn new(seed: String, offset: i32) -> Self {
        Self::from_schema(S::new(seed, offset))
    }
    /// Retain a caller's explicit provider and codec state.
    pub fn from_schema(schema: S) -> Self {
        let catalog = Arc::new(S::catalog());
        let model = S::describe();
        let record = SourceRecordSchema::from_metadata(
            &model,
            catalog.clone(),
            schema.seed().into(),
            schema.offset_minutes(),
        );
        Self {
            schema,
            record,
            model,
            catalog,
        }
    }
    /// Exact emitted SQL table identity.
    pub fn table(&self) -> &str {
        self.table_of(&self.model)
    }
    /// SQL spelling is independent of the corresponding Rust module/keyword escape.
    pub(crate) fn table_of<'a>(&'a self, model: &'a Model) -> &'a str {
        self.catalog
            .tables
            .iter()
            .find(|table| table.model == model.name)
            .map(|table| table.name.as_str())
            .unwrap_or(&model.module)
    }
    /// Sibling schema selected by a checked path; no hidden external tables are synthesized.
    pub(crate) fn sibling(&self, name: &str) -> Option<SourceRecordSchema> {
        self.catalog
            .models
            .iter()
            .find(|model| model.name == name)
            .map(|model| {
                SourceRecordSchema::from_metadata(
                    model,
                    self.catalog.clone(),
                    self.schema.seed().into(),
                    self.schema.offset_minutes(),
                )
            })
    }
    /// Source DDL including defaults, unique/index metadata and owning foreign keys.
    pub fn create(&self) -> Create {
        let mut create = Create::new(self.table());
        for column in self.record.columns() {
            let f = &column.field;
            if f.value_type == ValueType::Unknown {
                continue;
            }
            let declared = self
                .catalog
                .types
                .iter()
                .find(|t| t.model == self.model.name && t.field == f.name)
                .map(|t| t.name.as_str());
            let ty = if f.multiple {
                SqlType::Json
            } else {
                match f.value_type {
                    ValueType::String if declared == Some("Text") => SqlType::Text,
                    ValueType::Boolean => SqlType::Boolean,
                    ValueType::Integer => SqlType::Integer,
                    ValueType::Number if numeric_decimal(f) => SqlType::Real,
                    ValueType::Number => SqlType::Integer,
                    ValueType::Object | ValueType::Fieldset(_) => SqlType::Json,
                    ValueType::Date => SqlType::DateTime,
                    ValueType::Time => SqlType::DateTime,
                    ValueType::Datetime => SqlType::DateTime,

                    _ => SqlType::Varchar,
                }
            };
            let mut field = Field::new(&f.column, ty.clone());
            field.nullable = !f.required;
            if ty == SqlType::Varchar {
                let length = f
                    .rules
                    .iter()
                    .filter(|r| matches!(r.code.as_str(), "ceq" | "clt" | "cle"))
                    .filter_map(|r| r.args.first().and_then(|v| v.as_i64()))
                    .next_back()
                    .filter(|n| *n > 0)
                    .unwrap_or(255);
                field = field.length(Length::Size(length as i32));
            }
            if let Some(value) = &f.default {
                let default = if f.multiple
                    || matches!(f.value_type, ValueType::Object | ValueType::Fieldset(_))
                {
                    match value {
                        serde_json::Value::String(v) => SqlValue::Text(v.clone()),
                        v => SqlValue::Text(v.to_string()),
                    }
                } else {
                    match value {
                        serde_json::Value::Null => SqlValue::Null,
                        serde_json::Value::Bool(v) => SqlValue::Bool(*v),
                        serde_json::Value::Number(v) => v
                            .as_i64()
                            .map(SqlValue::Integer)
                            .unwrap_or_else(|| SqlValue::Real(v.as_f64().unwrap_or_default())),
                        serde_json::Value::String(v) => SqlValue::Text(v.clone()),
                        v => SqlValue::Text(v.to_string()),
                    }
                };
                field = field.default(DefaultValue::Value(default));
            } else if f.value_rule.as_ref().is_some_and(|r| r.provider == "now") {
                field = field.default(DefaultValue::CurrentTimestamp);
            }
            create = create.add_field(field);
            if f.unique {
                create = create.add_index(
                    Index::new(format!("{}_{}_unique", self.table(), f.column), [&f.column])
                        .unique(),
                );
            }
            if f.searchable
                || f.sortable
                || f.attributes
                    .iter()
                    .any(|a| a.name.starts_with("filter.") || a.name.starts_with("span."))
            {
                create = create.add_index(Index::new(
                    format!("{}_{}_index", self.table(), f.column),
                    [&f.column],
                ));
            }
        }
        let keys = self
            .model
            .fields
            .iter()
            .filter(|f| f.key)
            .map(|f| f.column.as_str());
        create = create.primary_key(keys);
        for f in &self.model.fields {
            if let (ValueType::Model(target), Some(relation)) = (&f.value_type, &f.relation) {
                let Some(target) = self.catalog.models.iter().find(|m| &m.name == target) else {
                    continue;
                };
                let local = self
                    .model
                    .fields
                    .iter()
                    .find(|f| f.name == relation.local)
                    .expect("normalized local key");
                let foreign = target
                    .fields
                    .iter()
                    .find(|f| f.name == relation.foreign)
                    .expect("normalized foreign key");
                let mut key = ForeignKey::new(
                    format!("{}_{}_foreign", self.table(), local.column),
                    [&local.column],
                    self.table_of(target),
                    [&foreign.column],
                );
                key.on_delete = ReferentialAction::Cascade;
                key.on_update = ReferentialAction::Restrict;
                create = create.add_foreign_key(key);
            }
        }
        create
    }
    /// Adjacent alteration uses the accepted SQL definition diff, without introspection.
    pub fn alter(&self, to: Option<&Create>) -> Result<Alter> {
        let Some(to) = to else {
            return Ok(Alter::new(self.table()));
        };
        Ok(match self.create().definition()?.diff(&to.definition()?)? {
            SchemaDiff::Changes { alteration, .. } => *alteration,
            _ => Alter::new(self.table()),
        })
    }
    /// Storage codecs map declared input to SQL spellings, including app-added members.
    pub fn scalarize(&self, input: &Record) -> Result<Vec<(String, SqlValue)>> {
        let mut encoded = Vec::new();
        for (name, value) in input {
            if let Some(column) = self.record.columns().iter().find(|c| c.name() == name)
                && let Some(value) = column.serialize(
                    Some(value),
                    self.schema.seed(),
                    true,
                    self.schema.offset_minutes(),
                )?
            {
                encoded.push((column.sql().to_owned(), self.to_sql_value(name, &value)?));
            }
        }
        Ok(encoded)
    }
    /// Raw SQL member names return to the original record names before storage decoding.
    pub fn unscalarize(&self, input: &Record) -> Result<Record> {
        let mut decoded = Record::new();
        for (name, value) in input {
            if let Some(column) = self.record.columns().iter().find(|c| c.sql() == name)
                && let Some(value) = column.unserialize(
                    Some(value),
                    self.schema.seed(),
                    true,
                    self.schema.offset_minutes(),
                )?
            {
                decoded.insert(column.name().to_owned(), value);
            }
        }
        Ok(decoded)
    }
    /// Convert the source scalar family after the declared column codec, not a strict DTO.
    pub fn to_sql_value(&self, name: &str, value: &Value) -> Result<SqlValue> {
        let Some(c) = self.record.columns().iter().find(|c| c.name() == name) else {
            return Ok(if value.is_null() {
                SqlValue::Null
            } else {
                SqlValue::Text(super::text::source_text(
                    value,
                    self.schema.offset_minutes(),
                ))
            });
        };
        if c.nullable() && value.is_null() {
            return Ok(SqlValue::Null);
        }
        Ok(match c.field.value_type {
            ValueType::Boolean => {
                SqlValue::Bool(stackpress_framework_schema::source::truthy(value))
            }
            ValueType::Integer => SqlValue::Integer(sql_number(value, true) as i64),
            ValueType::Number => SqlValue::Real(sql_number(value, false)),
            ValueType::Date | ValueType::Time | ValueType::Datetime => {
                SqlValue::Text(stackpress_framework_schema::source::iso(
                    stackpress_framework_schema::source::date_millis(
                        value,
                        self.schema.offset_minutes(),
                    ),
                )?)
            }
            _ => SqlValue::Text(if value.is_null() {
                String::new()
            } else {
                stackpress_framework_schema::source::string(value)
            }),
        })
    }
    /// MySQL omits unsupported RETURNING; source create's fallback remains in the action.
    pub fn insert(&self, input: &Record, dialect: DialectId) -> Result<Insert> {
        let values = self.scalarize(input)?;
        let insert = Insert::new(self.table())
            .columns(values.iter().map(|(n, _)| n.clone()))
            .values(values.into_iter().map(|(_, v)| v));
        Ok(if dialect.capabilities().returning {
            insert.returning_all()
        } else {
            insert
        })
    }
    /// Timestamp input is mutated before writing, after action results have been assembled.
    pub fn update(
        &self,
        query: &StoreQuery,
        input: &mut Record,
        dialect: DialectId,
    ) -> Result<SourceStatement<Update>> {
        for f in self.model.fields.iter().filter(|f| f.timestamp) {
            input.insert(
                f.name.clone(),
                stackpress_framework_schema::source::date(
                    stackpress_framework_schema::source::now(),
                ),
            );
        }
        let mut update = Update::new(self.table());
        for (name, value) in self.scalarize(input)? {
            update = update.set(name, value)
        }
        Ok(SourceStatement(
            if let Some(filter) = self.where_clause(query, dialect)? {
                update.and_where(filter)
            } else {
                update.all_rows()
            },
        ))
    }
    /// Physical deletion retains the same source active/filter defaults as a read.
    pub fn delete(
        &self,
        query: &StoreQuery,
        dialect: DialectId,
    ) -> Result<SourceStatement<Delete>> {
        let delete = Delete::new(self.table());
        Ok(SourceStatement(
            if let Some(filter) = self.where_clause(query, dialect)? {
                delete.and_where(filter)
            } else {
                delete.all_rows()
            },
        ))
    }
}

/// Source parseInt/parseFloat read a numeric prefix, unlike the Number column preparer.
fn sql_number(value: &Value, integer: bool) -> f64 {
    let text = super::text::source_text(value, 0);
    let text = stackpress_framework_schema::source::trim(&text);
    if integer {
        let (negative, digits) = if let Some(v) = text.strip_prefix('-') {
            (true, v)
        } else {
            (false, text.strip_prefix('+').unwrap_or(text))
        };
        let (radix, digits) = if let Some(v) = digits
            .strip_prefix("0x")
            .or_else(|| digits.strip_prefix("0X"))
        {
            (16, v)
        } else {
            (10, digits)
        };
        let mut result = 0.0;
        let mut seen = false;
        for c in digits.chars() {
            let Some(digit) = c.to_digit(radix) else {
                break;
            };
            seen = true;
            result = result * radix as f64 + digit as f64;
        }
        if !seen {
            return 0.0;
        }
        return if negative { -result } else { result };
    }
    // Only source Infinity spelling is special; Rust also accepts non-source "inf" aliases.
    if text.starts_with("Infinity") || text.starts_with("+Infinity") {
        return f64::INFINITY;
    }
    if text.starts_with("-Infinity") {
        return f64::NEG_INFINITY;
    }
    // Retain the longest decimal numeric prefix; trailing nonnumeric data is ignored.
    for (end, _) in text
        .char_indices()
        .chain(std::iter::once((text.len(), '\0')))
        .rev()
    {
        let prefix = &text[..end];
        if prefix
            .bytes()
            .all(|b| b.is_ascii_digit() || b"+-eE.".contains(&b))
            && let Ok(number) = prefix.parse::<f64>()
        {
            return if number.is_nan() { 0.0 } else { number };
        }
    }
    0.0
}

/// Source getNumberInfo chooses floating storage only when a bound or inferred step has decimals.
fn numeric_decimal(field: &stackpress_framework_schema::Field) -> bool {
    let decimal = |n: f64| {
        stackpress_framework_schema::source::number_text(n)
            .split_once('.')
            .is_some()
    };
    decimal(stackpress_framework_schema::source::number_step(field))
        || field
            .rules
            .iter()
            .filter(|rule| matches!(rule.code.as_str(), "eq" | "gt" | "ge" | "lt" | "le"))
            .filter_map(|rule| rule.args.first().and_then(serde_json::Value::as_f64))
            .any(decimal)
}
