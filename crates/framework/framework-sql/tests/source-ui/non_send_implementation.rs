//! A Send container cannot implement an action method whose suspended state contains Rc.
use stackpress_framework_sql::source::ActionError;
#[path = "../main/source_models/support.rs"]
mod support;
use stackpress_framework_sql::{DataSession, source::*};
use stackpress_sql::{Create, Engine};
use std::{rc::Rc, sync::Arc};
use support::{Query, Record};
type Good = <support::Model<support::CompileSpec<String>, ProfileRole> as SourceModel>::Actions;
struct Bad(Good);

// Only create_in changes; these mechanical forwards supply every other required method.
macro_rules! reads {
    ($name:ident,$output:ty) => {
        async fn $name<'a, 'db: 'a>(
            &'a self,
            db: &'a mut DataSession<'db>,
            query: &'a Query,
        ) -> Result<$output, ActionError> {
            self.0.$name(db, query).await
        }
    };
}
impl SourceActions for Bad {
    type Store = <Good as SourceActions>::Store;
    type BatchStatus = <Good as SourceActions>::BatchStatus;
    fn new(engine: Arc<Engine>, store: Self::Store) -> Self {
        Self(Good::new(engine, store))
    }
    fn engine(&self) -> &Engine {
        self.0.engine()
    }
    fn store(&self) -> &Self::Store {
        self.0.store()
    }
    async fn create_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a Record,
    ) -> Result<Record, ActionError> {
        let non_send = Rc::new(());
        let result = self.0.create_in(db, input).await;
        drop(non_send);
        result
    }
    reads!(find_in, Option<Record>);
    reads!(find_all_in, Vec<Record>);
    reads!(count_in, u64);
    reads!(delete_in, Vec<Record>);
    reads!(remove_in, Vec<Record>);
    reads!(restore_in, Vec<Record>);
    async fn update_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a Query,
        input: &'a mut Record,
    ) -> Result<Vec<Record>, ActionError> {
        self.0.update_in(db, query, input).await
    }
    async fn upsert_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a mut Record,
    ) -> Result<Option<Record>, ActionError> {
        self.0.upsert_in(db, input).await
    }
    async fn batch(&self, inputs: &mut [Record], mode: BatchMode) -> Vec<Self::BatchStatus> {
        self.0.batch(inputs, mode).await
    }
    async fn install(&self) -> Result<(), ActionError> {
        self.0.install().await
    }
    async fn upgrade(&self, to: Option<&Create>) -> Result<(), ActionError> {
        self.0.upgrade(to).await
    }
    async fn purge(&self, cascade: bool) -> Result<(), ActionError> {
        self.0.purge(cascade).await
    }
    async fn uninstall(&self) -> Result<(), ActionError> {
        self.0.uninstall().await
    }
}
fn main() {}
