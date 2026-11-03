//! Probe action implementations exercise the real SQL engine/exclusive session contracts.

use super::{Query, Record, Value, decode, get, put};
use stackpress_framework_schema::source::SourceSchema;
use stackpress_framework_sql::{
    DataSession,
    source::{ActionError, BatchMode, SourceActions, SourceStore},
};
use stackpress_sql::{Create, Delete, Engine};
use std::sync::Arc;

/// Immutable action state is shared by Arc; no model/config mutex can survive an await.
pub struct Actions<T: SourceStore> {
    pub engine: Arc<Engine>,
    pub store: T,
}
impl<T> SourceActions for Actions<T>
where
    T: SourceStore<
            Query = Query,
            Error = ActionError,
            Schema: SourceSchema<Value = Value, Record = Record>,
        >,
{
    type Store = T;
    type BatchStatus = Result<Record, ActionError>;
    fn new(engine: Arc<Engine>, store: T) -> Self {
        Self { engine, store }
    }
    fn engine(&self) -> &Engine {
        &self.engine
    }
    fn store(&self) -> &T {
        &self.store
    }
    async fn create_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a Record,
    ) -> Result<Record, ActionError> {
        db.execute(&self.store.insert(input, db.dialect())?).await?;
        Ok(input.clone())
    }
    async fn find_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a Query,
    ) -> Result<Option<Record>, ActionError> {
        Ok(self.find_all_in(db, query).await?.into_iter().next())
    }
    async fn find_all_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a Query,
    ) -> Result<Vec<Record>, ActionError> {
        Ok(db
            .fetch(&self.store.select(query, db.dialect())?)
            .await?
            .into_iter()
            .map(|row| decode(row, &T::Schema::describe()))
            .collect())
    }
    async fn count_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a Query,
    ) -> Result<u64, ActionError> {
        Ok(self.find_all_in(db, query).await?.len() as u64)
    }
    async fn update_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a Query,
        input: &'a mut Record,
    ) -> Result<Vec<Record>, ActionError> {
        let mut rows = self.find_all_in(db, query).await?;
        for row in &mut rows {
            for (name, value) in input.iter() {
                if name != "id" {
                    put(row, name, value.clone());
                }
            }
        }
        let update = self.store.update(query, input, db.dialect())?;
        if !rows.is_empty() {
            db.execute(&update).await?;
        }
        Ok(rows)
    }
    async fn delete_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a Query,
    ) -> Result<Vec<Record>, ActionError> {
        let rows = self.find_all_in(db, query).await?;
        db.execute(&self.store.delete(query, db.dialect())?).await?;
        Ok(rows)
    }
    async fn upsert_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a mut Record,
    ) -> Result<Option<Record>, ActionError> {
        if let Some(Value::String(id)) = get(input, "id") {
            let query = Query(Some(id.clone()));
            if self.find_in(db, &query).await?.is_none() {
                return Ok(None);
            }
            Ok(self.update_in(db, &query, input).await?.into_iter().next())
        } else {
            self.create_in(db, input).await.map(Some)
        }
    }
    async fn remove_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a Query,
    ) -> Result<Vec<Record>, ActionError> {
        self.update_in(
            db,
            query,
            &mut Record::from_iter([("active".into(), Value::Bool(false))]),
        )
        .await
    }
    async fn restore_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a Query,
    ) -> Result<Vec<Record>, ActionError> {
        self.update_in(
            db,
            query,
            &mut Record::from_iter([("active".into(), Value::Bool(true))]),
        )
        .await
    }
    async fn batch(&self, inputs: &mut [Record], mode: BatchMode) -> Vec<Self::BatchStatus> {
        let Ok(mut transaction) = self.engine.begin().await else {
            return Vec::new();
        };
        let mut results = Vec::new();
        {
            let mut db = DataSession::transaction(&mut transaction);
            for input in inputs {
                let result = match mode {
                    BatchMode::Create => self.create_in(&mut db, input).await,
                    BatchMode::Update | BatchMode::Upsert => self
                        .upsert_in(&mut db, input)
                        .await
                        .map(|row| row.unwrap_or_default()),
                };
                results.push(result);
            }
        }
        // This finite probe retains attempt results, independent of the transaction acknowledgement.
        if results.iter().any(Result::is_err) {
            let _ = transaction.rollback().await;
        } else {
            let _ = transaction.commit().await;
        }
        results
    }
    async fn install(&self) -> Result<(), ActionError> {
        let plan = self.store.create().render(self.engine.dialect())?;
        stackpress_framework_sql::setup::execute_plan(&self.engine, &plan).await?;
        Ok(())
    }
    async fn upgrade(&self, to: Option<&Create>) -> Result<(), ActionError> {
        // Only empty adjacent alters are in the M0 mechanics corpus; source migration work is M2.
        let alter = self.store.alter(to)?;
        let _ = alter;
        Ok(())
    }
    async fn purge(&self, _cascade: bool) -> Result<(), ActionError> {
        self.engine
            .execute(&Delete::new(T::Schema::describe().module).all_rows())
            .await?;
        Ok(())
    }
    async fn uninstall(&self) -> Result<(), ActionError> {
        self.engine
            .execute(&self.engine.drop_table(T::Schema::describe().module))
            .await?;
        Ok(())
    }
}
