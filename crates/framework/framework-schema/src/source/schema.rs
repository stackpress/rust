//! Native schema execution over generated columns; no model names or field-name policy.
use super::{Column, Record, Value, default_value, string, to_json};
use crate::{
    Field, SchemaResult, ValueContext, ValueProviders, check_rule, providers::BuiltinProviders,
};
use std::sync::Arc;

/// One generated storage column and its complete source metadata.
#[derive(Clone, Debug)]
pub struct SourceColumn {
    /// Legacy-compatible conversion only; its name-based default getter is never exposed here.
    codec: Column,
    /// Exact composed field, including assertions, defaults, roles and source attributes.
    pub field: Field,
}
impl SourceColumn {
    /// Pair a generated codec with its exact source declaration.
    pub fn new(codec: Column, field: Field) -> Self {
        Self { codec, field }
    }
    /// Exact source field name, independent of its SQL/member spelling.
    pub fn name(&self) -> &'static str {
        self.codec.name
    }
    /// Generated SQL column identity.
    pub fn sql(&self) -> &'static str {
        self.codec.sql
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
        self.codec.serialize(value, seed, storage, offset)
    }
    /// Decode using source readback behavior, including hash retention and native Dates.
    pub fn unserialize(
        &self,
        value: Option<&Value>,
        seed: &str,
        storage: bool,
        offset: i32,
    ) -> SchemaResult<Option<Value>> {
        self.codec.unserialize(value, seed, storage, offset)
    }
    /// Resolve a getter default with an explicit clock, provider and local Date offset.
    pub fn defaults_at(
        &self,
        now: i64,
        providers: &dyn ValueProviders,
        offset: i32,
    ) -> SchemaResult<Option<Value>> {
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
        if let Some(error) = self.codec.assert(value) {
            return Some(error);
        }
        // Nullable source shapes return before refinements; false is a present Boolean.
        if !self.field.required && value.is_none_or(Value::is_null) {
            return None;
        }
        let value = value?;
        if !self.field.multiple
            && self.field.required
            && !self.field.rules.iter().any(|rule| rule.code == "required")
            && string(value).is_empty()
        {
            return Some("Value is required.".into());
        }
        for rule in &self.field.rules {
            // Zod string lengths count UTF-16 units, unlike the separate Typed validator.
            let valid = match rule.code.as_str() {
                "required" => !value.is_null() && !string(value).is_empty(),
                "cge" => value.as_str().is_some_and(|text| {
                    text.encode_utf16().count() as f64 >= rule.args[0].as_f64().unwrap_or(0.0)
                }),
                _ => to_json(value).is_ok_and(|value| {
                    check_rule(rule, &value, &ValueContext::default(), false).unwrap_or(false)
                }),
            };
            if !valid {
                let label = self
                    .field
                    .documentation
                    .label
                    .as_deref()
                    .unwrap_or(&self.field.name);
                let mut message = rule
                    .message
                    .replace("{{name}}", &self.field.name)
                    .replace("{{label}}", label);
                // Source shape.ts substitutes the first three args; String(undefined) is
                // the literal "undefined". It never interpolates the rejected input value.
                for index in 0..3 {
                    let arg = rule
                        .args
                        .get(index)
                        .map(|value| string(&Value::from(value.clone())))
                        .unwrap_or_else(|| "undefined".into());
                    if index == 0 {
                        message = message.replace("{{arg}}", &arg);
                    }
                    message = message.replace(&format!("{{{{arg{}}}}}", index + 1), &arg);
                }
                return Some(message);
            }
        }
        None
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
        let errors: Record = self
            .columns
            .iter()
            .filter(|column| required || input.contains_key(column.name()))
            .filter_map(|column| {
                column
                    .assert(input.get(column.name()))
                    .map(|error| (column.name().into(), Value::String(error)))
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
                    Ok(Some(value)) => Some(Ok((name.into(), value))),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                }
            })
            .collect()
    }
}
