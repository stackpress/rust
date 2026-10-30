//! Source batch attempt statuses and an exact, exclusively borrowed transaction.
use super::actions::token;
use super::{Actions, ModelKind, Result, StoreQuery};
use crate::columns::Record;
use crate::columns::Value;
use serde::{Deserialize, Serialize};
use stackpress_framework_sql::DataSession;

/// Source batch mode; only caught item failures set rollback, not missing update keys.
#[derive(Clone, Copy, Debug, Default)]
pub enum BatchMode {
    /// Insert every input independently inside the selected batch transaction.
    Create,
    /// Update by present ID, or by an Auth token if its ID did not resolve.
    Update,
    /// Source ID-first/token-second upsert.
    #[default]
    Upsert,
}
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

impl<M: ModelKind> Actions<M> {
    /// Atomic source batch, with caught item statuses retained even after rollback/commit error.
    pub async fn batch(&self, inputs: &mut [Record], mode: BatchMode) -> Vec<BatchStatus> {
        let mut statuses = Vec::new();
        let Ok(mut transaction) = self.engine.begin().await else {
            return statuses;
        };
        let mut rollback = false;
        {
            let mut db = DataSession::transaction(&mut transaction);
            for input in inputs {
                match self.batch_item(&mut db, input, mode).await {
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
                if M::NAME == "Auth"
                    && let Some(token) = token(input)
                {
                    queries.push(StoreQuery::eq("token", token));
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
