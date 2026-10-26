//! Real-server acceptance cases for every builder family and the new schema-diff plan.
use super::{Native, TestResult};
use stackpress_sql::*;
/// Dispatch stable independent case names; every statement uses a rendered Query/QueryPlan.
pub(super) async fn run(
    index: usize,
    connection: &mut Native,
    dialect: DialectId,
) -> TestResult<()> {
    match index {
        0 => create(connection, dialect).await,
        1 => crud(connection, dialect).await,
        2 => joins(connection, dialect).await,
        3 => expressions(connection, dialect).await,
        4 => json(connection, dialect).await,
        5 => alter(connection, dialect).await,
        6 => diff(connection, dialect).await,
        7 => commands(connection, dialect).await,
        _ => unreachable!(),
    }
}
/// Create typed defaults, identity, compound indexes, FK actions and idempotent indexes.
async fn create(connection: &mut Native, dialect: DialectId) -> TestResult<()> {
    let parent = Create::new("parent")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .primary_key(["id"]);
    connection.plan(&parent.render(dialect)?).await?;
    connection
        .run(
            &Insert::new("parent")
                .columns(["id"])
                .values([1])
                .render(dialect)?,
        )
        .await?;
    let mut foreign = ForeignKey::new("profile_parent", ["parent_id"], "parent", ["id"]);
    foreign.on_delete = ReferentialAction::SetNull;
    foreign.on_update = ReferentialAction::Cascade;
    let mut name = Field::new("name", SqlType::Varchar)
        .length(Length::Size(80))
        .default("");
    name.comment = Some("display name".into());
    let profiles = Create::new("profiles")
        .add_field(
            Field::new("id", SqlType::Integer)
                .not_null()
                .auto_increment(),
        )
        .add_field(name)
        .add_field(Field::new("active", SqlType::Boolean).default(false))
        .add_field(Field::new("amount", SqlType::Real).default(0.0))
        .add_field(
            Field::new("price", SqlType::Decimal)
                .length(Length::Precision {
                    precision: 10,
                    scale: 2,
                })
                .default(0),
        )
        .add_field(Field::new("note", SqlType::Text).default("apostrophe ' slash \\ 💡"))
        .add_field(Field::new("bytes", SqlType::Blob).default(vec![0_u8, 255]))
        .add_field(Field::new("created", SqlType::DateTime).default(DefaultValue::CurrentTimestamp))
        .add_field(Field::new("data", SqlType::Json))
        .add_field(Field::new("parent_id", SqlType::Integer))
        .primary_key(["id"])
        .add_foreign_key(foreign)
        .add_index(Index::new("profile_name", ["name"]))
        .add_index(Index::new("profile_identity", ["id", "name"]).unique())
        .if_not_exists();
    let plan = profiles.render(dialect)?;
    connection.plan(&plan).await?;
    connection.plan(&plan).await?;
    Ok(())
}
/// Exercise bound injection-looking values, bulk DEFAULT, NULL and real RETURNING support.
async fn crud(connection: &mut Native, dialect: DialectId) -> TestResult<()> {
    let mut insert = Insert::new("profiles")
        .columns(["name", "active", "parent_id"])
        .values([WriteValue::from("Ada"), true.into(), 1.into()]);
    if dialect.capabilities().returning {
        insert = insert.returning(["id"]);
    }
    let ids = connection.run(&insert.render(dialect)?).await?;
    if dialect.capabilities().returning {
        assert_eq!(ids, [1]);
    }
    connection
        .run(
            &Insert::new("profiles")
                .columns(["name", "active"])
                .values([WriteValue::from("Grace"), WriteValue::Default])
                .values([WriteValue::from("remove"), true.into()])
                .render(dialect)?,
        )
        .await?;
    let mut update = Update::new("profiles")
        .set("name", "Ada '; DROP TABLE parent; -- \\ 💡")
        .where_eq("id", 1);
    if dialect.capabilities().returning {
        update = update.returning(["id"]);
    }
    connection.run(&update.render(dialect)?).await?;
    let mut delete = Delete::new("profiles").where_eq("id", 3);
    if dialect.capabilities().returning {
        delete = delete.returning(["id"]);
    }
    connection.run(&delete.render(dialect)?).await?;
    assert_eq!(
        connection
            .run(
                &Select::new(["id"])
                    .from("profiles")
                    .order("id")
                    .render(dialect)?
            )
            .await?,
        [1, 2]
    );
    assert_eq!(
        connection
            .run(
                &Select::new(["id"])
                    .from("profiles")
                    .where_eq("active", false)
                    .render(dialect)?
            )
            .await?,
        [2]
    );
    Ok(())
}
/// Typed operands and aliases run on each supported native join kind.
async fn joins(connection: &mut Native, dialect: DialectId) -> TestResult<()> {
    for kind in [
        JoinKind::Inner,
        JoinKind::Left,
        JoinKind::Right,
        JoinKind::Full,
        JoinKind::Cross,
    ] {
        let query = Select::new([Identifier::qualified(["p", "id"])])
            .from(TableRef::new("profiles").alias("p"))
            .join(Join {
                kind,
                table: TableRef::new("parent").alias("q"),
                on: if kind == JoinKind::Cross {
                    None
                } else {
                    Some(
                        Expr::column(Identifier::qualified(["p", "parent_id"])).compare(
                            Compare::Eq,
                            Expr::column(Identifier::qualified(["q", "id"])),
                        ),
                    )
                },
            })
            .order_by(Identifier::qualified(["p", "id"]), Order::Asc)
            .limit(1);
        if kind == JoinKind::Full && !dialect.capabilities().full_join {
            assert_eq!(
                query.render(dialect).unwrap_err().kind(),
                ErrorKind::Unsupported
            );
        } else {
            assert!(!connection.run(&query.render(dialect)?).await?.is_empty());
        }
    }
    Ok(())
}
/// Predicate groups, operators, pagination, zero limits and escaped LIKE remain bound.
async fn expressions(connection: &mut Native, dialect: DialectId) -> TestResult<()> {
    let query = Select::new(["id"])
        .from("profiles")
        .and_where(Expr::all([
            Expr::In(
                Box::new(Expr::column("id")),
                vec![1.into(), 2.into()],
                false,
            ),
            Expr::any([Expr::eq("active", false), Expr::eq("id", 1)]),
        ]))
        .order("id")
        .offset(1);
    assert_eq!(connection.run(&query.render(dialect)?).await?, [2]);
    assert!(
        connection
            .run(
                &Select::new(["id"])
                    .from("profiles")
                    .limit(0)
                    .render(dialect)?
            )
            .await?
            .is_empty()
    );
    let like = Expr::Like(Box::new(Expr::column("name")), "Grace".into(), '!');
    assert_eq!(
        connection
            .run(
                &Select::new(["id"])
                    .from("profiles")
                    .and_where(like)
                    .render(dialect)?
            )
            .await?,
        [2]
    );
    Ok(())
}
/// Bool/int/text/null, missing paths and nested-array nonmembership are real outcomes.
async fn json(connection: &mut Native, dialect: DialectId) -> TestResult<()> {
    let scalar = |value| JsonValue::Scalar(value);
    let document = JsonValue::Object(vec![
        (
            "tags".into(),
            JsonValue::Array(vec![
                scalar(JsonScalar::String("admin".into())),
                JsonValue::Array(vec![scalar(JsonScalar::String("nested".into()))]),
                scalar(JsonScalar::Bool(true)),
                scalar(JsonScalar::Integer(1)),
                scalar(JsonScalar::String("1".into())),
                scalar(JsonScalar::Null),
            ]),
        ),
        (
            "only_bool".into(),
            JsonValue::Array(vec![scalar(JsonScalar::Bool(true))]),
        ),
        (
            "only_number".into(),
            JsonValue::Array(vec![scalar(JsonScalar::Integer(1))]),
        ),
        (
            "only_text".into(),
            JsonValue::Array(vec![scalar(JsonScalar::String("1".into()))]),
        ),
        ("nil".into(), scalar(JsonScalar::Null)),
        (
            "01".into(),
            JsonValue::Object(vec![(
                "a\"b\\c".into(),
                scalar(JsonScalar::String("sensitive ' ; --".into())),
            )]),
        ),
    ]);
    connection
        .run(
            &Update::new("profiles")
                .set("data", WriteValue::Json(document))
                .where_eq("id", 1)
                .render(dialect)?,
        )
        .await?;
    let tags = JsonPath::new([JsonSegment::Key("tags".into())]);
    for value in [
        JsonScalar::String("admin".into()),
        JsonScalar::Bool(true),
        JsonScalar::Integer(1),
        JsonScalar::Real(1.0),
        JsonScalar::String("1".into()),
        JsonScalar::Null,
    ] {
        assert_eq!(
            connection
                .run(
                    &Select::new(["id"])
                        .from("profiles")
                        .where_json_contains("data", tags.clone(), value)
                        .render(dialect)?
                )
                .await?,
            [1]
        );
    }
    for value in [JsonScalar::String("nested".into()), JsonScalar::Bool(false)] {
        assert!(
            connection
                .run(
                    &Select::new(["id"])
                        .from("profiles")
                        .where_json_contains("data", tags.clone(), value)
                        .render(dialect)?
                )
                .await?
                .is_empty()
        );
    }
    for (key, expected) in [("nil", vec![1]), ("missing", vec![])] {
        assert_eq!(
            connection
                .run(
                    &Select::new(["id"])
                        .from("profiles")
                        .where_json(
                            "data",
                            JsonPath::new([JsonSegment::Key(key.into())]),
                            JsonScalar::Null
                        )
                        .render(dialect)?
                )
                .await?,
            expected
        );
    }
    let path = JsonPath::new([
        JsonSegment::Key("01".into()),
        JsonSegment::Key("a\"b\\c".into()),
    ]);
    assert_eq!(
        connection
            .run(
                &Select::new(["id"])
                    .from("profiles")
                    .where_json(
                        "data",
                        path.clone(),
                        JsonScalar::String("sensitive ' ; --".into())
                    )
                    .render(dialect)?
            )
            .await?,
        [1]
    );
    assert_eq!(
        connection
            .run(
                &Select::new([
                    Selector::from("id"),
                    Selector::expression(Expr::json("data", path)).alias("selected")
                ])
                .from("profiles")
                .where_eq("id", 1)
                .render(dialect)?
            )
            .await?,
        [1]
    );
    // Negative cross-type cases prove native JSON does not coerce bool/number/text.
    for (key, expected_kind) in [("only_bool", 0), ("only_number", 1), ("only_text", 2)] {
        for (kind, value) in [
            JsonScalar::Bool(true),
            JsonScalar::Integer(1),
            JsonScalar::String("1".into()),
        ]
        .into_iter()
        .enumerate()
        {
            let rows = connection
                .run(
                    &Select::new(["id"])
                        .from("profiles")
                        .where_json_contains(
                            "data",
                            JsonPath::new([JsonSegment::Key(key.into())]),
                            value,
                        )
                        .render(dialect)?,
                )
                .await?;
            assert_eq!(
                rows,
                if kind == expected_kind {
                    vec![1]
                } else {
                    vec![]
                }
            );
        }
    }
    let indexed = JsonPath::new([
        JsonSegment::Key("tags".into()),
        JsonSegment::Index(1),
        JsonSegment::Index(0),
    ]);
    assert_eq!(
        connection
            .run(
                &Select::new(["id"])
                    .from("profiles")
                    .where_json("data", indexed, JsonScalar::String("nested".into()))
                    .render(dialect)?
            )
            .await?,
        [1]
    );
    // Malformed JSON in ordinary text must be a native failure, never an empty success.
    connection
        .plan(
            &Create::new("bad_json")
                .add_field(Field::new("id", SqlType::Integer))
                .add_field(Field::new("data", SqlType::Text))
                .render(dialect)?,
        )
        .await?;
    connection
        .run(
            &Insert::new("bad_json")
                .columns(["id", "data"])
                .values([WriteValue::from(1), "{bad".into()])
                .render(dialect)?,
        )
        .await?;
    assert!(
        connection
            .run(
                &Select::new(["id"])
                    .from("bad_json")
                    .where_json("data", JsonPath::default(), JsonScalar::Null)
                    .render(dialect)?
            )
            .await
            .is_err()
    );
    assert_eq!(
        connection
            .run(&Select::new(["id"]).from("bad_json").render(dialect)?)
            .await?,
        [1]
    );
    Ok(())
}
/// Native ADD/DROP/RENAME/MODIFY and primary/FK actions execute against fixture data.
async fn alter(connection: &mut Native, dialect: DialectId) -> TestResult<()> {
    let create = Create::new("altering")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("old", SqlType::Text))
        .primary_key(["id"]);
    connection.plan(&create.render(dialect)?).await?;
    connection
        .run(
            &Insert::new("altering")
                .columns(["id", "old"])
                .values([WriteValue::from(1), "preserve".into()])
                .render(dialect)?,
        )
        .await?;
    let alter = Alter::new("altering")
        .source_field(Field::new("old", SqlType::Text))
        .rename_field("old", "new")
        .add_field(Field::new("extra", SqlType::Text))
        .add_index(Index::new("alter_new", ["new"]));
    // MySQL requires a prefix length for TEXT indexes; use an explicit VARCHAR change first.
    let alter = if dialect == DialectId::Mysql {
        Alter::new("altering")
            .source_field(Field::new("old", SqlType::Text))
            .rename_field("old", "new")
            .change_field(Field::new("new", SqlType::Varchar).length(Length::Size(80)))
            .add_field(Field::new("extra", SqlType::Text))
            .add_index(Index::new("alter_new", ["new"]))
    } else {
        alter
    };
    connection.plan(&alter.render(dialect)?).await?;
    let change = Alter::new("altering")
        .source_field(Field::new("id", SqlType::Integer).not_null())
        .source_field(Field::new("new", SqlType::Text))
        .change_field(
            Field::new("new", SqlType::Varchar)
                .length(Length::Size(100))
                .not_null()
                .default(""),
        )
        .remove_field("extra")
        .remove_primary_key()
        .primary_key(["id"])
        .add_foreign_key(ForeignKey::new("alter_parent", ["id"], "parent", ["id"]));
    connection.plan(&change.render(dialect)?).await?;
    assert_eq!(
        connection
            .run(
                &Select::new(["id"])
                    .from("altering")
                    .where_eq("new", "preserve")
                    .render(dialect)?
            )
            .await?,
        [1]
    );
    Ok(())
}
/// A changed-definition rename keeps both operations and preserves the original row.
async fn diff(connection: &mut Native, dialect: DialectId) -> TestResult<()> {
    let source = Create::new("diffed")
        .add_field(Field::new("id", SqlType::Integer))
        .add_field(Field::new("old", SqlType::Varchar).length(Length::Size(80)))
        .primary_key(["id"])
        .definition()?;
    connection.plan(&source.create().render(dialect)?).await?;
    connection
        .run(
            &Insert::new("diffed")
                .columns(["id", "old"])
                .values([WriteValue::from(1), "Ada".into()])
                .render(dialect)?,
        )
        .await?;
    let target = Create::new("diffed")
        .add_field(Field::new("id", SqlType::Integer))
        .add_field(
            Field::new("new", SqlType::Varchar)
                .length(Length::Size(100))
                .not_null()
                .default("fallback"),
        )
        .primary_key(["id"])
        .definition()?;
    let diff = source.diff(&target)?.rename_field("old", "new")?;
    connection.plan(&diff.render(dialect)?.unwrap()).await?;
    assert_eq!(
        connection
            .run(
                &Select::new(["id"])
                    .from("diffed")
                    .where_eq("new", "Ada")
                    .render(dialect)?
            )
            .await?,
        [1]
    );
    Ok(())
}
/// Rename/truncate are explicit; no standalone DROP TABLE/INDEX is issued by this harness.
async fn commands(connection: &mut Native, dialect: DialectId) -> TestResult<()> {
    connection
        .run(&TableCommand::rename("altering", "renamed").render(dialect)?)
        .await?;
    connection
        .run(&TableCommand::truncate("renamed").render(dialect)?)
        .await?;
    assert!(
        connection
            .run(&Select::new(["id"]).from("renamed").render(dialect)?)
            .await?
            .is_empty()
    );
    Ok(())
}
