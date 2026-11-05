//! Default API schema compatibility over the shipped generated model declarations.
use super::{Record, Value};
use stackpress_framework_schema::{SchemaResult, ValueProviders, source::SourceSchema as Contract};
use stackpress_framework_session::columns::RecordSchema;
use std::{marker::PhantomData, ops::Deref};
/// Finite source model identity; implementations supply owned schema columns.
pub trait ModelKind: Clone + Send + Sync + 'static {
    /// Exact source model name.
    const NAME: &'static str;
    /// Exact SQL table name.
    const TABLE: &'static str;
}
/// Source Application identity.
#[derive(Clone, Debug)]
pub struct ApplicationModel;
impl ModelKind for ApplicationModel {
    const NAME: &'static str = "Application";
    const TABLE: &'static str = "application";
}
/// Source persisted Session identity.
#[derive(Clone, Debug)]
pub struct SessionModel;
impl ModelKind for SessionModel {
    const NAME: &'static str = "Session";
    const TABLE: &'static str = "session";
}
/// Source Application schema, including plaintext generated secret.
pub type ApplicationSchema = Schema<ApplicationModel>;
/// Source API Session schema, including applicationId and profileId.
pub type SessionSchema = Schema<SessionModel>;

/// Generated default schema with the existing zero-argument API constructor.
#[derive(Clone, Debug)]
pub struct Schema<M: ModelKind> {
    pub(crate) record: RecordSchema,
    marker: PhantomData<M>,
}
impl<M: ModelKind> Default for Schema<M> {
    fn default() -> Self {
        Self::new()
    }
}
impl<M: ModelKind> Deref for Schema<M> {
    type Target = RecordSchema;
    fn deref(&self) -> &RecordSchema {
        &self.record
    }
}
impl<M: ModelKind> Schema<M> {
    /// Retain only immutable generated metadata; no defaults or database calls occur.
    pub fn new() -> Self {
        <Self as Contract>::new(String::new(), 0)
    }
    /// Complete generated model, without a synthetic Profile or relation patch.
    pub fn model(&self) -> stackpress_framework_schema::Model {
        self.record.model()
    }
    /// Explicit native local Date offset; process-global timezone state is never changed.
    pub fn with_offset_minutes(mut self, offset: i32) -> Self {
        self.record = self.record.with_offset_minutes(offset);
        self
    }
    /// Existing direct provider boundary, now selected by generated default metadata.
    pub fn defaults_with(&self, now: i64, providers: &dyn ValueProviders) -> SchemaResult<Record> {
        self.record.defaults_with(now, providers)
    }
}
impl<M: ModelKind> Contract for Schema<M> {
    type Value = Value;
    type Record = Record;
    fn new(seed: String, offset: i32) -> Self {
        let record = if M::NAME == "Application" {
            RecordSchema::from_generated::<stackpress_framework_models::ApplicationSchema>(
                seed, offset,
            )
        } else {
            RecordSchema::from_generated::<stackpress_framework_models::SessionSchema>(seed, offset)
        };
        Self {
            record,
            marker: PhantomData,
        }
    }
    fn describe() -> stackpress_framework_schema::Model {
        Self::new().model()
    }
    fn catalog() -> stackpress_framework_schema::SourceCatalog {
        Self::new().catalog().clone()
    }
    fn seed(&self) -> &str {
        self.record.seed()
    }
    fn offset_minutes(&self) -> i32 {
        self.record.offset_minutes()
    }
    fn defaults_at(&self, now: i64) -> SchemaResult<Record> {
        self.record.defaults_at(now)
    }
    fn defaults(&self) -> SchemaResult<Record> {
        self.record.defaults()
    }
    fn filter(&self, input: &Record, populate: bool) -> SchemaResult<Record> {
        self.record.filter(input, populate)
    }
    fn populate(&self, input: &Record) -> SchemaResult<Record> {
        self.record.populate(input)
    }
    fn assert(&self, input: &Record, required: bool) -> Option<Record> {
        self.record.assert(input, required)
    }
    fn serialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> {
        self.record.serialize(input, storage)
    }
    fn unserialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> {
        self.record.unserialize(input, storage)
    }
}
