//! Native generated-schema boundary; source records are not converted through strict JSON DTOs.
//!
//! A generated family chooses its owned value/record types. This permits native Dates,
//! missing/null and insertion order without a build-host dependency in the runtime graph.
//! These contracts add no validation policy; implementations preserve source preparation,
//! errors, defaults and storage codecs. No concrete framework models are supplied here.
//!
//! This module is native-only and needs no optional feature. A generated implementation
//! supplies the record and the field witness together, rather than a second role DTO:
//!
//! ```
//! # #[cfg(not(target_arch = "wasm32"))] {
//! use stackpress_framework_schema::source::{Id, SourceField, SourceSchema};
//!
//! fn identity<S: SourceSchema + SourceField<Id, FieldValue = String>>(
//!     record: &S::Record,
//! ) -> Option<String> {
//!     <S as SourceField<Id>>::read(record)
//! }
//! # }
//! ```

use crate::{Model, SchemaResult};

/// Schema operations emitted for one application model, with immutable seed/offset state.
///
/// Records own their contents and may cross await points. They need not implement Serde:
/// publication and transport conversion remain explicit operations at their existing owners.
pub trait SourceSchema: Send + Sync + Sized + 'static {
    /// Native value family, preserving Date separately from text and JSON objects.
    type Value: Clone + Send + Sync + 'static;
    /// Owned source record, including insertion order and application-added columns.
    type Record: Clone + Default + Send + Sync + 'static;

    /// Construct codec state without generating defaults, registering handlers or doing IO.
    fn new(seed: String, offset_minutes: i32) -> Self;
    /// Full generated metadata, not a projection onto framework-required core fields.
    fn describe() -> Model;
    /// Borrow the native storage seed; never publish schema instances in browser snapshots.
    fn seed(&self) -> &str;
    /// Explicit source Date conversion offset; no process-global timezone is implied.
    fn offset_minutes(&self) -> i32;
    /// Resolve fresh defaults at the supplied epoch milliseconds.
    fn defaults_at(&self, now: i64) -> SchemaResult<Self::Record>;
    /// Resolve defaults using the implementation's source-compatible native clock.
    fn defaults(&self) -> SchemaResult<Self::Record>;
    /// Keep declared stored members; optionally apply defaults as the source filter does.
    fn filter(&self, input: &Self::Record, populate: bool) -> SchemaResult<Self::Record>;
    /// Spread defaults then input, retaining extension members as the source populate does.
    fn populate(&self, input: &Self::Record) -> SchemaResult<Self::Record>;
    /// Original field-error record, or None; required selects all versus supplied columns.
    fn assert(&self, input: &Self::Record, required: bool) -> Option<Self::Record>;
    /// Prepare declared fields; storage selects the original hash/encryption boundary.
    fn serialize(&self, input: &Self::Record, storage: bool) -> SchemaResult<Self::Record>;
    /// Decode declared fields without guessing native Dates from arbitrary JSON spellings.
    fn unserialize(&self, input: &Self::Record, storage: bool) -> SchemaResult<Self::Record>;
}

/// Generated typed access to one plugin role field, tied to that schema's actual record.
///
/// `FieldValue` describes the declared field type. A missing or wrongly shaped unprepared
/// record returns None; this accessor adds no request validation or implicit coercion.
pub trait SourceField<Name>: SourceSchema {
    /// Declared Rust type; core family bounds reject incompatible application overrides.
    type FieldValue: Clone + Send + Sync + 'static;
    /// Read an owned field value without losing the rest of the extended record.
    fn read(record: &Self::Record) -> Option<Self::FieldValue>;
    /// Write that field while retaining every other member and its source record order.
    fn write(record: &mut Self::Record, value: Self::FieldValue);
}

/// Name marker for a model's string identity, shared by the session/API family contract.
///
/// Further role markers can be added without making records into fixed DTOs. The emitter
/// must derive their implementations from composed declarations, not handwritten models.
pub enum Id {}
