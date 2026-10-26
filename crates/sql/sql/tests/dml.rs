//! M1 core inventory: Select 1/3/4/5/10, Insert 1/6/7/8, Update/Delete 1/7,
//! SQLite CRUD/RETURNING/identifier/pagination rows and explicit Rust replacements.

use stackpress_sql::*;

/// Render one detached DML value without constructing any runtime or database.
fn sqlite(query: &impl Render) -> Query {
    query.render(DialectId::Sqlite).unwrap()
}

/// Explicit selectors, aliases, qualified operands and joins preserve order and binding.
#[test]
fn select_build_and_complete_sql() {
    let id = Identifier::qualified(["p", "id"]);
    let query = Select::new([
        Selector::from(id.clone()).alias("profile.id"),
        Selector::expression(Expr::value("bound selector")),
    ])
    .from(TableRef::new("profile").alias("p"))
    .join(Join {
        kind: JoinKind::Inner,
        table: TableRef::new("details").alias("d"),
        on: Some(Expr::column(id).compare(
            Compare::Eq,
            Expr::column(Identifier::qualified(["d", "id"])),
        )),
    })
    .where_eq("name", "O'Reilly")
    .and_where(Expr::any([Expr::eq("active", true), Expr::eq("id", 7_i64)]))
    .order("name")
    .order_by("id", Order::Desc)
    .limit(20)
    .offset(1)
    .distinct();
    let data = query.build();
    assert_eq!(data.selectors.len(), 2);
    assert_eq!(data.joins.len(), 1);
    assert_eq!(data.filters.len(), 2);
    assert_eq!(data.sort[0].order, Order::Asc);
    assert_eq!(data.limit, Some(20));
    let rendered = sqlite(&query);
    assert_eq!(
        rendered.sql(),
        "SELECT DISTINCT \"p\".\"id\" AS \"profile.id\", ? FROM \"profile\" AS \"p\" INNER JOIN \"details\" AS \"d\" ON \"p\".\"id\" = \"d\".\"id\" WHERE \"name\" = ? AND (\"active\" = ? OR \"id\" = ?) ORDER BY \"name\" ASC, \"id\" DESC LIMIT 20 OFFSET 1"
    );
    assert_eq!(
        rendered.binds(),
        &[
            "bound selector".into(),
            "O'Reilly".into(),
            true.into(),
            7_i64.into()
        ]
    );
}

/// Default wildcard, explicit wildcard qualification and lists have separate semantics.
#[test]
fn select_defaults_and_selector_boundaries() {
    assert_eq!(
        sqlite(&Select::default().from("p")).sql(),
        "SELECT * FROM \"p\""
    );
    let explicit = Select::new([Selector::wildcard(Some("p".into()))]).from("p");
    assert_eq!(sqlite(&explicit).sql(), "SELECT \"p\".* FROM \"p\"");
    assert_eq!(
        sqlite(&Select::new(["id,name", "count(*)"]).from("p")).sql(),
        "SELECT \"id,name\", \"count(*)\" FROM \"p\""
    );
    assert!(
        Select::new([Selector::wildcard(None).alias("x")])
            .from("p")
            .render(DialectId::Sqlite)
            .is_err()
    );
    assert!(
        Select::new(Vec::<String>::new())
            .from("p")
            .render(DialectId::Sqlite)
            .is_err()
    );
    assert!(Select::default().render(DialectId::Sqlite).is_err());
}

/// Every join kind has deliberate ON semantics; unsupported clause combinations error.
#[test]
fn joins_and_operand_validation() {
    for (kind, keyword) in [
        (JoinKind::Inner, "INNER"),
        (JoinKind::Left, "LEFT"),
        (JoinKind::Right, "RIGHT"),
        (JoinKind::Full, "FULL"),
        (JoinKind::Cross, "CROSS"),
    ] {
        let on = if kind == JoinKind::Cross {
            None
        } else {
            Some(Expr::eq("id", 1))
        };
        let query = Select::default().from("p").join(Join {
            kind,
            table: "q".into(),
            on,
        });
        assert!(
            sqlite(&query)
                .sql()
                .contains(&format!("{keyword} JOIN \"q\""))
        );
    }
    for (kind, on) in [
        (JoinKind::Inner, None),
        (JoinKind::Cross, Some(Expr::eq("id", 1))),
    ] {
        assert!(
            Select::default()
                .from("p")
                .join(Join {
                    kind,
                    table: "q".into(),
                    on
                })
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    let many = (0..65).fold(Select::default().from("p"), |query, _| {
        query.join(Join {
            kind: JoinKind::Cross,
            table: "q".into(),
            on: None,
        })
    });
    assert!(many.render(DialectId::Sqlite).is_err());
}

/// Zero/absent pagination differ, offset-only SQL is valid and signed bounds are enforced.
#[test]
fn select_sort_and_pagination() {
    assert_eq!(
        sqlite(&Select::default().from("p").limit(0)).sql(),
        "SELECT * FROM \"p\" LIMIT 0"
    );
    assert_eq!(
        sqlite(&Select::default().from("p").offset(0)).sql(),
        "SELECT * FROM \"p\" LIMIT -1 OFFSET 0"
    );
    assert!(
        sqlite(&Select::default().from("p").limit(i64::MAX as u64))
            .sql()
            .ends_with(&i64::MAX.to_string())
    );
    for query in [
        Select::default().from("p").limit(u64::MAX),
        Select::default().from("p").offset(u64::MAX),
    ] {
        assert!(query.render(DialectId::Sqlite).is_err());
    }
    let sorted = Select::default().from("p").sort(Sort {
        expression: Expr::value(1),
        order: Order::default(),
    });
    assert_eq!(sqlite(&sorted).sql(), "SELECT * FROM \"p\" ORDER BY ? ASC");
}

/// Bulk inserts retain explicit stable columns and row-major values, including NULL.
#[test]
fn insert_bulk_returning_and_inspection() {
    let query = Insert::new("p")
        .columns(["id", "name"])
        .values([WriteValue::from(1), "Ada".into()])
        .values([SqlValue::Integer(2), SqlValue::Null])
        .returning(["id", "name"]);
    assert_eq!(query.build().rows.len(), 2);
    assert_eq!(query.build().columns, ["id", "name"]);
    let rendered = sqlite(&query);
    assert_eq!(
        rendered.sql(),
        "INSERT INTO \"p\" (\"id\", \"name\") VALUES (?, ?), (?, ?) RETURNING \"id\", \"name\""
    );
    assert_eq!(
        rendered.binds(),
        &[1.into(), "Ada".into(), 2.into(), SqlValue::Null]
    );
    assert!(
        sqlite(&query.clone().returning_all())
            .sql()
            .ends_with(" RETURNING *")
    );
    assert!(
        sqlite(
            &Insert::new("p")
                .columns(["id"])
                .values([1])
                .returning(["id"])
        )
        .sql()
        .ends_with("RETURNING \"id\"")
    );
}

/// Missing columns use explicit defaults; NULL never silently requests a default.
#[test]
fn insert_defaults_are_explicit_and_never_inferred() {
    assert_eq!(
        sqlite(&Insert::new("p").default_values()).sql(),
        "INSERT INTO \"p\" DEFAULT VALUES"
    );
    let omitted = Insert::new("p")
        .columns(["id", "name"])
        .values([WriteValue::Default, "Ada".into()]);
    assert_eq!(
        sqlite(&omitted).sql(),
        "INSERT INTO \"p\" (\"name\") VALUES (?)"
    );
    assert_eq!(
        sqlite(
            &Insert::new("p")
                .columns(["id"])
                .values([WriteValue::Default])
        )
        .sql(),
        "INSERT INTO \"p\" DEFAULT VALUES"
    );
    let mixed = Insert::new("p")
        .columns(["id"])
        .values([WriteValue::Default])
        .values([WriteValue::from(1)]);
    assert_eq!(
        mixed.render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    let bulk_default = Insert::new("p")
        .columns(["id"])
        .values([WriteValue::Default])
        .values([WriteValue::Default]);
    assert!(bulk_default.render(DialectId::Sqlite).is_err());
    assert_eq!(
        sqlite(&Insert::new("p").columns(["id"]).values([SqlValue::Null])).binds(),
        &[SqlValue::Null]
    );
}

/// Invalid insert shapes cannot manufacture successful empty SQL.
#[test]
fn insert_rejects_empty_arity_duplicate_and_conflicting_input() {
    for query in [
        Insert::new("p"),
        Insert::new(""),
        Insert::new("p").columns(["id"]),
        Insert::new("p").columns(["id"]).values([1, 2]),
        Insert::new("p").columns(["id", "ID"]).values([1, 2]),
        Insert::new("p").columns([""]).values([1]),
        Insert::new("p").columns(["id"]).default_values(),
        Insert::new("p").values([1]).default_values(),
    ] {
        assert!(query.render(DialectId::Sqlite).is_err(), "{query:?}");
    }
    assert!(
        Insert::new("p")
            .columns(["id"])
            .values([WriteValue::Expression(Expr::value(3))])
            .render(DialectId::Sqlite)
            .is_ok()
    );
}

/// Nonempty assignments bind before filter and returning expressions; zero is retained.
#[test]
fn update_and_delete_build_and_returning() {
    let update = Update::new("p")
        .set("name", "Ada")
        .set("count", 0)
        .where_eq("id", 7)
        .returning(["id"]);
    assert_eq!(update.assignments().len(), 2);
    assert_eq!(update.build().filters.len(), 1);
    let rendered = sqlite(&update);
    assert_eq!(
        rendered.sql(),
        "UPDATE \"p\" SET \"name\" = ?, \"count\" = ? WHERE \"id\" = ? RETURNING \"id\""
    );
    assert_eq!(rendered.binds(), &["Ada".into(), 0.into(), 7.into()]);
    assert!(
        sqlite(&update.returning_all())
            .sql()
            .ends_with("RETURNING *")
    );
    let delete = Delete::new("p").where_eq("id", 7).returning(["id"]);
    assert!(!delete.build().all_rows);
    assert_eq!(
        sqlite(&delete).sql(),
        "DELETE FROM \"p\" WHERE \"id\" = ? RETURNING \"id\""
    );
    assert!(
        sqlite(&delete.returning_all())
            .sql()
            .ends_with("RETURNING *")
    );
}

/// Unfiltered mutation requires all_rows; invalid groups remain invalid after that choice.
#[test]
fn mutation_scope_never_silently_widens() {
    assert!(
        Update::new("p")
            .set("x", 1)
            .render(DialectId::Sqlite)
            .is_err()
    );
    assert!(Delete::new("p").render(DialectId::Sqlite).is_err());
    assert_eq!(
        sqlite(&Delete::new("p").all_rows()).sql(),
        "DELETE FROM \"p\""
    );
    assert_eq!(
        sqlite(&Update::new("p").set("x", 1).all_rows()).sql(),
        "UPDATE \"p\" SET \"x\" = ?"
    );
    assert!(
        Update::new("p")
            .all_rows()
            .render(DialectId::Sqlite)
            .is_err()
    );
    assert!(
        Update::new("p")
            .set("x", 1)
            .set("X", 2)
            .all_rows()
            .render(DialectId::Sqlite)
            .is_err()
    );
    assert!(
        Update::new("p")
            .set("x", WriteValue::Default)
            .all_rows()
            .render(DialectId::Sqlite)
            .is_err()
    );
    for expression in [Expr::all([]), Expr::any([])] {
        assert!(
            Delete::new("p")
                .all_rows()
                .and_where(expression.clone())
                .render(DialectId::Sqlite)
                .is_err()
        );
        assert!(
            Update::new("p")
                .set("x", 1)
                .all_rows()
                .and_where(expression)
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
}

/// Operator enums, null tests, IN and LIKE preserve semantics and exact bind order.
#[test]
fn expressions_cover_comparisons_groups_null_in_and_like() {
    for (operator, expected) in [
        (Compare::Eq, "="),
        (Compare::Ne, "<>"),
        (Compare::Lt, "<"),
        (Compare::Le, "<="),
        (Compare::Gt, ">"),
        (Compare::Ge, ">="),
        (Compare::Like, "LIKE"),
    ] {
        let query = Select::default()
            .from("p")
            .and_where(Expr::column("x").compare(operator, Expr::value(1)));
        assert!(
            sqlite(&query)
                .sql()
                .ends_with(&format!("\"x\" {expected} ?"))
        );
    }
    assert_eq!(
        sqlite(&Select::default().from("p").where_eq("x", SqlValue::Null)).sql(),
        "SELECT * FROM \"p\" WHERE \"x\" IS NULL"
    );
    let expression = Expr::all([
        Expr::column("x").compare(Compare::Ne, Expr::value(SqlValue::Null)),
        Expr::Not(Box::new(Expr::eq("a", false))),
        Expr::IsNull(Box::new(Expr::column("b")), false),
        Expr::IsNull(Box::new(Expr::column("c")), true),
        Expr::In(Box::new(Expr::column("n")), vec![1.into(), 2.into()], false),
        Expr::In(Box::new(Expr::column("m")), vec![3.into()], true),
        Expr::Like(Box::new(Expr::column("name")), "a\\_%".into(), '\\'),
    ]);
    let rendered = sqlite(&Select::default().from("p").and_where(expression));
    assert!(
        rendered
            .sql()
            .contains("\"x\" IS NOT NULL AND NOT (\"a\" = ?)")
    );
    assert!(
        rendered
            .sql()
            .contains("\"n\" IN (?, ?) AND \"m\" NOT IN (?)")
    );
    assert_eq!(
        rendered.binds(),
        &[
            false.into(),
            1.into(),
            2.into(),
            3.into(),
            "a\\_%".into(),
            "\\".into()
        ]
    );
    for invalid in [
        Expr::In(Box::new(Expr::column("x")), vec![], false),
        Expr::Like(Box::new(Expr::column("x")), "x".into(), '\0'),
    ] {
        assert!(
            Select::default()
                .from("p")
                .and_where(invalid)
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
}

/// Nested comparison operands preserve the tree rather than native left-associativity.
#[test]
fn nested_comparison_tree_preserves_parentheses() {
    let right = Expr::column("a").compare(Compare::Eq, Expr::eq("b", 1));
    let query = sqlite(&Select::default().from("p").and_where(right));
    assert_eq!(query.sql(), "SELECT * FROM \"p\" WHERE \"a\" = (\"b\" = ?)");
    let left = Expr::eq("a", 1).compare(Compare::Eq, Expr::value(false));
    assert_eq!(
        sqlite(&Select::default().from("p").and_where(left)).sql(),
        "SELECT * FROM \"p\" WHERE (\"a\" = ?) = ?"
    );
}

/// Audited native fragments remain dialect-specific and reject empty/batched escapes.
#[test]
fn explicit_trusted_expression_boundary() {
    let valid = Expr::Trusted(
        DialectId::Sqlite,
        TrustedSql::audited("length(?)"),
        vec!["abc".into()],
    );
    assert_eq!(
        sqlite(&Select::new([Selector::expression(valid).alias("n")]).from("p")).sql(),
        "SELECT (length(?)) AS \"n\" FROM \"p\""
    );
    for text in ["", "); DROP TABLE p; --", "1\0", "'unfinished"] {
        let expression = Expr::Trusted(DialectId::Sqlite, TrustedSql::audited(text), vec![]);
        assert!(
            Select::default()
                .from("p")
                .and_where(expression)
                .render(DialectId::Sqlite)
                .is_err(),
            "{text:?}"
        );
    }
    let wrong = Expr::Trusted(DialectId::Postgres, TrustedSql::audited("1"), vec![]);
    assert!(
        Select::default()
            .from("p")
            .and_where(wrong)
            .render(DialectId::Sqlite)
            .is_err()
    );
}

/// All identifiers use the same quoting policy, including aliases, qualified parts and DML.
#[test]
fn hostile_identifiers_and_values_stay_data() {
    let malicious = "a\"; DROP TABLE p;--";
    let value = "' ; SELECT 1; -- \0 \\ Unicode 💡";
    let rendered = sqlite(
        &Update::new(malicious)
            .set(malicious, value)
            .where_eq(malicious, value),
    );
    assert_eq!(rendered.binds(), &[value.into(), value.into()]);
    assert!(rendered.sql().contains("\"a\"\"; DROP TABLE p;--\""));
    assert_eq!(
        Identifier::qualified(["schema", "literal.dot"]).segments(),
        &["schema", "literal.dot"]
    );
    for name in ["", "a\0b", "a\nb", "a\u{7f}b"] {
        assert!(
            Select::new([name])
                .from("p")
                .render(DialectId::Sqlite)
                .is_err()
        );
        assert!(
            Select::default()
                .from(TableRef::new("p").alias(name))
                .render(DialectId::Sqlite)
                .is_err()
        );
        assert!(
            Delete::new(name)
                .all_rows()
                .render(DialectId::Sqlite)
                .is_err()
        );
        assert!(
            Insert::new("p")
                .columns([name])
                .values([1])
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    assert!(
        Select::new(["x".repeat(1025)])
            .from("p")
            .render(DialectId::Sqlite)
            .is_err()
    );
    for parts in [vec![], vec!["x"; 65]] {
        assert!(
            Select::new([Identifier::qualified(parts)])
                .from("p")
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
}

/// Clones remain independent; every DML family rejects unimplemented network rendering.
#[test]
fn detached_data_and_dialect_selection() {
    let base = Select::default().from("p");
    assert!(base.clone().where_eq("id", 1).build().filters.len() == 1);
    assert!(base.build().filters.is_empty());
    let replacement = base
        .clone()
        .where_eq("id", 1)
        .and_where(Expr::eq("active", true))
        .where_eq("id", 2);
    assert_eq!(sqlite(&replacement).binds(), &[2.into(), true.into()]);
    let insert = Insert::new("p").columns(["x"]).values([1]);
    let update = Update::new("p").set("x", 1).all_rows();
    let delete = Delete::new("p").all_rows();
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        for query in [&base as &dyn Render, &insert, &update, &delete] {
            assert_eq!(
                query.render(dialect).unwrap_err().kind(),
                ErrorKind::Unsupported
            );
        }
    }
    assert!(sqlite(&insert.clone().values([2])).binds().len() == 2);
    assert_eq!(insert.build().rows.len(), 1);
    assert_eq!(update.clone().set("y", 2).assignments().len(), 2);
    assert_eq!(update.assignments().len(), 1);
}

/// Input resource budgets catch nested trees, excess binds and excessively large SQL/data.
#[test]
fn dml_resource_limits() {
    let mut expression = Expr::eq("x", 1);
    for _ in 0..66 {
        expression = Expr::Not(Box::new(expression));
    }
    assert_eq!(
        Select::default()
            .from("p")
            .and_where(expression)
            .render(DialectId::Sqlite)
            .unwrap_err()
            .kind(),
        ErrorKind::Limit
    );
    let nodes = Expr::all((0..10_001).map(|_| Expr::column("x")));
    assert_eq!(
        Select::default()
            .from("p")
            .and_where(nodes)
            .render(DialectId::Sqlite)
            .unwrap_err()
            .kind(),
        ErrorKind::Limit
    );
    assert!(
        Select::new(vec!["x"; 1001])
            .from("p")
            .render(DialectId::Sqlite)
            .is_err()
    );
    let too_many = Insert::new("p").columns(["x", "y"]).values([1, 2]);
    let too_many = (0..500).fold(too_many, |query, _| query.values([1, 2]));
    assert_eq!(
        too_many.render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Limit
    );
    let oversized = Select::default()
        .from("p")
        .where_eq("x", "x".repeat(16 * 1024 * 1024 + 1));
    assert_eq!(
        oversized.render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Limit
    );
    let exact_identifier = "x".repeat(1024);
    let large_sql = Select::new(vec![exact_identifier; 1000]).from("p");
    assert!(large_sql.render(DialectId::Sqlite).is_ok());
}
