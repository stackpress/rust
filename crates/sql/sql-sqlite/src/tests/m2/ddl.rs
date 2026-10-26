//! Execute rendered DDL against files; compare both schema and data after rejected batches.

use std::error::Error;

use stackpress_sql::{
    Alter, AlterOperation, Create, DialectId, Engine, ErrorKind, Field, ForeignKey, Index, Insert,
    ReferentialAction, Select, SqlType, SqlValue, TableCommand,
};

use super::{SqliteDatabase, TempDir, options, plan, schema, sql};

/// All native ADD/DROP/RENAME/index commands preserve rows and explicit defaults.
pub(super) async fn supported() {
    let root = TempDir::new().unwrap();
    let engine = Engine::new(SqliteDatabase::open(options(&root, 2)).await.unwrap());
    let create = Create::new("profile")
        .add_field(Field::new("id", SqlType::Integer).auto_increment())
        .add_field(Field::new("name", SqlType::Text).not_null())
        .add_field(Field::new("enabled", SqlType::Boolean).default(false))
        .add_field(Field::new("score", SqlType::Integer).default(0_i64))
        .add_field(Field::new("empty", SqlType::Text).default(""))
        .add_field(Field::new("sentinel", SqlType::Text).default("' ); DROP TABLE profile; --"))
        .add_field(Field::new("optional", SqlType::Text).default(SqlValue::Null))
        .primary_key(["id"])
        .add_index(Index::new("profile_name", ["name"]).unique())
        .add_index(Index::new("profile_score", ["score"]))
        .if_not_exists();
    let rendered = create.render(DialectId::Sqlite).unwrap();
    assert_eq!(plan(&engine, &rendered).await.unwrap().len(), 3);
    let installed = schema(&engine).await;
    plan(&engine, &rendered).await.unwrap();
    assert_eq!(schema(&engine).await, installed);
    let inserted = engine
        .execute(
            &Insert::new("profile")
                .columns(["name"])
                .values(["O'Reilly; -- λ"])
                .returning_all(),
        )
        .await
        .unwrap();
    let row = &inserted.rows[0];
    assert!(!row.try_get::<bool>("enabled").unwrap());
    assert_eq!(row.try_get::<i64>("score").unwrap(), 0);
    assert_eq!(row.try_get::<String>("empty").unwrap(), "");
    assert_eq!(
        row.try_get::<String>("sentinel").unwrap(),
        "' ); DROP TABLE profile; --"
    );
    assert_eq!(row.try_get::<Option<String>>("optional").unwrap(), None);

    // The original metadata makes rename intent explicit; SQLite preserves the stored text.
    let alter = Alter::new("profile")
        .source_field(Field::new("name", SqlType::Text).not_null())
        .rename_field("name", "display.name")
        .add_field(
            Field::new("added", SqlType::Integer)
                .not_null()
                .default(0_i64),
        )
        .remove_field("optional")
        .remove_index("profile_score")
        .add_index(Index::new("display_index", ["display.name"]));
    plan(&engine, &alter.render(DialectId::Sqlite).unwrap())
        .await
        .unwrap();
    let rows = engine
        .fetch(&Select::new(["display.name", "added"]).from("profile"))
        .await
        .unwrap();
    assert_eq!(
        rows[0].try_get::<String>("display.name").unwrap(),
        "O'Reilly; -- λ"
    );
    assert_eq!(rows[0].try_get::<i64>("added").unwrap(), 0);
    engine
        .execute(&TableCommand::rename("profile", "profile\";--"))
        .await
        .unwrap();
    engine
        .execute(&TableCommand::truncate("profile\";--"))
        .await
        .unwrap();
    let next = engine
        .execute(
            &Insert::new("profile\";--")
                .columns(["display.name"])
                .values(["after truncate"])
                .returning(["id"]),
        )
        .await
        .unwrap();
    assert_eq!(next.rows[0].try_get::<i64>("id").unwrap(), 2);
    engine
        .execute(&TableCommand::drop_table("profile\";--"))
        .await
        .unwrap();
    foreign_keys(&engine).await;
    engine.close().await.unwrap();
}

/// Compound key ordering and native FK actions are real execution, not render snapshots.
async fn foreign_keys(engine: &Engine) {
    let parent = Create::new("parent")
        .add_field(Field::new("a", SqlType::Integer).not_null())
        .add_field(Field::new("b", SqlType::Integer).not_null())
        .primary_key(["b", "a"]);
    let mut foreign = ForeignKey::new("parent_ref", ["y", "x"], "parent", ["b", "a"]);
    foreign.on_delete = ReferentialAction::Cascade;
    let child = Create::new("child")
        .add_field(Field::new("x", SqlType::Integer))
        .add_field(Field::new("y", SqlType::Integer))
        .add_foreign_key(foreign);
    for create in [parent, child] {
        plan(engine, &create.render(DialectId::Sqlite).unwrap())
            .await
            .unwrap();
    }
    engine
        .execute(&Insert::new("parent").columns(["a", "b"]).values([1_i64, 2]))
        .await
        .unwrap();
    engine
        .execute(&Insert::new("child").columns(["x", "y"]).values([1_i64, 2]))
        .await
        .unwrap();
    let error = engine
        .execute(&Insert::new("child").columns(["x", "y"]).values([2_i64, 1]))
        .await
        .unwrap_err();
    assert!(error.source().is_some());
    engine
        .execute(&stackpress_sql::Delete::new("parent").all_rows())
        .await
        .unwrap();
    assert!(
        engine
            .fetch(&Select::new(["*"]).from("child"))
            .await
            .unwrap()
            .is_empty()
    );
}

/// A later native CREATE/ALTER error rolls back successful earlier statements and their data.
pub(super) async fn atomic() {
    let root = TempDir::new().unwrap();
    let engine = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    engine
        .execute(&sql("CREATE TABLE existing(value TEXT)"))
        .await
        .unwrap();
    engine
        .execute(&sql("CREATE INDEX occupied ON existing(value)"))
        .await
        .unwrap();
    let before = schema(&engine).await;
    let create = Create::new("never_installed")
        .add_field(Field::new("value", SqlType::Text))
        .add_index(Index::new("occupied", ["value"]));
    let error = plan(&engine, &create.render(DialectId::Sqlite).unwrap())
        .await
        .unwrap_err();
    assert!(error.source().is_some());
    assert!(error.cleanup_error().is_none());
    assert_eq!(schema(&engine).await, before);
    engine
        .execute(
            &Insert::new("existing")
                .columns(["value"])
                .values(["duplicate"])
                .values(["duplicate"]),
        )
        .await
        .unwrap();
    let rows = engine
        .fetch(&Select::new(["*"]).from("existing"))
        .await
        .unwrap();
    let alter = Alter::new("existing")
        .add_field(Field::new("partial", SqlType::Text))
        .add_index(Index::new("duplicates_forbidden", ["value"]).unique());
    let error = plan(&engine, &alter.render(DialectId::Sqlite).unwrap())
        .await
        .unwrap_err();
    assert!(error.source().is_some());
    assert_eq!(schema(&engine).await, before);
    assert_eq!(
        engine
            .fetch(&Select::new(["*"]).from("existing"))
            .await
            .unwrap(),
        rows
    );

    // A rename is transactional too, even if the next DROP targets a nonexistent index.
    let alter = Alter::new("existing")
        .source_field(Field::new("value", SqlType::Text))
        .rename_field("value", "renamed")
        .remove_index("absent");
    assert!(
        plan(&engine, &alter.render(DialectId::Sqlite).unwrap())
            .await
            .is_err()
    );
    assert_eq!(schema(&engine).await, before);
    assert_eq!(
        engine
            .fetch(&Select::new(["*"]).from("existing"))
            .await
            .unwrap(),
        rows
    );
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(schema(&reopened).await, before);
    assert_eq!(
        reopened
            .fetch(&Select::new(["*"]).from("existing"))
            .await
            .unwrap(),
        rows
    );
    reopened.close().await.unwrap();
}

/// Every unsupported native alteration rejects the whole plan, including an earlier valid ADD.
pub(super) async fn unsupported() {
    let root = TempDir::new().unwrap();
    let engine = super::super::engine(&root, 1).await;
    engine
        .execute(
            &Insert::new("profile")
                .columns(["name"])
                .values(["preserved"]),
        )
        .await
        .unwrap();
    let before = schema(&engine).await;
    let rows = engine
        .fetch(&Select::new(["*"]).from("profile"))
        .await
        .unwrap();
    let operations = [
        AlterOperation::ChangeField(Field::new("name", SqlType::Integer)),
        AlterOperation::ChangeField(Field::new("name", SqlType::Text)),
        AlterOperation::AddPrimary(vec!["id".into()]),
        AlterOperation::RemovePrimary,
        AlterOperation::AddForeignKey(ForeignKey::new("fk", ["id"], "profile", ["id"])),
        AlterOperation::RemoveForeignKey("fk".into()),
        AlterOperation::AddField(Field::new("required", SqlType::Text).not_null()),
    ];
    for operation in operations {
        let alter = Alter::new("profile")
            .source_field(Field::new("id", SqlType::Integer))
            .source_field(Field::new("name", SqlType::Text).not_null())
            .add_field(Field::new("partial", SqlType::Text))
            .operation(operation);
        assert_eq!(
            alter.render(DialectId::Sqlite).unwrap_err().kind(),
            ErrorKind::Unsupported
        );
        assert_eq!(schema(&engine).await, before);
        assert_eq!(
            engine
                .fetch(&Select::new(["*"]).from("profile"))
                .await
                .unwrap(),
            rows
        );
    }
    // An explicit rename cannot conceal a requested nullability change in remove/add halves.
    let changed_rename = Alter::new("profile")
        .source_field(Field::new("name", SqlType::Text).not_null())
        .remove_field("name")
        .add_field(Field::new("renamed", SqlType::Text))
        .rename_field("name", "renamed");
    assert_eq!(
        changed_rename.render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    assert_eq!(schema(&engine).await, before);
    assert_eq!(
        engine
            .fetch(&Select::new(["*"]).from("profile"))
            .await
            .unwrap(),
        rows
    );
    // Type annotations must not be silently accepted and then lost by SQLite affinity.
    let mut unsigned = Field::new("unsigned", SqlType::Integer);
    unsigned.unsigned = true;
    let mut commented = Field::new("commented", SqlType::Text);
    commented.comment = Some("must be stored".into());
    for field in [unsigned, commented, Field::new("decimal", SqlType::Decimal)] {
        let create = Create::new("unsupported").add_field(field);
        assert_eq!(
            create.render(DialectId::Sqlite).unwrap_err().kind(),
            ErrorKind::Unsupported
        );
        assert_eq!(schema(&engine).await, before);
    }
    engine.close().await.unwrap();
}

/// A warmed pooled connection must refresh both schema and cached result names after DDL elsewhere.
pub(super) async fn cached_schema() {
    use stackpress_sql::{Database, QueryLimits, Render};
    let root = TempDir::new().unwrap();
    let db = SqliteDatabase::open(options(&root, 2)).await.unwrap();
    let limits = QueryLimits::default();
    db.run(
        sql("CREATE TABLE cached(value TEXT)")
            .render(DialectId::Sqlite)
            .unwrap(),
        limits,
    )
    .await
    .unwrap();
    db.run(
        sql("INSERT INTO cached VALUES ('preserved')")
            .render(DialectId::Sqlite)
            .unwrap(),
        limits,
    )
    .await
    .unwrap();
    // Retain the exact warmed connection while the public plan runs on the other pool slot.
    let mut old = db.pool.acquire().await.unwrap();
    let star = sql("SELECT * FROM cached")
        .render(DialectId::Sqlite)
        .unwrap();
    let initial = crate::execute::run(&mut old, star.clone(), limits)
        .await
        .unwrap();
    assert_eq!(initial.rows[0].cells()[0].0, "value");
    let engine = Engine::new(db);
    let alter = Alter::new("cached")
        .source_field(Field::new("value", SqlType::Text))
        .rename_field("value", "display.name")
        .add_field(Field::new("added", SqlType::Integer).default(0_i64));
    plan(&engine, &alter.render(DialectId::Sqlite).unwrap())
        .await
        .unwrap();
    let after = crate::execute::run(&mut old, star, limits).await.unwrap();
    assert_eq!(after.rows[0].cells().len(), 2);
    assert_eq!(
        after.rows[0].try_get::<String>("display.name").unwrap(),
        "preserved"
    );
    assert_eq!(after.rows[0].try_get::<i64>("added").unwrap(), 0);
    let quoted = Select::new(["display.name", "added"])
        .from("cached")
        .render(DialectId::Sqlite)
        .unwrap();
    let after = crate::execute::run(&mut old, quoted, limits).await.unwrap();
    assert_eq!(
        after.rows[0].try_get::<String>("display.name").unwrap(),
        "preserved"
    );
    drop(old);
    engine.close().await.unwrap();
}
