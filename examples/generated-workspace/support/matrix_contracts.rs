//! The exact generated default/value/access rules run in both pure and native consumers.
use super::{
    contracts::{
        matrix::{CreateMatrix, Matrix},
        public::{MatrixKey, MatrixQuery, MatrixSort, PatchMatrix, PublicCreateMatrix},
    },
    metadata,
};
use serde_json::{Value, json};
use stackpress_framework_schema::{
    DatetimeValue, Input, Patch, SchemaResult, ValidationErrors, ValueContext, ValueProviders,
};
struct Host;
impl ValueProviders for Host {
    fn generate(&self, name: &str, args: &[Value]) -> SchemaResult<Value> {
        match name {
            "nanoid" => {
                assert_eq!(args, &[json!(8)]);
                Ok(json!("fixed-id"))
            }
            "default:Matrix.secret" => Ok(json!("M1_PRIVATE_DEFAULT_SENTINEL")),
            _ => panic!("unexpected provider"),
        }
    }
    fn validate(&self, name: &str, value: &Value, _: &[Value]) -> SchemaResult<bool> {
        assert_eq!(name, "app.approved");
        Ok(value.as_str() == Some("PH"))
    }
}
/// Preparation is deterministic with the injected clock, retains literal false/zero/empty, and projects safely.
#[test]
fn real_matrix_preparation_projection_and_patch() {
    let clock = DatetimeValue::parse("2024-06-01T00:00:00.123456789Z").unwrap();
    let host = Host;
    let context = ValueContext {
        now: Some(&clock),
        providers: Some(&host),
    };
    let input: CreateMatrix =
        serde_json::from_value(json!({"id":1,"address":{"city":"Paris","zip":"PRIVATE_ZIP"}}))
            .unwrap();
    let value = input.prepare_with(&context).unwrap();
    assert_eq!(value.count, 0);
    assert!(!value.flag);
    assert!(value.tags.is_empty());
    assert_eq!(value.real.get(), 0.25);
    assert_eq!(value.created, clock);
    assert_eq!(value.time.as_str(), "12:00:00.12");
    assert_eq!(value.stamp.as_str(), "2024-03-01T00:00:00Z");
    assert_eq!(value.token, "fixed-id");
    assert!(value.validate().is_ok());
    let roundtrip: Matrix = serde_json::from_value(serde_json::to_value(&value).unwrap()).unwrap();
    assert_eq!(value, roundtrip);
    let public = serde_json::to_value(value.project().unwrap()).unwrap();
    let encoded = public.to_string();
    assert!(!encoded.contains("PRIVATE"));
    assert_eq!(public["address"], json!({"city":"Paris","country":"PH"}));
    let ui = metadata::ui::schema().unwrap();
    assert_eq!(ui.contract_revision, 3);
    assert_eq!(ui.schema_fingerprint, metadata::schema::FINGERPRINT);
    assert!(!serde_json::to_string(&ui).unwrap().contains("PRIVATE"));
    assert_eq!(
        ui.models
            .iter()
            .find(|model| model.id == "Matrix")
            .unwrap()
            .key,
        vec!["id"]
    );
    let input: CreateMatrix = serde_json::from_value(json!({"id":1})).unwrap();
    assert!(input.prepare().is_err());
    for input in [
        json!({"id":1,"token":"forged"}),
        json!({"id":1,"created":"2024-01-01T00:00:00Z"}),
        json!({"id":1,"choice":2}),
        json!({"id":1,"truth":"false"}),
        json!({"id":1,"day":"2023-02-29"}),
        json!({"id":1,"numbers":[9007199254740993_i64]}),
    ] {
        assert!(serde_json::from_value::<CreateMatrix>(input).is_err());
    }
    let bad: CreateMatrix = serde_json::from_value(json!({"id":1,"text":"x"})).unwrap();
    let error = bad.prepare_with(&context).unwrap_err();
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<ValidationErrors>()
            .is_some()
    );
    let empty = PatchMatrix::default();
    assert!(empty.validate(&context).is_err());
    let patch: PatchMatrix =
        serde_json::from_value(json!({"text":"okay","flag":false,"float":null})).unwrap();
    assert_eq!(patch.flag, Patch::Set(false));
    assert_eq!(patch.float, Patch::Null);
    patch.validate(&context).unwrap();
    let bad: PatchMatrix = serde_json::from_value(json!({"text":null})).unwrap();
    assert!(bad.validate(&context).is_err());
    assert!(serde_json::from_value::<PatchMatrix>(json!({"token":"no"})).is_err());
    assert!(serde_json::from_value::<PatchMatrix>(json!({"id":2})).is_err());
    assert!(serde_json::from_value::<PublicCreateMatrix>(json!({"secret":"no"})).is_err());
    let public_input: PublicCreateMatrix =
        serde_json::from_value(json!({"id":1,"text":"okay"})).unwrap();
    assert_eq!(public_input.id, Input::Value(1));
    let key = MatrixKey { id: 1 };
    assert_eq!(serde_json::to_value(key).unwrap(), json!({"id":1}));
    let mut query = MatrixQuery::default();
    query.sort.push(stackpress_framework_schema::Sort {
        field: MatrixSort::Text,
        direction: stackpress_framework_schema::SortDirection::Asc,
        nulls: stackpress_framework_schema::NullOrder::Last,
    });
    query.validate().unwrap();
    query.take = 0;
    assert!(query.validate().is_err());
    query.take = 100;
    query.skip = 100_001;
    assert!(query.validate().is_err());
}

/// Public inputs share checked business rules while host preparation stays explicit and single-use.
#[test]
fn public_input_and_typed_query_contracts() {
    use super::contracts::public::{MatrixFilter, PublicPatchMatrix};
    use stackpress_framework_schema::Predicate;
    let host = Host;
    let context = ValueContext {
        now: None,
        providers: Some(&host),
    };
    let input: PublicCreateMatrix = serde_json::from_value(json!({"id":1})).unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    let prepared = input.prepare_create(&context).unwrap();
    assert_eq!(prepared["text"], "Café");
    assert_eq!(prepared["flag"], false);
    assert!(!prepared.as_object().unwrap().contains_key("created"));
    assert!(!prepared.as_object().unwrap().contains_key("secret"));
    let bad: PublicCreateMatrix = serde_json::from_value(json!({"id":1,"text":"x"})).unwrap();
    assert!(!bad.validate_client(&context).unwrap().is_empty());
    assert!(!bad.validate_create(&context).unwrap().is_empty());
    let empty: PublicPatchMatrix = serde_json::from_value(json!({})).unwrap();
    assert!(!empty.validate_patch(&context).unwrap().is_empty());
    let patch: PublicPatchMatrix =
        serde_json::from_value(json!({"flag":false,"float":null})).unwrap();
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    for input in [
        json!({"secret":"blocked"}),
        json!({"id":2}),
        json!({"token":"forged"}),
    ] {
        assert!(serde_json::from_value::<PublicPatchMatrix>(input).is_err());
    }
    let too_large: PublicCreateMatrix =
        serde_json::from_value(json!({"id":9007199254740992_i64})).unwrap();
    assert!(too_large.validate_create(&context).is_err());
    let mut query: MatrixQuery = serde_json::from_value(json!({})).unwrap();
    assert_eq!(query.take, 20);
    query.filters.push(MatrixFilter::Id(Predicate::Eq(1)));
    query.validate().unwrap();
    query.filters.push(MatrixFilter::Id(Predicate::Like(1)));
    assert!(query.validate().is_err());
    query.filters.pop();
    query
        .filters
        .push(MatrixFilter::Id(Predicate::IsNull(true)));
    assert!(query.validate().is_err());
    query.filters.pop();
    query.includes.push("secret".into());
    assert!(query.validate().is_err());
    query.includes.clear();
    query.q = Some("private query".into());
    assert!(query.validate().is_err());
    for input in [
        json!({"filters":[{"field":"secret","predicate":{"op":"eq","value":"x"}}]}),
        json!({"sort":[{"field":"secret","direction":"asc","nulls":"last"}]}),
        json!({"force":true}),
    ] {
        assert!(serde_json::from_value::<MatrixQuery>(input).is_err());
    }
    let mut record: CreateMatrix =
        serde_json::from_value(json!({"id":9007199254740992_i64})).unwrap();
    record.secret = Input::Null;
    let host = Host;
    let clock = DatetimeValue::parse("2024-01-01T00:00:00Z").unwrap();
    let host_context = ValueContext {
        now: Some(&clock),
        providers: Some(&host),
    };
    let record = record.prepare_create(&host_context).unwrap();
    assert!(record.project().is_err());
}

/// Embedded defaults/generators and host rules execute once per instance on one explicit clock.
#[test]
fn nested_preparation_is_single_and_bounded() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Counting {
        generated: AtomicUsize,
        validated: AtomicUsize,
    }
    impl ValueProviders for Counting {
        fn generate(&self, name: &str, _: &[Value]) -> SchemaResult<Value> {
            if name == "nanoid" {
                let index = self.generated.fetch_add(1, Ordering::SeqCst);
                Ok(json!(format!("id-{index}")))
            } else {
                assert_eq!(name, "default:Matrix.secret");
                Ok(Value::Null)
            }
        }
        fn validate(&self, name: &str, value: &Value, _: &[Value]) -> SchemaResult<bool> {
            assert_eq!(name, "app.approved");
            self.validated.fetch_add(1, Ordering::SeqCst);
            Ok(value.as_str() == Some("PH"))
        }
    }
    let clock = DatetimeValue::parse("2024-06-01T00:00:00.000000001Z").unwrap();
    let host = Counting {
        generated: AtomicUsize::new(0),
        validated: AtomicUsize::new(0),
    };
    let context = ValueContext {
        now: Some(&clock),
        providers: Some(&host),
    };
    let input: CreateMatrix = serde_json::from_value(
        json!({"id":1,"address":{"city":"A"},"siblings":[{"city":"B"},{"city":"C"}]}),
    )
    .unwrap();
    let value = input.prepare_create(&context).unwrap();
    assert_eq!(host.generated.load(Ordering::SeqCst), 5);
    assert_eq!(host.validated.load(Ordering::SeqCst), 4);
    assert_eq!(value.favorite.city, "Default");
    assert_eq!(value.favorite.country, "PH");
    assert_eq!(value.created, clock);
    assert_eq!(value.siblings.len(), 2);
    let public = value.project().unwrap();
    assert_eq!(public.favorite.country, "PH");
    assert_eq!(host.validated.load(Ordering::SeqCst), 4);
    assert!(
        serde_json::from_value::<CreateMatrix>(
            json!({"id":1,"address":{"city":"A","nonce":"forged"}})
        )
        .is_err()
    );
    use super::contracts::{count::Count, truth::Truth};
    for value in [Count::Zero, Count::One] {
        let wire = serde_json::to_value(&value).unwrap();
        assert_eq!(serde_json::from_value::<Count>(wire).unwrap(), value);
    }
    for value in [Truth::No, Truth::Yes] {
        let wire = serde_json::to_value(&value).unwrap();
        assert_eq!(serde_json::from_value::<Truth>(wire).unwrap(), value);
    }
}

/// Supplied values survive every codec/default boundary; null never falls through to a default.
#[test]
fn supplied_matrix_values_and_null_policy() {
    let clock = DatetimeValue::parse("2024-06-01T00:00:00Z").unwrap();
    let host = Host;
    let context = ValueContext {
        now: Some(&clock),
        providers: Some(&host),
    };
    let supplied = json!({
        "id": 7, "text": "Supplied", "count": 8, "real": 1.5, "float": 2.5,
        "flag": true, "day": "2025-01-02", "time": "10:11:12.3",
        "stamp": "2025-01-02T01:02:03Z", "object": {"deep": [false, null]},
        "choice": 1, "truth": true, "tags": ["one"], "numbers": [0.5, 2],
        "dates": ["2025-01-02"], "times": ["10:11:12"],
        "instants": ["2025-01-02T02:02:03+01:00"], "objects": [{"x":false}],
        "address": {"city":"Oslo", "country":"PH", "zip":"PRIVATE"},
        "siblings": [{"city":"Paris", "country":"PH"}],
        "favorite": {"city":"Tokyo", "country":"PH"}, "secret": "PRIVATE"
    });
    let input: CreateMatrix = serde_json::from_value(supplied).unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    let stored = input.prepare_create(&context).unwrap();
    let wire = serde_json::to_value(&stored).unwrap();
    assert_eq!(wire["text"], "Supplied");
    assert_eq!(wire["count"], 8);
    assert_eq!(wire["real"], 1.5);
    assert_eq!(wire["float"], 2.5);
    assert_eq!(wire["flag"], true);
    assert_eq!(wire["choice"], 1);
    assert_eq!(wire["truth"], true);
    assert_eq!(wire["instants"][0], "2025-01-02T01:02:03Z");
    assert_eq!(wire["objects"][0]["x"], false);
    assert_eq!(wire["favorite"]["city"], "Tokyo");
    let roundtrip: Matrix = serde_json::from_value(wire).unwrap();
    assert_eq!(stored, roundtrip);
    assert!(
        !serde_json::to_string(&stored.project().unwrap())
            .unwrap()
            .contains("PRIVATE")
    );
    // The full required-value matrix shares one public rule: null is an error even with a default.
    for field in [
        "id", "text", "count", "real", "flag", "day", "time", "stamp", "object", "choice", "truth",
        "tags", "numbers", "dates", "times", "instants", "objects", "siblings", "favorite",
    ] {
        let mut wire = json!({"id":1});
        wire[field] = Value::Null;
        let input: CreateMatrix = serde_json::from_value(wire).unwrap();
        let error = input.prepare_with(&context).unwrap_err();
        assert_eq!(
            error.kind(),
            stackpress_framework_schema::SchemaErrorKind::Invalid,
            "{field}"
        );
        assert_eq!(
            error.to_string(),
            "required create field is null",
            "{field}"
        );
    }
    // Nullable members preserve explicit null while provider-backed required fields still resolve.
    let input: CreateMatrix =
        serde_json::from_value(json!({"id":1,"float":null,"address":null,"secret":null})).unwrap();
    let value = input.prepare_with(&context).unwrap();
    assert!(value.float.is_none() && value.address.is_none() && value.secret.is_none());
}

/// Standalone fieldsets enforce the same defaults and host/client validation as nested inputs.
#[test]
fn standalone_fieldset_and_host_client_policy() {
    use super::contracts::{
        matrix_address::CreateMatrixAddress, public::PublicCreateMatrixAddress,
    };
    let host = Host;
    let context = ValueContext {
        now: None,
        providers: Some(&host),
    };
    let input: CreateMatrixAddress =
        serde_json::from_value(json!({"city":"Oslo","country":"PH","zip":"PRIVATE"})).unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    let value = input.prepare_create(&context).unwrap();
    assert_eq!(value.nonce, "fixed-id");
    assert_eq!(
        serde_json::to_value(value.project().unwrap()).unwrap(),
        json!({"city":"Oslo","country":"PH"})
    );
    let input: PublicCreateMatrixAddress = serde_json::from_value(json!({"city":"Oslo"})).unwrap();
    assert!(
        input
            .validate_client(&ValueContext::default())
            .unwrap()
            .is_empty()
    );
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert_eq!(
        input.prepare_create(&context).unwrap(),
        json!({"city":"Oslo","country":"PH"})
    );
    let input: PublicCreateMatrixAddress =
        serde_json::from_value(json!({"city":"Oslo","country":"XX"})).unwrap();
    assert!(
        input
            .validate_client(&ValueContext::default())
            .unwrap()
            .is_empty()
    );
    let errors = input.validate_create(&context).unwrap();
    assert_eq!(errors.issues[0].path, vec!["country"]);
    assert!(!serde_json::to_string(&errors).unwrap().contains("XX"));
    assert!(input.prepare_create(&context).is_err());
    assert!(
        serde_json::from_value::<PublicCreateMatrixAddress>(
            json!({"city":"Oslo","nonce":"forged"})
        )
        .is_err()
    );
}
