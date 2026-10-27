//! Native generated input modes retain missing/null/set and private-model exposure policy.
#![cfg(feature = "names")]
use generated_workspace_app::generated::contracts::{
    operations::CreateOperations,
    public::{
        PublicCreateIdentity, PublicCreateOperations, PublicCreateSchema, PublicPatchOperations,
        PublicPatchSchema,
    },
    schema::CreateSchema,
};
use serde_json::json;
use stackpress_framework_schema::ValueContext;
/// Both optional and required named-model fields exercise client, host and trusted preparation.
#[test]
fn actual_named_input_modes() {
    let context = ValueContext::default();
    let input: PublicCreateOperations =
        serde_json::from_value(json!({"id":"named","value":"Value","schemaId":null})).unwrap();
    assert!(input.validate_client(&context).unwrap().is_empty());
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert_eq!(
        input.prepare_create(&context).unwrap()["schemaId"],
        serde_json::Value::Null
    );
    let patch: PublicPatchOperations =
        serde_json::from_value(json!({"value":"Patch","schemaId":null})).unwrap();
    assert!(patch.validate_client(&context).unwrap().is_empty());
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    let input: PublicCreateSchema =
        serde_json::from_value(json!({"id":"named","value":"Value","type":null,"active":true}))
            .unwrap();
    assert!(input.validate_client(&context).unwrap().is_empty());
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert_eq!(input.prepare_create(&context).unwrap()["active"], true);
    let patch: PublicPatchSchema =
        serde_json::from_value(json!({"value":"Patch","type":null,"active":true})).unwrap();
    assert!(patch.validate_client(&context).unwrap().is_empty());
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    // Trusted creation and input validation agree, without granting public access to Identity.id.
    let input: CreateSchema =
        serde_json::from_value(json!({"id":"named","value":"Value","type":7,"active":true}))
            .unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert_eq!(input.prepare_create(&context).unwrap().r#type_, Some(7));
    let input: CreateOperations =
        serde_json::from_value(json!({"id":"named","value":"Value","schemaId":"parent"})).unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert_eq!(
        input.prepare_create(&context).unwrap().schema_id.as_deref(),
        Some("parent")
    );
    let input: PublicCreateIdentity = serde_json::from_value(json!({})).unwrap();
    assert!(input.validate_client(&context).unwrap().is_empty());
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert_eq!(input.prepare_create(&context).unwrap(), json!({}));
    assert!(serde_json::from_value::<PublicCreateIdentity>(json!({"id":99})).is_err());
    for value in [
        json!({"id":null}),
        json!({"id":"named","value":null}),
        json!({"id":"named","active":null}),
    ] {
        let input: CreateSchema = serde_json::from_value(value).unwrap();
        assert!(input.prepare_with(&context).is_err());
    }
}
