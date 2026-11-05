//! Compatibility schema state over generated source descriptors; no field inventory lives here.
use super::{Record, Value};
use stackpress_framework_schema::{
    Model, SchemaResult, SourceCatalog, ValueProviders,
    source::{SourceColumn, SourceRecordSchema, SourceSchema},
};
use std::{ops::Deref, sync::Arc};

/// Owned native schema; its generated columns are inspected through accessor methods.
#[derive(Clone)]
pub struct RecordSchema {
    /// Exact emitted model name, without a runtime model registry.
    pub name: String,
    pub(crate) seed: String,
    pub(crate) offset: i32,
    model: Model,
    catalog: SourceCatalog,
    native: SourceRecordSchema,
}
impl std::fmt::Debug for RecordSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordSchema")
            .field("name", &self.name)
            .field("columns", &self.columns())
            .finish_non_exhaustive()
    }
}
impl RecordSchema {
    /// Bind the application's generated metadata without defaults, database IO or registration.
    pub fn from_generated<S: SourceSchema<Value = Value, Record = Record>>(
        seed: String,
        offset: i32,
    ) -> Self {
        let model = S::describe();
        let catalog = S::catalog();
        let native = SourceRecordSchema::from_metadata(
            &model,
            Arc::new(catalog.clone()),
            seed.clone(),
            offset,
        );
        Self {
            name: model.name.clone(),
            seed,
            offset,
            model,
            catalog,
            native,
        }
    }
    /// Exact storage columns in source order; reverse relations remain model metadata.
    pub fn columns(&self) -> &[SourceColumn] {
        self.native.columns()
    }
    /// Resolve an original source field name.
    pub fn column(&self, name: &str) -> Option<&SourceColumn> {
        self.native.column(name)
    }
    /// Complete generated declaration, including roles, rules and relations.
    pub fn describe(&self) -> &Model {
        &self.model
    }
    /// Complete composed generated catalog used by store joins.
    pub fn catalog(&self) -> &SourceCatalog {
        &self.catalog
    }
    /// Preserve the caller's explicit local Date offset without any global timezone state.
    pub fn with_offset_minutes(mut self, offset: i32) -> Self {
        self.offset = offset;
        self.native = SourceRecordSchema::from_metadata(
            &self.model,
            Arc::new(self.catalog.clone()),
            self.seed.clone(),
            offset,
        );
        self
    }
    /// Source defaults with a caller's provider, including generated API secrets.
    pub fn defaults_with(&self, now: i64, providers: &dyn ValueProviders) -> SchemaResult<Record> {
        self.columns()
            .iter()
            .filter_map(
                |column| match column.defaults_at(now, providers, self.offset) {
                    Ok(Some(value)) => Some(Ok((column.name().into(), value))),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                },
            )
            .collect()
    }
}
impl Deref for RecordSchema {
    type Target = SourceRecordSchema;
    fn deref(&self) -> &Self::Target {
        &self.native
    }
}
/// Native source clock, retaining the legacy private path used by auth/account logic.
#[cfg(feature = "sql")]
pub(crate) fn now() -> i64 {
    stackpress_framework_schema::source::now()
}
