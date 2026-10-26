//! Prevalidated Serde views: borrow containers and copy only requested owned scalar outputs.
//!
//! Validation finishes before any serializer or target visitor runs. JSON's numeric,
//! string and enum-key interpretation remains delegated to serde_json where possible.

use super::{DataMap, ParseLimits, Value};
use crate::{Error, Result};
use serde::de::{self, DeserializeOwned, DeserializeSeed, IntoDeserializer, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserializer, Serialize, Serializer};

mod key;

/// Borrow a complete JSON-compatible tree after finite-budget and variant preflight.
pub(super) struct Json<'a>(&'a Value);

impl<'a> Json<'a> {
    /// Check budgets first, then unsupported variants in wire order, before invoking user code.
    pub(super) fn checked(value: &'a Value, limits: &ParseLimits) -> Result<Self> {
        // Preserve the existing precedence: a budget violation anywhere wins over non-JSON data.
        value.validate(limits)?;
        compatible(value, &mut Vec::new())?;
        Ok(Self(value))
    }

    /// Decode a typed result while retaining serde_json's concrete error in the source chain.
    pub(super) fn decode<T: DeserializeOwned>(self) -> Result<T> {
        // Only the requested owned output is constructed; ignored subtrees remain borrowed.
        T::deserialize(self).map_err(Error::from)
    }

    /// Delegate scalar semantics without ever constructing a container tree.
    fn scalar(&self) -> Option<serde_json::Value> {
        // Owned strings can move directly into a DeserializeOwned target's visit_string callback.
        match self.0 {
            Value::Null => Some(serde_json::Value::Null),
            Value::Bool(value) => Some((*value).into()),
            Value::Number(value) => Some(value.clone().into()),
            Value::String(value) => Some(value.clone().into()),
            _ => None,
        }
    }

    /// Describe a container mismatch using the requested visitor's expectation.
    fn invalid(&self, expected: &dyn de::Expected) -> serde_json::Error {
        // Scalar mismatches go through serde_json; only container mismatches reach this helper.
        let unexpected = match self.0 {
            Value::Array(_) => de::Unexpected::Seq,
            Value::Object(_) => de::Unexpected::Map,
            _ => unreachable!("preflight and scalar delegation cover every other variant"),
        };
        de::Error::invalid_type(unexpected, expected)
    }
}

/// Borrowed diagnostic components keep literal empty keys distinct from absent components.
enum Component<'a> {
    Key(&'a str),
    Index(usize),
}

/// Reject unsupported variants, formatting a complete diagnostic path only on failure.
fn compatible<'a>(value: &'a Value, path: &mut Vec<Component<'a>>) -> Result<()> {
    // Recursion is bounded by the completed budget preflight's hard depth ceiling.
    match value {
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                path.push(Component::Index(index));
                compatible(child, path)?;
                path.pop();
            }
        }
        Value::Object(values) => {
            for (key, child) in values {
                path.push(Component::Key(key));
                compatible(child, path)?;
                path.pop();
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        _ => {
            // Match the explicit converter's root-to-leaf joining, including empty literal keys.
            let path = path
                .iter()
                .fold(String::new(), |path, component| match component {
                    Component::Key(key) => super::value::qualify(&path, key),
                    Component::Index(index) => super::value::qualify(&path, &index.to_string()),
                });
            return Err(
                Error::invalid("non-JSON value requires an explicit wire representation")
                    .at_path(path),
            );
        }
    }
    Ok(())
}

impl Serialize for Json<'_> {
    /// Serialize already checked descendants without repeating budgets or allocating JSON copies.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self.0 {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(value) => serializer.serialize_bool(*value),
            Value::Number(value) => value.serialize(serializer),
            Value::String(value) => serializer.serialize_str(value),
            Value::Array(values) => {
                let mut sequence = serializer.serialize_seq(Some(values.len()))?;
                // Preserve index order without reentering Value's root prevalidation.
                for child in values {
                    sequence.serialize_element(&Json(child))?;
                }
                sequence.end()
            }
            Value::Object(values) => {
                let mut map = serializer.serialize_map(Some(values.len()))?;
                // IndexMap iteration preserves literal key insertion order for every serializer.
                for (key, child) in values {
                    map.serialize_entry(key, &Json(child))?;
                }
                map.end()
            }
            _ => unreachable!("complete preflight rejected unsupported variants"),
        }
    }
}

/// Keep typed scalar interpretation identical to serde_json, including 128-bit integers.
macro_rules! scalar_methods {
    ($($method:ident),* $(,)?) => {$ (
        #[doc = "Delegate a scalar visitor to serde_json without copying any containers."]
        fn $method<V: Visitor<'de>>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> {
            match self.scalar() {
                Some(value) => value.$method(visitor),
                None => Err(self.invalid(&visitor)),
            }
        }
    )*};
}

impl<'de> Deserializer<'de> for Json<'de> {
    type Error = serde_json::Error;

    /// Visit a JSON node; only owned scalar results allocate payload storage.
    fn deserialize_any<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        match self.0 {
            Value::Array(values) => sequence(values, visitor),
            Value::Object(values) => object(values, visitor),
            _ => self
                .scalar()
                .expect("preflight proved JSON scalar")
                .deserialize_any(visitor),
        }
    }

    scalar_methods!(
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_i128,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_u128,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_unit,
        deserialize_identifier
    );

    /// Null selects None; every other checked JSON node selects Some.
    fn deserialize_option<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        if matches!(self.0, Value::Null) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    /// A unit struct uses the same null-only representation as serde_json.
    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        self.deserialize_unit(visitor)
    }

    /// Preserve JSON's special newtype protocols as well as ordinary transparent wrappers.
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        // Delegate uncommon protocol handling so dependency feature unification remains valid.
        self.0
            .json_at("")
            .map_err(de::Error::custom)?
            .deserialize_newtype_struct(name, visitor)
    }

    /// Byte targets accept JSON text/arrays; native byte variants were rejected by preflight.
    fn deserialize_bytes<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        if let Value::Array(values) = self.0 {
            sequence(values, visitor)
        } else if let Some(value) = self.scalar() {
            value.deserialize_bytes(visitor)
        } else {
            Err(self.invalid(&visitor))
        }
    }

    /// An owned byte target uses serde_json's text/array policy.
    fn deserialize_byte_buf<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        self.deserialize_bytes(visitor)
    }

    /// Typed sequences accept only JSON arrays.
    fn deserialize_seq<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        if let Value::Array(values) = self.0 {
            sequence(values, visitor)
        } else if let Some(value) = self.scalar() {
            value.deserialize_seq(visitor)
        } else {
            Err(self.invalid(&visitor))
        }
    }

    /// Tuple length validation belongs to the target visitor, with no unconsumed children allowed.
    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    /// Tuple structs share the JSON array representation.
    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    /// Typed maps accept only literal JSON objects.
    fn deserialize_map<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        if let Value::Object(values) = self.0 {
            object(values, visitor)
        } else if let Some(value) = self.scalar() {
            value.deserialize_map(visitor)
        } else {
            Err(self.invalid(&visitor))
        }
    }

    /// Serde JSON structs accept both named fields and positional array representations.
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        match self.0 {
            Value::Array(values) => sequence(values, visitor),
            _ => self.deserialize_map(visitor),
        }
    }

    /// Delegate the uncommon enum boundary to JSON to retain every variant representation.
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        // Retain an owned fallback at this enum boundary; surrounding plain containers stay borrowed.
        self.0
            .json_at("")
            .map_err(de::Error::custom)?
            .deserialize_enum(name, variants, visitor)
    }

    /// Preflight already validated ignored descendants; do not copy or visit their payloads again.
    fn deserialize_ignored_any<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }
}

impl<'de> IntoDeserializer<'de, serde_json::Error> for Json<'de> {
    type Deserializer = Self;
    /// Pass a checked borrowed child directly into Serde's sequence adapter.
    fn into_deserializer(self) -> Self {
        self
    }
}

/// Visit borrowed array children and retain JSON's unconsumed-element failure.
fn sequence<'de, V: Visitor<'de>>(
    values: &'de [Value],
    visitor: V,
) -> std::result::Result<V::Value, serde_json::Error> {
    let mut access = de::value::SeqDeserializer::new(values.iter().map(Json));
    let result = visitor.visit_seq(&mut access)?;
    // Translate only the adapter's end check so JSON's established diagnostic remains stable.
    access.end().map_err(|_: serde_json::Error| {
        de::Error::invalid_length(values.len(), &"fewer elements in array")
    })?;
    Ok(result)
}

/// Visit borrowed object children while preserving insertion order and JSON key coercions.
fn object<'de, V: Visitor<'de>>(
    values: &'de DataMap<String, Value>,
    visitor: V,
) -> std::result::Result<V::Value, serde_json::Error> {
    let mut access = Object {
        entries: values.iter(),
        pending: None,
    };
    let result = visitor.visit_map(&mut access)?;
    if access.entries.len() > 0 {
        return Err(de::Error::invalid_length(
            values.len(),
            &"fewer elements in map",
        ));
    }
    Ok(result)
}

/// A map cursor holds at most the next value; no child ownership is duplicated.
struct Object<'a> {
    /// Remaining insertion-ordered fields; advancing never clones their child values.
    entries: indexmap::map::Iter<'a, String, Value>,
    /// The value paired with the last yielded key, consumed by next_value_seed.
    pending: Option<&'a Value>,
}

impl<'de> de::MapAccess<'de> for Object<'de> {
    type Error = serde_json::Error;
    /// Decode literal keys through JSON's own key deserializer, including numeric/newtype keys.
    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> std::result::Result<Option<K::Value>, Self::Error> {
        let Some((key, value)) = self.entries.next() else {
            return Ok(None);
        };
        self.pending = Some(value);
        // String keys use a direct owned scalar; uncommon coercions delegate to JSON privately.
        seed.deserialize(key::Key(key)).map(Some)
    }
    /// Consume exactly the value paired with the preceding key.
    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> std::result::Result<V::Value, Self::Error> {
        let value = self
            .pending
            .take()
            .ok_or_else(|| de::Error::custom("value is missing"))?;
        seed.deserialize(Json(value))
    }
    /// Let owned targets reserve their eventual output without allocating a duplicate map.
    fn size_hint(&self) -> Option<usize> {
        Some(self.entries.len())
    }
}
