//! Original per-item statuses remain attempt evidence after transaction rollback.
use super::{BatchMode, NativeActions, SourceResult as Result, StoreQuery};
use crate::DataSession;
use serde::{Deserialize, Serialize};
use stackpress_framework_schema::source::{Record, SourceSchema, Value};
/// Source per-item status; successful rows remain reported even when a later item rolls back.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchStatus {
    /// Absent for the source's missing-update-selector item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
    /// Source status phrase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Success row or source null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Value>,
    /// Source success count, always one even for a null upsert result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    /// Source exception/missing-selector message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Original field errors, when supplied by an exception.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Record>,
}

impl<S: SourceSchema<Value = Value, Record = Record>> NativeActions<S> {
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
                    Value::Null => self.null_item(mode),
                    // Object.entries on boxed primitives supplies no declared stored fields.
                    _ => self.batch_item(&mut db, &mut Record::new(), mode).await,
                };
                match outcome {
                    Ok(Some(row)) => {
                        let mut status = success(row);
                        // MySQL create returns its original input when no native row/ID is
                        // returned. The typed Record layer boxes primitives as {}, so restore
                        // the original dynamic value only for that exact fallback branch.
                        if db.dialect() == stackpress_sql::DialectId::Mysql
                            && !matches!(mode, BatchMode::Update)
                            && !input.is_object()
                            && status
                                .results
                                .as_ref()
                                .and_then(Value::as_object)
                                .is_some_and(Record::is_empty)
                        {
                            status.results = Some(input.clone());
                        }
                        statuses.push(status);
                    }
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

    /// Source property access fails on the first declared selector, rather than a fixed `id`.
    fn null_item(&self, mode: BatchMode) -> Result<Option<Option<Record>>> {
        let selector = self.ids().first().copied().or_else(|| {
            self.store
                .model
                .fields
                .iter()
                .find(|field| field.unique)
                .map(|field| field.name.as_str())
        });
        if matches!(mode, BatchMode::Update) && selector.is_none() {
            return Ok(None);
        }
        let message =
            if let Some(selector) = selector.filter(|_| !matches!(mode, BatchMode::Create)) {
                format!("Cannot read properties of null (reading '{selector}')")
            } else {
                "Cannot convert undefined or null to object".into()
            };
        Err(super::ActionError::new(500, message, Record::new()))
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
                let ids = self.ids();
                if !ids.is_empty() && ids.iter().all(|id| input.contains_key(*id)) {
                    let query = StoreQuery {
                        eq: ids
                            .into_iter()
                            .map(|id| (id.to_owned(), input[id].clone()))
                            .collect(),
                        ..Default::default()
                    };
                    queries.push(query);
                }
                for f in self.store.model.fields.iter().filter(|f| f.unique) {
                    if let Some(value) = super::native_actions::nonempty(input, &f.name) {
                        queries.push(StoreQuery::eq(&f.name, value.clone()));
                    }
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
