//! Source batch attempt statuses and an exact, exclusively borrowed transaction.
use super::Record;
use super::Value;
use super::{Actions, ModelKind, Result, StoreQuery};
use super::{BatchMode, BatchStatus};
use stackpress_framework_sql::DataSession;

impl<M: ModelKind> Actions<M> {
    /// Atomic source batch, with caught item statuses retained even after rollback/commit error.
    pub async fn batch(&self, inputs: &mut [Record], mode: BatchMode) -> Vec<BatchStatus> {
        let mut values = inputs
            .iter()
            .cloned()
            .map(Value::Object)
            .collect::<Vec<_>>();
        let statuses = self.batch_values(&mut values, mode).await;
        for (input, value) in inputs.iter_mut().zip(values) {
            if let Value::Object(value) = value {
                *input = value;
            }
        }
        statuses
    }
    /// Dynamic event rows preserve source primitive/null item behavior and mutations.
    /// Only caught failures trigger rollback; statuses still describe attempted work.
    pub async fn batch_values(&self, inputs: &mut [Value], mode: BatchMode) -> Vec<BatchStatus> {
        let mut statuses = Vec::new();
        let Ok(mut transaction) = self.engine.begin().await else {
            return statuses;
        };
        let mut rollback = false;
        {
            let mut db = DataSession::transaction(&mut transaction);
            for input in inputs {
                let outcome = match input {
                    Value::Object(input) => self.batch_item(&mut db, input, mode).await,
                    Value::Null => Err(super::ActionError::new(
                        500,
                        if matches!(mode, BatchMode::Create) {
                            "Cannot convert undefined or null to object"
                        } else {
                            "Cannot read properties of null (reading 'id')"
                        },
                        Record::new(),
                    )),
                    // Object.entries on boxed primitives supplies no declared stored fields.
                    _ => self.batch_item(&mut db, &mut Record::new(), mode).await,
                };
                match outcome {
                    Ok(Some(row)) => statuses.push(success(row)),
                    Ok(None) => statuses.push(BatchStatus {
                        code: None,
                        status: None,
                        results: None,
                        total: None,
                        error: Some("ID or unique field is required for update mode".into()),
                        errors: None,
                    }),
                    Err(error) => {
                        rollback = true;
                        statuses.push(BatchStatus {
                            code: Some(error.code),
                            status: Some(error.status().into()),
                            results: None,
                            total: None,
                            error: Some(error.error),
                            errors: Some(error.errors),
                        });
                    }
                }
            }
        }
        // Source catches the outer transaction failure. Item statuses describe attempts,
        // not committed rows. Dropping a cancelled future retains adapter rollback-on-drop.
        if rollback {
            let _ = transaction.rollback().await;
        } else {
            let _ = transaction.commit().await;
        }
        statuses
    }

    /// Dispatch one item; missing update selectors deliberately do not trigger rollback.
    async fn batch_item(
        &self,
        db: &mut DataSession<'_>,
        input: &mut Record,
        mode: BatchMode,
    ) -> Result<Option<Option<Record>>> {
        match mode {
            BatchMode::Create => self.create_in(db, input).await.map(|row| Some(Some(row))),
            BatchMode::Upsert => self.upsert_in(db, input).await.map(Some),
            BatchMode::Update => {
                let mut queries = Vec::new();
                if let Some(id) = input.get("id") {
                    queries.push(StoreQuery::eq("id", id.clone()));
                }
                for query in queries {
                    if self.find_in(db, &query).await?.is_some() {
                        return Ok(Some(
                            self.update_in(db, &query, input).await?.into_iter().next(),
                        ));
                    }
                }
                Ok(None)
            }
        }
    }
}
/// Source success status always has total=1, even when a supplied ID matched no row.
fn success(row: Option<Record>) -> BatchStatus {
    BatchStatus {
        code: Some(200),
        status: Some("OK".into()),
        results: Some(row.map(Value::Object).unwrap_or(Value::Null)),
        total: Some(1),
        error: None,
        errors: None,
    }
}
