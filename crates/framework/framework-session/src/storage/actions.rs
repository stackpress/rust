//! Constructor compatibility only; generated metadata and the lower runtime own CRUD behavior.
use super::{BatchMode, BatchStatus, ModelKind, Result, Store, StoreQuery};
use crate::columns::Record;
use stackpress_framework_sql::DataSession;
use stackpress_framework_sql::source::NativeActions;
use stackpress_sql::{Create, Engine};
use std::sync::Arc;
/// Direct actions retain only an app-owned Engine and detached generated store state.
#[derive(Clone)]
pub struct Actions<M: ModelKind> {
    /// Application-owned shared executor; construction opens no connection.
    pub engine: Arc<Engine>,
    /// Generated schema/store state, without an ambient transaction.
    pub store: Store<M>,
    pub(crate) password: super::PasswordConfig,
}
impl<M: ModelKind> Actions<M> {
    /// Retain existing constructor arity, without defaults, registration or database work.
    pub fn new(engine: Arc<Engine>, seed: impl Into<String>) -> Self {
        Self {
            engine,
            store: Store::new(seed),
            password: super::PasswordConfig::default(),
        }
    }
    fn native(&self) -> NativeActions<super::store::RuntimeSchema<M>> {
        NativeActions::new(self.engine.clone(), self.store.native())
    }
    /// Source create on the ordinary app-owned executor.
    pub async fn create(&self, input: &Record) -> Result<Record> {
        self.native().create(input).await
    }
    /// Source create on the caller's exact executor; the caller owns transaction completion.
    pub async fn create_in(&self, db: &mut DataSession<'_>, input: &Record) -> Result<Record> {
        self.native().create_in(db, input).await
    }
    /// Source upsert on the ordinary app-owned executor.
    pub async fn upsert(&self, input: &mut Record) -> Result<Option<Record>> {
        self.native().upsert(input).await
    }
    /// Source upsert on the caller's exact executor; the caller owns transaction completion.
    pub async fn upsert_in(
        &self,
        db: &mut DataSession<'_>,
        input: &mut Record,
    ) -> Result<Option<Record>> {
        self.native().upsert_in(db, input).await
    }
    /// Source find, retaining generated selectors, errors and native row semantics.
    pub async fn find(&self, query: &StoreQuery) -> Result<Option<Record>> {
        self.native().find(&query.native()).await
    }
    /// Same find on the caller's exact executor, without a hidden transaction.
    pub async fn find_in(
        &self,
        db: &mut DataSession<'_>,
        query: &StoreQuery,
    ) -> Result<Option<Record>> {
        self.native().find_in(db, &query.native()).await
    }
    /// Source find_all, retaining generated selectors, errors and native row semantics.
    pub async fn find_all(&self, query: &StoreQuery) -> Result<Vec<Record>> {
        self.native().find_all(&query.native()).await
    }
    /// Same find_all on the caller's exact executor, without a hidden transaction.
    pub async fn find_all_in(
        &self,
        db: &mut DataSession<'_>,
        query: &StoreQuery,
    ) -> Result<Vec<Record>> {
        self.native().find_all_in(db, &query.native()).await
    }
    /// Source count, retaining generated selectors, errors and native row semantics.
    pub async fn count(&self, query: &StoreQuery) -> Result<u64> {
        self.native().count(&query.native()).await
    }
    /// Same count on the caller's exact executor, without a hidden transaction.
    pub async fn count_in(&self, db: &mut DataSession<'_>, query: &StoreQuery) -> Result<u64> {
        self.native().count_in(db, &query.native()).await
    }
    /// Source delete, retaining generated selectors, errors and native row semantics.
    pub async fn delete(&self, query: &StoreQuery) -> Result<Vec<Record>> {
        self.native().delete(&query.native()).await
    }
    /// Same delete on the caller's exact executor, without a hidden transaction.
    pub async fn delete_in(
        &self,
        db: &mut DataSession<'_>,
        query: &StoreQuery,
    ) -> Result<Vec<Record>> {
        self.native().delete_in(db, &query.native()).await
    }
    /// Source remove, retaining generated selectors, errors and native row semantics.
    pub async fn remove(&self, query: &StoreQuery) -> Result<Vec<Record>> {
        self.native().remove(&query.native()).await
    }
    /// Source restore, retaining generated selectors, errors and native row semantics.
    pub async fn restore(&self, query: &StoreQuery) -> Result<Vec<Record>> {
        self.native().restore(&query.native()).await
    }
    /// Source update retains the original mutable input and pre-read result contract.
    pub async fn update(&self, query: &StoreQuery, input: &mut Record) -> Result<Vec<Record>> {
        self.native().update(&query.native(), input).await
    }
    /// Same update on the caller's exclusively borrowed executor.
    pub async fn update_in(
        &self,
        db: &mut DataSession<'_>,
        query: &StoreQuery,
        input: &mut Record,
    ) -> Result<Vec<Record>> {
        self.native().update_in(db, &query.native(), input).await
    }
    /// Explicit setup selects atomic PostgreSQL/SQLite and acknowledged non-atomic MySQL.
    pub async fn install(&self) -> Result<()> {
        self.native().install().await
    }
    /// Explicit adjacent upgrade preserves G48 progress and caller-owned revision policy.
    pub async fn upgrade(&self, to: &Create) -> Result<()> {
        self.native().upgrade(to).await
    }
    /// Explicit purge retains the accepted dialect/FK mappings, including MySQL DELETE.
    pub async fn purge(&self, cascade: bool) -> Result<()> {
        self.native().purge(cascade).await
    }
    /// Explicit uninstall affects only this generated table.
    pub async fn uninstall(&self) -> Result<()> {
        self.native().uninstall().await
    }
    /// Source batch status rows describe attempts, including rolled-back work.
    pub async fn batch(&self, input: &mut [Record], mode: BatchMode) -> Vec<BatchStatus> {
        self.native().batch(input, mode).await
    }
}
/// Direct generated-default Profile actions, retaining the old constructor.
pub type ProfileActions = Actions<super::ProfileModel>;
/// Direct Auth actions retain the handwritten account collaborator methods.
pub type AuthActions = Actions<super::AuthModel>;
