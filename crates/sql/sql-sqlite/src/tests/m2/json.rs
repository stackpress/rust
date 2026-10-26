//! JSON paths and scalar predicates are proved through native SQLite's JSON functions.

use std::error::Error;

use stackpress_sql::{
    Engine, ErrorKind, Expr, Insert, JsonPath, JsonScalar, JsonSegment, Order, Select, Selector,
    SqlValue,
};

use super::{SqliteDatabase, TempDir, options, sql};

/// Preserve the distinction between an object key and a typed array index.
fn keys(names: &[&str]) -> JsonPath {
    JsonPath::new(names.iter().map(|key| JsonSegment::Key((*key).into())))
}

/// Execute a predicate and return deterministic IDs, rather than snapshotting rendered text.
async fn matching(engine: &Engine, predicate: Expr) -> Vec<i64> {
    engine
        .fetch(
            &Select::new(["id"])
                .from("documents")
                .and_where(predicate)
                .order_by("id", Order::Asc),
        )
        .await
        .unwrap()
        .iter()
        .map(|row| row.try_get("id").unwrap())
        .collect()
}

/// Object/array/nested/twice-nested inventory rows include direct membership nonmatches.
pub(super) async fn paths() {
    let root = TempDir::new().unwrap();
    let engine = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    engine
        .execute(&sql(
            "CREATE TABLE documents(id INTEGER PRIMARY KEY, document TEXT)",
        ))
        .await
        .unwrap();
    let hostile = "O'Reilly'; DROP TABLE documents; -- λ";
    let document = serde_json::json!({
        "object": {"name": hostile},
        "array": ["direct", ["descendant"], {"tag": "hidden"}],
        "nested": {"items": [{"tags": ["nested"]}]},
        "twice": {"items": [{"inner": {"tags": ["twice"]}}]},
        "01": {"a.b\"\\x": "special"},
        "path']; SELECT 1; --": "literal key",
        "control\nkey": "newline"
    })
    .to_string();
    engine
        .execute(
            &Insert::new("documents")
                .columns(["id", "document"])
                .values([SqlValue::Integer(1), SqlValue::Text(document.clone())]),
        )
        .await
        .unwrap();
    for predicate in [
        Expr::json_eq(
            "document",
            keys(&["object", "name"]),
            JsonScalar::String(hostile.into()),
        ),
        Expr::json_contains(
            "document",
            keys(&["array"]),
            JsonScalar::String("direct".into()),
        ),
        Expr::json_contains(
            "document",
            JsonPath::new([
                JsonSegment::Key("nested".into()),
                JsonSegment::Key("items".into()),
                JsonSegment::Index(0),
                JsonSegment::Key("tags".into()),
            ]),
            JsonScalar::String("nested".into()),
        ),
        Expr::json_contains(
            "document",
            JsonPath::new([
                JsonSegment::Key("twice".into()),
                JsonSegment::Key("items".into()),
                JsonSegment::Index(0),
                JsonSegment::Key("inner".into()),
                JsonSegment::Key("tags".into()),
            ]),
            JsonScalar::String("twice".into()),
        ),
        Expr::json_eq(
            "document",
            keys(&["01", "a.b\"\\x"]),
            JsonScalar::String("special".into()),
        ),
        Expr::json_eq(
            "document",
            keys(&["path']; SELECT 1; --"]),
            JsonScalar::String("literal key".into()),
        ),
        Expr::json_eq(
            "document",
            keys(&["control\nkey"]),
            JsonScalar::String("newline".into()),
        ),
    ] {
        assert_eq!(matching(&engine, predicate).await, [1]);
    }
    for predicate in [
        Expr::json_contains(
            "document",
            keys(&["array"]),
            JsonScalar::String("descendant".into()),
        ),
        Expr::json_contains(
            "document",
            keys(&["array"]),
            JsonScalar::String("hidden".into()),
        ),
        Expr::json_contains(
            "document",
            keys(&["object"]),
            JsonScalar::String(hostile.into()),
        ),
        Expr::json_eq(
            "document",
            JsonPath::new([JsonSegment::Index(1)]),
            JsonScalar::String("special".into()),
        ),
    ] {
        assert!(matching(&engine, predicate).await.is_empty());
    }
    let rows = engine
        .fetch(
            &Select::new([
                Selector::expression(Expr::json("document", keys(&["object", "name"])))
                    .alias("extracted"),
                Selector::expression(Expr::json("document", keys(&["missing"]))).alias("absent"),
            ])
            .from("documents"),
        )
        .await
        .unwrap();
    assert_eq!(rows[0].try_get::<String>("extracted").unwrap(), hostile);
    assert_eq!(rows[0].try_get::<Option<String>>("absent").unwrap(), None);
    assert_eq!(
        engine
            .fetch(&Select::new(["document"]).from("documents"))
            .await
            .unwrap()[0]
            .try_get::<String>("document")
            .unwrap(),
        document
    );
    engine.close().await.unwrap();
}

/// JSON null, absent, SQL NULL, booleans, integer/real and text never collapse to one type.
pub(super) async fn types() {
    let root = TempDir::new().unwrap();
    let engine = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    engine
        .execute(&sql(
            "CREATE TABLE documents(id INTEGER PRIMARY KEY, document TEXT)",
        ))
        .await
        .unwrap();
    let documents = [
        Some(r#"{"v":null,"a":[null]}"#),
        Some(r#"{"v":false,"a":[false]}"#),
        Some(r#"{"v":true,"a":[true]}"#),
        Some(r#"{"v":0,"a":[0]}"#),
        Some(r#"{"v":1,"a":[1]}"#),
        Some(r#"{"v":1.0,"a":[1.0]}"#),
        Some(r#"{"v":"1","a":["1"]}"#),
        Some("{}"),
        None,
    ];
    for (index, document) in documents.iter().enumerate() {
        engine
            .execute(
                &Insert::new("documents")
                    .columns(["id", "document"])
                    .values([
                        SqlValue::Integer(index as i64 + 1),
                        document
                            .map(|text| SqlValue::Text(text.into()))
                            .unwrap_or(SqlValue::Null),
                    ]),
            )
            .await
            .unwrap();
    }
    for (index, scalar) in [
        JsonScalar::Null,
        JsonScalar::Bool(false),
        JsonScalar::Bool(true),
        JsonScalar::Integer(0),
        JsonScalar::Integer(1),
        JsonScalar::Real(1.0),
        JsonScalar::String("1".into()),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            matching(
                &engine,
                Expr::json_eq("document", keys(&["v"]), scalar.clone())
            )
            .await,
            [index as i64 + 1]
        );
        assert_eq!(
            matching(
                &engine,
                Expr::json_contains("document", keys(&["a"]), scalar)
            )
            .await,
            [index as i64 + 1]
        );
    }
    assert!(
        matching(
            &engine,
            Expr::json_eq("document", keys(&["missing"]), JsonScalar::Null)
        )
        .await
        .is_empty()
    );
    // An explicit encoding API returns text; native rows do not automatically parse documents.
    let encoded = SqlValue::json(&stackpress_sql::JsonValue::Array(vec![
        stackpress_sql::JsonValue::Scalar(JsonScalar::Bool(false)),
        stackpress_sql::JsonValue::Scalar(JsonScalar::Null),
    ]))
    .unwrap();
    assert_eq!(encoded, SqlValue::Text("[false,null]".into()));

    // Invalid JSON is allowed as TEXT storage, but using a JSON function must preserve its error.
    engine
        .execute(
            &Insert::new("documents")
                .columns(["id", "document"])
                .values([SqlValue::Integer(10), SqlValue::Text("{broken".into())]),
        )
        .await
        .unwrap();
    let error = engine
        .fetch(
            &Select::new(["id"])
                .from("documents")
                .and_where(Expr::json_eq("document", keys(&["v"]), JsonScalar::Null)),
        )
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert!(error.source().is_some());
    assert_eq!(
        engine
            .fetch(
                &Select::new(["document"])
                    .from("documents")
                    .where_eq("id", 10_i64)
            )
            .await
            .unwrap()[0]
            .try_get::<String>("document")
            .unwrap(),
        "{broken"
    );
    engine.close().await.unwrap();
}
