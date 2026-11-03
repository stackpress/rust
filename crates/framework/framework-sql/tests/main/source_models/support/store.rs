//! Small probe SQL codec: Dates use INTEGER here solely to prove native ownership, not source parity.

use super::{Record, Value, get};
use stackpress_framework_schema::{ValueType, source::SourceSchema};
use stackpress_framework_sql::source::{ActionError, SourceStore};
use stackpress_sql::{
    Alter, Create, Delete, DialectId, Expr, Field, Insert, Select, SqlType, SqlValue, Update,
};

/// Probe query owns an optional identity and can be reused after an operation finishes.
#[derive(Clone, Debug, Default)]
pub struct Query(pub Option<String>);
/// Metadata and codec state are immutable; no mutex, engine or ambient transaction is hidden.
pub struct Store<S: SourceSchema<Value = Value, Record = Record>> {
    schema: S,
}

/// Resolve native probe data into typed SQL binds; rich Dates remain distinct before/after IO.
fn value(value: &Value) -> SqlValue {
    match value {
        Value::String(value) => value.clone().into(),
        Value::Number(value) => value.as_i64().unwrap().into(),
        Value::Date { millis, .. } => (*millis).into(),
        Value::Bool(value) => (*value).into(),
        _ => SqlValue::Null,
    }
}
impl<S: SourceSchema<Value = Value, Record = Record>> SourceStore for Store<S> {
    type Schema = S;
    type Query = Query;
    type Error = ActionError;
    fn new(seed: String, offset_minutes: i32) -> Self {
        Self {
            schema: S::new(seed, offset_minutes),
        }
    }
    fn schema(&self) -> &S {
        &self.schema
    }
    fn query(&self, input: &Record) -> Result<Query, ActionError> {
        Ok(Query(match get(input, "id") {
            Some(Value::String(id)) => Some(id.clone()),
            _ => None,
        }))
    }
    fn create(&self) -> Create {
        let model = S::describe();
        let mut table = Create::new(model.module);
        for field in &model.fields {
            let kind = match field.value_type {
                ValueType::Boolean => SqlType::Boolean,
                ValueType::Datetime | ValueType::Integer => SqlType::Integer,
                _ => SqlType::Text,
            };
            let column = Field::new(&field.column, kind);
            table = table.add_field(if field.required {
                column.not_null()
            } else {
                column
            });
        }
        table.primary_key(["id"])
    }
    fn alter(&self, _to: Option<&Create>) -> Result<Alter, ActionError> {
        Ok(Alter::new(S::describe().module))
    }
    fn scalarize(&self, input: &Record) -> Result<Vec<(String, SqlValue)>, ActionError> {
        Ok(S::describe()
            .fields
            .iter()
            .filter_map(|field| get(input, &field.name).map(|v| (field.column.clone(), value(v))))
            .collect())
    }
    fn unscalarize(&self, input: &Record) -> Result<Record, ActionError> {
        Ok(input.clone())
    }
    fn select(&self, query: &Query, _dialect: DialectId) -> Result<Select, ActionError> {
        let model = S::describe();
        let select = Select::new(model.fields.iter().map(|f| f.column.clone())).from(model.module);
        Ok(if let Some(id) = &query.0 {
            select.and_where(Expr::eq("id", id.clone()))
        } else {
            select
        })
    }
    fn insert(&self, input: &Record, _dialect: DialectId) -> Result<Insert, ActionError> {
        let cells = self.scalarize(input)?;
        Ok(Insert::new(S::describe().module)
            .columns(cells.iter().map(|(name, _)| name.clone()))
            .values(cells.into_iter().map(|(_, value)| value)))
    }
    fn update(
        &self,
        query: &Query,
        input: &mut Record,
        _dialect: DialectId,
    ) -> Result<Update, ActionError> {
        // The exclusive input borrow carries observable mutation through a suspended call.
        input.retain(|name, _| name != "id");
        let mut update = Update::new(S::describe().module);
        for (column, value) in self.scalarize(input)? {
            update = update.set(column, value);
        }
        Ok(if let Some(id) = &query.0 {
            update.and_where(Expr::eq("id", id.clone()))
        } else {
            update.all_rows()
        })
    }
    fn delete(&self, query: &Query, _dialect: DialectId) -> Result<Delete, ActionError> {
        let delete = Delete::new(S::describe().module);
        Ok(if let Some(id) = &query.0 {
            delete.and_where(Expr::eq("id", id.clone()))
        } else {
            delete.all_rows()
        })
    }
}
