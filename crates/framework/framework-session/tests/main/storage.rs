//! Real SQLite storage/lookup roundtrip with original deterministic source encodings.
use stackpress_framework_session::codecs::{SourceCodecs, date_string, hash};
use stackpress_sql::{DialectId, Engine, Sql};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

#[tokio::test]
async fn sql_codec_roundtrip_equality_uniqueness_and_consumed_hash() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("codec.sqlite");
    let db = SqliteDatabase::open(SqliteOptions::file(&path).create_if_missing(true))
        .await
        .unwrap();
    let engine = Engine::new(db);
    engine.execute(&Sql::literal(DialectId::Sqlite,"CREATE TABLE auth (token TEXT UNIQUE NOT NULL, secret TEXT NOT NULL, consumed TEXT NOT NULL)")).await.unwrap();
    let codecs = SourceCodecs::new("database-seed");
    let token = codecs.serialize_token("user@example.test");
    assert_eq!(
        token,
        "5dd687dad4a14580ec8d46fd439ee9b3cee6fe648c0851b8d642161392107ea8"
    );
    let secret = codecs.serialize_secret("hello");
    let consumed = "2023-11-14T22:13:20.123Z";
    engine
        .execute(
            &Sql::literal(DialectId::Sqlite, "INSERT INTO auth VALUES (?, ?, ?)")
                .bind(token.clone())
                .bind(secret.clone())
                .bind(consumed),
        )
        .await
        .unwrap();
    let rows = engine
        .fetch(
            &Sql::literal(
                DialectId::Sqlite,
                "SELECT token,secret,consumed FROM auth WHERE token = ?",
            )
            .bind(codecs.serialize_token("user@example.test")),
        )
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        codecs
            .unserialize_token(&rows[0].try_get::<String>("token").unwrap())
            .unwrap(),
        "user@example.test"
    );
    assert_eq!(rows[0].try_get::<String>("secret").unwrap(), secret);
    assert_eq!(rows[0].try_get::<String>("consumed").unwrap(), consumed);
    // Global unique token lookup uses the same encrypted bytes; no new blind index or method key.
    assert!(
        engine
            .execute(
                &Sql::literal(DialectId::Sqlite, "INSERT INTO auth VALUES (?, ?, ?)")
                    .bind(token)
                    .bind(secret)
                    .bind(consumed)
            )
            .await
            .is_err()
    );
    engine.close().await.unwrap();
}
#[test]
fn original_node_date_to_string_and_timestamp_challenges() {
    let vectors: serde_json::Value = serde_json::from_str(include_str!(
        "../../../framework-schema/src/providers/vectors.json"
    ))
    .unwrap();
    for case in vectors["dateVectors"].as_array().unwrap() {
        let date = date_string(
            case["ms"].as_i64().unwrap(),
            0,
            "Coordinated Universal Time",
        )
        .unwrap();
        assert_eq!(date, case["date"]);
        assert_eq!(hash(&date), case["hash"]);
        assert_eq!(hash(&(date + "000123")), case["otpHash"]);
    }
    assert_eq!(
        date_string(0, 480, "Philippine Standard Time").unwrap(),
        "Thu Jan 01 1970 08:00:00 GMT+0800 (Philippine Standard Time)"
    );
    assert_eq!(
        date_string(0, -300, "Eastern Standard Time").unwrap(),
        "Wed Dec 31 1969 19:00:00 GMT-0500 (Eastern Standard Time)"
    );
    assert!(date_string(0, i32::MAX, "invalid").is_err());
    assert!(date_string(i64::MAX, 0, "invalid").is_err());
}
