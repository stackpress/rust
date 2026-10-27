//! Actual RoleProof contract behavior at the UI/public/private boundary.
use generated_admin_shared::contracts::{
    public::{
        PatchRoleProof, PublicCreateRoleProof, PublicPatchRoleProof, RoleProofFilter,
        RoleProofQuery,
    },
    role_proof::CreateRoleProof,
};
use serde_json::{Value, json};
use stackpress_framework_schema::{Input, Predicate, SchemaResult, ValueContext, ValueProviders};

/// A named host capability supplies the private default without embedding it into the UI.
struct Host;
impl ValueProviders for Host {
    /// Only the explicitly declared private default may execute in this fixture.
    fn generate(&self, name: &str, args: &[Value]) -> SchemaResult<Value> {
        assert_eq!(name, "default:RoleProof.sensitive");
        assert!(args.is_empty());
        Ok(json!("host-only-sensitive-value"))
    }

    /// This schema declares no host validators, so an unexpected call is a fixture failure.
    fn validate(&self, _: &str, _: &Value, _: &[Value]) -> SchemaResult<bool> {
        panic!("RoleProof has no registered host validators")
    }
}

/// Missing defaults, explicit null and supplied false/zero survive actual generated contracts.
#[test]
fn normalized_boundary_defaults_and_exact_supplied_values() {
    let context = ValueContext {
        providers: Some(&Host),
        ..Default::default()
    };
    let create: CreateRoleProof = serde_json::from_value(json!({"id":"proof"})).unwrap();
    assert!(
        create.clone().prepare().is_err(),
        "private host default requires an explicit provider"
    );
    let stored = create.prepare_with(&context).unwrap();
    assert_eq!(stored.text.as_deref(), Some("fixture default"));
    assert!(!stored.flag);
    assert_eq!(stored.zero, 0);
    assert_eq!(
        stored.sensitive.as_deref(),
        Some("host-only-sensitive-value")
    );
    stored.validate().unwrap();
    stored.validate_with(&context, false).unwrap();
    let public = serde_json::to_value(stored.project().unwrap()).unwrap();
    assert_eq!(
        public,
        json!({"id":"proof","text":"fixture default","flag":false,"zero":0,"rows":null,"metadata":null,"score":null,"slug":null})
    );
    assert!(!public.to_string().contains("sensitive"));
    for text in [Value::Null, json!(""), json!("supplied")] {
        let create: CreateRoleProof = serde_json::from_value(json!({"id":"proof","text":text,"flag":false,"zero":0,"sensitive":null,"hostOnly":null})).unwrap();
        let stored = create.prepare().unwrap();
        assert_eq!(serde_json::to_value(&stored).unwrap()["text"], text);
        assert!(!stored.flag);
        assert_eq!(stored.zero, 0);
        assert!(stored.sensitive.is_none());
        assert!(stored.host_only.is_none());
    }
    let supplied: CreateRoleProof = serde_json::from_value(json!({"id":"proof","text":"x","flag":true,"zero":12,"sensitive":"secret","hostOnly":"private"})).unwrap();
    let stored = supplied.prepare().unwrap();
    assert!(stored.flag);
    assert_eq!(stored.zero, 12);
    let public = serde_json::to_value(stored.project().unwrap()).unwrap();
    assert!(public.get("sensitive").is_none());
    assert!(public.get("hostOnly").is_none());
    for invalid in [
        json!({}),
        json!({"id":null}),
        json!({"id":"proof","flag":null}),
        json!({"id":"proof","zero":null}),
    ] {
        let create: CreateRoleProof = serde_json::from_value(invalid).unwrap();
        assert!(create.prepare_with(&context).is_err());
    }
}

/// Public create/patch checks preserve data and refuse host-only fields before any write.
#[test]
fn public_inputs_and_queries_keep_the_same_projection_policy() {
    let context = ValueContext {
        providers: Some(&Host),
        ..Default::default()
    };
    let public: PublicCreateRoleProof = serde_json::from_value(
        json!({"id":"proof","text":null,"flag":false,"zero":0,"sensitive":"new input"}),
    )
    .unwrap();
    assert!(public.validate_client(&context).unwrap().issues.is_empty());
    assert!(public.validate_create(&context).unwrap().issues.is_empty());
    let prepared = public.prepare_create(&context).unwrap();
    assert_eq!(prepared["flag"], false);
    assert_eq!(prepared["zero"], 0);
    assert!(prepared["text"].is_null());
    assert!(
        serde_json::from_value::<PublicCreateRoleProof>(json!({"id":"proof","hostOnly":"denied"}))
            .is_err()
    );
    let trusted = CreateRoleProof {
        id: Input::Value("proof".into()),
        sensitive: Input::Null,
        ..Default::default()
    };
    assert!(trusted.validate_create(&context).unwrap().issues.is_empty());
    trusted.prepare_create(&context).unwrap();
    let patch: PublicPatchRoleProof =
        serde_json::from_value(json!({"text":null,"flag":false,"zero":0,"sensitive":"changed"}))
            .unwrap();
    assert!(patch.validate_client(&context).unwrap().issues.is_empty());
    assert!(patch.validate_patch(&context).unwrap().issues.is_empty());
    let patch: PatchRoleProof =
        serde_json::from_value(json!({"text":null,"flag":false,"zero":0,"hostOnly":"trusted"}))
            .unwrap();
    patch.validate(&context).unwrap();
    assert!(patch.validate_patch(&context).unwrap().issues.is_empty());
    assert!(PatchRoleProof::default().validate(&context).is_err());
    let filter = RoleProofFilter::Text(Predicate::Eq(String::new()));
    filter.validate().unwrap();
    RoleProofFilter::Id(Predicate::Eq("proof".into()))
        .validate()
        .unwrap();
    RoleProofFilter::Text(Predicate::IsNull(true))
        .validate()
        .unwrap();
    let mut query = RoleProofQuery {
        filters: vec![filter],
        ..Default::default()
    };
    query.validate().unwrap();
    query.take = 0;
    assert!(query.validate().is_err());
    query.take = 20;
    query.q = Some("unavailable search".into());
    assert!(query.validate().is_err());
    query.q = None;
    query.includes = vec!["hostOnly".into()];
    assert!(query.validate().is_err());
}

/// A handwritten app replacement has the exact generated component interface on SSR too.
#[cfg(feature = "ssr")]
#[test]
fn app_component_accepts_the_public_role_interface() {
    use generated_admin_shared::{plugin::AccentField, ui::profile};
    use leptos::prelude::*;
    use stackpress_framework_admin::{Draft, RoleContext};
    use stackpress_framework_schema::Role;
    let owner = Owner::new();
    owner.with(|| {
        let fields = profile::descriptors().unwrap();
        let draft = Draft::new(&fields, &json!({"name":"plugin"}), Role::Field).unwrap();
        let context = RoleContext {
            id: "custom".into(),
            field: fields
                .into_iter()
                .find(|field| field.wire_name == "name")
                .unwrap(),
            role: Role::Field,
            value: draft.get("name").unwrap(),
        };
        assert!(
            view! { <AccentField context/> }
                .to_html()
                .contains("data-admin-plugin=\"accent\"")
        );
    });
    owner.cleanup();
}

/// Collection replacements compile against the same metadata/state interface as generated calls.
#[cfg(feature = "ssr")]
#[test]
fn app_collection_component_keeps_the_exact_role_interface() {
    use generated_admin_shared::{plugin::AccentTags, ui::project};
    use leptos::prelude::*;
    use stackpress_framework_admin::{Draft, RoleContext};
    use stackpress_framework_schema::Role;
    let owner = Owner::new();
    owner.with(|| {
        let fields = project::descriptors().unwrap();
        let draft = Draft::new(&fields, &json!({"tags":["", "0"]}), Role::Field).unwrap();
        let context = RoleContext {
            id: "plugin-tags".into(),
            field: fields
                .into_iter()
                .find(|field| field.wire_name == "tags")
                .unwrap(),
            role: Role::Field,
            value: draft.get("tags").unwrap(),
        };
        assert!(
            view! {<AccentTags context/>}
                .to_html()
                .contains("data-admin-plugin=\"tags\"")
        );
        assert_eq!(draft.snapshot().unwrap()["tags"], json!(["", "0"]));
    });
    owner.cleanup();
}
