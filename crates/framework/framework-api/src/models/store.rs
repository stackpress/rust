//! Default API constructor compatibility; all store mechanics use emitted descriptors.
use super::{ModelKind, Record, Result, Schema, StoreQuery, StoreSelector, Value};
use crate::sql::SourceStatement;
use stackpress_framework_sql::source::NativeStore;
use stackpress_sql::{
    Alter, Create, Delete, DialectId, Expr, Insert, Join, Select, SqlValue, Update,
};
use std::ops::Deref;
/// Detached generated-default store, with the existing direct constructor and schema path.
#[derive(Clone, Debug)]
pub struct Store<M: ModelKind> {
    /// Generated schema state, not a second field/relationship inventory.
    pub schema: Schema<M>,
    pub(crate) offset: i32,
}
impl<M: ModelKind> Deref for Store<M> {
    type Target = Schema<M>;
    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}
impl<M: ModelKind> Store<M> {
    /// Retain the existing seed argument; construction performs no defaults or database IO.
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            schema: <Schema<M> as stackpress_framework_schema::source::SourceSchema>::new(
                seed.into(),
                0,
            ),
            offset: 0,
        }
    }
    /// Explicit source Date offset, shared by every codec and predicate.
    pub fn with_offset_minutes(mut self, offset: i32) -> Self {
        self.offset = offset;
        self.schema = self.schema.with_offset_minutes(offset);
        self
    }
    /// Exact legacy SQL identity.
    pub fn table(&self) -> &'static str {
        M::TABLE
    }
    pub(super) fn native(&self) -> NativeStore<Schema<M>> {
        NativeStore::from_schema(self.schema.clone())
    }
    /// Source DDL from the generated graph, including source index/FK order.
    pub fn create(&self) -> Create {
        self.native().create()
    }
    /// Explicit adjacent definition diff, without introspection.
    pub fn alter(&self, to: Option<&Create>) -> Result<Alter> {
        self.native().alter(to)
    }
    /// Generated source storage encoding.
    pub fn scalarize(&self, input: &Record) -> Result<Vec<(String, SqlValue)>> {
        self.native().scalarize(input)
    }
    /// Generated source partial decode, preserving missing fields.
    pub fn unscalarize(&self, input: &Record) -> Result<Record> {
        self.native().unscalarize(input)
    }
    /// Source native scalar conversion.
    pub fn to_sql_value(&self, name: &str, value: &Value) -> Result<SqlValue> {
        self.native().to_sql_value(name, value)
    }
    /// Dialect-selected source insert return behavior.
    pub fn insert_for(&self, input: &Record, dialect: DialectId) -> Result<Insert> {
        self.native().insert(input, dialect)
    }
    /// Existing direct insertion requests RETURNING for supporting dialects.
    pub fn insert(&self, input: &Record) -> Result<Insert> {
        self.native().insert(input, DialectId::Postgres)
    }
    /// Source update retains its original input timestamp mutation.
    pub fn update(
        &self,
        q: &StoreQuery,
        input: &mut Record,
        dialect: DialectId,
    ) -> Result<SourceStatement<Update>> {
        self.native().update(q, input, dialect)
    }
    /// Source physical deletion with its original active/filter policy.
    pub fn delete(&self, q: &StoreQuery, dialect: DialectId) -> Result<SourceStatement<Delete>> {
        self.native().delete(q, dialect)
    }
    /// Source search selectors, joins, filters and G46 rendering.
    pub fn select(&self, q: &StoreQuery, dialect: DialectId) -> Result<SourceStatement<Select>> {
        self.native().select(q, dialect)
    }
    /// Source predicate compilation from generated metadata.
    pub fn where_clause(&self, q: &StoreQuery, dialect: DialectId) -> Result<Option<Expr>> {
        self.native().where_clause(q, dialect)
    }
    /// Source owning joins, without reverse relation invention.
    pub fn joins(&self, q: &StoreQuery) -> Vec<Join> {
        self.native().joins(q)
    }
    /// Resolve one source path through the generated catalog.
    pub fn paths(&self, path: &str) -> Vec<StoreSelector> {
        self.selectors(&[path.into()])
    }
    /// Preserve the legacy static selector carrier; generic app stores use open source names.
    pub fn selectors(&self, paths: &[String]) -> Vec<StoreSelector> {
        self.native()
            .selectors(paths)
            .into_iter()
            .map(|value| StoreSelector::from_native(value, M::NAME))
            .collect()
    }
}
/// Existing direct Application store path and constructor.
pub type ApplicationStore = Store<super::ApplicationModel>;
/// Existing direct persisted Session store path and constructor.
pub type SessionStore = Store<super::SessionModel>;
