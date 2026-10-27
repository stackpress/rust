//! Explicit generated MySQL target facts do not alter ordinary VARCHAR behavior.
use stackpress_sql::{Create, DialectId, Field, ForeignKey, Index, Length, SqlType};

/// Build a deterministic compound string key without inferring widths.
fn compound(parts: usize, length: i32) -> Create {
    let names: Vec<_> = (0..parts).map(|i| format!("part{i}")).collect();
    let mut table = Create::new("bounded");
    for name in &names {
        table = table.add_field(
            Field::new(name, SqlType::Varchar)
                .length(Length::Size(length))
                .not_null(),
        );
    }
    table.primary_key(names)
}

/// Per-page full-key limits apply to the whole primary/unique/secondary/FK-support key.
#[test]
fn whole_key_budgets() {
    for (page, fitting, oversized) in [
        (4096, 192, 193),
        (8192, 384, 385),
        (16384, 768, 769),
        (32768, 768, 769),
        (65536, 768, 769),
    ] {
        assert!(
            compound(1, fitting)
                .mysql_utf8mb4_dynamic("utf8mb4_bin", page)
                .unwrap()
                .render(DialectId::Mysql)
                .is_ok()
        );
        assert!(
            compound(1, oversized)
                .mysql_utf8mb4_dynamic("utf8mb4_bin", page)
                .unwrap()
                .render(DialectId::Mysql)
                .is_err()
        );
    }
    assert!(
        compound(4, 191)
            .mysql_utf8mb4_dynamic("utf8mb4_bin", 16384)
            .unwrap()
            .render(DialectId::Mysql)
            .is_ok()
    );
    assert!(
        compound(5, 191)
            .mysql_utf8mb4_dynamic("utf8mb4_bin", 16384)
            .unwrap()
            .render(DialectId::Mysql)
            .is_err()
    );
    for unique in [false, true] {
        let mut index = Index::new("compound", ["part0", "part1", "part2", "part3", "part4"]);
        index.unique = unique;
        let mut table = compound(5, 191).build().unwrap();
        table.primary.clear();
        let table: stackpress_sql::TableDefinition = table.into();
        assert!(
            table
                .create()
                .add_index(index)
                .mysql_utf8mb4_dynamic("utf8mb4_bin", 16384)
                .unwrap()
                .render(DialectId::Mysql)
                .is_err()
        );
    }
    let mut table: stackpress_sql::TableDefinition = compound(5, 191).build().unwrap().into();
    table.primary.clear();
    let names: Vec<_> = (0..5).map(|i| format!("part{i}")).collect();
    table
        .foreign_keys
        .push(ForeignKey::new("support", names.clone(), "parent", names));
    assert!(
        table
            .create()
            .mysql_utf8mb4_dynamic("utf8mb4_bin", 16384)
            .unwrap()
            .render(DialectId::Mysql)
            .is_err()
    );
    assert!(
        compound(17, 1)
            .mysql_utf8mb4_dynamic("utf8mb4_bin", 16384)
            .unwrap()
            .render(DialectId::Mysql)
            .is_err()
    );
}

/// Facts are checked tokens; unsupported layouts and incomplete generated facts refuse purely.
#[test]
fn explicit_facts_and_layouts() {
    for (collation, page) in [
        ("utf8mb4_bin; DROP TABLE x", 16384),
        ("latin1_bin", 16384),
        ("utf8mb4_bin", 2048),
    ] {
        assert!(
            compound(1, 10)
                .mysql_utf8mb4_dynamic(collation, page)
                .is_err()
        );
    }
    let table = compound(1, 191).require_mysql_target();
    assert!(table.render(DialectId::Mysql).is_err());
    assert!(table.render(DialectId::Postgres).is_ok());
    assert!(table.render(DialectId::Sqlite).is_ok());
    let selected = table.mysql_utf8mb4_dynamic("utf8mb4_bin", 16384).unwrap();
    let definition = selected.definition().unwrap();
    assert_eq!(
        definition.mysql_storage.as_ref().unwrap().facts(),
        Some(("utf8mb4_bin", 16384))
    );
    assert_eq!(definition.normalized().unwrap(), definition);
    assert!(
        selected.render(DialectId::Mysql).unwrap().queries()[0]
            .sql()
            .ends_with(
                "ENGINE=InnoDB ROW_FORMAT=DYNAMIC DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_bin"
            )
    );
    assert!(
        Create::new("large")
            .add_field(Field::new("v", SqlType::Varchar).length(Length::Size(16383)))
            .mysql_utf8mb4_dynamic("utf8mb4_bin", 16384)
            .unwrap()
            .render(DialectId::Mysql)
            .is_err()
    );
    assert!(
        Create::new("unknown")
            .add_field(Field::new("v", SqlType::Text))
            .mysql_utf8mb4_dynamic("utf8mb4_bin", 16384)
            .unwrap()
            .render(DialectId::Mysql)
            .is_err()
    );
    assert!(
        compound(1, 10_485_761)
            .require_mysql_target()
            .render(DialectId::Postgres)
            .is_err()
    );
    // Unselected low-level behavior is unchanged, including legacy TEXT-key refusal.
    assert!(compound(1, 1000).render(DialectId::Mysql).is_ok());
    assert!(
        Create::new("legacy")
            .add_field(Field::new("id", SqlType::Text))
            .primary_key(["id"])
            .render(DialectId::Mysql)
            .is_err()
    );
}
