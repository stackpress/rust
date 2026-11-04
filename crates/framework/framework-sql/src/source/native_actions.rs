//! Generic source CRUD over generated descriptors and an explicitly owned executor.
use super::{ActionError, NativeStore, SourceResult as Result, StoreQuery};
use crate::DataSession;
use stackpress_framework_schema::{
    ValueType,
    source::{Record, SourceSchema, Value},
};
use stackpress_sql::{Engine, Row, SqlValue};
use std::sync::Arc;
/// Immutable native actions; no locks, ambient model lookup, policy or startup IO.
pub struct NativeActions<S: SourceSchema<Value = Value, Record = Record>> {
    /// Explicit app-owned executor.
    pub engine: Arc<Engine>,
    /// Exact generated schema and metadata, including app extensions.
    pub store: NativeStore<S>,
}
impl<S: SourceSchema<Value = Value, Record = Record>> NativeActions<S> {
    /// Bind the caller's engine without acquiring a connection.
    pub fn new(engine: Arc<Engine>, store: NativeStore<S>) -> Self {
        Self { engine, store }
    }
    pub(super) fn ids(&self) -> Vec<&str> {
        self.store
            .model
            .fields
            .iter()
            .filter(|f| {
                f.key
                    && !matches!(
                        f.value_type,
                        ValueType::Boolean
                            | ValueType::Enum(_)
                            | ValueType::Fieldset(_)
                            | ValueType::Model(_)
                    )
            })
            .map(|f| f.name.as_str())
            .collect()
    }
    pub(super) fn active(&self, value: bool) -> Record {
        self.store
            .model
            .fields
            .iter()
            .filter(|f| f.active)
            .map(|f| (f.name.clone(), value.into()))
            .collect()
    }
    /// Prepare and validate before IO, then insert and preserve the source native-row fallback.
    pub async fn create(&self, input: &Record) -> Result<Record> {
        self.create_in(&mut DataSession::pooled(&self.engine), input)
            .await
    }
    /// The exact caller-selected executor, including a borrowed batch transaction.
    pub async fn create_in(&self, db: &mut DataSession<'_>, input: &Record) -> Result<Record> {
        let sanitized = self.prepare(input, true)?;
        let mut errors = self
            .store
            .schema
            .assert(&sanitized, true)
            .unwrap_or_default();
        // Source preflight sees active rows only. The UNIQUE constraint remains authoritative
        // for inactive duplicates and concurrent creates, across all authentication types.
        for field in self
            .store
            .model
            .fields
            .iter()
            .filter(|f| validates_unique(f))
        {
            if let Some(value) = nonempty(&sanitized, &field.name)
                && self
                    .find_in(db, &StoreQuery::eq(&field.name, value.clone()))
                    .await?
                    .is_some()
            {
                errors.insert(
                    field.name.clone(),
                    self.store
                        .record
                        .columns()
                        .iter()
                        .find(|c| c.name() == field.name)
                        .expect("stored unique column")
                        .uniqueness_error(value)
                        .into(),
                );
            }
        }
        validate(errors)?;
        let result = db
            .execute(&self.store.insert(&sanitized, db.dialect())?)
            .await?;
        if let Some(row) = result.rows.first() {
            // Deliberately schema.unserialize, not unscalarize: source returning rows keep
            // encrypted tokens and raw snake_case does not become profileId in this branch.
            return Ok(self.store.schema.unserialize(&raw(row), false)?);
        }
        let ids = self.ids();
        if ids.len() == 1 {
            if let Some(id) = result.insert_id.filter(|id| *id != 0) {
                return Ok(self
                    .find_in(db, &StoreQuery::eq(ids[0], id))
                    .await?
                    .unwrap_or_else(|| input.clone()));
            }
        } else if ids.len() > 1 {
            let query = StoreQuery {
                eq: ids
                    .into_iter()
                    .filter_map(|key| input.get(key).map(|v| (key.to_owned(), v.clone())))
                    .collect(),
                ..Default::default()
            };
            return Ok(self
                .find_in(db, &query)
                .await?
                .unwrap_or_else(|| input.clone()));
        }
        Ok(input.clone())
    }

    /// Read the first source result; no extra limit or inactive-profile gate is added.
    pub async fn find(&self, query: &StoreQuery) -> Result<Option<Record>> {
        self.find_in(&mut DataSession::pooled(&self.engine), query)
            .await
    }
    /// Read the first result from the caller-selected executor.
    pub async fn find_in(
        &self,
        db: &mut DataSession<'_>,
        query: &StoreQuery,
    ) -> Result<Option<Record>> {
        Ok(self.find_all_in(db, query).await?.into_iter().next())
    }
    /// Source projected records, including requested nested Profile fields.
    pub async fn find_all(&self, query: &StoreQuery) -> Result<Vec<Record>> {
        self.find_all_in(&mut DataSession::pooled(&self.engine), query)
            .await
    }
    /// Fetch and decode complete owned results before releasing the executor borrow.
    pub async fn find_all_in(
        &self,
        db: &mut DataSession<'_>,
        query: &StoreQuery,
    ) -> Result<Vec<Record>> {
        let selectors = self.store.selectors(&query.columns);
        let rows = db.fetch(&self.store.select(query, db.dialect())?).await?;

        rows.iter()
            .map(|row| {
                let mut record = Record::new();
                for (alias, value) in raw(row) {
                    if let Some(selector) =
                        selectors.iter().find(|selector| selector.alias == alias)
                    {
                        let Some(schema) = self.store.sibling(&selector.model) else {
                            continue;
                        };
                        if let Some(column) =
                            schema.columns().iter().find(|c| c.name() == selector.field)
                            && let Some(value) = column.unserialize(
                                Some(&value),
                                schema.seed(),
                                true,
                                schema.offset_minutes(),
                            )?
                        {
                            set(&mut record, &selector.path, value);
                        }
                    } else {
                        // Native fallback retains raw output, applying the source alias heuristic.
                        set(
                            &mut record,
                            &alias
                                .replace("__", ".")
                                .split('.')
                                .map(camel)
                                .collect::<Vec<_>>(),
                            value,
                        );
                    }
                }
                Ok(record)
            })
            .collect()
    }

    /// Count base matches, with the source joins/filters but without paging or sort.
    pub async fn count(&self, query: &StoreQuery) -> Result<u64> {
        self.count_in(&mut DataSession::pooled(&self.engine), query)
            .await
    }
    /// Count using the same executor; selecting a constant avoids decoding unrelated secrets.
    pub async fn count_in(&self, db: &mut DataSession<'_>, query: &StoreQuery) -> Result<u64> {
        let mut query = query.clone();
        query.take = 0;
        query.skip = 0;
        query.sort.clear();
        // Source COUNT has no pagination. The SQL AST supplies the aggregate expression.
        let mut select = stackpress_sql::Select::new([stackpress_sql::Selector::expression(
            stackpress_sql::Expr::Trusted(
                db.dialect(),
                stackpress_sql::TrustedSql::audited("COUNT(*)"),
                Vec::new(),
            ),
        )
        .alias("total")])
        .from(self.store.table());
        for join in self.store.joins(&query) {
            select = select.join(join);
        }
        if let Some(filter) = self.store.where_clause(&query, db.dialect())? {
            select = select.and_where(filter);
        }
        let rows = db.fetch(&super::SourceStatement(select)).await?;
        Ok(rows
            .first()
            .map(|row| row.try_get::<i64>("total"))
            .transpose()?
            .unwrap_or_default() as u64)
    }

    /// Validate prepared values, return pre-read merged rows, then write original input.
    pub async fn update(&self, query: &StoreQuery, input: &mut Record) -> Result<Vec<Record>> {
        self.update_in(&mut DataSession::pooled(&self.engine), query, input)
            .await
    }
    /// Source nontransactional read/validate/write sequence, or exact caller transaction.
    pub async fn update_in(
        &self,
        db: &mut DataSession<'_>,
        query: &StoreQuery,
        input: &mut Record,
    ) -> Result<Vec<Record>> {
        let sanitized = self.prepare(input, false)?;
        validate(
            self.store
                .schema
                .assert(&sanitized, false)
                .unwrap_or_default(),
        )?;
        let rows = self.find_all_in(db, query).await?;
        let results = rows
            .iter()
            .map(|row| {
                let mut row = row.clone();
                row.extend(sanitized.clone());
                row
            })
            .collect();
        if rows.is_empty() {
            return Ok(results);
        }
        let mut errors = Record::new();
        for field in self
            .store
            .model
            .fields
            .iter()
            .filter(|f| validates_unique(f))
        {
            if let Some(value) = nonempty(&sanitized, &field.name) {
                let exists = self
                    .find_all_in(db, &StoreQuery::eq(&field.name, value.clone()))
                    .await?;
                if !exists.is_empty() && !rows.iter().any(|row| row.get(&field.name) == Some(value))
                {
                    errors.insert(
                        field.name.clone(),
                        self.store
                            .record
                            .columns()
                            .iter()
                            .find(|c| c.name() == field.name)
                            .expect("stored unique column")
                            .uniqueness_error(value)
                            .into(),
                    );
                }
            }
        }
        validate(errors)?;
        db.execute(&self.store.update(query, input, db.dialect())?)
            .await?;
        Ok(results)
    }

    /// Read before physically deleting; return those rows even when delete returns no rows.
    pub async fn delete(&self, query: &StoreQuery) -> Result<Vec<Record>> {
        self.delete_in(&mut DataSession::pooled(&self.engine), query)
            .await
    }
    /// Use one exact borrowed executor, without making the source multi-statement call atomic.
    pub async fn delete_in(
        &self,
        db: &mut DataSession<'_>,
        query: &StoreQuery,
    ) -> Result<Vec<Record>> {
        let rows = self.find_all_in(db, query).await?;
        if !rows.is_empty() {
            db.execute(&self.store.delete(query, db.dialect())?).await?;
        }
        Ok(rows)
    }
    /// Source soft removal; it does not invalidate existing JWTs.
    pub async fn remove(&self, query: &StoreQuery) -> Result<Vec<Record>> {
        if self.store.model.fields.iter().any(|field| field.active) {
            self.update(query, &mut self.active(false)).await
        } else {
            self.delete(query).await
        }
    }
    /// Source restore still needs eq.active=false/-1 to select an inactive row.
    pub async fn restore(&self, query: &StoreQuery) -> Result<Vec<Record>> {
        self.update(query, &mut self.active(true)).await
    }

    /// Source ID-first/token-second upsert; a supplied nonexistent ID returns null.
    pub async fn upsert(&self, input: &mut Record) -> Result<Option<Record>> {
        self.upsert_in(&mut DataSession::pooled(&self.engine), input)
            .await
    }
    /// Preserve ID precedence without an implicit insert-on-missing behavior.
    pub async fn upsert_in(
        &self,
        db: &mut DataSession<'_>,
        input: &mut Record,
    ) -> Result<Option<Record>> {
        let ids = self.ids();
        if !ids.is_empty() && ids.iter().all(|id| input.contains_key(*id)) {
            let query = StoreQuery {
                eq: ids
                    .into_iter()
                    .map(|id| (id.to_owned(), input[id].clone()))
                    .collect(),
                ..Default::default()
            };
            return Ok(self.update_in(db, &query, input).await?.into_iter().next());
        }
        for field in self.store.model.fields.iter().filter(|f| f.unique) {
            if let Some(value) = nonempty(input, &field.name) {
                let query = StoreQuery::eq(&field.name, value.clone());
                if self.find_in(db, &query).await?.is_some() {
                    return Ok(self.update_in(db, &query, input).await?.into_iter().next());
                }
            }
        }
        self.create_in(db, input).await.map(Some)
    }

    /// Original sanitize sequence, including list defaults on updates and empty-string removal.
    fn prepare(&self, input: &Record, populate: bool) -> Result<Record> {
        let filtered = self.store.schema.filter(input, populate)?;
        let serialized = self.store.schema.serialize(&filtered, false)?;
        let mut sanitized = self.store.schema.unserialize(&serialized, false)?;
        sanitized.retain(|_, value| value.as_str() != Some(""));
        Ok(sanitized)
    }
}
/// Source invalid-parameters exception, preserving the lowercase generic CRUD spelling.
fn validate(errors: Record) -> Result<()> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ActionError::new(400, "Invalid parameters", errors))
    }
}
/// Source uniqueness ignores null/empty text but retains false/zero values.
pub(super) fn nonempty<'a>(input: &'a Record, name: &str) -> Option<&'a Value> {
    input
        .get(name)
        .filter(|v| !v.is_null() && v.as_str() != Some(""))
}
/// Convert owned native cells without guessing that text contains JSON or dates.
fn raw(row: &Row) -> Record {
    row.cells()
        .iter()
        .map(|(name, value)| {
            (
                name.clone(),
                match value {
                    SqlValue::Null => Value::Null,
                    SqlValue::Text(value) => Value::String(value.clone()),
                    SqlValue::Bool(value) => Value::Bool(*value),
                    SqlValue::Integer(value) => Value::from(*value),
                    SqlValue::Real(value) => Value::from(*value),
                    SqlValue::Blob(value) => {
                        Value::Array(value.iter().map(|value| Value::from(*value)).collect())
                    }
                },
            )
        })
        .collect()
}
/// Nest a selected alias without overwriting its sibling fields.
fn set(record: &mut Record, path: &[String], value: Value) {
    let Some((first, rest)) = path.split_first() else {
        return;
    };
    if rest.is_empty() {
        record.insert(first.clone(), value);
        return;
    }
    let child = record
        .entry(first.clone())
        .or_insert_with(|| Value::Object(Record::new()));
    if !child.is_object() {
        *child = Value::Object(Record::new());
    }
    if let Some(child) = child.as_object_mut() {
        set(child, rest, value);
    }
}
/// Source snake_case alias fallback becomes camelCase without touching unrelated capitals.
fn camel(value: &str) -> String {
    let mut output = String::new();
    let mut upper = false;
    for character in value.chars() {
        if character == '_' {
            upper = true;
        } else if upper {
            output.push(character.to_ascii_uppercase());
            upper = false;
        } else {
            output.push(character);
        }
    }
    output
}

/// G51 adds action validation for is.unique; only @unique owns DDL/upsert selector metadata.
fn validates_unique(field: &stackpress_framework_schema::Field) -> bool {
    field.unique || field.rules.iter().any(|rule| rule.code == "unique")
}
