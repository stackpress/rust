//! Exact typed access to unprepared native fields; no implicit coercion or validation.
use super::{Record, Value};

/// Native Date witness, preserving optional host display text without JSON round-tripping.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Date {
    /// Epoch milliseconds.
    pub millis: i64,
    /// Explicit host Date.toString for array coercion.
    pub display: Option<String>,
}

/// Primitive conversion used by generated SourceField implementations.
/// Missing keys are handled by SourceField; nullable values retain explicit null here.
pub trait NativeField: Sized {
    /// Read exactly the declared native family, without repairing overrides or coercing input.
    fn read(value: &Value) -> Option<Self>;
    /// Preserve the native value when writing a generated field.
    fn value(self) -> Value;
}
impl NativeField for String {
    fn read(value: &Value) -> Option<Self> {
        value.as_str().map(str::to_owned)
    }
    fn value(self) -> Value {
        Value::String(self)
    }
}
impl NativeField for bool {
    fn read(value: &Value) -> Option<Self> {
        value.as_bool()
    }
    fn value(self) -> Value {
        Value::Bool(self)
    }
}
impl NativeField for i64 {
    fn read(value: &Value) -> Option<Self> {
        value.as_i64()
    }
    fn value(self) -> Value {
        Value::from(self)
    }
}
impl NativeField for f64 {
    fn read(value: &Value) -> Option<Self> {
        value.as_f64()
    }
    fn value(self) -> Value {
        Value::from(self)
    }
}
impl NativeField for Date {
    fn read(value: &Value) -> Option<Self> {
        if let Value::Date { millis, display } = value {
            Some(Self {
                millis: *millis,
                display: display.clone(),
            })
        } else {
            None
        }
    }
    fn value(self) -> Value {
        Value::Date {
            millis: self.millis,
            display: self.display,
        }
    }
}
impl NativeField for Record {
    fn read(value: &Value) -> Option<Self> {
        value.as_object().cloned()
    }
    fn value(self) -> Value {
        Value::Object(self)
    }
}
impl<T: NativeField> NativeField for Vec<T> {
    fn read(value: &Value) -> Option<Self> {
        value.as_array()?.iter().map(T::read).collect()
    }
    fn value(self) -> Value {
        Value::Array(self.into_iter().map(T::value).collect())
    }
}
impl<T: NativeField> NativeField for Option<T> {
    fn read(value: &Value) -> Option<Self> {
        if value.is_null() {
            Some(None)
        } else {
            T::read(value).map(Some)
        }
    }
    fn value(self) -> Value {
        self.map(T::value).unwrap_or(Value::Null)
    }
}
