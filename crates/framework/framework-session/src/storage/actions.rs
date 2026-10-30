//! Source CRUD sequencing, explicit batch transactions and opt-in maintenance.

use super::{ActionError, AuthModel, ModelKind, ProfileModel, Result, Store, StoreQuery};
use crate::columns::Record;
use crate::columns::Value;
use stackpress_framework_sql::DataSession;
use stackpress_sql::{Create, Engine, Row, SqlValue};
use std::sync::Arc;

/// Direct Profile actions over the application's engine.
pub type ProfileActions = Actions<ProfileModel>;
/// Direct Auth actions with the original deterministic token/secret codecs.
pub type AuthActions = Actions<AuthModel>;

/// An engine reference and a detached source store; transactions are never cloned.
#[derive(Clone)]
pub struct Actions<M: ModelKind> {
    /// App-owned executor shared by direct calls; no database is opened here.
    pub engine: Arc<Engine>,
    /// Source generated Store API, available without Actions.
    pub store: Store<M>,
    /// Source password assertions, consulted only by Auth's account methods.
    pub(crate) password: super::PasswordConfig,
}
impl<M: ModelKind> Actions<M> {
    /// Construct direct actions with source empty-seed/password defaults when supplied so.
    pub fn new(engine: Arc<Engine>, seed: impl Into<String>) -> Self {
        Self {
            engine,
            store: Store::new(seed),
            password: super::PasswordConfig::default(),
        }
    }

    /// Prepare and validate before IO, then insert and preserve the source native-row fallback.
    pub async fn create(&self, input: &Record) -> Result<Record> {
        self.create_in(&mut DataSession::pooled(&self.engine), input)
            .await
    }
    /// The exact caller-selected executor, including a borrowed batch transaction.
    pub async fn create_in(&self, db: &mut DataSession<'_>, input: &Record) -> Result<Record> {
        let sanitized = self.prepare(input, true)?;
        let mut errors = self.store.assert(&sanitized, true).unwrap_or_default();
        // Source preflight sees active rows only. The UNIQUE constraint remains authoritative
        // for inactive duplicates and concurrent creates, across all authentication types.
        if M::NAME == "Auth"
            && let Some(token) = token(&sanitized)
            && self
                .find_in(db, &StoreQuery::eq("token", token))
                .await?
                .is_some()
        {
            errors.insert("token".into(), "Already exists".into());
        }
        validate(errors)?;
        let result = db
            .execute(&self.store.insert_for(&sanitized, db.dialect())?)
            .await?;
        if let Some(row) = result.rows.first() {
            // Deliberately schema.unserialize, not unscalarize: source returning rows keep
            // encrypted tokens and raw snake_case does not become profileId in this branch.
            return Ok(self.store.unserialize(&raw(row), false)?);
        }
        if let Some(id) = result.insert_id.filter(|id| *id != 0) {
            return Ok(self
                .find_in(db, &StoreQuery::eq("id", id))
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
        let profile = super::ProfileStore::new(self.store.seed.clone());
        rows.iter()
            .map(|row| {
                let mut record = Record::new();
                for (alias, value) in raw(row) {
                    if let Some(selector) =
                        selectors.iter().find(|selector| selector.alias == alias)
                    {
                        let schema = if selector.profile {
                            &profile.schema
                        } else {
                            &self.store.schema
                        };
                        if let Some(column) = schema.column(&selector.field)
                            && let Some(value) = column.unserialize(
                                Some(&value),
                                &schema.seed,
                                true,
                                schema.offset,
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
        .from(M::TABLE);
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
        validate(self.store.assert(&sanitized, false).unwrap_or_default())?;
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
        if M::NAME == "Auth"
            && let Some(token) = token(&sanitized)
        {
            let exists = self
                .find_all_in(db, &StoreQuery::eq("token", token))
                .await?;
            if !exists.is_empty()
                && !rows
                    .iter()
                    .any(|row| row.get("token").and_then(Value::as_str) == Some(token))
            {
                return Err(ActionError::new(
                    400,
                    "Invalid parameters",
                    Record::from_iter([("token".into(), "Already exists".into())]),
                ));
            }
        }
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
        self.update(
            query,
            &mut Record::from_iter([("active".into(), false.into())]),
        )
        .await
    }
    /// Source restore still needs eq.active=false/-1 to select an inactive row.
    pub async fn restore(&self, query: &StoreQuery) -> Result<Vec<Record>> {
        self.update(
            query,
            &mut Record::from_iter([("active".into(), true.into())]),
        )
        .await
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
        if let Some(id) = input.get("id") {
            return Ok(self
                .update_in(db, &StoreQuery::eq("id", id.clone()), input)
                .await?
                .into_iter()
                .next());
        }
        if M::NAME == "Auth"
            && let Some(token) = token(input)
        {
            let query = StoreQuery::eq("token", token);
            if self.find_in(db, &query).await?.is_some() {
                return Ok(self.update_in(db, &query, input).await?.into_iter().next());
            }
        }
        self.create_in(db, input).await.map(Some)
    }

    /// Explicit table installation; this never runs implicitly during plugin startup.
    pub async fn install(&self) -> Result<()> {
        self.engine
            .execute_plan(&self.store.create().render(self.engine.dialect())?)
            .await?;
        Ok(())
    }
    /// Explicit source-to-target alteration, with unsupported SQL mechanics reported by SQL.
    pub async fn upgrade(&self, to: &Create) -> Result<()> {
        if let Some(plan) = self
            .store
            .create()
            .definition()?
            .diff(&to.definition()?)?
            .render(self.engine.dialect())?
        {
            self.engine.execute_plan(&plan).await?;
        }
        Ok(())
    }
    /// Explicit source purge; SQLite's DELETE truncation already applies declared FK cascade.
    pub async fn purge(&self, cascade: bool) -> Result<()> {
        // Source cascade matters only to PostgreSQL; SQLite/MySQL ignore this flag.
        let command = stackpress_sql::TableCommand::Truncate {
            table: M::TABLE.into(),
            cascade: cascade && self.engine.dialect() == stackpress_sql::DialectId::Postgres,
        };
        self.engine.execute(&command).await?;
        Ok(())
    }
    /// Explicit physical uninstall; no automatic related-table teardown is added.
    pub async fn uninstall(&self) -> Result<()> {
        self.engine
            .execute(&self.engine.drop_table(M::TABLE))
            .await?;
        Ok(())
    }

    /// Original sanitize sequence, including list defaults on updates and empty-string removal.
    fn prepare(&self, input: &Record, populate: bool) -> Result<Record> {
        let filtered = self.store.filter(input, populate)?;
        let serialized = self.store.serialize(&filtered, false)?;
        let mut sanitized = self.store.unserialize(&serialized, false)?;
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
/// A present nonempty Auth token enables source uniqueness/upsert lookup.
pub(super) fn token(input: &Record) -> Option<&str> {
    input
        .get("token")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
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
