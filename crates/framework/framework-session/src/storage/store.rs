//! Legacy constructors over generated descriptors and the shared source store implementation.
use super::{Result, SourceStatement, StoreQuery, StoreSelector};
use crate::columns::{Record, RecordSchema, Value};
use stackpress_framework_sql::source::NativeStore;
use stackpress_sql::{
    Alter, Create, Delete, DialectId, Expr, Insert, Join, Select, SqlValue, Update,
};
use std::{marker::PhantomData, ops::Deref};
/// Closed model identity used by the two generated stores.
pub trait ModelKind: Clone + Send + Sync + 'static {
    /// Exact source schema name.
    const NAME: &'static str;
    /// Source SQL table name.
    const TABLE: &'static str;
    /// Construct the corresponding source schema.
    fn schema(seed: String) -> RecordSchema;
}
/// Profile's generated storage identity.
#[derive(Clone, Debug)]
pub struct ProfileModel;
impl ModelKind for ProfileModel {
    const NAME: &'static str = "Profile";
    const TABLE: &'static str = "profile";
    fn schema(seed: String) -> RecordSchema {
        crate::profile::ProfileSchema::new(seed).into_schema()
    }
}
/// Auth's generated storage identity.
#[derive(Clone, Debug)]
pub struct AuthModel;
impl ModelKind for AuthModel {
    const NAME: &'static str = "Auth";
    const TABLE: &'static str = "auth";
    fn schema(seed: String) -> RecordSchema {
        crate::auth::AuthSchema::new(seed).into_schema()
    }
}
/// Complete source Profile store.
pub type ProfileStore = Store<ProfileModel>;
/// Complete source Auth store, including its required inner Profile relation.
pub type AuthStore = Store<AuthModel>;

/// Compatibility state; generated metadata is the sole column/relation inventory.
#[derive(Clone, Debug)]
pub struct Store<M: ModelKind> {
    /// Generated schema state, independently usable without an Engine.
    pub schema: RecordSchema,
    marker: PhantomData<M>,
}
impl<M: ModelKind> Deref for Store<M> {
    type Target = RecordSchema;
    fn deref(&self) -> &RecordSchema {
        &self.schema
    }
}
/// Adapt only the legacy constructor contract; model mechanics remain in the lower owner.
#[derive(Clone)]
pub(super) struct RuntimeSchema<M: ModelKind>(RecordSchema, PhantomData<M>);

impl<M: ModelKind> stackpress_framework_schema::source::SourceSchema for RuntimeSchema<M> {
    type Value = Value;
    type Record = Record;
    fn new(seed: String, offset: i32) -> Self {
        Self(
            M::schema(seed).with_offset_minutes(offset),
            std::marker::PhantomData,
        )
    }
    fn describe() -> stackpress_framework_schema::Model {
        M::schema(String::new()).model()
    }
    fn catalog() -> stackpress_framework_schema::SourceCatalog {
        M::schema(String::new()).catalog().clone()
    }
    fn seed(&self) -> &str {
        self.0.seed()
    }
    fn offset_minutes(&self) -> i32 {
        self.0.offset_minutes()
    }
    fn defaults_at(&self, now: i64) -> stackpress_framework_schema::SchemaResult<Record> {
        self.0.defaults_at(now)
    }
    fn defaults(&self) -> stackpress_framework_schema::SchemaResult<Record> {
        self.0.defaults()
    }
    fn filter(
        &self,
        input: &Record,
        populate: bool,
    ) -> stackpress_framework_schema::SchemaResult<Record> {
        self.0.filter(input, populate)
    }
    fn populate(&self, input: &Record) -> stackpress_framework_schema::SchemaResult<Record> {
        self.0.populate(input)
    }
    fn assert(&self, input: &Record, required: bool) -> Option<Record> {
        self.0.assert(input, required)
    }
    fn serialize(
        &self,
        input: &Record,
        storage: bool,
    ) -> stackpress_framework_schema::SchemaResult<Record> {
        self.0.serialize(input, storage)
    }
    fn unserialize(
        &self,
        input: &Record,
        storage: bool,
    ) -> stackpress_framework_schema::SchemaResult<Record> {
        self.0.unserialize(input, storage)
    }
}

impl<M: ModelKind> Store<M> {
    /// Retain source constructor spelling without defaults, registration or database IO.
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            schema: M::schema(seed.into()),
            marker: PhantomData,
        }
    }
    /// Exact existing SQL table identity.
    pub fn table(&self) -> &'static str {
        M::TABLE
    }
    pub(super) fn native(&self) -> NativeStore<RuntimeSchema<M>> {
        NativeStore::from_schema(RuntimeSchema(self.schema.clone(), PhantomData))
    }
    /// Generated source table definition, including its source-grouped indexes.
    pub fn create(&self) -> Create {
        self.native().create()
    }
    /// Compare only caller-provided adjacent definitions; no introspection or database opens.
    pub fn alter(&self, to: Option<&Create>) -> Result<Alter> {
        self.native().alter(to)
    }
    /// Source storage codecs from generated declarations.
    pub fn scalarize(&self, input: &Record) -> Result<Vec<(String, SqlValue)>> {
        self.native().scalarize(input)
    }
    /// Partial native SQL decode preserves missing fields and insertion order.
    pub fn unscalarize(&self, input: &Record) -> Result<Record> {
        self.native().unscalarize(input)
    }
    /// Source scalar conversion selected by generated column metadata.
    pub fn to_sql_value(&self, name: &str, value: &Value) -> Result<SqlValue> {
        self.native().to_sql_value(name, value)
    }
    /// Insert with the selected dialect's original return behavior.
    pub fn insert_for(&self, input: &Record, dialect: DialectId) -> Result<Insert> {
        self.native().insert(input, dialect)
    }
    /// Original direct insertion requests RETURNING when rendered by a supporting dialect.
    pub fn insert(&self, input: &Record) -> Result<Insert> {
        self.native().insert(input, DialectId::Postgres)
    }
    /// Generated update preserves source input clock mutation.
    pub fn update(
        &self,
        q: &StoreQuery,
        input: &mut Record,
        dialect: DialectId,
    ) -> Result<SourceStatement<Update>> {
        self.native().update(&q.native(), input, dialect)
    }
    /// Generated physical delete, retaining source filters and active policy.
    pub fn delete(&self, q: &StoreQuery, dialect: DialectId) -> Result<SourceStatement<Delete>> {
        self.native().delete(&q.native(), dialect)
    }
    /// Source selection and G46 rendering; the legacy q adapter never broadens decoding.
    pub fn select(&self, q: &StoreQuery, dialect: DialectId) -> Result<SourceStatement<Select>> {
        self.native().select(&q.native(), dialect)
    }
    /// Source path filters and joins come from the complete generated graph.
    pub fn where_clause(&self, q: &StoreQuery, dialect: DialectId) -> Result<Option<Expr>> {
        self.native().where_clause(&q.native(), dialect)
    }
    /// Source owning joins in declaration order.
    pub fn joins(&self, q: &StoreQuery) -> Vec<Join> {
        self.native().joins(&q.native())
    }
    /// Resolve one legacy selector, retaining its compatibility profile flag.
    pub fn paths(&self, path: &str) -> Vec<StoreSelector> {
        self.selectors(&[path.into()])
    }
    /// Expand source selectors in declaration order, without a second path interpreter.
    pub fn selectors(&self, paths: &[String]) -> Vec<StoreSelector> {
        self.native()
            .selectors(paths)
            .into_iter()
            .map(|value| StoreSelector::from_native(value, M::NAME))
            .collect()
    }
}
