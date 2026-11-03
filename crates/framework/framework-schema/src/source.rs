//! Native generated-schema boundary; source records are not converted through strict JSON DTOs.
//!
//! Low-level implementations associate owned value/record types; framework role families
//! require the shared native Value/Record. Dates, missing/null and insertion order survive
//! without a build-host dependency in the runtime graph.
//! These contracts add no validation policy; implementations preserve source preparation,
//! errors, defaults and storage codecs. No concrete framework models are supplied here.
//!
//! Low-level traits are native-only. The `source` feature adds reusable mechanics and
//! complete core-field bounds. Generated accessors use the actual record, not a second DTO:
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

/// Exact source field name `id`; generated declarations supply its type.
pub enum Id {}
/// Exact source field name `image`; generated declarations supply its type.
pub enum Image {}
/// Exact source field name `name`; generated declarations supply its type.
pub enum Name {}
/// Exact source field name `type`; generated declarations supply its type.
pub enum Type {}
/// Exact source field name `roles`; generated declarations supply its type.
pub enum Roles {}
/// Exact source field name `tags`; generated declarations supply its type.
pub enum Tags {}
/// Exact source field name `references`; generated declarations supply its type.
pub enum References {}
/// Exact source field name `active`; generated declarations supply its type.
pub enum Active {}
/// Exact source field name `created`; generated declarations supply its type.
pub enum Created {}
/// Exact source field name `updated`; generated declarations supply its type.
pub enum Updated {}
/// Exact source field name `auth`; generated declarations supply its type.
pub enum Auth {}
/// Exact source field name `applications`; generated declarations supply its type.
pub enum Applications {}
/// Exact source field name `sessions`; generated declarations supply its type.
pub enum Sessions {}
/// Exact source field name `profileId`; generated declarations supply its type.
pub enum ProfileId {}
/// Exact source field name `token`; generated declarations supply its type.
pub enum Token {}
/// Exact source field name `secret`; generated declarations supply its type.
pub enum Secret {}
/// Exact source field name `verified`; generated declarations supply its type.
pub enum Verified {}
/// Exact source field name `consumed`; generated declarations supply its type.
pub enum Consumed {}
/// Exact source field name `profile`; generated declarations supply its type.
pub enum Profile {}
/// Exact source field name `logo`; generated declarations supply its type.
pub enum Logo {}
/// Exact source field name `website`; generated declarations supply its type.
pub enum Website {}
/// Exact source field name `scopes`; generated declarations supply its type.
pub enum Scopes {}
/// Exact source field name `expires`; generated declarations supply its type.
pub enum Expires {}
/// Exact source field name `session`; generated declarations supply its type.
pub enum Session {}
/// Exact source field name `applicationId`; generated declarations supply its type.
pub enum ApplicationId {}
/// Exact source field name `application`; generated declarations supply its type.
pub enum Application {}
#[cfg(feature = "source")]
mod codecs;
#[cfg(feature = "source")]
mod columns;
#[cfg(feature = "source")]
mod dates;
#[cfg(feature = "source")]
mod value;
#[cfg(feature = "source")]
pub use codecs::{SourceCodecs, date_string};
#[cfg(feature = "source")]
pub use columns::{Column, Kind, as_date, date, date_with_display, string, to_json, truthy};
#[cfg(feature = "source")]
pub use dates::{date_millis, date_text, iso};
#[cfg(feature = "source")]
pub use value::Value;
/// Insertion-ordered native source fields; absence and explicit null are distinct.
#[cfg(feature = "source")]
pub type Record = stackpress_lib::DataMap<String, Value>;
/// Source Date.toString range failure; legacy session wrappers retain their error type.
#[cfg(feature = "source")]
#[derive(Debug)]
pub struct DateError(pub &'static str);
#[cfg(feature = "source")]
impl std::fmt::Display for DateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
#[cfg(feature = "source")]
impl std::error::Error for DateError {}
/// Native source clock, without a process-global provider or timezone.
#[cfg(feature = "source")]
pub fn now() -> i64 {
    (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64
}

#[cfg(feature = "source")]
mod fields;
#[cfg(feature = "source")]
pub use fields::{Date, NativeField};
#[cfg(feature = "source")]
mod declarations;
#[cfg(feature = "source")]
pub use declarations::*;
#[cfg(feature = "source")]
mod providers;
#[cfg(feature = "source")]
pub use providers::default_value;

/// Generated Profile core contract, including stored fields and source relation reads.
#[cfg(feature = "source")]
pub trait ProfileFields:
    SourceSchema<Value = Value, Record = Record>
    + SourceField<Id, FieldValue = String>
    + SourceDeclaration<Id, Declaration = StringType>
    + SourceField<Image, FieldValue = Option<String>>
    + SourceDeclaration<Image, Declaration = Nullable<StringType>>
    + SourceField<Name, FieldValue = String>
    + SourceDeclaration<Name, Declaration = StringType>
    + SourceField<Type, FieldValue = String>
    + SourceDeclaration<Type, Declaration = StringType>
    + SourceField<Roles, FieldValue = Vec<String>>
    + SourceDeclaration<Roles, Declaration = List<StringType>>
    + SourceField<Tags, FieldValue = Vec<String>>
    + SourceDeclaration<Tags, Declaration = List<StringType>>
    + SourceField<References, FieldValue = Option<Record>>
    + SourceDeclaration<References, Declaration = Nullable<ObjectType>>
    + SourceField<Active, FieldValue = bool>
    + SourceDeclaration<Active, Declaration = BooleanType>
    + SourceField<Created, FieldValue = Date>
    + SourceDeclaration<Created, Declaration = DatetimeType>
    + SourceField<Updated, FieldValue = Date>
    + SourceDeclaration<Updated, Declaration = DatetimeType>
    + SourceField<Auth, FieldValue = Vec<Record>>
    + SourceDeclaration<Auth, Declaration = List<AuthType>>
    + SourceField<Applications, FieldValue = Vec<Record>>
    + SourceDeclaration<Applications, Declaration = List<ApplicationType>>
    + SourceField<Sessions, FieldValue = Vec<Record>>
    + SourceDeclaration<Sessions, Declaration = List<SessionType>>
{
}
#[cfg(feature = "source")]
impl<S> ProfileFields for S where
    S: SourceSchema<Value = Value, Record = Record>
        + SourceField<Id, FieldValue = String>
        + SourceDeclaration<Id, Declaration = StringType>
        + SourceField<Image, FieldValue = Option<String>>
        + SourceDeclaration<Image, Declaration = Nullable<StringType>>
        + SourceField<Name, FieldValue = String>
        + SourceDeclaration<Name, Declaration = StringType>
        + SourceField<Type, FieldValue = String>
        + SourceDeclaration<Type, Declaration = StringType>
        + SourceField<Roles, FieldValue = Vec<String>>
        + SourceDeclaration<Roles, Declaration = List<StringType>>
        + SourceField<Tags, FieldValue = Vec<String>>
        + SourceDeclaration<Tags, Declaration = List<StringType>>
        + SourceField<References, FieldValue = Option<Record>>
        + SourceDeclaration<References, Declaration = Nullable<ObjectType>>
        + SourceField<Active, FieldValue = bool>
        + SourceDeclaration<Active, Declaration = BooleanType>
        + SourceField<Created, FieldValue = Date>
        + SourceDeclaration<Created, Declaration = DatetimeType>
        + SourceField<Updated, FieldValue = Date>
        + SourceDeclaration<Updated, Declaration = DatetimeType>
        + SourceField<Auth, FieldValue = Vec<Record>>
        + SourceDeclaration<Auth, Declaration = List<AuthType>>
        + SourceField<Applications, FieldValue = Vec<Record>>
        + SourceDeclaration<Applications, Declaration = List<ApplicationType>>
        + SourceField<Sessions, FieldValue = Vec<Record>>
        + SourceDeclaration<Sessions, Declaration = List<SessionType>>
{
}

/// Generated Auth core contract, including stored fields and source relation reads.
#[cfg(feature = "source")]
pub trait AuthFields:
    SourceSchema<Value = Value, Record = Record>
    + SourceField<Id, FieldValue = String>
    + SourceDeclaration<Id, Declaration = StringType>
    + SourceField<ProfileId, FieldValue = String>
    + SourceDeclaration<ProfileId, Declaration = StringType>
    + SourceField<Type, FieldValue = String>
    + SourceDeclaration<Type, Declaration = StringType>
    + SourceField<Token, FieldValue = String>
    + SourceDeclaration<Token, Declaration = StringType>
    + SourceField<Secret, FieldValue = String>
    + SourceDeclaration<Secret, Declaration = StringType>
    + SourceField<Verified, FieldValue = bool>
    + SourceDeclaration<Verified, Declaration = BooleanType>
    + SourceField<Consumed, FieldValue = Date>
    + SourceDeclaration<Consumed, Declaration = DatetimeType>
    + SourceField<Active, FieldValue = bool>
    + SourceDeclaration<Active, Declaration = BooleanType>
    + SourceField<Created, FieldValue = Date>
    + SourceDeclaration<Created, Declaration = DatetimeType>
    + SourceField<Updated, FieldValue = Date>
    + SourceDeclaration<Updated, Declaration = DatetimeType>
    + SourceField<Profile, FieldValue = Record>
    + SourceDeclaration<Profile, Declaration = ProfileType>
{
}
#[cfg(feature = "source")]
impl<S> AuthFields for S where
    S: SourceSchema<Value = Value, Record = Record>
        + SourceField<Id, FieldValue = String>
        + SourceDeclaration<Id, Declaration = StringType>
        + SourceField<ProfileId, FieldValue = String>
        + SourceDeclaration<ProfileId, Declaration = StringType>
        + SourceField<Type, FieldValue = String>
        + SourceDeclaration<Type, Declaration = StringType>
        + SourceField<Token, FieldValue = String>
        + SourceDeclaration<Token, Declaration = StringType>
        + SourceField<Secret, FieldValue = String>
        + SourceDeclaration<Secret, Declaration = StringType>
        + SourceField<Verified, FieldValue = bool>
        + SourceDeclaration<Verified, Declaration = BooleanType>
        + SourceField<Consumed, FieldValue = Date>
        + SourceDeclaration<Consumed, Declaration = DatetimeType>
        + SourceField<Active, FieldValue = bool>
        + SourceDeclaration<Active, Declaration = BooleanType>
        + SourceField<Created, FieldValue = Date>
        + SourceDeclaration<Created, Declaration = DatetimeType>
        + SourceField<Updated, FieldValue = Date>
        + SourceDeclaration<Updated, Declaration = DatetimeType>
        + SourceField<Profile, FieldValue = Record>
        + SourceDeclaration<Profile, Declaration = ProfileType>
{
}

/// Generated Application core contract, including stored fields and source relation reads.
#[cfg(feature = "source")]
pub trait ApplicationFields:
    SourceSchema<Value = Value, Record = Record>
    + SourceField<Id, FieldValue = String>
    + SourceDeclaration<Id, Declaration = StringType>
    + SourceField<ProfileId, FieldValue = String>
    + SourceDeclaration<ProfileId, Declaration = StringType>
    + SourceField<Name, FieldValue = String>
    + SourceDeclaration<Name, Declaration = StringType>
    + SourceField<Logo, FieldValue = Option<String>>
    + SourceDeclaration<Logo, Declaration = Nullable<StringType>>
    + SourceField<Website, FieldValue = Option<String>>
    + SourceDeclaration<Website, Declaration = Nullable<StringType>>
    + SourceField<Secret, FieldValue = String>
    + SourceDeclaration<Secret, Declaration = StringType>
    + SourceField<Scopes, FieldValue = Vec<String>>
    + SourceDeclaration<Scopes, Declaration = List<StringType>>
    + SourceField<Active, FieldValue = bool>
    + SourceDeclaration<Active, Declaration = BooleanType>
    + SourceField<Expires, FieldValue = Date>
    + SourceDeclaration<Expires, Declaration = DatetimeType>
    + SourceField<Created, FieldValue = Date>
    + SourceDeclaration<Created, Declaration = DatetimeType>
    + SourceField<Updated, FieldValue = Date>
    + SourceDeclaration<Updated, Declaration = DatetimeType>
    + SourceField<Session, FieldValue = Vec<Record>>
    + SourceDeclaration<Session, Declaration = List<SessionType>>
    + SourceField<Profile, FieldValue = Record>
    + SourceDeclaration<Profile, Declaration = ProfileType>
{
}
#[cfg(feature = "source")]
impl<S> ApplicationFields for S where
    S: SourceSchema<Value = Value, Record = Record>
        + SourceField<Id, FieldValue = String>
        + SourceDeclaration<Id, Declaration = StringType>
        + SourceField<ProfileId, FieldValue = String>
        + SourceDeclaration<ProfileId, Declaration = StringType>
        + SourceField<Name, FieldValue = String>
        + SourceDeclaration<Name, Declaration = StringType>
        + SourceField<Logo, FieldValue = Option<String>>
        + SourceDeclaration<Logo, Declaration = Nullable<StringType>>
        + SourceField<Website, FieldValue = Option<String>>
        + SourceDeclaration<Website, Declaration = Nullable<StringType>>
        + SourceField<Secret, FieldValue = String>
        + SourceDeclaration<Secret, Declaration = StringType>
        + SourceField<Scopes, FieldValue = Vec<String>>
        + SourceDeclaration<Scopes, Declaration = List<StringType>>
        + SourceField<Active, FieldValue = bool>
        + SourceDeclaration<Active, Declaration = BooleanType>
        + SourceField<Expires, FieldValue = Date>
        + SourceDeclaration<Expires, Declaration = DatetimeType>
        + SourceField<Created, FieldValue = Date>
        + SourceDeclaration<Created, Declaration = DatetimeType>
        + SourceField<Updated, FieldValue = Date>
        + SourceDeclaration<Updated, Declaration = DatetimeType>
        + SourceField<Session, FieldValue = Vec<Record>>
        + SourceDeclaration<Session, Declaration = List<SessionType>>
        + SourceField<Profile, FieldValue = Record>
        + SourceDeclaration<Profile, Declaration = ProfileType>
{
}

/// Generated Session core contract, including stored fields and source relation reads.
#[cfg(feature = "source")]
pub trait SessionFields:
    SourceSchema<Value = Value, Record = Record>
    + SourceField<Id, FieldValue = String>
    + SourceDeclaration<Id, Declaration = StringType>
    + SourceField<ApplicationId, FieldValue = String>
    + SourceDeclaration<ApplicationId, Declaration = StringType>
    + SourceField<ProfileId, FieldValue = String>
    + SourceDeclaration<ProfileId, Declaration = StringType>
    + SourceField<Secret, FieldValue = String>
    + SourceDeclaration<Secret, Declaration = StringType>
    + SourceField<Scopes, FieldValue = Vec<String>>
    + SourceDeclaration<Scopes, Declaration = List<StringType>>
    + SourceField<Active, FieldValue = bool>
    + SourceDeclaration<Active, Declaration = BooleanType>
    + SourceField<Expires, FieldValue = Date>
    + SourceDeclaration<Expires, Declaration = DatetimeType>
    + SourceField<Created, FieldValue = Date>
    + SourceDeclaration<Created, Declaration = DatetimeType>
    + SourceField<Updated, FieldValue = Date>
    + SourceDeclaration<Updated, Declaration = DatetimeType>
    + SourceField<Application, FieldValue = Record>
    + SourceDeclaration<Application, Declaration = ApplicationType>
    + SourceField<Profile, FieldValue = Record>
    + SourceDeclaration<Profile, Declaration = ProfileType>
{
}
#[cfg(feature = "source")]
impl<S> SessionFields for S where
    S: SourceSchema<Value = Value, Record = Record>
        + SourceField<Id, FieldValue = String>
        + SourceDeclaration<Id, Declaration = StringType>
        + SourceField<ApplicationId, FieldValue = String>
        + SourceDeclaration<ApplicationId, Declaration = StringType>
        + SourceField<ProfileId, FieldValue = String>
        + SourceDeclaration<ProfileId, Declaration = StringType>
        + SourceField<Secret, FieldValue = String>
        + SourceDeclaration<Secret, Declaration = StringType>
        + SourceField<Scopes, FieldValue = Vec<String>>
        + SourceDeclaration<Scopes, Declaration = List<StringType>>
        + SourceField<Active, FieldValue = bool>
        + SourceDeclaration<Active, Declaration = BooleanType>
        + SourceField<Expires, FieldValue = Date>
        + SourceDeclaration<Expires, Declaration = DatetimeType>
        + SourceField<Created, FieldValue = Date>
        + SourceDeclaration<Created, Declaration = DatetimeType>
        + SourceField<Updated, FieldValue = Date>
        + SourceDeclaration<Updated, Declaration = DatetimeType>
        + SourceField<Application, FieldValue = Record>
        + SourceDeclaration<Application, Declaration = ApplicationType>
        + SourceField<Profile, FieldValue = Record>
        + SourceDeclaration<Profile, Declaration = ProfileType>
{
}
