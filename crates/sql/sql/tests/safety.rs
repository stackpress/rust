//! Dedicated public-boundary and auto-trait proofs; native effects are tested separately.

use stackpress_sql::*;

/// Every owned shareable description/result is Send+Sync; exclusive guards/futures are Send.
#[test]
fn public_auto_traits_match_ownership_contracts() {
    fn share<T: Send + Sync>() {}
    fn transfer<T: Send>() {}
    macro_rules! shared {
        ($($kind:ty),+ $(,)?) => { $(share::<$kind>();)+ };
    }
    shared!(
        Alter,
        AlterData,
        AlterOperation,
        ColumnMetadata,
        ColumnMetadataRef<'static>,
        Compare,
        Create,
        CreateData,
        DefaultValue,
        Delete,
        DialectCapabilities,
        DialectId,
        Engine,
        ErrorKind,
        Expr,
        Field,
        ForeignKey,
        Identifier,
        Index,
        Insert,
        InsertData,
        Join,
        JoinKind,
        JsonPath,
        JsonPathOptions,
        JsonScalar,
        JsonSegment,
        JsonValue,
        Length,
        MutationData,
        Order,
        Query,
        QueryLimits,
        QueryPlan,
        QueryResult,
        ReferentialAction,
        Row,
        SchemaDiff,
        Select,
        SelectData,
        Selector,
        Sort,
        Sql,
        SqlError,
        SqlType,
        SqlValue,
        TableCommand,
        TableDefinition,
        TableRef,
        TransactionOptions,
        TrustedSql,
        Update,
        WriteValue,
        Box<dyn Database>,
        Result<QueryResult>,
    );
    transfer::<Transaction>();
    transfer::<Box<dyn TransactionSession>>();
    transfer::<SqlFuture<'static, ()>>();
}

/// A valid maximum must remain usable while the next expression/JSON level or node fails.
#[test]
fn expression_and_json_exact_depth_and_node_limits() {
    let mut expression = Expr::column("active");
    let mut document = JsonValue::Scalar(JsonScalar::Null);
    for _ in 0..64 {
        expression = Expr::Not(Box::new(expression));
        document = JsonValue::Array(vec![document]);
    }
    let query = Select::default().from("p").and_where(expression.clone());
    assert!(query.render(DialectId::Sqlite).is_ok());
    assert!(document.to_json().is_ok());
    assert_eq!(
        Select::default()
            .from("p")
            .and_where(Expr::Not(Box::new(expression)))
            .render(DialectId::Sqlite)
            .unwrap_err()
            .kind(),
        ErrorKind::Limit
    );
    assert_eq!(
        JsonValue::Array(vec![document])
            .to_json()
            .unwrap_err()
            .kind(),
        ErrorKind::Limit
    );
    for (children, valid) in [(9_999, true), (10_000, false)] {
        let expression = Expr::all((0..children).map(|_| Expr::column("active")));
        assert_eq!(
            Select::default()
                .from("p")
                .and_where(expression)
                .render(DialectId::Sqlite)
                .is_ok(),
            valid
        );
        let document = JsonValue::Array(vec![JsonValue::Scalar(JsonScalar::Null); children]);
        assert_eq!(document.to_json().is_ok(), valid);
    }
}

/// UTF-8 input, invalid controls, paths and extreme ordinals return errors without unwinding.
#[test]
fn hostile_utf8_boundaries_do_not_panic() {
    for text in [
        "",
        "\0",
        "'é",
        "SELECT '💡",
        "SELECT $é$",
        "SELECT /* é",
        "SELECT [é",
    ] {
        let outcome = std::panic::catch_unwind(|| {
            for dialect in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
                let _ = Sql::trusted(dialect, TrustedSql::audited(text)).render(dialect);
            }
            let _ = SqlValue::utc_text(text);
            let _ = JsonPath::parse(text, JsonPathOptions::default());
        });
        assert!(outcome.is_ok(), "input {text:?}");
    }
    let row = Row::new(vec![("id".into(), 7.into())]);
    assert_eq!(
        row.try_get_at::<i64>(usize::MAX).unwrap_err().kind(),
        ErrorKind::Decode
    );
    for name in ["", "\0", "a\n", "a\u{7f}"] {
        assert!(
            Select::new([name])
                .from("p")
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
}

/// Statement bytes and structured JSON bytes count delimiters, not only payload characters.
#[test]
fn exact_sql_and_json_byte_budgets() {
    let limit = QueryLimits::default().sql_bytes;
    for (bytes, valid) in [(limit, true), (limit + 1, false)] {
        let text = format!("SELECT 1{}", " ".repeat(bytes - "SELECT 1".len()));
        assert_eq!(
            Sql::trusted(DialectId::Sqlite, TrustedSql::audited(text))
                .render(DialectId::Sqlite)
                .is_ok(),
            valid
        );
        let document = JsonValue::Scalar(JsonScalar::String("a".repeat(bytes - 2)));
        assert_eq!(document.to_json().is_ok(), valid);
    }
}

/// Large valid fragments must hit the SQL budget before expanding later invalid clauses.
#[test]
fn sql_budget_stops_later_clause_expansion() {
    let fragment = Expr::Trusted(
        DialectId::Sqlite,
        TrustedSql::audited(format!("'{}'", "a".repeat(600_000))),
        vec![],
    );
    let values = [
        WriteValue::Expression(fragment.clone()),
        WriteValue::Expression(fragment),
        WriteValue::Value(SqlValue::Real(f64::NAN)),
    ];
    let insert = Insert::new("p")
        .columns(["a", "b", "c"])
        .values(values.clone());
    let update = Update::new("p")
        .set("a", values[0].clone())
        .set("b", values[1].clone())
        .set("c", values[2].clone())
        .all_rows();
    for error in [
        insert.render(DialectId::Sqlite).unwrap_err(),
        update.render(DialectId::Sqlite).unwrap_err(),
    ] {
        assert_eq!(error.kind(), ErrorKind::Limit);
    }
    let create = Create::new("p")
        .add_field(Field::new("a", SqlType::Text).default("a".repeat(600_000)))
        .add_field(Field::new("b", SqlType::Text).default("b".repeat(600_000)))
        .add_field(Field::new(
            "c",
            SqlType::Trusted(DialectId::Mysql, TrustedSql::audited("BIGINT")),
        ));
    assert_eq!(
        create.render(DialectId::Postgres).unwrap_err().kind(),
        ErrorKind::Limit
    );
}

/// Dialect rejection precedes invalid raw bind validation, even after moving validation before copy.
#[test]
fn raw_dialect_error_precedes_nonfinite_bind_error() {
    let raw = Sql::literal(DialectId::Postgres, "SELECT $1").bind(f64::NAN);
    assert_eq!(
        raw.render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    assert_eq!(
        raw.render(DialectId::Postgres).unwrap_err().kind(),
        ErrorKind::InvalidQuery
    );
}

/// ALTER must apply the same explicit primary-key/SET NULL rule as CREATE, in either order.
#[test]
fn alter_primary_rejects_set_null_relationships() {
    let mut foreign = ForeignKey::new("parent_fk", ["id"], "parent", ["id"]);
    foreign.on_delete = ReferentialAction::SetNull;
    for operations in [
        [
            AlterOperation::AddPrimary(vec!["id".into()]),
            AlterOperation::AddForeignKey(foreign.clone()),
        ],
        [
            AlterOperation::AddForeignKey(foreign.clone()),
            AlterOperation::AddPrimary(vec!["id".into()]),
        ],
    ] {
        let alter = Alter::new("p")
            .source_field(Field::new("id", SqlType::Integer))
            .operation(operations[0].clone())
            .operation(operations[1].clone());
        assert_eq!(alter.build().unwrap_err().kind(), ErrorKind::InvalidQuery);
    }
    assert!(
        Alter::new("p")
            .source_field(Field::new("id", SqlType::Integer))
            .add_foreign_key(foreign.clone())
            .build()
            .is_ok()
    );
}

/// Removing a primary leaves native NOT NULL in place until an explicit column change.
#[test]
fn primary_removal_requires_explicit_nullable_change_for_set_null() {
    let mut foreign = ForeignKey::new("parent_fk", ["id"], "parent", ["id"]);
    foreign.on_delete = ReferentialAction::SetNull;
    let alter = Alter::new("p")
        .source_field(Field::new("id", SqlType::Integer))
        .primary_key(["id"])
        .remove_primary_key();
    assert!(
        alter
            .clone()
            .add_foreign_key(foreign.clone())
            .build()
            .is_err()
    );
    // A later explicit nullability change restores a valid SET NULL target.
    let restored = alter
        .change_field(Field::new("id", SqlType::Integer))
        .add_foreign_key(foreign);
    assert!(restored.build().is_ok());
    assert!(restored.render(DialectId::Postgres).is_ok());
    assert!(restored.render(DialectId::Mysql).is_ok());
}

/// Finite floating storage preserves sign/subnormals; unsigned and narrowing conversions reject loss.
#[test]
fn numeric_edges_preserve_exact_owned_values() {
    for number in [-0.0, 0.0, f64::from_bits(1), f64::MIN, f64::MAX] {
        let value = SqlValue::Real(number);
        assert_eq!(f64::from_sql(&value).unwrap().to_bits(), number.to_bits());
        assert!(
            Sql::literal(DialectId::Sqlite, "SELECT ?")
                .bind(value)
                .render(DialectId::Sqlite)
                .is_ok()
        );
    }
    for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(f64::from_sql(&SqlValue::Real(number)).is_err());
        assert!(
            Sql::literal(DialectId::Sqlite, "SELECT ?")
                .bind(number)
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    assert_eq!(
        SqlValue::try_from(i64::MAX as u64).unwrap(),
        SqlValue::Integer(i64::MAX)
    );
    assert!(SqlValue::try_from(i64::MAX as u64 + 1).is_err());
    assert_eq!(
        i8::from_sql(&SqlValue::Integer(i8::MIN.into())).unwrap(),
        i8::MIN
    );
    assert!(i8::from_sql(&SqlValue::Integer(i64::from(i8::MAX) + 1)).is_err());
    assert!(u64::from_sql(&SqlValue::Integer(-1)).is_err());
}

/// Builder-generated plans and bind lists accept their documented maximum and reject the next item.
#[test]
fn plan_and_bind_cardinality_boundaries() {
    let mut create = Create::new("p");
    for index in 0..1000 {
        create = create.add_field(Field::new(format!("field_{index}"), SqlType::Integer));
    }
    let plan = create.render(DialectId::Sqlite).unwrap();
    plan.validate(DialectId::Sqlite, QueryLimits::default())
        .unwrap();
    assert!(
        create
            .add_field(Field::new("too_many", SqlType::Integer))
            .build()
            .is_err()
    );
    let mut raw = Sql::literal(DialectId::Sqlite, "SELECT ?");
    for _ in 0..QueryLimits::default().binds {
        raw = raw.bind(0);
    }
    // Core checks budgets here; native prepared-parameter arity belongs to the adapter.
    assert_eq!(raw.render(DialectId::Sqlite).unwrap().binds().len(), 1000);
    assert_eq!(
        raw.bind(0).render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Limit
    );
}

/// Audited SQL does not bypass native text representation checks on its ordinary bound data.
#[test]
fn raw_postgres_text_nul_is_rejected_during_rendering() {
    let pg = Sql::literal(DialectId::Postgres, "SELECT $1").bind("a\0b");
    assert_eq!(
        pg.render(DialectId::Postgres).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    for dialect in [DialectId::Sqlite, DialectId::Mysql] {
        let query = Sql::literal(dialect, "SELECT ?")
            .bind("a\0b")
            .render(dialect)
            .unwrap();
        assert_eq!(query.binds(), &[SqlValue::Text("a\0b".into())]);
    }
}
