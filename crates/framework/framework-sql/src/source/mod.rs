//! Compile-time source model families over explicitly owned or exclusively borrowed executors.
//!
//! No server, default model, ambient transaction or registration is selected here. Generated
//! implementations own metadata/query/codec policy; handwritten plugins keep their flows.
//! Source errors/statuses remain associated types, rather than strict public DTO envelopes.
//!
//! This native-only module needs no optional feature. An API family inherits its session
//! family, including the very same Profile type. This is a type equality, not a runtime lookup:
//!
//! ```
//! # #[cfg(not(target_arch = "wasm32"))] {
//! use stackpress_framework_sql::source::{ApiModels, SessionModels, SourceModel};
//!
//! fn profile_name<M: ApiModels>() -> String {
//!     <M as SessionModels>::Profile::describe().name
//! }
//! # }
//! ```

use stackpress_framework_schema::source::{Id, SourceField, SourceSchema};
use stackpress_sql::{Alter, Create, Delete, DialectId, Engine, Insert, Select, SqlValue, Update};
use std::{error::Error, sync::Arc};

mod actions;
pub use actions::{ActionQuery, ActionRecord, ActionResult, SourceActions};

/// Native generated store: pure SQL preparation and codecs, with no hidden engine or IO.
pub trait SourceStore: Send + Sync + Sized + 'static {
    /// The very same schema used by this model's actions and typed role accessors.
    type Schema: SourceSchema;
    /// Owned source query representation; source query semantics remain generated code.
    type Query: Clone + Send + Sync + 'static;
    /// Original model error shape/cause, including source field errors when applicable.
    type Error: Error + Send + Sync + 'static;

    /// Construct immutable model/codec state, without database or default-provider calls.
    fn new(seed: String, offset_minutes: i32) -> Self;
    /// Borrow the generated schema; no lock or global model lookup is needed.
    fn schema(&self) -> &Self::Schema;
    /// Build source query options from a native record, retaining source skips/errors.
    fn query(
        &self,
        input: &<Self::Schema as SourceSchema>::Record,
    ) -> Result<Self::Query, Self::Error>;
    /// Source table DDL; explicit setup callers choose G48 execution semantics.
    fn create(&self) -> Create;
    /// Source adjacent alteration; callers retain revision/force/ordering policy.
    fn alter(&self, to: Option<&Create>) -> Result<Alter, Self::Error>;
    /// Convert source values to declared SQL columns without routing through public DTOs.
    fn scalarize(
        &self,
        input: &<Self::Schema as SourceSchema>::Record,
    ) -> Result<Vec<(String, SqlValue)>, Self::Error>;
    /// Decode raw native source records using the declared storage/relationship mapping.
    fn unscalarize(
        &self,
        input: &<Self::Schema as SourceSchema>::Record,
    ) -> Result<<Self::Schema as SourceSchema>::Record, Self::Error>;
    /// Build source selectors, active policy and joins for the selected dialect.
    fn select(&self, query: &Self::Query, dialect: DialectId) -> Result<Select, Self::Error>;
    /// Build an insert with the original dialect return behavior.
    fn insert(
        &self,
        input: &<Self::Schema as SourceSchema>::Record,
        dialect: DialectId,
    ) -> Result<Insert, Self::Error>;
    /// Prepare a source update, including source-visible input mutation where specified.
    fn update(
        &self,
        query: &Self::Query,
        input: &mut <Self::Schema as SourceSchema>::Record,
        dialect: DialectId,
    ) -> Result<Update, Self::Error>;
    /// Build a source physical delete; no implicit transaction, retry or guard is added.
    fn delete(&self, query: &Self::Query, dialect: DialectId) -> Result<Delete, Self::Error>;
}

/// Generated model associates one schema, one store and the matching action implementation.
pub trait SourceModel: Send + Sync + 'static {
    /// Generated framework role, preventing a family from binding Auth as its Profile.
    type Role: Send + Sync + 'static;
    /// Complete application schema, including extra fields.
    type Schema: SourceSchema;
    /// Store bound to exactly this schema, not a parallel handwritten field catalog.
    type Store: SourceStore<Schema = Self::Schema>;
    /// Actions bound to exactly this store.
    type Actions: SourceActions<Store = Self::Store>;

    /// Inspect complete generated metadata without constructing an engine or model instance.
    fn describe() -> stackpress_framework_schema::Model {
        Self::Schema::describe()
    }

    /// Construct only the generated schema/codec state, without defaults or database work.
    fn schema(seed: String, offset_minutes: i32) -> Self::Schema {
        Self::Schema::new(seed, offset_minutes)
    }

    /// Construct actions from an application-owned engine and immutable generated store.
    fn actions(engine: Arc<Engine>, seed: String, offset_minutes: i32) -> Self::Actions {
        Self::Actions::new(engine, Self::Store::new(seed, offset_minutes))
    }
}

/// Minimal source session role family; ID types are checked from generated field accessors.
///
/// This M0 boundary does not freeze the complete core-role inventory. Before M1 the
/// generator must add witnesses for the remaining role fields; it must not infer them
/// from JSON or erase incompatible app overrides. No concrete default family is supplied.
pub trait SessionModels: Send + Sync + 'static {
    /// Application's generated Profile (the entire schema, not a scalar projection).
    type Profile: SourceModel<Role = ProfileRole, Schema: SourceField<Id, FieldValue = String>>;
    /// Application's generated Auth, preserving source storage codecs.
    type Auth: SourceModel<Role = AuthRole, Schema: SourceField<Id, FieldValue = String>>;
}

/// API composition inherits exactly one Profile/Auth family, so Profile cannot diverge.
///
/// API's source imports session models; API/aggregate families therefore supply all four.
/// Session-only families implement only SessionModels and need no hidden API tables.
pub trait ApiModels: SessionModels {
    /// Application's generated Application model.
    type Application: SourceModel<Role = ApplicationRole, Schema: SourceField<Id, FieldValue = String>>;
    /// Application's generated API Session model (distinct from the JWT session type).
    type Session: SourceModel<Role = ApiSessionRole, Schema: SourceField<Id, FieldValue = String>>;
}

/// Generated Profile role; a marker, not a handwritten model or table definition.
pub enum ProfileRole {}
/// Generated Auth role; authentication flows remain outside the model contract.
pub enum AuthRole {}
/// Generated API Application role.
pub enum ApplicationRole {}
/// Generated API Session role, distinct from native JWT sessions.
pub enum ApiSessionRole {}

/// Original source batch selection; only the generated implementation owns batch policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BatchMode {
    /// Create each input.
    Create,
    /// Update by the original ID/unique selector rules.
    Update,
    /// Original ID/unique-selector upsert.
    #[default]
    Upsert,
}
