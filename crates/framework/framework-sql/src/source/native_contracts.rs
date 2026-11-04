//! Contract forwards keep owned and borrowed actions on one implementation path.
use super::*;
use crate::DataSession;
use stackpress_framework_schema::source::{Record, SourceSchema, Value};
use stackpress_sql::{Alter, Create, Delete, DialectId, Engine, Insert, Select, SqlValue, Update};
use std::sync::Arc;
impl<S: SourceSchema<Value = Value, Record = Record>> SourceStore for NativeStore<S> {
    type Schema = S;
    type Query = StoreQuery;
    type Error = ActionError;
    fn new(seed: String, offset: i32) -> Self {
        Self::new(seed, offset)
    }
    fn schema(&self) -> &S {
        &self.schema
    }
    fn query(&self, input: &Record) -> SourceResult<StoreQuery> {
        StoreQuery::from_record(input)
    }
    fn create(&self) -> Create {
        self.create()
    }
    fn alter(&self, to: Option<&Create>) -> SourceResult<Alter> {
        self.alter(to)
    }
    fn scalarize(&self, input: &Record) -> SourceResult<Vec<(String, SqlValue)>> {
        self.scalarize(input)
    }
    fn unscalarize(&self, input: &Record) -> SourceResult<Record> {
        self.unscalarize(input)
    }
    fn select(&self, q: &StoreQuery, d: DialectId) -> SourceResult<Select> {
        self.select(q, d).map(|s| s.0)
    }
    fn insert(&self, r: &Record, d: DialectId) -> SourceResult<Insert> {
        self.insert(r, d)
    }
    fn update(&self, q: &StoreQuery, r: &mut Record, d: DialectId) -> SourceResult<Update> {
        self.update(q, r, d).map(|s| s.0)
    }
    fn delete(&self, q: &StoreQuery, d: DialectId) -> SourceResult<Delete> {
        self.delete(q, d).map(|s| s.0)
    }
}
impl<S: SourceSchema<Value = Value, Record = Record>> SourceActions for NativeActions<S> {
    type Store = NativeStore<S>;
    type BatchStatus = BatchStatus;
    fn new(engine: Arc<Engine>, store: Self::Store) -> Self {
        Self::new(engine, store)
    }
    fn engine(&self) -> &Engine {
        &self.engine
    }
    fn store(&self) -> &Self::Store {
        &self.store
    }
    async fn create_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a Record,
    ) -> SourceResult<Record> {
        self.create_in(db, input).await
    }
    async fn find_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> SourceResult<Option<Record>> {
        self.find_in(db, query).await
    }
    async fn find_all_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> SourceResult<Vec<Record>> {
        self.find_all_in(db, query).await
    }
    async fn count_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> SourceResult<u64> {
        self.count_in(db, query).await
    }
    async fn update_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
        input: &'a mut Record,
    ) -> SourceResult<Vec<Record>> {
        self.update_in(db, query, input).await
    }
    async fn delete_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> SourceResult<Vec<Record>> {
        self.delete_in(db, query).await
    }
    async fn upsert_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a mut Record,
    ) -> SourceResult<Option<Record>> {
        self.upsert_in(db, input).await
    }
    async fn remove_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> SourceResult<Vec<Record>> {
        if self.store.model.fields.iter().any(|field| field.active) {
            self.update_in(db, query, &mut self.active(false)).await
        } else {
            self.delete_in(db, query).await
        }
    }
    async fn restore_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> SourceResult<Vec<Record>> {
        self.update_in(db, query, &mut self.active(true)).await
    }
    async fn batch(&self, inputs: &mut [Record], mode: BatchMode) -> Vec<BatchStatus> {
        self.batch(inputs, mode).await
    }
    async fn install(&self) -> SourceResult<()> {
        self.install().await
    }
    async fn upgrade(&self, to: Option<&Create>) -> SourceResult<()> {
        if let Some(to) = to {
            self.upgrade(to).await
        } else {
            Ok(())
        }
    }
    async fn purge(&self, cascade: bool) -> SourceResult<()> {
        self.purge(cascade).await
    }
    async fn uninstall(&self) -> SourceResult<()> {
        self.uninstall().await
    }
}
