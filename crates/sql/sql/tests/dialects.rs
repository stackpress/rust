//! M2 PostgreSQL/MySQL source inventory: all six families, types/defaults, JSON,
//! idempotence, quoting and explicit unsupported capabilities. Network IO lives elsewhere.
use stackpress_sql::*;
/// Render without a native driver.
fn render(query: &impl Render, dialect: DialectId) -> Query {
    query.render(dialect).unwrap()
}
/// Capabilities describe actual output behavior, not installed production drivers.
#[test]
fn explicit_capability_matrix() {
    let sqlite = DialectId::Sqlite.capabilities();
    let pg = DialectId::Postgres.capabilities();
    let mysql = DialectId::Mysql.capabilities();
    assert!(sqlite.returning && pg.returning && !mysql.returning);
    assert!(sqlite.transactional_ddl && pg.transactional_ddl && !mysql.transactional_ddl);
    assert!(!sqlite.alter_constraints && pg.alter_constraints && mysql.alter_constraints);
    assert!(sqlite.full_join && pg.full_join && !mysql.full_join);
    assert!(!sqlite.cascade && pg.cascade && !mysql.cascade);
    assert!(sqlite.idempotent_indexes && pg.idempotent_indexes && !mysql.idempotent_indexes);
    assert!(
        sqlite.distinct_json_number_kinds
            && !pg.distinct_json_number_kinds
            && !mysql.distinct_json_number_kinds
    );
    assert!(sqlite.text_nul && !pg.text_nul && mysql.text_nul);
}
/// Bound AST traversal numbers PG parameters without rewriting literal/comment tokens.
#[test]
fn all_dml_families_native_snapshots() {
    let select = Select::new([
        Selector::from(Identifier::qualified(["p", "id"])).alias("selected"),
        Selector::expression(Expr::value("selector?")),
    ])
    .from(TableRef::new("profile.table").alias("p"))
    .where_eq("name", "O'Reilly?")
    .order("id")
    .limit(0)
    .offset(1);
    let pg = render(&select, DialectId::Postgres);
    assert_eq!(
        pg.sql(),
        "SELECT \"p\".\"id\" AS \"selected\", $1 FROM \"profile.table\" AS \"p\" WHERE \"name\" = $2 ORDER BY \"id\" ASC LIMIT 0 OFFSET 1"
    );
    assert_eq!(pg.binds(), &["selector?".into(), "O'Reilly?".into()]);
    assert_eq!(
        render(&select, DialectId::Mysql).sql(),
        "SELECT `p`.`id` AS `selected`, ? FROM `profile.table` AS `p` WHERE `name` = ? ORDER BY `id` ASC LIMIT 0 OFFSET 1"
    );
    let insert = Insert::new("p")
        .columns(["id", "name"])
        .values([WriteValue::from(1), "Ada".into()])
        .values([2.into(), WriteValue::Default]);
    assert_eq!(
        render(&insert, DialectId::Postgres).sql(),
        "INSERT INTO \"p\" (\"id\", \"name\") VALUES ($1, $2), ($3, DEFAULT)"
    );
    assert_eq!(
        render(&insert, DialectId::Mysql).sql(),
        "INSERT INTO `p` (`id`, `name`) VALUES (?, ?), (?, DEFAULT)"
    );
    assert_eq!(
        render(&Insert::new("p").default_values(), DialectId::Mysql).sql(),
        "INSERT INTO `p` () VALUES ()"
    );
    assert_eq!(
        render(&Insert::new("p").default_values(), DialectId::Postgres).sql(),
        "INSERT INTO \"p\" DEFAULT VALUES"
    );
    let update = Update::new("p")
        .set("name", "Ada")
        .where_eq("id", 1)
        .returning(["id"]);
    assert_eq!(
        render(&update, DialectId::Postgres).sql(),
        "UPDATE \"p\" SET \"name\" = $1 WHERE \"id\" = $2 RETURNING \"id\""
    );
    let delete = Delete::new("p").where_eq("id", 1).returning_all();
    assert_eq!(
        render(&delete, DialectId::Postgres).sql(),
        "DELETE FROM \"p\" WHERE \"id\" = $1 RETURNING *"
    );
    assert_eq!(
        render(
            &Update::new("p").set("x", WriteValue::Default).all_rows(),
            DialectId::Mysql
        )
        .sql(),
        "UPDATE `p` SET `x` = DEFAULT"
    );
    assert_eq!(
        render(&Delete::new("p").where_eq("id", 1), DialectId::Mysql).sql(),
        "DELETE FROM `p` WHERE `id` = ?"
    );
}
/// Unsupported DML features error at render time rather than silently losing a clause.
#[test]
fn unsupported_returning_full_join_and_invalid_shapes() {
    assert_eq!(
        Insert::new("p")
            .default_values()
            .returning_all()
            .render(DialectId::Mysql)
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
    assert!(
        Update::new("p")
            .set("x", 1)
            .all_rows()
            .returning_all()
            .render(DialectId::Mysql)
            .is_err()
    );
    assert!(
        Delete::new("p")
            .all_rows()
            .returning_all()
            .render(DialectId::Mysql)
            .is_err()
    );
    let join = Select::default().from("p").join(Join {
        kind: JoinKind::Full,
        table: "q".into(),
        on: Some(Expr::eq("id", 1)),
    });
    assert_eq!(
        join.render(DialectId::Mysql).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        assert!(Select::default().render(dialect).is_err());
        assert!(Insert::new("p").render(dialect).is_err());
        assert!(Update::new("p").all_rows().render(dialect).is_err());
        assert!(Delete::new("p").render(dialect).is_err());
        assert!(Create::new("p").render(dialect).is_err());
        assert!(Alter::new("p").render(dialect).is_err());
        assert!(Select::default().from("").render(dialect).is_err());
    }
}
/// Table statements and offset-only syntax expose native cascade/sequence behavior.
#[test]
fn commands_and_offset_only() {
    let command = TableCommand::Truncate {
        table: "p".into(),
        cascade: true,
    };
    assert_eq!(
        render(&command, DialectId::Postgres).sql(),
        "TRUNCATE TABLE \"p\" CASCADE"
    );
    assert!(command.render(DialectId::Mysql).is_err());
    assert_eq!(
        render(&TableCommand::truncate("p"), DialectId::Mysql).sql(),
        "TRUNCATE TABLE `p`"
    );
    assert_eq!(
        render(
            &TableCommand::Drop {
                table: "p".into(),
                if_exists: true,
                cascade: true
            },
            DialectId::Postgres
        )
        .sql(),
        "DROP TABLE IF EXISTS \"p\" CASCADE"
    );
    assert_eq!(
        render(&TableCommand::rename("a`b", "c.d"), DialectId::Mysql).sql(),
        "ALTER TABLE `a``b` RENAME TO `c.d`"
    );
    assert_eq!(
        render(&Select::default().from("p").offset(1), DialectId::Postgres).sql(),
        "SELECT * FROM \"p\" OFFSET 1"
    );
    assert_eq!(
        render(&Select::default().from("p").offset(1), DialectId::Mysql).sql(),
        "SELECT * FROM `p` LIMIT 18446744073709551615 OFFSET 1"
    );
}
/// Native create types, defaults, comments and idempotence remain inspectable snapshots.
#[test]
fn create_types_defaults_and_idempotent_indexes() {
    let base = Create::new("p")
        .add_field(
            Field::new("id", SqlType::Integer)
                .not_null()
                .auto_increment(),
        )
        .add_field(Field::new("flag", SqlType::Boolean))
        .add_field(Field::new("name", SqlType::Text).default(""))
        .primary_key(["id"])
        .add_index(Index::new("flag_index", ["flag"]))
        .if_not_exists();
    let pg = base.render(DialectId::Postgres).unwrap();
    assert!(
        pg.queries()[0]
            .sql()
            .contains("\"id\" BIGINT GENERATED BY DEFAULT AS IDENTITY NOT NULL")
    );
    assert!(
        pg.queries()[0]
            .sql()
            .contains("\"flag\" BOOLEAN, \"name\" TEXT DEFAULT ''")
    );
    assert_eq!(
        pg.queries()[1].sql(),
        "CREATE INDEX IF NOT EXISTS \"flag_index\" ON \"p\" (\"flag\")"
    );
    let mysql = base.render(DialectId::Mysql).unwrap();
    assert_eq!(mysql.queries().len(), 1);
    assert!(
        mysql.queries()[0]
            .sql()
            .contains("CREATE TABLE IF NOT EXISTS `p`")
    );
    assert!(
        mysql.queries()[0]
            .sql()
            .contains("`id` BIGINT AUTO_INCREMENT NOT NULL")
    );
    assert!(
        mysql.queries()[0]
            .sql()
            .contains("`name` TEXT DEFAULT ('')")
    );
    assert!(
        mysql.queries()[0]
            .sql()
            .contains("INDEX `flag_index` (`flag`)")
    );
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        let plan = Alter::new("p")
            .add_index(Index::new("flag_index", ["flag"]))
            .render(dialect)
            .unwrap();
        assert!(!plan.queries()[0].sql().contains("IF NOT EXISTS"));
        let defaults = Create::new("defaults")
            .add_field(Field::new("nil", SqlType::Text).default(SqlValue::Null))
            .add_field(Field::new("zero", SqlType::Integer).default(0))
            .add_field(Field::new("false", SqlType::Boolean).default(false))
            .add_field(Field::new("quoted", SqlType::Text).default("' \\ 💡"));
        let plan = defaults.render(dialect).unwrap();
        assert!(plan.queries()[0].sql().contains("DEFAULT NULL"));
        assert!(plan.queries()[0].sql().contains("DEFAULT 0"));
        assert!(plan.queries()[0].sql().contains("DEFAULT FALSE"));
    }
}
/// Unsupported type/attribute combinations are rejected before a complete plan is exposed.
#[test]
fn fields_foreign_actions_and_invalid_lengths() {
    let mut unsigned = Field::new("id", SqlType::Integer);
    unsigned.unsigned = true;
    assert!(
        Create::new("p")
            .add_field(unsigned.clone())
            .render(DialectId::Mysql)
            .unwrap()
            .queries()[0]
            .sql()
            .contains("BIGINT UNSIGNED")
    );
    assert_eq!(
        Create::new("p")
            .add_field(unsigned)
            .render(DialectId::Postgres)
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        for length in [Length::Size(0), Length::Size(-1)] {
            assert!(
                Create::new("p")
                    .add_field(Field::new("x", SqlType::Varchar).length(length))
                    .render(dialect)
                    .is_err()
            );
        }
        assert!(
            Create::new("p")
                .add_field(Field::new("x", SqlType::Text).default(f64::NAN))
                .render(dialect)
                .is_err()
        );
        assert!(
            Create::new("p")
                .add_field(Field::new(
                    "x",
                    SqlType::Trusted(dialect, TrustedSql::audited("CUSTOM"))
                ))
                .render(dialect)
                .is_ok()
        );
    }
    assert!(
        Create::new("p")
            .add_field(Field::new("x", SqlType::Varchar))
            .render(DialectId::Mysql)
            .is_err()
    );
    assert!(
        Create::new("p")
            .add_field(Field::new("x", SqlType::Varchar))
            .render(DialectId::Postgres)
            .is_ok()
    );
    let mut foreign = ForeignKey::new("fk", ["id"], "parent", ["id"]);
    foreign.on_delete = ReferentialAction::SetDefault;
    let create = Create::new("p")
        .add_field(Field::new("id", SqlType::Integer).default(0))
        .add_foreign_key(foreign);
    assert!(create.render(DialectId::Postgres).is_ok());
    assert_eq!(
        create.render(DialectId::Mysql).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
}
/// All native ALTER categories render, including changed defaults/type/nullability and key removal.
#[test]
fn alter_full_native_plan() {
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        let source = Field::new("name", SqlType::Varchar)
            .length(Length::Size(20))
            .default("old");
        let target = Field::new("new", SqlType::Varchar)
            .length(Length::Size(100))
            .not_null()
            .default("");
        let alter = Alter::new("p")
            .source_field(Field::new("id", SqlType::Integer))
            .source_field(source)
            .rename_field("name", "new")
            .change_field(target)
            .add_field(Field::new("extra", SqlType::Integer))
            .remove_field("extra")
            .remove_primary_key()
            .primary_key(["id"])
            .remove_index("index")
            .add_index(Index::new("index", ["new"]).unique())
            .remove_foreign_key("fk")
            .add_foreign_key(ForeignKey::new("fk", ["id"], "parent", ["id"]));
        // Duplicate add/remove on the same field is intentionally rejected by normalization.
        assert!(alter.render(dialect).is_err());
        let alter = Alter::new("p")
            .source_field(Field::new("id", SqlType::Integer))
            .source_field(Field::new("name", SqlType::Text))
            .rename_field("name", "new")
            .change_field(Field::new("new", SqlType::Varchar).length(Length::Size(80)))
            .remove_primary_key()
            .primary_key(["id"])
            .remove_index("index")
            .add_index(Index::new("index", ["new"]).unique())
            .remove_foreign_key("fk")
            .add_foreign_key(ForeignKey::new("fk", ["id"], "parent", ["id"]));
        let plan = alter.render(dialect).unwrap();
        assert!(plan.queries().len() >= 8);
        assert!(
            plan.queries()
                .iter()
                .all(|query| query.dialect() == dialect)
        );
    }
}
/// Typed JSON paths bind quote/dot/index data, native JSON null and direct array membership.
#[test]
fn json_native_rendering_types_and_unsupported_nul() {
    let path = JsonPath::new([
        JsonSegment::Key("01".into()),
        JsonSegment::Index(2),
        JsonSegment::Key("a\"b".into()),
    ]);
    let query =
        Select::new(["id"])
            .from("p")
            .where_json("data", path.clone(), JsonScalar::Bool(false));
    let pg = render(&query, DialectId::Postgres);
    assert!(pg.sql().contains("-> $1::text"));
    assert!(pg.sql().contains("-> $2::integer"));
    assert_eq!(
        pg.binds(),
        &["01".into(), 2.into(), "a\"b".into(), "false".into()]
    );
    let mysql = render(&query, DialectId::Mysql);
    assert!(
        mysql
            .sql()
            .contains("JSON_EXTRACT(`data`, ?) = CAST(? AS JSON)")
    );
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        let query = Select::new([Selector::expression(Expr::json("data", path.clone()))])
            .from("p")
            .where_json_contains("data", JsonPath::default(), JsonScalar::Null);
        assert!(query.render(dialect).is_ok());
        assert!(
            Insert::new("p")
                .columns(["data"])
                .values([WriteValue::Json(JsonValue::Scalar(JsonScalar::Bool(true)))])
                .render(dialect)
                .is_ok()
        );
    }
    assert!(
        Select::default()
            .from("p")
            .where_eq("text", "a\0b")
            .render(DialectId::Postgres)
            .is_err()
    );
    assert!(
        Select::default()
            .from("p")
            .where_json(
                "data",
                JsonPath::default(),
                JsonScalar::String("a\0b".into())
            )
            .render(DialectId::Postgres)
            .is_err()
    );
    assert!(
        Select::default()
            .from("p")
            .where_eq("text", "a\0b")
            .render(DialectId::Mysql)
            .is_ok()
    );
}
/// Raw native SQL respects tokens and native parameters, never global quote/marker replacement.
#[test]
fn network_raw_lexical_gate_and_identifier_bounds() {
    for text in [
        "SELECT $1, '$2;?', $$';?$1$$",
        "SELECT $tag$;?$1$tag$",
        "SELECT $1::text /* $2; */; -- tail",
    ] {
        assert!(
            Sql::trusted(DialectId::Postgres, TrustedSql::audited(text))
                .bind("data")
                .render(DialectId::Postgres)
                .is_ok(),
            "{text}"
        );
    }
    for text in [
        "SELECT $0",
        "SELECT $1001",
        "SELECT $unfinished",
        "SELECT ?",
        "SELECT 1; SELECT 2",
        "SELECT E'\\'; SELECT 2",
    ] {
        assert!(
            Sql::trusted(DialectId::Postgres, TrustedSql::audited(text))
                .render(DialectId::Postgres)
                .is_err(),
            "{text}"
        );
    }
    for text in [
        "SELECT '?;`', ? # ignored ;\n",
        "SELECT `a``b` FROM `p` -- tail\n",
    ] {
        assert!(
            Sql::trusted(DialectId::Mysql, TrustedSql::audited(text))
                .render(DialectId::Mysql)
                .is_ok()
        );
    }
    for text in [
        "SELECT 1 /*!; DELETE FROM p*/",
        "SELECT '\\'; SELECT 2",
        "SELECT 1; SELECT 2",
    ] {
        assert!(
            Sql::trusted(DialectId::Mysql, TrustedSql::audited(text))
                .render(DialectId::Mysql)
                .is_err()
        );
    }
    assert!(
        Select::new(["x".repeat(64)])
            .from("p")
            .render(DialectId::Postgres)
            .is_err()
    );
    assert!(
        Select::new(["x".repeat(65)])
            .from("p")
            .render(DialectId::Mysql)
            .is_err()
    );
    assert_eq!(
        render(&Select::new(["a`b"]).from("literal.dot"), DialectId::Mysql).sql(),
        "SELECT `a``b` FROM `literal.dot`"
    );
}

/// Known native schema restrictions fail before even the first CREATE/ALTER statement.
#[test]
fn native_key_and_changed_attribute_validation() {
    assert!(
        Create::new("p")
            .add_field(Field::new("id", SqlType::Integer))
            .primary_key(["ID"])
            .render(DialectId::Postgres)
            .is_err()
    );
    for data_type in [SqlType::Text, SqlType::Blob, SqlType::Json] {
        assert_eq!(
            Create::new("p")
                .add_field(Field::new("data", data_type))
                .add_index(Index::new("data", ["data"]))
                .render(DialectId::Mysql)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
    }
    let mut field = Field::new("id", SqlType::Integer);
    field.unsigned = true;
    assert_eq!(
        Alter::new("p")
            .source_field(Field::new("id", SqlType::Integer))
            .change_field(field)
            .render(DialectId::Postgres)
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
}
