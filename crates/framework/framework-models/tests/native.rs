//! First source-native generated-default parity against the original Node column oracle.
#![cfg(not(target_arch = "wasm32"))]
use serde_json::{Value as Json, json};
use stackpress_framework_models::{ApplicationSchema, AuthSchema, ProfileSchema, SessionSchema};
use stackpress_framework_schema::{SchemaResult, ValueProviders, source::*};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

/// Keep native Date construction explicit, as in the original source fixture runner.
fn input(case: &Json) -> Option<Value> {
    if case["nativeDate"] == true {
        return Some(date(1700000000123));
    }
    if case["nestedDate"] == true {
        return Some(Value::Array(vec![
            date(0),
            Value::Null,
            Value::Object(Record::new()),
        ]));
    }
    case.get("input").cloned().map(Value::from)
}

/// Represent undefined/error separately from JSON null, matching the Node oracle wrapper.
fn wire(value: SchemaResult<Option<Value>>) -> Json {
    match value {
        Ok(Some(value)) => json!({"value": to_json(&value).unwrap()}),
        Ok(None) => json!({"undefined":true}),
        Err(_) => json!({"error":true}),
    }
}

#[test]
fn generated_profile_auth_columns_match_original_node_goldens() {
    let fixture: Json =
        serde_json::from_str(include_str!("../../framework-session/tests/records.json")).unwrap();
    let profile = ProfileSchema::new("database-seed".into(), 0);
    let auth = AuthSchema::new("database-seed".into(), 0);
    let mut count = 0;
    for case in fixture["columns"].as_array().unwrap() {
        let columns = if case["model"] == "Profile" {
            profile.columns()
        } else {
            auth.columns()
        };
        let column = columns
            .iter()
            .find(|column| column.name() == case["name"])
            .unwrap();
        let value = input(case);
        assert_eq!(column.sql(), column.field.column);
        assert_eq!(
            column.nullable(),
            !column.field.required && !column.field.multiple
        );
        if column.name() == "token" {
            assert_eq!(column.kind(), Kind::Token);
        }
        if column.name() == "secret" {
            assert_eq!(column.kind(), Kind::Secret);
        }
        for (name, result) in [
            (
                "serialize",
                column.serialize(value.as_ref(), "database-seed", false, 0),
            ),
            (
                "unserialize",
                column.unserialize(value.as_ref(), "database-seed", false, 0),
            ),
        ] {
            assert_eq!(
                wire(result),
                case[name],
                "{} {} {name} input={value:?}",
                case["model"],
                case["name"]
            );
        }
        assert_eq!(
            column
                .assert(value.as_ref())
                .map(Json::String)
                .unwrap_or(Json::Null),
            case["assert"],
            "{} {} assertion input={value:?}",
            case["model"],
            case["name"]
        );
        count += 1;
    }
    assert!(count > 100);
}

/// Count host defaults without invoking randomness, and expose provider errors unchanged.
struct Providers(AtomicUsize);
impl ValueProviders for Providers {
    fn generate(&self, name: &str, _: &[Json]) -> SchemaResult<Json> {
        assert_eq!(name, "cuid");
        Ok(json!(format!(
            "id-{}",
            self.0.fetch_add(1, Ordering::Relaxed)
        )))
    }
    fn validate(&self, _: &str, _: &Json, _: &[Json]) -> SchemaResult<bool> {
        unreachable!("defaults never invoke a host assertion")
    }
}

#[test]
fn defaults_are_metadata_driven_fresh_and_preserve_dates_order_and_false() {
    let providers = Arc::new(Providers(AtomicUsize::new(0)));
    let profile = ProfileSchema::new("seed".into(), 480).with_providers(providers.clone());
    assert_eq!(
        providers.0.load(Ordering::Relaxed),
        0,
        "construction has no provider IO"
    );
    let first = profile.defaults_at(42).unwrap();
    let second = profile.defaults_at(43).unwrap();
    assert_ne!(first["id"], second["id"]);
    assert_eq!(first["created"], date(42));
    assert_eq!(first["updated"], date(42));
    assert!(!first.contains_key("image"));
    assert_eq!(
        first.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "id",
            "type",
            "roles",
            "tags",
            "references",
            "active",
            "created",
            "updated"
        ]
    );
    let auth = AuthSchema::new("seed".into(), 0);
    assert_eq!(auth.defaults_at(42).unwrap()["verified"], false);
    // A generated secret default is not tied to the legacy `id` field-name getter.
    let application = ApplicationSchema::new("seed".into(), 0).with_providers(providers.clone());
    let defaults = application.defaults_at(42).unwrap();
    assert!(defaults["secret"].as_str().unwrap().starts_with("id-"));
    assert!(!defaults.contains_key("expires"));
    let session = SessionSchema::new("seed".into(), 0).with_providers(providers);
    assert!(
        session.defaults_at(42).unwrap()["secret"]
            .as_str()
            .unwrap()
            .starts_with("id-")
    );
    assert_eq!(profile.offset_minutes(), 480);
    assert_eq!(profile.seed(), "seed");
}

#[test]
fn source_filter_populate_partial_assertion_and_storage_phases() {
    let profile = ProfileSchema::new("private-seed".into(), 0);
    let input: Record =
        serde_json::from_value(json!({"name":"Ada","image":null,"extension":"owned"})).unwrap();
    assert_eq!(
        profile
            .filter(&input, false)
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["name", "image"]
    );
    assert!(profile.populate(&input).unwrap().contains_key("extension"));
    assert!(
        !profile
            .filter(&input, true)
            .unwrap()
            .contains_key("extension")
    );
    assert!(profile.assert(&Record::new(), false).is_none());
    assert!(
        profile
            .assert(&Record::new(), true)
            .unwrap()
            .contains_key("id")
    );
    assert_eq!(
        profile.serialize(&Record::new(), false).unwrap()["roles"],
        "[]"
    );
    assert_eq!(
        profile.unserialize(&Record::new(), false).unwrap()["tags"],
        json!([])
    );
    let auth = AuthSchema::new("database-seed".into(), 0);
    // Source/Zod length is UTF-16 units: two surrogate pairs plus one scalar reaches five.
    let token = auth
        .columns()
        .iter()
        .find(|column| column.name() == "token")
        .unwrap();
    assert!(token.assert(Some(&Value::String("😀😀a".into()))).is_none());
    assert_eq!(
        token.assert(Some(&Value::String("😀a".into()))).as_deref(),
        Some("Must be at least 5 characters")
    );
    let input: Record = serde_json::from_value(
        json!({"token":"john@doe.com","secret":"password","verified":false}),
    )
    .unwrap();
    assert!(auth.assert(&input, false).is_none());
    assert_eq!(
        auth.serialize(&input, false).unwrap()["token"],
        "john@doe.com"
    );
    let encoded = auth.serialize(&input, true).unwrap();
    assert_eq!(
        encoded["token"],
        Value::String(stackpress_framework_schema::providers::encrypt(
            "john@doe.com",
            "database-seed"
        ))
    );
    assert_eq!(
        encoded["secret"],
        Value::String(stackpress_framework_schema::providers::hash("password"))
    );
    assert_eq!(
        auth.unserialize(&encoded, true).unwrap()["token"],
        "john@doe.com"
    );
    assert_eq!(
        auth.unserialize(&encoded, true).unwrap()["secret"],
        encoded["secret"]
    );
    assert!(
        !serde_json::to_string(&ProfileSchema::describe())
            .unwrap()
            .contains("private-seed")
    );
}

#[test]
fn generated_core_witnesses_share_the_native_family_without_repairs() {
    fn profile<S: ProfileFields>() {}
    fn auth<S: AuthFields>() {}
    fn application<S: ApplicationFields>() {}
    fn session<S: SessionFields>() {}
    profile::<ProfileSchema>();
    auth::<AuthSchema>();
    application::<ApplicationSchema>();
    session::<SessionSchema>();
    fn owned<S: Send + Sync + Clone + 'static>() {}
    owned::<ProfileSchema>();
    owned::<AuthSchema>();
    owned::<ApplicationSchema>();
    owned::<SessionSchema>();
    let mut record = Record::new();
    <ProfileSchema as SourceField<Name>>::write(&mut record, "Ada".into());
    record.insert("extra".into(), Value::Null);
    assert_eq!(
        <ProfileSchema as SourceField<Name>>::read(&record),
        Some("Ada".into())
    );
    assert!(record.contains_key("extra"));
}

/// Exercise the declared value contract, including reverse relations, without a second DTO.
fn accessor<S, Marker>(value: Value)
where
    S: SourceSchema<Record = Record> + SourceField<Marker>,
    <S as SourceField<Marker>>::FieldValue: NativeField,
{
    let native = <<S as SourceField<Marker>>::FieldValue as NativeField>::read(&value).unwrap();
    let mut record = Record::new();
    record.insert("app-owned".into(), Value::Null);
    <S as SourceField<Marker>>::write(&mut record, native);
    assert_eq!(
        <S as SourceField<Marker>>::read(&record).unwrap().value(),
        value
    );
    assert_eq!(record.keys().next().unwrap(), "app-owned");
}

/// The complete 48-field core contract is exercised, not only identity accessors.
macro_rules! core {
    ($schema:ty; $($marker:ty => $value:expr),* $(,)?) => { $(accessor::<$schema, $marker>($value);)* };
}

#[test]
fn all_generated_accessors_preserve_their_actual_native_types() {
    let text = Value::String("source".into());
    let boolean = Value::Bool(false);
    let time = date(42);
    let list = Value::from(json!(["one", "two"]));
    let object = Value::from(json!({"z":1,"a":null}));
    let relations = Value::Array(vec![object.clone()]);
    core!(ProfileSchema;
        Id=>text.clone(), Image=>Value::Null, Name=>text.clone(), Type=>text.clone(),
        Roles=>list.clone(), Tags=>list.clone(), References=>object.clone(), Active=>boolean.clone(),
        Created=>time.clone(), Updated=>time.clone(), Auth=>relations.clone(), Applications=>relations.clone(), Sessions=>relations.clone());
    core!(AuthSchema;
        Id=>text.clone(), ProfileId=>text.clone(), Type=>text.clone(), Token=>text.clone(), Secret=>text.clone(),
        Verified=>boolean.clone(), Consumed=>time.clone(), Active=>boolean.clone(), Created=>time.clone(), Updated=>time.clone(), Profile=>object.clone());
    core!(ApplicationSchema;
        Id=>text.clone(), ProfileId=>text.clone(), Name=>text.clone(), Logo=>Value::Null, Website=>Value::Null,
        Secret=>text.clone(), Scopes=>list.clone(), Active=>boolean.clone(), Expires=>time.clone(), Created=>time.clone(), Updated=>time.clone(), Session=>relations.clone(), Profile=>object.clone());
    core!(SessionSchema;
        Id=>text.clone(), ApplicationId=>text.clone(), ProfileId=>text.clone(), Secret=>text, Scopes=>list, Active=>boolean,
        Expires=>time.clone(), Created=>time.clone(), Updated=>time, Application=>object.clone(), Profile=>object);
}

/// All generated schema delegates operate on the same shared runtime and declared metadata.
fn schema_surface<S: SourceSchema<Value = Value, Record = Record>>() {
    let schema = S::new("seed".into(), 0);
    assert_eq!(schema.seed(), "seed");
    assert_eq!(schema.offset_minutes(), 0);
    let defaults = schema.defaults().unwrap();
    assert!(defaults.contains_key("id"));
    assert_eq!(schema.defaults_at(42).unwrap()["created"], date(42));
    assert!(schema.assert(&defaults, false).is_none());
    assert!(schema.assert(&Record::new(), true).is_some());
    assert_eq!(schema.filter(&defaults, false).unwrap(), defaults);
    assert!(
        schema
            .filter(&Record::new(), true)
            .unwrap()
            .contains_key("id")
    );
    assert!(schema.populate(&Record::new()).unwrap().contains_key("id"));
    let encoded = schema.serialize(&defaults, false).unwrap();
    assert!(
        schema
            .unserialize(&encoded, false)
            .unwrap()
            .contains_key("id")
    );
    assert!(!S::describe().fields.is_empty());
}

#[test]
fn all_four_schemas_and_generated_catalog_are_independent_of_plugins() {
    schema_surface::<ProfileSchema>();
    schema_surface::<AuthSchema>();
    schema_surface::<ApplicationSchema>();
    schema_surface::<SessionSchema>();
    let catalog = stackpress_framework_models::generated::source::catalog::describe();
    assert_eq!(
        catalog.models,
        stackpress_framework_models::generated::source::catalog::models()
    );
    assert_eq!(catalog.models.len(), 4);
    assert_eq!(catalog.relations.len(), 8);
    assert!(stackpress_framework_models::generated::source::catalog::EXTERNAL_MODELS.is_empty());
    assert_eq!(
        catalog
            .models
            .iter()
            .map(|model| model.fields.len())
            .sum::<usize>(),
        48
    );
}

#[test]
fn generated_storage_codec_selection_matches_original_node_crypto_vectors() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../framework-schema/src/providers/vectors.json"
    ))
    .unwrap();
    for case in fixture["cryptoVectors"].as_array().unwrap() {
        let value = case["value"].as_str().unwrap();
        let schema = AuthSchema::new(case["seed"].as_str().unwrap().into(), 0);
        let input: Record = [
            ("token".into(), Value::String(value.into())),
            ("secret".into(), Value::String(value.into())),
        ]
        .into();
        let encoded = schema.serialize(&input, true).unwrap();
        // The source column bypasses its codec for empty text; helper vectors encode it.
        if value.is_empty() {
            assert_eq!(encoded["token"], "");
            assert_eq!(encoded["secret"], "");
        } else {
            assert_eq!(encoded["token"], case["ciphertext"]);
            assert_eq!(encoded["secret"], case["hash"]);
        }
        assert_eq!(schema.unserialize(&encoded, true).unwrap()["token"], value);
    }
}
