//! JSON object-key decoding: direct strings, delegated numeric/boolean coercions.
//!
//! serde_json's map-key deserializer is private. A one-key scratch map reaches
//! it only when a target requests coercion, keeping the common string-key path lean.

use serde::Deserializer;
use serde::de::{self, DeserializeSeed, Visitor};

/// One borrowed literal key; outputs preserve owned JSON's string visitor behavior.
pub(super) struct Key<'a>(pub(super) &'a str);

/// Delegate numeric/boolean key policy without copying parser rules or error spelling.
macro_rules! coerced_keys {
    ($($method:ident),* $(,)?) => {$ (
        #[doc = "Decode a coerced key through serde_json's authoritative key implementation."]
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            // The seed relays this exact requested method to JSON's private map-key decoder.
            struct Seed<V>(V);
            impl<'de, V: Visitor<'de>> DeserializeSeed<'de> for Seed<V> {
                type Value = V::Value;
                /// Retain the target visitor while selecting its requested key interpretation.
                fn deserialize<D: Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
                    decoder.$method(self.0)
                }
            }
            let mut map = serde_json::Map::new();
            map.insert(self.0.to_owned(), serde_json::Value::Null);
            map.deserialize_any(KeySeed(Seed(visitor)))
        }
    )*};
}

impl<'de> Deserializer<'de> for Key<'de> {
    type Error = serde_json::Error;

    /// Move a single cloned key into the visitor, matching owned JSON's callback choice.
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_string(self.0.to_owned())
    }

    coerced_keys!(
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
        deserialize_f64
    );

    /// Object keys cannot be null, so optional key targets always receive Some.
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_some(self)
    }

    /// A key newtype unwraps into the same JSON key policy rather than value semantics.
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    /// String enum keys use Serde's unit-variant convention, as serde_json does.
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        de::value::StringDeserializer::<Self::Error>::new(self.0.to_owned())
            .deserialize_enum(name, variants, visitor)
    }

    // JSON's remaining key methods all forward their string representation to the visitor.
    serde::forward_to_deserialize_any! {
        char str string bytes byte_buf unit unit_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
}

/// Bridge a target key seed to serde_json's private map-key implementation.
struct KeySeed<S>(S);
impl<'de, S: DeserializeSeed<'de>> Visitor<'de> for KeySeed<S> {
    type Value = S::Value;
    /// Describe the private single-key delegation invariant.
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("one JSON key")
    }
    /// Consume the placeholder too, satisfying JSON's complete-map visitor contract.
    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let key = map
            .next_key_seed(self.0)?
            .ok_or_else(|| de::Error::custom("key is missing"))?;
        let _: de::IgnoredAny = map.next_value()?;
        Ok(key)
    }
}
