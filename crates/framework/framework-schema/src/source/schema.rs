//! Native schema execution over generated columns; no model names or field-name policy.
use super::{Column, Record, Value, default_value};
use crate::{Field, SchemaResult, ValueProviders, providers::BuiltinProviders};
use std::sync::Arc;

/// One generated storage column and its complete source metadata.
#[derive(Clone, Debug)]
pub struct SourceColumn {
    /// Legacy-compatible conversion only; its name-based default getter is never exposed here.
    codec: Column,
    /// Exact composed field, including assertions, defaults, roles and source attributes.
    pub field: Field,
    /// Immutable sibling definitions; no global model registry or runtime import.
    pub(super) catalog: Arc<crate::SourceCatalog>,
}
impl SourceColumn {
    /// Pair a generated codec with its exact source declaration.
    pub fn new(codec: Column, field: Field) -> Self {
        Self {
            codec,
            field,
            catalog: Arc::default(),
        }
    }
    /// Bind emitted sibling enum/fieldset definitions for direct column use as well as schemas.
    pub fn with_catalog(mut self, catalog: Arc<crate::SourceCatalog>) -> Self {
        self.catalog = catalog;
        self
    }
    /// Exact source field name, independent of its SQL/member spelling.
    pub fn name(&self) -> &str {
        &self.field.name
    }
    /// Generated SQL column identity.
    pub fn sql(&self) -> &str {
        &self.field.column
    }
    /// Resolve this generated field's source default with the ordinary native clock/provider.
    /// This retains the former column convenience path without its name-based default catalog.
    pub fn defaults(&self) -> SchemaResult<Option<Value>> {
        self.defaults_at(super::now(), &BuiltinProviders, 0)
    }
    /// Source Date formatting convenience, independent of this column's declared scalar kind.
    pub fn serialize_date(&self, millis: i64, offset: i32) -> SchemaResult<Value> {
        Ok(Value::String(super::date_text(millis, offset)?))
    }
    /// Source conversion family, selected from the declaration and codec attributes.
    pub fn kind(&self) -> super::Kind {
        self.codec.kind
    }
    /// Whether source null/undefined bypasses scalar conversion.
    pub fn nullable(&self) -> bool {
        self.codec.nullable
    }
    /// Prepare/storage-encode with the existing source codec; defaults use this field's metadata.
    pub fn serialize(
        &self,
        value: Option<&Value>,
        seed: &str,
        storage: bool,
        offset: i32,
    ) -> SchemaResult<Option<Value>> {
        super::coercion::convert(self, value, seed, storage, offset, true)
    }
    /// Decode using source readback behavior, including hash retention and native Dates.
    pub fn unserialize(
        &self,
        value: Option<&Value>,
        seed: &str,
        storage: bool,
        offset: i32,
    ) -> SchemaResult<Option<Value>> {
        super::coercion::convert(self, value, seed, storage, offset, false)
    }
    /// Resolve a getter default with an explicit clock, provider and local Date offset.
    pub fn defaults_at(
        &self,
        now: i64,
        providers: &dyn ValueProviders,
        offset: i32,
    ) -> SchemaResult<Option<Value>> {
        if self.field.default.is_none()
            && self.field.value_rule.is_none()
            && let Some(schema) = self.embedded(String::new(), offset)
        {
            let record: SchemaResult<Record> = schema
                .columns
                .iter()
                .filter_map(|column| match column.defaults_at(now, providers, offset) {
                    Ok(Some(value)) => Some(Ok((column.name().into(), value))),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                })
                .collect();
            return record.map(|record| Some(Value::Object(record)));
        }
        let Some(value) = default_value(&self.field, now, providers)? else {
            return Ok(None);
        };
        // Source generator calls unserialize for literals. Providers return directly,
        // including a native Date from now(); no storage decryption is selected here.
        if self.field.value_rule.is_some() {
            return Ok(Some(value));
        }
        self.unserialize(Some(&value), "", false, offset)
    }
    /// Assert the source base shape first, then implicit required and ordered refinements.
    pub fn assert(&self, value: Option<&Value>) -> Option<String> {
        self.assert_at(value, super::now(), 0)
    }
    /// Test time-relative rules using an explicit native clock and local offset.
    pub fn assert_at(&self, value: Option<&Value>, now: i64, offset: i32) -> Option<String> {
        super::checks::assert(&self.field, value, &self.codec, now, offset, &self.catalog)
    }
    /// Preserve nested fieldset error records instead of flattening them into one message.
    pub fn assert_value_at(&self, value: Option<&Value>, now: i64, offset: i32) -> Option<Value> {
        if let Some(schema) = self.embedded(String::new(), offset) {
            let empty = Record::new();
            let input = value.and_then(Value::as_object).unwrap_or(&empty);
            return schema.assert_at(input, false, now).map(Value::Object);
        }
        self.assert_at(value, now, offset).map(Value::String)
    }
    /// G51 uniqueness is action-owned. An explicit rule retains its original/custom message;
    /// a storage-only @unique uses the source generated action's unpunctuated diagnostic.
    pub fn uniqueness_error(&self, value: &Value) -> String {
        self.field
            .rules
            .iter()
            .find(|rule| rule.code == "unique")
            .map(|rule| super::checks::message(&self.field, rule, value))
            .unwrap_or_else(|| "Already exists".into())
    }
    /// Borrow the unchanged legacy scalar codec for shared primitive conversions.
    pub(super) fn codec(&self) -> &Column {
        &self.codec
    }
    /// Resolve an embedded schema lazily, avoiding recursive construction/default calls.
    pub(super) fn embedded(&self, seed: String, offset: i32) -> Option<SourceRecordSchema> {
        let crate::ValueType::Fieldset(name) = &self.field.value_type else {
            return None;
        };
        let model = self
            .catalog
            .fieldsets
            .iter()
            .find(|model| model.name == *name)?;
        Some(SourceRecordSchema::from_metadata(
            model,
            self.catalog.clone(),
            seed,
            offset,
        ))
    }
}

/// Immutable generated-schema state, retaining ordered native records and explicit providers.
#[derive(Clone)]
pub struct SourceRecordSchema {
    /// Source storage-column inventory emitted by the public generator, immutable after creation.
    columns: Vec<SourceColumn>,
    /// Application-owned private codec seed, retained only on the native schema instance.
    seed: String,
    /// Explicit source local Date offset; no global timezone changes.
    offset: i32,
    /// Owned provider whose calls occur only when defaults are requested.
    providers: Arc<dyn ValueProviders>,
}
impl SourceRecordSchema {
    /// Bind generated sibling enum/fieldset definitions without any provider or database IO.
    pub fn with_catalog(mut self, catalog: crate::SourceCatalog) -> Self {
        let catalog = Arc::new(catalog);
        for column in &mut self.columns {
            column.catalog = catalog.clone();
        }
        self
    }
    /// Build a nested schema from emitted metadata, with no handwritten field inventory.
    pub fn from_metadata(
        model: &crate::Model,
        catalog: Arc<crate::SourceCatalog>,
        seed: String,
        offset: i32,
    ) -> Self {
        let columns = model
            .fields
            .iter()
            .filter(|field| !matches!(field.value_type, crate::ValueType::Model(_)))
            .map(|field| {
                let kind = match field.value_type {
                    crate::ValueType::String if field.multiple => super::Kind::Strings,
                    crate::ValueType::String
                        if field.attributes.iter().any(|attr| attr.name == "encrypted") =>
                    {
                        super::Kind::Token
                    }
                    crate::ValueType::String
                        if field.attributes.iter().any(|attr| attr.name == "hashed") =>
                    {
                        super::Kind::Secret
                    }
                    crate::ValueType::Boolean => super::Kind::Boolean,
                    crate::ValueType::Date
                    | crate::ValueType::Time
                    | crate::ValueType::Datetime => super::Kind::Date,
                    crate::ValueType::Object | crate::ValueType::Fieldset(_) => super::Kind::Object,
                    _ => super::Kind::String,
                };
                let mut column = SourceColumn::new(
                    Column {
                        name: "",
                        sql: "",
                        kind,
                        nullable: !field.required && !field.multiple,
                        required_message: None,
                        minimum: None,
                        default: None,
                    },
                    field.clone(),
                );
                column.catalog = catalog.clone();
                column
            })
            .collect();
        Self::new(columns, seed, offset)
    }
    /// Borrow the immutable generated column inventory.
    pub fn columns(&self) -> &[SourceColumn] {
        &self.columns
    }
    /// Construct only owned codec metadata; no defaults, database or registration IO runs.
    pub fn new(columns: Vec<SourceColumn>, seed: String, offset: i32) -> Self {
        Self {
            columns,
            seed,
            offset,
            providers: Arc::new(BuiltinProviders),
        }
    }
    /// Select an application-owned provider without changing generated metadata.
    pub fn with_providers(mut self, providers: Arc<dyn ValueProviders>) -> Self {
        self.providers = providers;
        self
    }
    /// Borrow the private native codec seed.
    pub fn seed(&self) -> &str {
        &self.seed
    }
    /// Read the explicit source local offset.
    pub fn offset_minutes(&self) -> i32 {
        self.offset
    }
    /// Resolve a generated storage column by its exact source name.
    pub fn column(&self, name: &str) -> Option<&SourceColumn> {
        self.columns.iter().find(|column| column.name() == name)
    }
    /// Resolve fresh defaults in source column order.
    pub fn defaults_at(&self, now: i64) -> SchemaResult<Record> {
        self.columns
            .iter()
            .filter_map(|column| {
                match column.defaults_at(now, self.providers.as_ref(), self.offset) {
                    Ok(Some(value)) => Some(Ok((column.name().into(), value))),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                }
            })
            .collect()
    }
    /// Resolve the source getter against the native clock.
    pub fn defaults(&self) -> SchemaResult<Record> {
        self.defaults_at(super::now())
    }
    /// Keep declared storage fields; optional population uses fresh defaults before input.
    pub fn filter(&self, input: &Record, populate: bool) -> SchemaResult<Record> {
        let filtered = input
            .iter()
            .filter(|(name, _)| self.column(name).is_some())
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
        if populate {
            self.populate(&filtered)
        } else {
            Ok(filtered)
        }
    }
    /// Spread defaults then the original input, retaining extension members and insertion order.
    pub fn populate(&self, input: &Record) -> SchemaResult<Record> {
        let mut result = self.defaults()?;
        result.extend(input.clone());
        Ok(result)
    }
    /// Return only the first source error per selected field, in declaration order.
    pub fn assert(&self, input: &Record, required: bool) -> Option<Record> {
        self.assert_at(input, required, super::now())
    }
    /// Assert with a supplied clock for reproducible future/past/present rules.
    pub fn assert_at(&self, input: &Record, required: bool, now: i64) -> Option<Record> {
        let errors: Record = self
            .columns
            .iter()
            .filter(|column| required || input.contains_key(column.name()))
            .filter_map(|column| {
                column
                    .assert_value_at(input.get(column.name()), now, self.offset)
                    .map(|error| (column.name().into(), error))
            })
            .collect();
        (!errors.is_empty()).then_some(errors)
    }
    /// Prepare or storage-encode every generated column, removing undefined only.
    pub fn serialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> {
        self.convert(input, storage, true)
    }
    /// Decode native columns, preserving source omitted-list behavior and hash readback.
    pub fn unserialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> {
        self.convert(input, storage, false)
    }
    /// Traverse one inventory for both directions without projecting through a strict DTO.
    fn convert(&self, input: &Record, storage: bool, encode: bool) -> SchemaResult<Record> {
        self.columns
            .iter()
            .filter_map(|column| {
                let name = column.name();
                let result = if encode {
                    column.serialize(input.get(name), &self.seed, storage, self.offset)
                } else {
                    column.unserialize(input.get(name), &self.seed, storage, self.offset)
                };
                match result {
                    Ok(Some(value)) => {
                        // Original schema.ts JSON.stringify wraps fieldset-column serialization
                        // again. Keep that source storage spelling rather than silently fixing it.
                        let value = if encode
                            && matches!(column.field.value_type, crate::ValueType::Fieldset(_))
                        {
                            serde_json::to_string(&value)
                                .map(Value::String)
                                .map_err(|cause| {
                                    crate::SchemaError::new(
                                        crate::SchemaErrorKind::Codec,
                                        "fieldset schema encoding failed",
                                    )
                                    .with_cause(cause)
                                })
                        } else {
                            Ok(value)
                        };
                        Some(value.map(|value| (name.into(), value)))
                    }
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                }
            })
            .collect()
    }
}
