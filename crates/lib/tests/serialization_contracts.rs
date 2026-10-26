//! Wire compatibility and preflight guarantees for borrowed JSON adapters.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use stackpress_lib::{
    DataMap, ErrorKind, FileData, Nest, ParseLimits, Result, SharedNative, Timestamp, Value,
};
use std::collections::BTreeMap;
use std::time::Instant;

/// Compare direct selection to the established owned-JSON decoder, including error text.
fn equivalent<T: DeserializeOwned + std::fmt::Debug + PartialEq>(
    json: serde_json::Value,
) -> Result<()> {
    let expected = serde_json::from_value::<T>(json.clone());
    let mut nest = Nest::new();
    nest.set("selected", Value::try_from(json)?)?;
    match (nest.path_as::<T>("selected"), expected) {
        (Ok(actual), Ok(expected)) => assert_eq!(actual, Some(expected)),
        (Err(actual), Err(expected)) => {
            assert_eq!(actual.path(), Some("selected"));
            assert_eq!(
                std::error::Error::source(&actual).unwrap().to_string(),
                expected.to_string()
            );
        }
        (actual, expected) => panic!("decoder disagreement: {actual:?} versus {expected:?}"),
    }
    Ok(())
}

/// Transparent wrappers retain JSON newtype semantics.
#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct NumberKey(i64);

/// String keys may select unit enum variants.
#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum NamedKey {
    First,
    Second,
}

/// Ordinary unit and tuple structs use null and array representations respectively.
#[derive(Debug, Deserialize, PartialEq)]
struct Unit;
#[derive(Debug, Deserialize, PartialEq)]
struct Pair(String, bool);

/// A byte-buffer target explicitly exercises JSON's text/array byte representation.
#[derive(Debug, PartialEq)]
struct Bytes(Vec<u8>);
impl<'de> Deserialize<'de> for Bytes {
    /// Decode the same byte forms accepted by serde_json's owned value deserializer.
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> std::result::Result<Self, D::Error> {
        struct ByteVisitor;
        impl<'de> serde::de::Visitor<'de> for ByteVisitor {
            type Value = Bytes;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("bytes")
            }
            fn visit_string<E: serde::de::Error>(
                self,
                text: String,
            ) -> std::result::Result<Bytes, E> {
                Ok(Bytes(text.into_bytes()))
            }
            /// Support borrowed text as well as the owned callback used by the JSON baseline.
            fn visit_str<E: serde::de::Error>(self, text: &str) -> std::result::Result<Bytes, E> {
                Ok(Bytes(text.as_bytes().to_vec()))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut values: A,
            ) -> std::result::Result<Bytes, A::Error> {
                let mut bytes = Vec::new();
                while let Some(value) = values.next_element()? {
                    bytes.push(value);
                }
                Ok(Bytes(bytes))
            }
        }
        decoder.deserialize_byte_buf(ByteVisitor)
    }
}

/// Expected and rejected typed forms agree across each JSON node category.
#[test]
fn typed_nodes_and_coerced_keys_match_owned_json() -> Result<()> {
    for json in [
        serde_json::json!(null),
        serde_json::json!(true),
        serde_json::json!(7),
        serde_json::json!(-1),
        serde_json::json!(1.25),
        serde_json::json!("x"),
        serde_json::json!([]),
        serde_json::json!({}),
    ] {
        equivalent::<bool>(json.clone())?;
        equivalent::<i8>(json.clone())?;
        equivalent::<u8>(json.clone())?;
        equivalent::<i128>(json.clone())?;
        equivalent::<u128>(json.clone())?;
        equivalent::<f32>(json.clone())?;
        equivalent::<f64>(json.clone())?;
        equivalent::<char>(json.clone())?;
        equivalent::<String>(json.clone())?;
        equivalent::<Unit>(json.clone())?;
        equivalent::<()>(json.clone())?;
        equivalent::<Vec<i64>>(json.clone())?;
        equivalent::<Pair>(json.clone())?;
        equivalent::<Bytes>(json.clone())?;
        equivalent::<Row>(json.clone())?;
        equivalent::<NumberKey>(json.clone())?;
        equivalent::<BTreeMap<String, String>>(json)?;
    }
    equivalent::<Pair>(serde_json::json!(["x", false]))?;
    equivalent::<Bytes>(serde_json::json!([0, 255]))?;
    equivalent::<Pair>(serde_json::json!(["x", false, true]))?;
    equivalent::<Row>(serde_json::json!(["Ada", false, null]))?;
    equivalent::<BTreeMap<bool, String>>(serde_json::json!({"false":"no","true":"yes"}))?;
    equivalent::<BTreeMap<bool, String>>(serde_json::json!({"invalid":"no"}))?;
    equivalent::<BTreeMap<Option<i32>, String>>(serde_json::json!({"-2":"x"}))?;
    equivalent::<BTreeMap<NumberKey, String>>(serde_json::json!({"2":"x"}))?;
    equivalent::<BTreeMap<NamedKey, String>>(serde_json::json!({"First":"one","Second":"two"}))?;
    for key in [
        "-1",
        "0",
        "1.0",
        "1e2",
        " 1",
        "1 ",
        "01",
        "18446744073709551615",
        "invalid",
    ] {
        let json = serde_json::Value::Object(serde_json::Map::from_iter([(
            key.into(),
            serde_json::json!("x"),
        )]));
        equivalent::<BTreeMap<i64, String>>(json.clone())?;
        equivalent::<BTreeMap<u128, String>>(json)?;
    }
    Ok(())
}

/// No target visitor may run before compatibility and selected-budget checks complete.
#[test]
fn typed_prevalidation_runs_before_target_visitors() -> Result<()> {
    static VISITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    struct Observed;
    impl<'de> Deserialize<'de> for Observed {
        fn deserialize<D: serde::Deserializer<'de>>(
            decoder: D,
        ) -> std::result::Result<Self, D::Error> {
            VISITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            serde::de::IgnoredAny::deserialize(decoder)?;
            Ok(Self)
        }
    }
    let mut nest = Nest::new();
    nest.set_path("selected.ignored", Value::Bytes(vec![]))?;
    assert!(nest.path_as::<Observed>("selected").is_err());
    assert_eq!(VISITS.load(std::sync::atomic::Ordering::Relaxed), 0);
    nest.set("selected", Value::Null)?;
    assert!(nest.path_as::<Observed>("selected")?.is_some());
    assert_eq!(VISITS.load(std::sync::atomic::Ordering::Relaxed), 1);
    Ok(())
}

/// A serializer must receive no bytes when any rich child is unsupported or over budget.
#[test]
fn prevalidation_precedes_output_and_retains_error_precedence() -> Result<()> {
    let unsupported = [
        Value::Bytes(vec![1]),
        Value::DateTime(Timestamp::new(0, 0)?),
        Value::File(FileData::new(vec![], "name", "text/plain")),
        Value::Shared(SharedNative::new(1u32)),
    ];
    for value in unsupported {
        let value = Value::Object(DataMap::from([
            ("first".into(), Value::from("valid")),
            ("nested".into(), Value::Array(vec![Value::Null, value])),
        ]));
        let expected = value.to_json().unwrap_err().to_string();
        let mut bytes = Vec::new();
        let error = value
            .serialize(&mut serde_json::Serializer::new(&mut bytes))
            .unwrap_err();
        assert_eq!(error.to_string(), expected);
        assert!(error.to_string().contains("nested.1"));
        assert!(bytes.is_empty());
    }
    let value = Value::Array(vec![
        Value::Shared(SharedNative::new(1)),
        Value::from("x".repeat(1_048_577)),
    ]);
    let mut bytes = Vec::new();
    let error = value
        .serialize(&mut serde_json::Serializer::new(&mut bytes))
        .unwrap_err();
    assert!(error.to_string().contains("byte limit"));
    assert!(bytes.is_empty());
    Ok(())
}

/// Direct serialization retains insertion order, integer ranges, nulls and escaped strings.
#[test]
fn wire_output_matches_explicit_json_conversion() -> Result<()> {
    let value = Value::Object(DataMap::from([
        (
            "z".into(),
            Value::Array(vec![
                Value::Null,
                false.into(),
                u64::MAX.into(),
                (-7i64).into(),
            ]),
        ),
        ("a".into(), Value::from("quoted \" text\n")),
        ("float".into(), Value::number(1.25)?),
    ]));
    assert_eq!(
        serde_json::to_vec(&value)?,
        serde_json::to_vec(&value.to_json()?)?
    );
    assert!(serde_json::to_string(&value)?.starts_with("{\"z\":"));
    Ok(())
}

/// Empty literal keys in raw objects retain the exact legacy path-joining behavior.
#[test]
fn unsupported_paths_preserve_empty_literal_key_components() {
    for keys in [["", "x"], ["a", ""], ["", ""], ["a.b", "x"]] {
        let leaf = Value::Object(DataMap::from([("leaf".into(), Value::Bytes(vec![1]))]));
        let child = Value::Object(DataMap::from([(keys[1].into(), leaf)]));
        let value = Value::Object(DataMap::from([(keys[0].into(), child)]));
        assert_eq!(
            serde_json::to_string(&value).unwrap_err().to_string(),
            value.to_json().unwrap_err().to_string()
        );
    }
}

/// Serde's JSON rules include enums, numeric map keys, newtypes and tuple-shaped structs.
#[derive(Debug, Deserialize, PartialEq)]
enum Choice {
    Unit,
    Number(u64),
    Pair(String, bool),
    Record { count: i64 },
}

/// Owned target fields exercise output allocation without an intermediate payload copy.
#[derive(Debug, Deserialize, PartialEq)]
struct Row {
    name: String,
    active: bool,
    count: Option<u64>,
}

/// Typed selection preserves missing versus null and rejects unsupported ignored fields.
#[test]
fn typed_selection_preserves_json_deserialization_contracts() -> Result<()> {
    let mut nest = Nest::new();
    nest.set(
        "row",
        Value::try_from(
            serde_json::json!({"name":"Ada","active":false,"count":null,"ignored":[1,2]}),
        )?,
    )?;
    assert_eq!(
        nest.path_as::<Row>("row")?,
        Some(Row {
            name: "Ada".into(),
            active: false,
            count: None
        })
    );
    nest.set("null", Value::Null)?;
    assert_eq!(nest.path_as::<Option<u64>>("missing")?, None);
    assert_eq!(nest.path_as::<Option<u64>>("null")?, Some(None));
    for json in [
        serde_json::json!("Unit"),
        serde_json::json!({"Number":u64::MAX}),
        serde_json::json!({"Pair":["x",false]}),
        serde_json::json!({"Record":{"count":-7}}),
    ] {
        let expected: Choice = serde_json::from_value(json.clone())?;
        nest.set("choice", Value::try_from(json)?)?;
        assert_eq!(nest.path_as::<Choice>("choice")?, Some(expected));
    }
    nest.set(
        "keys",
        Value::try_from(serde_json::json!({"2":"two","-1":"negative"}))?,
    )?;
    assert_eq!(
        nest.path_as::<BTreeMap<i32, String>>("keys")?,
        Some(BTreeMap::from([(-1, "negative".into()), (2, "two".into())]))
    );
    nest.set_path("row.ignored", Value::Bytes(vec![1]))?;
    let error = nest.path_as::<Row>("row").unwrap_err();
    assert_eq!(error.path(), Some("row.ignored"));
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    Ok(())
}

/// Compare the old owned conversion with the selected adapter without a timing assertion.
#[test]
#[ignore = "focused performance receipt; run explicitly with --nocapture"]
fn json_adapter_performance_receipt() -> Result<()> {
    let value = Value::Array((0..500).map(|_| Value::from("x".repeat(1024))).collect());
    let rounds = 100;
    let start = Instant::now();
    for _ in 0..rounds {
        std::hint::black_box(serde_json::to_vec(&value.to_json()?)?);
    }
    let owned = start.elapsed();
    let start = Instant::now();
    for _ in 0..rounds {
        std::hint::black_box(serde_json::to_vec(&value)?);
    }
    let direct = start.elapsed();
    eprintln!(
        "serialization: {rounds} x 500 x 1024-byte strings; owned={owned:?}, adapter={direct:?}"
    );
    let mut nest = Nest::with_limits(ParseLimits::default());
    nest.set("values", value)?;
    let value = nest.get("values").unwrap();
    let start = Instant::now();
    for _ in 0..rounds {
        std::hint::black_box(serde_json::from_value::<Vec<String>>(value.to_json()?)?);
    }
    let owned = start.elapsed();
    let start = Instant::now();
    for _ in 0..rounds {
        std::hint::black_box(nest.path_as::<Vec<String>>("values")?);
    }
    eprintln!(
        "typed selection: owned={owned:?}, adapter={:?}",
        start.elapsed()
    );
    Ok(())
}
