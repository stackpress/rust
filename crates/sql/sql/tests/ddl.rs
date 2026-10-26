//! M1 Create/Alter inspection, defaults and native SQLite rendering. Schema diff/network
//! dialect validation and real transactional DDL execution remain at the M2 gate.

use stackpress_sql::*;

/// Define a small table with stable fields for key/constraint tests.
fn table() -> Create {
    Create::new("p")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("name", SqlType::Text))
}

/// Create data, typed defaults, constraints and indexes render in deterministic order.
#[test]
fn create_full_definition_and_ordered_plan() {
    let mut foreign = ForeignKey::new("parent_fk", ["parent"], "parent.table", ["id"]);
    foreign.on_delete = ReferentialAction::SetNull;
    foreign.on_update = ReferentialAction::Cascade;
    let create = table()
        .add_field(Field::new("parent", SqlType::Integer))
        .primary_key(["id"])
        .add_index(Index::new("name_index", ["name"]))
        .add_index(Index::new("parent_index", ["parent"]).unique())
        .add_foreign_key(foreign)
        .if_not_exists();
    let built = create.build().unwrap();
    assert_eq!(built.fields.len(), 3);
    assert_eq!(built.primary, ["id"]);
    assert_eq!(built.foreign_keys.len(), 1);
    assert!(built.if_not_exists);
    let plan = create.render(DialectId::Sqlite).unwrap();
    assert_eq!(plan.queries().len(), 3);
    assert_eq!(
        plan.queries()[0].sql(),
        "CREATE TABLE IF NOT EXISTS \"p\" (\"id\" INTEGER NOT NULL, \"name\" TEXT, \"parent\" INTEGER, PRIMARY KEY (\"id\"), CONSTRAINT \"parent_fk\" FOREIGN KEY (\"parent\") REFERENCES \"parent.table\" (\"id\") ON DELETE SET NULL ON UPDATE CASCADE)"
    );
    assert_eq!(
        plan.queries()[1].sql(),
        "CREATE INDEX IF NOT EXISTS \"name_index\" ON \"p\" (\"name\")"
    );
    assert_eq!(
        plan.queries()[2].sql(),
        "CREATE UNIQUE INDEX IF NOT EXISTS \"parent_index\" ON \"p\" (\"parent\")"
    );
    assert!(plan.queries().iter().all(|query| query.binds().is_empty()));
    assert_eq!(
        plan.validate(DialectId::Mysql, QueryLimits::default())
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
    assert_eq!(
        plan.validate(
            DialectId::Sqlite,
            QueryLimits {
                sql_bytes: 5,
                ..QueryLimits::default()
            }
        )
        .unwrap_err()
        .kind(),
        ErrorKind::Limit
    );
}

/// Both insertion orders preserve unique strength; incompatible fields error explicitly.
#[test]
fn create_and_alter_unique_precedence_and_conflicts() {
    let regular = Index::new("identity", ["name"]);
    let unique = regular.clone().unique();
    for indexes in [
        [regular.clone(), unique.clone()],
        [unique.clone(), regular.clone()],
    ] {
        let create = table()
            .add_index(indexes[0].clone())
            .add_index(indexes[1].clone());
        assert_eq!(
            create.build().unwrap().indexes.as_slice(),
            std::slice::from_ref(&unique)
        );
        assert_eq!(create.render(DialectId::Sqlite).unwrap().queries().len(), 2);
        let alter = Alter::new("p")
            .add_index(indexes[0].clone())
            .add_index(indexes[1].clone());
        assert_eq!(
            alter.build().unwrap().operations,
            [AlterOperation::AddIndex(unique.clone())]
        );
        assert!(
            alter.render(DialectId::Sqlite).unwrap().queries()[0]
                .sql()
                .starts_with("CREATE UNIQUE INDEX \"identity\"")
        );
    }
    let conflicting = Index::new("identity", ["id"]).unique();
    assert!(
        table()
            .add_index(regular.clone())
            .add_index(conflicting.clone())
            .build()
            .is_err()
    );
    assert!(
        Alter::new("p")
            .add_index(regular)
            .add_index(conflicting)
            .build()
            .is_err()
    );
}

/// NULL/no default, false, zero, empty and quoted strings are all distinct SQL defaults.
#[test]
fn typed_defaults_preserve_false_zero_empty_and_quotes() {
    let create = Create::new("defaults")
        .add_field(Field::new("missing", SqlType::Text))
        .add_field(Field::new("null", SqlType::Text).default(SqlValue::Null))
        .add_field(Field::new("zero", SqlType::Integer).default(0))
        .add_field(Field::new("false", SqlType::Boolean).default(false))
        .add_field(Field::new("empty", SqlType::Text).default(""))
        .add_field(Field::new("quote", SqlType::Text).default("x'); DROP TABLE p; --"))
        .add_field(Field::new("real", SqlType::Real).default(1.5))
        .add_field(Field::new("blob", SqlType::Blob).default(vec![0_u8, 255]))
        .add_field(
            Field::new("created", SqlType::DateTime).default(DefaultValue::CurrentTimestamp),
        );
    let plan = create.render(DialectId::Sqlite).unwrap();
    let sql = plan.queries()[0].sql();
    for fragment in [
        "\"missing\" TEXT,",
        "\"null\" TEXT DEFAULT NULL",
        "\"zero\" INTEGER DEFAULT 0",
        "\"false\" BOOLEAN DEFAULT 0",
        "\"empty\" TEXT DEFAULT ''",
        "\"quote\" TEXT DEFAULT 'x''); DROP TABLE p; --'",
        "\"real\" REAL DEFAULT 1.5",
        "\"blob\" BLOB DEFAULT X'00FF'",
        "\"created\" TEXT DEFAULT CURRENT_TIMESTAMP",
    ] {
        assert!(sql.contains(fragment), "{fragment}: {sql}");
    }
    let named_function =
        Create::new("p").add_field(Field::new("name", SqlType::Text).default("evil()"));
    assert!(
        named_function.render(DialectId::Sqlite).unwrap().queries()[0]
            .sql()
            .contains("DEFAULT 'evil()'")
    );
}

/// Known storage types, positive declared lengths and strict auto-increment constraints.
#[test]
fn fields_validate_types_lengths_and_auto_increment() {
    let create = Create::new("p")
        .add_field(
            Field::new("id", SqlType::Integer)
                .not_null()
                .auto_increment(),
        )
        .add_field(Field::new("name", SqlType::Varchar).length(Length::Size(100)))
        .add_field(Field::new("other", SqlType::Varchar))
        .add_field(Field::new("json", SqlType::Json))
        .add_field(
            Field::new("date", SqlType::DateTime)
                .default(SqlValue::utc_text("2026-10-26T00:00:00Z").unwrap()),
        )
        .primary_key(["id"]);
    let plan = create.render(DialectId::Sqlite).unwrap();
    let sql = plan.queries()[0].sql();
    assert!(sql.contains("\"id\" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL"));
    assert_eq!(sql.matches("PRIMARY KEY").count(), 1);
    assert!(sql.contains("\"name\" VARCHAR(100)"));
    assert!(sql.contains("\"other\" VARCHAR"));
    for field in [
        Field::new("x", SqlType::Varchar).length(Length::Size(0)),
        Field::new("x", SqlType::Varchar).length(Length::Size(-1)),
        Field::new("x", SqlType::Integer).length(Length::Size(11)),
        Field::new("x", SqlType::Decimal).length(Length::Precision {
            precision: 0,
            scale: 0,
        }),
        Field::new("x", SqlType::Decimal).length(Length::Precision {
            precision: 2,
            scale: 3,
        }),
        Field::new("x", SqlType::Decimal).length(Length::Precision {
            precision: 2,
            scale: -1,
        }),
    ] {
        assert!(Create::new("p").add_field(field).build().is_err());
    }
    let decimal =
        Create::new("p").add_field(Field::new("x", SqlType::Decimal).length(Length::Precision {
            precision: 10,
            scale: 2,
        }));
    assert!(decimal.build().is_ok());
    assert_eq!(
        decimal.render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    for create in [
        Create::new("p").add_field(Field::new("id", SqlType::Integer).auto_increment()),
        Create::new("p")
            .add_field(Field::new("id", SqlType::Text).auto_increment())
            .primary_key(["id"]),
        Create::new("p")
            .add_field(
                Field::new("id", SqlType::Integer)
                    .auto_increment()
                    .default(0),
            )
            .primary_key(["id"]),
        table()
            .add_field(Field::new("other", SqlType::Integer).auto_increment())
            .primary_key(["id", "other"]),
    ] {
        assert!(create.render(DialectId::Sqlite).is_err());
    }
}

/// Invalid type/default/attribute inputs fail rather than silently losing requested intent.
#[test]
fn default_and_attribute_errors_are_typed() {
    for field in [
        Field::new("x", SqlType::Text)
            .not_null()
            .default(SqlValue::Null),
        Field::new("x", SqlType::Integer).default("12"),
        Field::new("x", SqlType::Boolean).default(1),
        Field::new("x", SqlType::Text).default(1),
        Field::new("x", SqlType::DateTime).default("invalid-date"),
        Field::new("x", SqlType::Integer).default(DefaultValue::CurrentTimestamp),
        Field::new("x", SqlType::Real).default(f64::NAN),
        Field::new("x", SqlType::Text).default("a\0b"),
    ] {
        assert!(
            Create::new("p")
                .add_field(field)
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    let mut unsigned = Field::new("x", SqlType::Integer);
    unsigned.unsigned = true;
    let mut commented = Field::new("x", SqlType::Text);
    commented.comment = Some("intent must survive".into());
    for field in [unsigned, commented] {
        assert_eq!(
            Create::new("p")
                .add_field(field)
                .render(DialectId::Sqlite)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
    }
}

/// Custom types/default expressions need an explicit trust marker and dialect match.
#[test]
fn explicit_trusted_ddl_syntax() {
    let custom = Create::new("p").add_field(
        Field::new(
            "x",
            SqlType::Trusted(DialectId::Sqlite, TrustedSql::audited("CUSTOM_TYPE")),
        )
        .default(DefaultValue::Trusted(
            DialectId::Sqlite,
            TrustedSql::audited("lower('A')"),
        )),
    );
    assert!(
        custom.render(DialectId::Sqlite).unwrap().queries()[0]
            .sql()
            .contains("CUSTOM_TYPE DEFAULT (lower('A'))")
    );
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        assert!(
            Create::new("p")
                .add_field(Field::new(
                    "x",
                    SqlType::Trusted(dialect, TrustedSql::audited("CUSTOM"))
                ))
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    for text in [
        "",
        "TEXT; DROP TABLE p",
        "TEXT -- hidden",
        "TEXT /* hidden */",
        "TEXT\0",
    ] {
        assert!(
            Create::new("p")
                .add_field(Field::new(
                    "x",
                    SqlType::Trusted(DialectId::Sqlite, TrustedSql::audited(text))
                ))
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
}

/// Empty/duplicate names, absent local keys and incompatible foreign actions are rejected.
#[test]
fn create_validates_keys_and_foreign_constraints() {
    for create in [
        Create::new("p"),
        Create::new("").add_field(Field::new("x", SqlType::Text)),
        table().add_field(Field::new("ID", SqlType::Integer)),
        table().primary_key(["absent"]),
        table().primary_key(["id", "ID"]),
        table().add_index(Index::new("x", ["absent"])),
        table().add_index(Index::new("x", Vec::<String>::new())),
    ] {
        assert!(create.build().is_err());
    }
    let fk = ForeignKey::new("fk", ["id"], "parent", ["id"]);
    let mut not_nullable = fk.clone();
    not_nullable.on_delete = ReferentialAction::SetNull;
    let mut no_default = fk.clone();
    no_default.on_update = ReferentialAction::SetDefault;
    for foreign in [
        not_nullable,
        no_default,
        ForeignKey::new("fk", ["id"], "parent", ["id", "other"]),
        ForeignKey::new("fk", ["missing"], "parent", ["id"]),
    ] {
        assert!(table().add_foreign_key(foreign).build().is_err());
    }
    assert!(
        table()
            .add_foreign_key(fk.clone())
            .add_foreign_key(fk)
            .build()
            .is_err()
    );
    let mut default_action = ForeignKey::new("fk", ["id"], "parent", ["id"]);
    default_action.on_delete = ReferentialAction::SetDefault;
    default_action.on_update = ReferentialAction::Restrict;
    assert!(
        Create::new("p")
            .add_field(Field::new("id", SqlType::Integer).default(0))
            .add_foreign_key(default_action)
            .render(DialectId::Sqlite)
            .is_ok()
    );
    // Compound order is retained for primary, unique/index and foreign keys.
    let compound = table()
        .primary_key(["name", "id"])
        .add_index(Index::new("both", ["id", "name"]))
        .add_foreign_key(ForeignKey::new(
            "both_fk",
            ["name", "id"],
            "parent",
            ["name", "id"],
        ));
    assert!(
        compound.render(DialectId::Sqlite).unwrap().queries()[0]
            .sql()
            .contains("FOREIGN KEY (\"name\", \"id\") REFERENCES \"parent\" (\"name\", \"id\")")
    );
}

/// Supported native alterations preserve explicit operation order and non-idempotent indexes.
#[test]
fn alter_native_plan_and_inspection() {
    let alter = Alter::new("p")
        .add_field(Field::new("new", SqlType::Text).not_null().default(""))
        .remove_field("old")
        .add_index(Index::new("new_idx", ["new"]))
        .remove_index("old_idx");
    assert_eq!(alter.build().unwrap().operations.len(), 4);
    let plan = alter.render(DialectId::Sqlite).unwrap();
    assert_eq!(
        plan.queries().iter().map(Query::sql).collect::<Vec<_>>(),
        [
            "ALTER TABLE \"p\" ADD COLUMN \"new\" TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE \"p\" DROP COLUMN \"old\"",
            "CREATE INDEX \"new_idx\" ON \"p\" (\"new\")",
            "DROP INDEX \"old_idx\""
        ]
    );
    assert!(Alter::new("p").build().is_err());
    assert!(Alter::new("").remove_field("x").build().is_err());
    assert!(
        Alter::new("p")
            .add_field(Field::new("x", SqlType::Text))
            .add_field(Field::new("X", SqlType::Text))
            .build()
            .is_err()
    );
}

/// Explicit rename reconciliation removes matching halves and exposes semantic changes.
#[test]
fn alter_direct_rename_and_reconciliation() {
    let source = Field::new("old", SqlType::Text);
    let direct = Alter::new("p")
        .source_field(source.clone())
        .rename_field("old", "new");
    assert_eq!(
        direct.render(DialectId::Sqlite).unwrap().queries()[0].sql(),
        "ALTER TABLE \"p\" RENAME COLUMN \"old\" TO \"new\""
    );
    let reconciled = Alter::new("p")
        .source_field(source.clone())
        .remove_field("old")
        .add_field(Field::new("new", SqlType::Text))
        .rename_field("old", "new");
    assert_eq!(
        reconciled.build().unwrap().operations,
        direct.build().unwrap().operations
    );
    let changed = Alter::new("p")
        .source_field(source)
        .remove_field("old")
        .add_field(Field::new("new", SqlType::Text).not_null())
        .rename_field("old", "new");
    assert!(matches!(
        changed.build().unwrap().operations.as_slice(),
        [
            AlterOperation::RenameField { .. },
            AlterOperation::ChangeField(_)
        ]
    ));
    assert_eq!(
        changed.render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
}

/// Every invalid rename half/collision fails in the pure build phase, before any SQL plan.
#[test]
fn alter_rejects_ambiguous_or_incomplete_renames() {
    let base = Alter::new("p")
        .source_field(Field::new("old", SqlType::Text))
        .source_field(Field::new("other", SqlType::Text));
    for alter in [
        base.clone().rename_field("old", "old"),
        base.clone().rename_field("old", "OLD"),
        base.clone().rename_field("old", "other"),
        base.clone().rename_field("missing", "new"),
        base.clone()
            .rename_field("old", "new")
            .rename_field("other", "new"),
        base.clone()
            .rename_field("old", "new")
            .rename_field("old", "second"),
        base.clone().remove_field("old").rename_field("old", "new"),
        base.clone()
            .add_field(Field::new("new", SqlType::Text))
            .rename_field("old", "new"),
        base.clone()
            .remove_field("old")
            .remove_field("old")
            .add_field(Field::new("new", SqlType::Text))
            .rename_field("old", "new"),
        Alter::new("p").rename_field("old", "new"),
    ] {
        assert!(alter.build().is_err(), "{alter:?}");
    }
}

/// A later unsupported operation prevents exposing even the earlier supported SQL.
#[test]
fn sqlite_alter_rejects_unsupported_operations_as_a_whole() {
    let source = Field::new("id", SqlType::Integer);
    for operation in [
        AlterOperation::ChangeField(Field::new("id", SqlType::Text)),
        AlterOperation::AddPrimary(vec!["id".into()]),
        AlterOperation::RemovePrimary,
        AlterOperation::AddForeignKey(ForeignKey::new("fk", ["id"], "parent", ["id"])),
        AlterOperation::RemoveForeignKey("fk".into()),
    ] {
        let alter = Alter::new("p")
            .source_field(source.clone())
            .add_field(Field::new("first", SqlType::Text))
            .operation(operation);
        assert!(alter.build().is_ok());
        assert_eq!(
            alter.render(DialectId::Sqlite).unwrap_err().kind(),
            ErrorKind::Unsupported
        );
    }
    for field in [
        Field::new("x", SqlType::Integer).auto_increment(),
        Field::new("x", SqlType::DateTime).default(DefaultValue::CurrentTimestamp),
        Field::new("x", SqlType::Text).default(DefaultValue::Trusted(
            DialectId::Sqlite,
            TrustedSql::audited("lower('A')"),
        )),
        Field::new("x", SqlType::Text).not_null(),
    ] {
        assert_eq!(
            Alter::new("p")
                .add_field(field)
                .render(DialectId::Sqlite)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
    }
}

/// Table commands validate names, preserve punctuation and reject SQLite cascade.
#[test]
fn table_drop_rename_truncate_and_invalid_names() {
    assert_eq!(
        TableCommand::drop_table("p")
            .render(DialectId::Sqlite)
            .unwrap()
            .sql(),
        "DROP TABLE \"p\""
    );
    assert_eq!(
        TableCommand::Drop {
            table: "p".into(),
            if_exists: true,
            cascade: false
        }
        .render(DialectId::Sqlite)
        .unwrap()
        .sql(),
        "DROP TABLE IF EXISTS \"p\""
    );
    assert_eq!(
        TableCommand::rename("a\"b", "c.d")
            .render(DialectId::Sqlite)
            .unwrap()
            .sql(),
        "ALTER TABLE \"a\"\"b\" RENAME TO \"c.d\""
    );
    assert_eq!(
        TableCommand::truncate("p")
            .render(DialectId::Sqlite)
            .unwrap()
            .sql(),
        "DELETE FROM \"p\""
    );
    for command in [
        TableCommand::drop_table(""),
        TableCommand::truncate(""),
        TableCommand::rename("", "p"),
        TableCommand::rename("p", "P"),
        TableCommand::Drop {
            table: "p".into(),
            if_exists: false,
            cascade: true,
        },
        TableCommand::Truncate {
            table: "p".into(),
            cascade: true,
        },
    ] {
        assert!(command.render(DialectId::Sqlite).is_err());
    }
}

/// All DDL surfaces share identifier/default bounds and explicit dialect selection.
#[test]
fn ddl_hostile_input_dialect_and_clone_independence() {
    let hostile = "x\"; DROP TABLE parent; --";
    let plan = Create::new(hostile)
        .add_field(Field::new(hostile, SqlType::Text))
        .add_index(Index::new(hostile, [hostile]))
        .render(DialectId::Sqlite)
        .unwrap();
    assert!(
        plan.queries()[0]
            .sql()
            .contains("\"x\"\"; DROP TABLE parent; --\"")
    );
    for name in ["", "x\0y", "x\ny"] {
        assert!(
            Create::new("p")
                .add_field(Field::new(name, SqlType::Text))
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        assert_eq!(
            table().render(dialect).unwrap_err().kind(),
            ErrorKind::Unsupported
        );
        assert_eq!(
            Alter::new("p")
                .remove_field("x")
                .render(dialect)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
    }
    let base = table();
    assert_eq!(
        base.clone()
            .add_field(Field::new("third", SqlType::Text))
            .build()
            .unwrap()
            .fields
            .len(),
        3
    );
    assert_eq!(base.build().unwrap().fields.len(), 2);
    assert!(
        Create::new("p")
            .add_field(Field::new("x", SqlType::Text).default("x".repeat(1024 * 1024 + 1)))
            .render(DialectId::Sqlite)
            .is_err()
    );
    assert!(
        Create::new("p")
            .add_field(Field::new("x", SqlType::Blob).default(vec![0_u8; 512 * 1024 + 1]))
            .render(DialectId::Sqlite)
            .is_err()
    );
    assert!(
        Create::new("p")
            .add_field(Field::new(
                "x",
                SqlType::Trusted(
                    DialectId::Sqlite,
                    TrustedSql::audited("X".repeat(16 * 1024 + 1))
                )
            ))
            .render(DialectId::Sqlite)
            .is_err()
    );
}
