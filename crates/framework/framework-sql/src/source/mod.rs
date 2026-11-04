//! Compile-time source model families over explicitly owned or exclusively borrowed executors.
//!
//! No server, default model, ambient transaction or registration is selected here. Generated
//! implementations own metadata/query/codec policy; handwritten plugins keep their flows.
//! Standalone errors/statuses remain associated types; framework families share ActionError.
//!
//! The `source` feature selects native family bounds and mechanics. An API family inherits its session
//! family, including the very same Profile type. This is a type equality, not a runtime lookup:
//!
//! ```
//! # #[cfg(all(not(target_arch = "wasm32"), feature = "source"))] {
//! use stackpress_framework_sql::source::{ApiModels, SessionModels, SourceModel};
//!
//! fn profile_name<M: ApiModels>() -> String {
//!     <M as SessionModels>::Profile::describe().name
//! }
//! # }
//! ```

use stackpress_framework_schema::source::SourceSchema;
#[cfg(feature = "source")]
use stackpress_framework_schema::source::{
    ApplicationFields, AuthFields, ProfileFields, SessionFields,
};
use stackpress_sql::{Alter, Create, Delete, DialectId, Engine, Insert, Select, SqlValue, Update};
use std::{error::Error, sync::Arc};

#[cfg(feature = "source")]
mod error;
#[cfg(feature = "source")]
pub use error::{ActionError, Result as SourceResult};
mod actions;
#[cfg(feature = "source")]
mod values;
pub use actions::{ActionQuery, ActionRecord, ActionResult, SourceActions};
#[cfg(feature = "source")]
pub use values::to_sql_value;

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
    /// This trait returns a bare SQL AST. With `source`, render it through `SourceStatement` for
    /// G46 PostgreSQL ILIKE spelling; native actions already use that wrapper.
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

/// A native family shares the schema owner's Value/Record and this owner's ActionError.
/// Wrong core declarations fail through the generated field witnesses, without repair.
#[cfg(feature = "source")]
pub trait SessionModels: Send + Sync + 'static {
    /// Complete generated Profile, including application extensions.
    type Profile: SourceModel<
            Role = ProfileRole,
            Schema: ProfileFields,
            Store: SourceStore<Error = ActionError>,
        >;
    /// Complete generated Auth with the very same native record and error family.
    type Auth: SourceModel<Role = AuthRole, Schema: AuthFields, Store: SourceStore<Error = ActionError>>;
}
/// API inherits exactly one Profile/Auth family; session-only implementations need no API tables.
#[cfg(feature = "source")]
pub trait ApiModels: SessionModels {
    /// Complete generated Application.
    type Application: SourceModel<
            Role = ApplicationRole,
            Schema: ApplicationFields,
            Store: SourceStore<Error = ActionError>,
        >;
    /// Complete generated API Session (distinct from JWT identity).
    type Session: SourceModel<
            Role = ApiSessionRole,
            Schema: SessionFields,
            Store: SourceStore<Error = ActionError>,
        >;
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

#[cfg(feature = "source")]
mod statement;
#[cfg(feature = "source")]
pub use statement::SourceStatement;
#[cfg(feature = "source")]
mod store;
#[cfg(feature = "source")]
pub use store::NativeStore;
#[cfg(feature = "source")]
mod query;
#[cfg(feature = "source")]
pub use query::StoreSelector;
#[cfg(feature = "source")]
mod query_input;
#[cfg(feature = "source")]
pub use query_input::StoreQuery;
#[cfg(feature = "source")]
mod membership;
#[cfg(feature = "source")]
mod native_actions;
#[cfg(feature = "source")]
mod predicates;
#[cfg(feature = "source")]
mod select;
#[cfg(feature = "source")]
mod text;
#[cfg(feature = "source")]
pub use native_actions::NativeActions;

#[cfg(feature = "source")]
mod batch;
#[cfg(feature = "source")]
mod native_contracts;
#[cfg(feature = "source")]
pub use batch::BatchStatus;

#[cfg(all(feature = "source", feature = "server"))]
mod event_defaults;
#[cfg(all(feature = "source", feature = "server"))]
mod events;
#[cfg(all(feature = "source", feature = "server"))]
pub use events::{Event, SourceEnvironment, action, register};
/// Stable support paths for generated registrars; no direct server dependency is required by apps.
#[cfg(all(feature = "source", feature = "server"))]
#[doc(hidden)]
pub mod __private {
    pub use stackpress_lib::Result as ServerResult;
    pub use stackpress_server::ServerBuilder;
}

#[cfg(feature = "source")]
mod maintenance;
