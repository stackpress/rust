//! Shared native lifecycle matrix: each generated callable observes one real SQLite identity.
macro_rules! native_calls {
    ($module:ident, $models:ident, $store:ident, $create:ty, $key_type:ty, $patch_type:ty,
     $input:expr, $identity:expr, $patch:expr, $session:expr, $context:expr) => {{
        use generated_workspace_app::generated::stores::$module as model;
        use stackpress_sql::Render;
        let raw = $input;
        let identity = $identity;
        let context = $context;
        let session = &mut *$session;
        let key: $key_type = serde_json::from_value(identity.clone()).unwrap();
        let input: $create = serde_json::from_value(raw.clone()).unwrap();
        // The first insert acknowledges a row; the second acknowledges no change without a row.
        assert!(matches!(model::$models::create_if_absent(session, input, context).await.unwrap(), stackpress_framework_sql::InsertOutcome::Inserted(_)));
        let input: $create = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(model::$models::create_if_absent(session, input, context).await.unwrap(), stackpress_framework_sql::InsertOutcome::AlreadyExists));
        let stored = model::$models::get_key(session, &key).await.unwrap().unwrap();
        assert_eq!(model::$models::get_maintenance(session, &key, stackpress_framework_sql::stores::Visibility::All).await.unwrap(), Some(stored));
        // Each scalar key family compiles to a conjunction, never to interpolated SQL text.
        let filters = identity.as_object().unwrap().iter().map(|(field,value)| serde_json::json!({"field":field,"predicate":{"op":"eq","value":value}})).collect::<Vec<_>>();
        let query: model::Query = serde_json::from_value(serde_json::json!({"filters":filters})).unwrap();
        let plan = model::$store::search_plan(&query, session.dialect()).unwrap().render(session.dialect()).unwrap();
        assert!(plan.binds().len() >= identity.as_object().unwrap().len());
        assert!(model::$models::find(session, query.clone()).await.unwrap().is_some());
        assert_eq!(model::$models::find_all(session, query.clone()).await.unwrap().len(),1);
        assert_eq!(model::$models::count(session, query.clone()).await.unwrap(),1);
        let engine=match &*session {
            stackpress_framework_sql::DataSession::Pooled(engine)=>*engine,
            stackpress_framework_sql::DataSession::Transaction(_)=>panic!("fixture uses pooled session"),
        };
        let mut transaction=engine.begin().await.unwrap();
        // A caller-owned snapshot exercises the public allowlist on the same complete identity.
        let public_query = serde_json::from_value(serde_json::json!({"filters":filters,"total":true})).unwrap();
        let page = model::$models::search_public(&mut stackpress_framework_sql::DataSession::transaction(&mut transaction), public_query).await.unwrap();
        assert_eq!(page.total,Some(1));
        assert_eq!(page.rows.len(),1);
        let selected = model::$models::detail(&mut stackpress_framework_sql::DataSession::transaction(&mut transaction),&key).await.unwrap().unwrap();
        assert!(selected.value().is_object());
        transaction.rollback().await.unwrap();
        let mut counted=query.clone(); counted.total=true;
        let counted=model::$models::search_atomic(engine,counted).await.unwrap();
        assert_eq!(counted.total,Some(1));
        // Mixed native batch modes prepare fully, keep one transaction and return ordered acknowledgements.
        let input:$create=serde_json::from_value(raw.clone()).unwrap();
        let first_patch:$patch_type=serde_json::from_value($patch).unwrap();
        let second_patch:$patch_type=serde_json::from_value($patch).unwrap();
        let batch=model::$models::batch_atomic(engine,vec![
            model::BatchInput::Update{key:key.clone(),patch:first_patch},
            model::BatchInput::UpsertOn{target:model::ConflictKey::Primary(key.clone()),input,patch:second_patch},
        ],context).await.unwrap();
        let output=serde_json::to_value(batch).unwrap();
        assert_eq!(output[0]["mode"],"updated");
        assert_eq!(output[1]["mode"],"upserted");
        for (field,value) in identity.as_object().unwrap() { assert_eq!(output[0]["row"][field],*value); assert_eq!(output[1]["row"][field],*value); }
        // Native explicit primary targets use the same atomic compiler as declared unique targets.
        let patch: $patch_type = serde_json::from_value($patch).unwrap();
        let input: $create = serde_json::from_value(raw.clone()).unwrap();
        let updated = model::$models::upsert_on(session, model::ConflictKey::Primary(key.clone()), input, patch, context).await.unwrap();
        let returned = serde_json::to_value(updated).unwrap();
        for (field,value) in identity.as_object().unwrap() { assert_eq!(returned[field],*value); }
    }};
}
pub(crate) use native_calls;
