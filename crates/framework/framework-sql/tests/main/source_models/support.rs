//! M0 mechanics probe only: tiny native records/Date and ordinary scalar storage, not source codecs.
#![allow(dead_code)]

use stackpress_framework_schema::source::{Id, SourceField, SourceSchema};
use stackpress_framework_schema::{Model as Metadata, SchemaResult, ValueType};
use stackpress_framework_sql::source::SourceModel;
use std::{marker::PhantomData, sync::Arc};

#[path = "support/actions.rs"]
mod actions;
pub use actions::Actions;
#[path = "support/store.rs"]
mod store;
pub use store::{Query, Store};

/// A real native Date is distinct from the same textual epoch; no reserved JSON member.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(String),
    Integer(i64),
    Bool(bool),
    Date(i64),
    Null,
}
/// Insertion-ordered owned records let lifetime tests retain extension columns and Dates.
pub type Record = Vec<(String, Value)>;

/// Read one literal member, without discarding unknown columns or guessing Date strings.
pub fn get<'a>(record: &'a Record, name: &str) -> Option<&'a Value> {
    record
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
}
/// Updating a member preserves its existing position; new fields append.
pub fn put(record: &mut Record, name: &str, value: Value) {
    if let Some((_, current)) = record.iter_mut().find(|(key, _)| key == name) {
        *current = value;
    } else {
        record.push((name.into(), value));
    }
}

/// Primitive witnesses used by the generated ID accessor, including the wrong-type control.
pub trait IdValue: Clone + Send + Sync + 'static {
    /// Read a correctly typed field from an unprepared record, or return None.
    fn read(value: &Value) -> Option<Self>;
    /// Construct a native field without converting the surrounding record through JSON.
    fn value(self) -> Value;
}
impl IdValue for String {
    fn read(value: &Value) -> Option<Self> {
        if let Value::Text(value) = value {
            Some(value.clone())
        } else {
            None
        }
    }
    fn value(self) -> Value {
        Value::Text(self)
    }
}
impl IdValue for i64 {
    fn read(value: &Value) -> Option<Self> {
        if let Value::Integer(value) = value {
            Some(*value)
        } else {
            None
        }
    }
    fn value(self) -> Value {
        Value::Integer(self)
    }
}

/// Structural emitter supplies all metadata and the declared ID type, never a fixed Profile list.
pub trait Spec: Send + Sync + 'static {
    /// Actual declared ID primitive, so family mismatches are compiler-visible.
    type Id: IdValue;
    /// Complete normalized metadata for this small probe's supported declarations.
    fn describe() -> Metadata;
}

/// Compile-only fixture has no runtime table; emitted specs are used for actual IO proofs.
pub struct CompileSpec<T>(PhantomData<T>);
impl<T: IdValue> Spec for CompileSpec<T> {
    type Id = T;
    fn describe() -> Metadata {
        Metadata {
            name: "CompileOnly".into(),
            rust_name: "CompileOnly".into(),
            module: "compile_only".into(),
            fields: Vec::new(),
            documentation: Default::default(),
            query: Vec::new(),
            attributes: Vec::new(),
        }
    }
}

/// Probe schema state is immutable and local to each generated instance.
pub struct Schema<S: Spec> {
    seed: String,
    offset: i32,
    spec: PhantomData<S>,
}
impl<S: Spec> SourceSchema for Schema<S> {
    type Value = Value;
    type Record = Record;
    fn new(seed: String, offset_minutes: i32) -> Self {
        Self {
            seed,
            offset: offset_minutes,
            spec: PhantomData,
        }
    }
    fn describe() -> Metadata {
        S::describe()
    }
    fn seed(&self) -> &str {
        &self.seed
    }
    fn offset_minutes(&self) -> i32 {
        self.offset
    }
    fn defaults_at(&self, now: i64) -> SchemaResult<Record> {
        // Only literal/Boolean/timestamp defaults are needed by the M0 mechanics corpus.
        Ok(S::describe()
            .fields
            .iter()
            .filter_map(|field| {
                if field.timestamp {
                    Some((field.name.clone(), Value::Date(now)))
                } else {
                    field.default.as_ref().map(|value| {
                        (
                            field.name.clone(),
                            match value {
                                serde_json::Value::String(value) => Value::Text(value.clone()),
                                serde_json::Value::Bool(value) => Value::Bool(*value),
                                _ => Value::Null,
                            },
                        )
                    })
                }
            })
            .collect())
    }
    fn defaults(&self) -> SchemaResult<Record> {
        self.defaults_at(123)
    }
    fn filter(&self, input: &Record, populate: bool) -> SchemaResult<Record> {
        let fields = S::describe().fields;
        let filtered = input
            .iter()
            .filter(|(name, _)| fields.iter().any(|f| f.name == *name))
            .cloned()
            .collect();
        if populate {
            self.populate(&filtered)
        } else {
            Ok(filtered)
        }
    }
    fn populate(&self, input: &Record) -> SchemaResult<Record> {
        let mut record = self.defaults()?;
        for (name, value) in input {
            put(&mut record, name, value.clone());
        }
        Ok(record)
    }
    fn assert(&self, input: &Record, required: bool) -> Option<Record> {
        let errors: Record = S::describe()
            .fields
            .iter()
            .filter(|f| f.required && (required || get(input, &f.name).is_some()))
            .filter(|f| get(input, &f.name).is_none_or(|value| *value == Value::Null))
            .map(|f| (f.name.clone(), Value::Text("required probe field".into())))
            .collect();
        (!errors.is_empty()).then_some(errors)
    }
    fn serialize(&self, input: &Record, _storage: bool) -> SchemaResult<Record> {
        self.filter(input, false)
    }
    fn unserialize(&self, input: &Record, _storage: bool) -> SchemaResult<Record> {
        self.filter(input, false)
    }
}
impl<S: Spec> SourceField<Id> for Schema<S> {
    type FieldValue = S::Id;
    fn read(record: &Record) -> Option<S::Id> {
        S::Id::read(get(record, "id")?)
    }
    fn write(record: &mut Record, value: S::Id) {
        put(record, "id", value.value());
    }
}

/// A model links generated metadata, the exact store and its owned/borrowed action pair.
pub struct Model<S: Spec, R: Send + Sync + 'static>(PhantomData<(S, R)>);
impl<S: Spec, R: Send + Sync + 'static> SourceModel for Model<S, R> {
    type Role = R;
    type Schema = Schema<S>;
    type Store = Store<Self::Schema>;
    type Actions = Actions<Self::Store>;
}

/// Map SQLite values back to owned native values using metadata, never text/date guessing.
pub fn decode(row: stackpress_sql::Row, model: &Metadata) -> Record {
    row.cells()
        .iter()
        .map(|(column, value)| {
            let field = model
                .fields
                .iter()
                .find(|field| field.column == *column)
                .unwrap();
            let value = match value {
                stackpress_sql::SqlValue::Text(value) => Value::Text(value.clone()),
                stackpress_sql::SqlValue::Integer(value)
                    if field.value_type == ValueType::Datetime =>
                {
                    Value::Date(*value)
                }
                stackpress_sql::SqlValue::Integer(value)
                    if field.value_type == ValueType::Boolean =>
                {
                    Value::Bool(*value != 0)
                }
                stackpress_sql::SqlValue::Integer(value) => Value::Integer(*value),
                stackpress_sql::SqlValue::Bool(value) => Value::Bool(*value),
                _ => Value::Null,
            };
            (field.name.clone(), value)
        })
        .collect()
}

/// Assert Send for an actual operation future rather than just its action container.
pub fn send<T: Send>(value: T) -> T {
    value
}
/// Keep the engine Arc constructor visible in disposable compile probes.
pub fn owned_engine(database: impl stackpress_sql::Database) -> Arc<stackpress_sql::Engine> {
    Arc::new(stackpress_sql::Engine::new(database))
}
