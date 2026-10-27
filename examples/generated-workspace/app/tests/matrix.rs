//! Actual generated storage over the complete type matrix and every scalar identity family.
#![cfg(feature = "matrix")]
use generated_workspace_app::generated::{
    contracts::{
        compound::CreateCompound,
        link::CreateLink,
        matrix::CreateMatrix,
        public::{CompoundKey, LinkKey, MatrixKey, PatchCompound, PatchMatrix, QueryKey},
        query::CreateQuery,
    },
    stores::{
        compound::{Compounds, Query as CompoundQuery},
        link::{Links, Query as LinkQuery},
        matrix::{Filter as MatrixFilter, MatrixStore, Matrixs, Query as MatrixQuery, SortField},
        query::Querys,
    },
};
use serde_json::{Value, json};
use stackpress_framework_schema::{
    DatetimeValue, FiniteNumber, NullOrder, Predicate, SchemaResult, Sort, SortDirection,
    ValueContext, ValueProviders,
};
use stackpress_framework_sql::{
    DataErrorKind, DataSession,
    stores::{Loaded, Visibility},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
#[path = "support/native_calls.rs"]
mod native_calls;
mod support;
#[path = "../../support/watchdog.rs"]
mod watchdog;
/// Deterministic app-owned host capability; counters prove once-per-instance preparation.
#[derive(Default)]
struct Values {
    generated: AtomicUsize,
    validated: AtomicUsize,
}
impl ValueProviders for Values {
    fn generate(&self, name: &str, _: &[Value]) -> SchemaResult<Value> {
        Ok(match name {
            "nanoid" => json!(format!(
                "token-{}",
                self.generated.fetch_add(1, Ordering::SeqCst)
            )),
            "default:Matrix.secret" => json!("PRIVATE_DATABASE_PASSWORD_SENTINEL"),
            _ => panic!("unexpected provider"),
        })
    }
    fn validate(&self, name: &str, value: &Value, _: &[Value]) -> SchemaResult<bool> {
        assert_eq!(name, "app.approved");
        self.validated.fetch_add(1, Ordering::SeqCst);
        Ok(value == "PH")
    }
}
/// Bound all generated type/clock/compound/relation execution with a parent process watchdog.
#[test]
fn full_storage_matrix() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) =
        watchdog::run("full_storage_matrix", "matrix", Duration::from_secs(20));
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// One complete key uses every supported scalar family, including a true compound identity.
fn compound_key(enabled: bool) -> Value {
    json!({"region":"APAC","code":i64::MAX,"enabled":enabled,"day":"2024-02-29","clock":"12:00:00.1","instant":"2024-03-01T00:00:00Z","ratio":0.25,"choice":0,"truth":false})
}
/// Minimal type defaults and rich supplied values round-trip through actual generated writes.
async fn prove() {
    let fixture = support::fixture().await;
    let mut db = DataSession::pooled(&fixture.engine);
    let host = Arc::new(Values::default());
    let now = DatetimeValue::parse("2024-03-01T00:00:00Z").unwrap();
    let later = DatetimeValue::parse("2024-03-01T00:00:00.100000000Z").unwrap();
    let context = ValueContext {
        now: Some(&now),
        providers: Some(host.as_ref()),
    };
    let input:CreateMatrix=serde_json::from_value(json!({"id":1,"float":0.5,"tags":["a","b"],"numbers":[0.5],"dates":["2024-02-29"],"times":["12:00:00.1"],"instants":["2024-03-01T00:00:00Z"],"objects":[{"x":null}],"address":{"city":"Café"},"siblings":[{"city":"Manila"}]})).unwrap();
    let row = Matrixs::create_with(&mut db, input, &context)
        .await
        .unwrap();
    assert_eq!(
        host.generated.load(Ordering::SeqCst),
        4,
        "one nonce per three embedded instances plus token"
    );
    assert_eq!(
        host.validated.load(Ordering::SeqCst),
        3,
        "host rule once per country instance"
    );
    assert_eq!(row.updated, now);
    assert_eq!(row.created, now);
    assert_eq!(
        row.secret.as_deref(),
        Some("PRIVATE_DATABASE_PASSWORD_SENTINEL")
    );
    assert_eq!(row, Matrixs::get(&mut db, 1).await.unwrap().unwrap());
    let public = row.project().unwrap();
    let public = serde_json::to_value(public).unwrap();
    assert!(public.get("secret").is_none());
    assert!(public["favorite"].get("nonce").is_none());
    assert!(public["favorite"].get("zip").is_none());
    assert_eq!(public["favorite"]["country"], "PH");
    let input: CreateMatrix = serde_json::from_value(
        json!({"id":2,"text":"Second","stamp":"2024-03-01T00:00:00.1Z","secret":null}),
    )
    .unwrap();
    Matrixs::create_with(&mut db, input, &context)
        .await
        .unwrap();
    let page = Matrixs::search(
        &mut db,
        MatrixQuery {
            sort: vec![Sort {
                field: SortField::Stamp,
                direction: SortDirection::Asc,
                nulls: NullOrder::Last,
            }],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        page.rows.iter().map(|row| row.row.id).collect::<Vec<_>>(),
        [1, 2]
    );
    let page = Matrixs::search(
        &mut db,
        MatrixQuery {
            filters: vec![MatrixFilter::Stamp(Predicate::Ge(
                DatetimeValue::parse("2024-03-01T00:00:00.05Z").unwrap(),
            ))],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(page.rows[0].row.id, 2);
    assert_eq!(page.rows.len(), 1);
    let physical = fixture
        .engine
        .fetch(
            &stackpress_sql::Select::new(["stamp"])
                .from("matrix")
                .order_by("id", stackpress_sql::Order::Asc),
        )
        .await
        .unwrap();
    assert_eq!(
        physical[0].try_get::<String>("stamp").unwrap(),
        "2024-03-01T00:00:00Z"
    );
    assert_eq!(
        physical[1].try_get::<String>("stamp").unwrap(),
        "2024-03-01T00:00:00.1Z"
    );
    let filters = [
        MatrixFilter::Tags(Predicate::Has("a".into())),
        MatrixFilter::Numbers(Predicate::Has(FiniteNumber::new(0.5).unwrap())),
        MatrixFilter::Instants(Predicate::Has(now.clone())),
        MatrixFilter::Secret(Predicate::IsNull(false)),
    ];
    for filter in filters {
        assert_eq!(
            Matrixs::search(
                &mut db,
                MatrixQuery {
                    filters: vec![filter],
                    ..Default::default()
                }
            )
            .await
            .unwrap()
            .rows[0]
                .row
                .id,
            1
        );
    }
    // Every scalar family compiles its typed predicate into actual SQLite semantics.
    for (field, op, value) in [
        ("text", "eq", json!("Café")),
        ("text", "ne", json!("other")),
        ("count", "ge", json!(0)),
        ("count", "le", json!(0)),
        ("real", "eq", json!(0.25)),
        ("flag", "eq", json!(false)),
        ("day", "ge", json!("2024-02-29")),
        ("time", "le", json!("12:00:00.12")),
        ("stamp", "eq", json!("2024-03-01T01:00:00+01:00")),
        ("choice", "eq", json!(0)),
        ("truth", "ne", json!(true)),
        ("dates", "has", json!("2024-02-29")),
        ("times", "has", json!("12:00:00.1")),
        ("numbers", "hasnt", json!(2)),
    ] {
        let query: MatrixQuery = serde_json::from_value(json!({"filters":[{"field":"id","predicate":{"op":"eq","value":1}},{"field":field,"predicate":{"op":op,"value":value}}]})).unwrap();
        let page = Matrixs::search(&mut db, query).await.unwrap();
        assert_eq!(page.rows.len(), 1, "{field}/{op}");
        assert_eq!(page.rows[0].row.id, 1);
    }
    let page = Matrixs::search_atomic(
        &fixture.engine,
        MatrixQuery {
            sort: vec![Sort {
                field: SortField::Stamp,
                direction: SortDirection::Desc,
                nulls: NullOrder::Last,
            }],
            total: true,
            take: 1,
            skip: 1,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        (page.total, page.rows[0].row.id, page.skip, page.take),
        (Some(2), 1, 1, 1)
    );
    assert_eq!(
        Matrixs::search(
            &mut db,
            MatrixQuery {
                filters: vec![MatrixFilter::Secret(Predicate::IsNull(true))],
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .rows[0]
            .row
            .id,
        2
    );
    for (nulls, expected) in [(NullOrder::First, 2), (NullOrder::Last, 1)] {
        let page = Matrixs::search(
            &mut db,
            MatrixQuery {
                sort: vec![Sort {
                    field: SortField::Float,
                    direction: SortDirection::Asc,
                    nulls,
                }],
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(page.rows[0].row.id, expected);
    }
    let before = host.generated.load(Ordering::SeqCst);
    let patch: PatchMatrix =
        serde_json::from_value(json!({"text":"Updated","float":null})).unwrap();
    let updated = Matrixs::update(
        &mut db,
        1,
        patch,
        &ValueContext {
            now: Some(&later),
            providers: Some(host.as_ref()),
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(updated.updated, later);
    assert_eq!(updated.created, now);
    assert!(updated.float.is_none());
    assert_eq!(
        host.generated.load(Ordering::SeqCst),
        before,
        "updates cannot regenerate identity/default fields"
    );
    assert!(
        Matrixs::update(
            &mut db,
            1,
            serde_json::from_value(json!({"text":"Next"})).unwrap(),
            &ValueContext::default()
        )
        .await
        .is_err()
    );
    assert_eq!(
        Matrixs::get(&mut db, 1).await.unwrap().unwrap().text,
        "Updated"
    );
    // Full i64 identity remains valid for trusted native calls; browser projection deliberately fails.
    let input: CreateMatrix =
        serde_json::from_value(json!({"id":i64::MAX,"text":"Large"})).unwrap();
    Matrixs::create_with(&mut db, input, &context)
        .await
        .unwrap();
    assert_eq!(
        Matrixs::get(&mut db, i64::MAX).await.unwrap().unwrap().id,
        i64::MAX
    );
    assert_eq!(
        Matrixs::search(
            &mut db,
            MatrixQuery {
                filters: vec![MatrixFilter::Id(Predicate::Eq(i64::MAX))],
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .rows
        .len(),
        1
    );
    // Required and optional eager relations retain loaded None versus a complete reverse collection.
    for (id, owner) in [("linked", Some(1)), ("unlinked", None)] {
        let input: CreateLink = serde_json::from_value(json!({"id":id,"ownerId":owner})).unwrap();
        Links::create(&mut db, input).await.unwrap();
    }
    let page = Links::search_atomic(
        &fixture.engine,
        LinkQuery {
            includes: vec!["owner.links.owner".into()],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        &page
            .rows
            .iter()
            .find(|row| row.row.id == "unlinked")
            .unwrap()
            .owner,
        Loaded::Loaded(None)
    ));
    let projected = page
        .rows
        .into_iter()
        .map(|row| row.project().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(projected.len(), 2);
    let page = Matrixs::search_atomic(
        &fixture.engine,
        MatrixQuery {
            filters: vec![MatrixFilter::Id(Predicate::Eq(1))],
            includes: vec!["links.owner.links".into()],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let public = page.rows.into_iter().next().unwrap().project().unwrap();
    assert!(!serde_json::to_string(&public).unwrap().contains("PRIVATE_"));
    // Every compound member scopes get/update/upsert/delete; no prefix-only mutation can escape.
    for enabled in [false, true] {
        let mut value = compound_key(enabled);
        value["value"] = json!(if enabled { "True" } else { "False" });
        let input: CreateCompound = serde_json::from_value(value).unwrap();
        Compounds::create(&mut db, input).await.unwrap();
    }
    let key: CompoundKey = serde_json::from_value(compound_key(false)).unwrap();
    let changed = Compounds::update_key(
        &mut db,
        &key,
        serde_json::from_value::<PatchCompound>(json!({"value":"Changed"})).unwrap(),
        &context,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(changed.value, "Changed");
    let other: CompoundKey = serde_json::from_value(compound_key(true)).unwrap();
    assert_eq!(
        Compounds::get_key(&mut db, &other)
            .await
            .unwrap()
            .unwrap()
            .value,
        "True"
    );
    let mut input = compound_key(false);
    input["value"] = json!("Ignored");
    Compounds::upsert_key(
        &mut db,
        &key,
        serde_json::from_value(input).unwrap(),
        serde_json::from_value(json!({"value":"Upserted"})).unwrap(),
        &context,
    )
    .await
    .unwrap();
    assert_eq!(
        Compounds::count(&mut db, CompoundQuery::default())
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        Compounds::delete(&mut db, &key)
            .await
            .unwrap()
            .unwrap()
            .value,
        "Upserted"
    );
    assert!(Compounds::get_key(&mut db, &key).await.unwrap().is_none());
    assert!(
        Compounds::get_maintenance(&mut db, &other, Visibility::All)
            .await
            .unwrap()
            .is_some()
    );
    // Model/field names overlapping native helper types and Rust keywords remain valid generated Rust.
    let input: CreateQuery = serde_json::from_value(
        json!({"id":"q","payload":{"row":"literal row","state":"literal state","value":7}}),
    )
    .unwrap();
    Querys::create(&mut db, input).await.unwrap();
    assert_eq!(
        Querys::get(&mut db, "q").await.unwrap().unwrap().self_,
        "safe"
    );
    let selected = Querys::detail(&mut db, &QueryKey { id: "q".into() })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        selected.value(),
        &json!({"id":"q","payload":{"row":"literal row","state":"literal state"}})
    );
    let mut tx = fixture.engine.begin().await.unwrap();
    let selected = Links::detail(
        &mut DataSession::transaction(&mut tx),
        &LinkKey {
            id: "linked".into(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        selected.value(),
        &json!({"id":"linked","owner":{"text":"Updated","favorite":{"city":"Default"}}})
    );
    let absent = Links::detail(
        &mut DataSession::transaction(&mut tx),
        &LinkKey {
            id: "unlinked".into(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(absent.value(), &json!({"id":"unlinked","owner":null}));
    tx.rollback().await.unwrap();
    let view = Querys::search(&mut db, Default::default())
        .await
        .unwrap()
        .rows
        .remove(0)
        .project()
        .unwrap();
    let full =
        stackpress_framework_sql::projection::select(&view, &["payload", "payload.row"], false)
            .unwrap();
    assert_eq!(
        full["payload"]["value"], 7,
        "a selected whole fieldset cannot be narrowed by a redundant child path"
    );
    Querys::delete(&mut db, &QueryKey { id: "q".into() })
        .await
        .unwrap();
    // Unique-target UPSERT returns the authoritative original primary identity, never the candidate one.
    let updated = Matrixs::upsert_on(
        &mut db,
        generated_workspace_app::generated::stores::matrix::ConflictKey::UniqueText(
            "Updated".into(),
        ),
        serde_json::from_value(json!({"id":999,"text":"Updated"})).unwrap(),
        serde_json::from_value(json!({"text":"Unique updated"})).unwrap(),
        &context,
    )
    .await
    .unwrap();
    assert_eq!(updated.id, 1);
    assert!(Matrixs::get(&mut db, 999).await.unwrap().is_none());
    let results = Matrixs::batch_atomic(
        &fixture.engine,
        vec![
            generated_workspace_app::generated::stores::matrix::BatchInput::UpsertOn {
                target: generated_workspace_app::generated::stores::matrix::ConflictKey::UniqueText(
                    "Unique updated".into(),
                ),
                input: serde_json::from_value(json!({"id":1000,"text":"Unique updated"})).unwrap(),
                patch: serde_json::from_value(json!({"count":2})).unwrap(),
            },
        ],
        &context,
    )
    .await
    .unwrap();
    assert!(
        matches!(&results[0],generated_workspace_app::generated::stores::matrix::BatchOutcome::Upserted(row) if row.id==1&&row.count==2)
    );
    // Nullable unique targets remain nonnull keys; the conflict branch may explicitly clear the field.
    let linked = Links::upsert_on(
        &mut db,
        generated_workspace_app::generated::stores::link::ConflictKey::UniqueOwnerId(1),
        serde_json::from_value(json!({"id":"candidate","ownerId":1})).unwrap(),
        serde_json::from_value(json!({"ownerId":null})).unwrap(),
        &context,
    )
    .await
    .unwrap();
    assert_eq!(linked.id, "linked");
    assert!(linked.owner_id.is_none());
    Links::update(
        &mut db,
        "linked",
        serde_json::from_value(json!({"ownerId":1})).unwrap(),
        &context,
    )
    .await
    .unwrap();
    assert!(
        Links::upsert_on(
            &mut db,
            generated_workspace_app::generated::stores::link::ConflictKey::UniqueOwnerId(1),
            serde_json::from_value(json!({"id":"no-owner"})).unwrap(),
            serde_json::from_value(json!({"ownerId":null})).unwrap(),
            &context
        )
        .await
        .is_err()
    );
    // Query comparison preserves all nine fractional digits without changing canonical stored values.
    for (index, nanos) in [
        1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000001, 900000000,
    ]
    .into_iter()
    .enumerate()
    {
        let stamp = format!("2024-03-01T00:00:00.{nanos:09}Z");
        let input: CreateMatrix = serde_json::from_value(
            json!({"id":100+index,"text":format!("Fraction {index}"),"stamp":stamp}),
        )
        .unwrap();
        Matrixs::create_with(&mut db, input, &context)
            .await
            .unwrap();
    }
    let rows = Matrixs::search(
        &mut db,
        MatrixQuery {
            take: 100,
            sort: vec![Sort {
                field: SortField::Stamp,
                direction: SortDirection::Asc,
                nulls: NullOrder::Last,
            }],
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .rows;
    assert!(
        rows.windows(2)
            .all(|rows| rows[0].row.stamp.unix_nanos() <= rows[1].row.stamp.unix_nanos())
    );
    let threshold = DatetimeValue::parse("2024-03-01T00:00:00.100000001Z").unwrap();
    let rows = Matrixs::search(
        &mut db,
        MatrixQuery {
            take: 100,
            filters: vec![MatrixFilter::Stamp(Predicate::Ge(threshold.clone()))],
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .rows;
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .all(|row| row.row.stamp.unix_nanos() >= threshold.unix_nanos())
    );
    // FK RESTRICT and strict codecs remain authoritative even after typed plans are submitted.
    assert_eq!(
        Matrixs::delete(&mut db, &MatrixKey { id: 1 })
            .await
            .unwrap_err()
            .kind(),
        DataErrorKind::UnknownOutcome
    );
    Links::delete(
        &mut db,
        &LinkKey {
            id: "linked".into(),
        },
    )
    .await
    .unwrap();
    assert!(
        Matrixs::delete(&mut db, &MatrixKey { id: 1 })
            .await
            .unwrap()
            .is_some()
    );
    let table = MatrixStore::table().unwrap();
    assert!(table.fields.iter().any(|field| field.name == "stamp"));
    // Detached/native convenience calls cover every generated identity and query family too.
    native_calls::native_calls!(
        matrix,
        Matrixs,
        MatrixStore,
        CreateMatrix,
        MatrixKey,
        PatchMatrix,
        json!({"id":10000,"text":"Native breadth"}),
        json!({"id":10000}),
        json!({"text":"Native changed"}),
        &mut db,
        &context
    );
    native_calls::native_calls!(
        link,
        Links,
        LinkStore,
        CreateLink,
        LinkKey,
        generated_workspace_app::generated::contracts::public::PatchLink,
        json!({"id":"native-breadth"}),
        json!({"id":"native-breadth"}),
        json!({"ownerId":null}),
        &mut db,
        &context
    );
    native_calls::native_calls!(
        query,
        Querys,
        QueryStore,
        CreateQuery,
        QueryKey,
        generated_workspace_app::generated::contracts::public::PatchQuery,
        json!({"id":"native-breadth"}),
        json!({"id":"native-breadth"}),
        json!({"self":"Changed"}),
        &mut db,
        &context
    );
    let key = json!({"region":"BREADTH","code":7,"enabled":true,"day":"2024-02-29","clock":"12:00:00.1","instant":"2024-03-01T00:00:00Z","ratio":0.25,"choice":0,"truth":false});
    let mut input = key.clone();
    input["value"] = json!("Before");
    native_calls::native_calls!(
        compound,
        Compounds,
        CompoundStore,
        CreateCompound,
        CompoundKey,
        PatchCompound,
        input,
        key,
        json!({"value":"Changed"}),
        &mut db,
        &context
    );
    let ui = generated_workspace_app::generated::metadata::ui::schema().unwrap();
    assert!(
        ui.models
            .iter()
            .any(|model| model.id == "Compound" && model.key.len() == 9)
    );
    fixture.engine.close().await.unwrap();
}

/// Policy assembles hidden values but cannot replace a public input or supply a generated member.
struct MatrixPolicy {
    host: Arc<Values>,
    now: DatetimeValue,
}
impl stackpress_framework_sql::operations::OperationPolicy for MatrixPolicy {
    fn authorize(
        &self,
        _: &'static str,
        _: stackpress_framework_sql::operations::Operation,
        _: &stackpress_lib::Request,
    ) -> stackpress_lib::Result<()> {
        Ok(())
    }
    fn values(
        &self,
        model: &'static str,
        operation: stackpress_framework_sql::operations::Operation,
        _: &stackpress_lib::Request,
    ) -> stackpress_framework_sql::DataResult<stackpress_framework_sql::operations::HostInputs>
    {
        if model != "Matrix" {
            return Ok(Default::default());
        }
        Ok(stackpress_framework_sql::operations::HostInputs {
            now: Some(self.now.clone()),
            providers: Some(self.host.clone()),
            create: json!({"secret":"PRIVATE_HOST_DEFAULT","favorite":{"zip":"PRIVATE_ZIP"}}),
            patch: if matches!(
                operation,
                stackpress_framework_sql::operations::Operation::Update
                    | stackpress_framework_sql::operations::Operation::Upsert
                    | stackpress_framework_sql::operations::Operation::Batch
            ) {
                json!({"secret":"PRIVATE_PATCH_VALUE","favorite":{"zip":"PRIVATE_PATCH_ZIP"}})
            } else {
                json!({})
            },
        })
    }
}
/// Parent-supervised nested public/private assembly and recursive disclosure proof.
#[test]
fn matrix_operation_disclosure() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove_public());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "matrix_operation_disclosure",
        "matrix-public",
        Duration::from_secs(20),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// Public input/defaults and private host members compose once, and every included node projects.
async fn prove_public() {
    use generated_workspace_app::generated::stores::operations::{
        compound as compound_ops, link as link_ops, matrix as ops, query as query_ops,
    };
    use stackpress_framework_sql::operations::{Operation, Policy, Registration};
    use stackpress_lib::{Flow, queue::task};
    use stackpress_server::{ServerBuilder, StartupContext};
    let fixture = support::fixture().await;
    let engine = Arc::new(fixture.engine);
    let provider = Arc::new(Values::default());
    let host = provider.clone();
    let db = engine.clone();
    let mut builder = ServerBuilder::new();
    builder
        .on_config(task(move |start: &mut StartupContext| {
            let host = host.clone();
            let db = db.clone();
            Box::pin(async move {
                start.register(db)?;
                start.register(Arc::new(Policy(Arc::new(MatrixPolicy {
                    host,
                    now: DatetimeValue::parse("2024-03-01T00:00:00Z").unwrap(),
                }))))?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    builder
        .on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                let mut registry = Registration::default();
                ops::register(
                    start,
                    &mut registry,
                    "matrix",
                    &[
                        Operation::Create,
                        Operation::Get,
                        Operation::Detail,
                        Operation::Search,
                        Operation::Update,
                        Operation::Upsert,
                        Operation::Batch,
                        Operation::Delete,
                    ],
                )?;
                let selected = [
                    Operation::Create,
                    Operation::Get,
                    Operation::Detail,
                    Operation::Search,
                    Operation::Update,
                    Operation::Upsert,
                    Operation::Batch,
                    Operation::Delete,
                ];
                link_ops::register(start, &mut registry, "links", &selected)?;
                compound_ops::register(start, &mut registry, "compound", &selected)?;
                query_ops::register(start, &mut registry, "query", &selected)?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    let app = builder.build().await.unwrap();
    let reply = app
        .call("matrix.create", json!({"input":{"id":11}}))
        .await
        .unwrap();
    let value = reply.response.try_to_json().unwrap();
    assert_eq!(value["code"], 200, "{value}");
    assert_eq!(value["results"]["favorite"]["city"], "Default");
    assert_eq!(provider.generated.load(Ordering::SeqCst), 2);
    assert_eq!(provider.validated.load(Ordering::SeqCst), 1);
    assert!(!value.to_string().contains("PRIVATE_"));
    let raw = Matrixs::get(&mut DataSession::pooled(&engine), 11)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(raw.secret.as_deref(), Some("PRIVATE_HOST_DEFAULT"));
    assert_eq!(raw.favorite.zip.as_deref(), Some("PRIVATE_ZIP"));
    let reply = app
        .call(
            "matrix.update",
            json!({"key":{"id":11},"patch":{"favorite":{"city":"Updated"}}}),
        )
        .await
        .unwrap();
    let value = reply.response.try_to_json().unwrap();
    assert_eq!(value["code"], 200, "{value}");
    assert_eq!(value["results"]["favorite"]["city"], "Updated");
    assert_eq!(provider.generated.load(Ordering::SeqCst), 3);
    assert_eq!(
        provider.validated.load(Ordering::SeqCst),
        2,
        "nested replacement validation must run once"
    );
    assert!(!value.to_string().contains("PRIVATE_"));
    for (event, input) in [
        (
            "matrix.create",
            json!({"input":{"id":12,"secret":"PRIVATE_ATTACK"}}),
        ),
        (
            "matrix.create",
            json!({"input":{"id":12,"token":"PRIVATE_ATTACK"}}),
        ),
        ("matrix.update", json!({"key":{"id":11},"patch":{}})),
        ("matrix.update", json!({"key":{"id":11},"patch":{"id":12}})),
        (
            "matrix.search",
            json!({"filters":[{"field":"secret","predicate":{"op":"eq","value":"PRIVATE_ATTACK"}}]}),
        ),
    ] {
        fixture.calls.lock().unwrap().clear();
        let reply = app.call(event, input).await.unwrap();
        let value = reply.response.try_to_json().unwrap();
        assert_eq!(value["code"], 400, "{value}");
        assert!(!value.to_string().contains("PRIVATE_"));
        assert!(fixture.calls.lock().unwrap().is_empty());
    }
    let link: CreateLink = serde_json::from_value(json!({"id":"link","ownerId":11})).unwrap();
    Links::create(&mut DataSession::pooled(&engine), link)
        .await
        .unwrap();
    let reply = app
        .call(
            "links.search",
            json!({"includes":["owner.links.owner"],"total":true}),
        )
        .await
        .unwrap();
    let value = reply.response.try_to_json().unwrap();
    assert_eq!(value["code"], 200);
    assert!(!value.to_string().contains("PRIVATE_"));
    assert_eq!(value["results"]["rows"][0]["owner"]["state"], "loaded");
    for event in ["matrix.get", "matrix.detail"] {
        let reply = app.call(event, json!({"key":{"id":11}})).await.unwrap();
        let value = reply.response.try_to_json().unwrap();
        assert_eq!(value["code"], 200);
        assert!(!value.to_string().contains("PRIVATE_"));
    }
    // Reverse the repeated relation path so both depth parities project every approved node.
    let reply = app
        .call(
            "matrix.search",
            json!({"includes":["links.owner.links"],"total":true}),
        )
        .await
        .unwrap();
    let value = reply.response.try_to_json().unwrap();
    assert_eq!(value["code"], 200);
    assert!(!value.to_string().contains("PRIVATE_"));
    operation_lifecycle(
        &app,
        "matrix",
        json!({"id":20}),
        json!({"id":20,"text":"Lifecycle"}),
        json!({"text":"Changed","favorite":{"city":"Updated","country":"PH"}}),
    )
    .await;
    operation_lifecycle(
        &app,
        "links",
        json!({"id":"lifecycle"}),
        json!({"id":"lifecycle"}),
        json!({"ownerId":null}),
    )
    .await;
    let key = json!({"region":"CP","code":7,"enabled":true,"day":"2024-02-29","clock":"12:00:00.1","instant":"2024-03-01T00:00:00Z","ratio":0.25,"choice":0,"truth":false});
    let mut input = key.clone();
    input["value"] = json!("Before");
    operation_lifecycle(&app, "compound", key, input, json!({"value":"Changed"})).await;
    operation_lifecycle(
        &app,
        "query",
        json!({"id":"lifecycle"}),
        json!({"id":"lifecycle"}),
        json!({"self":"Changed","payload":{"row":"one","state":"two","value":3}}),
    )
    .await;
    engine.close().await.unwrap();
}

/// Actual CRUD through each rich generated handler retains identity, defaults, order and public shape.
async fn operation_lifecycle(
    app: &stackpress_server::Server,
    prefix: &str,
    key: Value,
    input: Value,
    patch: Value,
) {
    async fn invoke(
        app: &stackpress_server::Server,
        prefix: &str,
        operation: &str,
        input: Value,
    ) -> Value {
        let reply = app
            .call(&format!("{prefix}.{operation}"), input)
            .await
            .unwrap();
        let value = reply.response.try_to_json().unwrap();
        assert_eq!(value["code"], 200, "{prefix}.{operation}: {value}");
        assert!(!value.to_string().contains("PRIVATE_"));
        value["results"].clone()
    }
    let created = invoke(app, prefix, "create", json!({"input":input})).await;
    let read = invoke(app, prefix, "get", json!({"key":key})).await;
    assert_eq!(created, read);
    let selected = invoke(app, prefix, "detail", json!({"key":key})).await;
    assert!(selected.is_object());
    let page = invoke(app, prefix, "search", json!({"total":true})).await;
    assert!(page["total"].as_u64().unwrap() > 0);
    let updated = invoke(app, prefix, "update", json!({"key":key,"patch":patch})).await;
    for (field, value) in patch.as_object().unwrap() {
        assert_eq!(updated[field], *value, "{prefix}/{field}");
    }
    let upserted = invoke(
        app,
        prefix,
        "upsert",
        json!({"key":key,"input":input,"patch":patch}),
    )
    .await;
    assert_eq!(updated, upserted);
    // A new identity plus repeated mutations proves all batch modes are ordered in one transaction.
    let mut extra_key = key.clone();
    let field = if extra_key.get("id").is_some() {
        "id"
    } else {
        "region"
    };
    extra_key[field] = match &extra_key[field] {
        Value::String(value) => json!(format!("{value}-batch")),
        Value::Number(value) => json!(value.as_i64().unwrap() + 1000),
        _ => panic!("fixture identity"),
    };
    let mut extra = input.clone();
    extra[field] = extra_key[field].clone();
    if prefix == "matrix" {
        extra["text"] = json!("Lifecycle batch");
    }
    let batch=invoke(app,prefix,"batch",json!({"items":[{"mode":"create","input":extra},{"mode":"update","key":key,"patch":patch},{"mode":"upsert","key":key,"input":input,"patch":patch}]})).await;
    assert_eq!(batch.as_array().unwrap().len(), 3);
    assert_eq!(batch[0][field], extra_key[field]);
    assert_eq!(batch[1][field], key[field]);
    assert_eq!(batch[2][field], key[field]);
    for key in [extra_key, key] {
        let deleted = invoke(app, prefix, "delete", json!({"key":key})).await;
        for (field, value) in key.as_object().unwrap() {
            assert_eq!(deleted[field], *value);
        }
        let reply = app
            .call(&format!("{prefix}.get"), json!({"key":key}))
            .await
            .unwrap();
        assert_eq!(reply.response.try_to_json().unwrap()["code"], 404);
    }
}
