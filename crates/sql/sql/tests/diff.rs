//! M2 Engine 1-5/7-17 and Helpers 3: same-table structural equality and explicit renames.
use stackpress_sql::*;
/// A normalized small definition with two ordered fields.
fn source() -> TableDefinition {
    Create::new("profiles")
        .add_field(Field::new("id", SqlType::Integer))
        .add_field(Field::new("name", SqlType::Varchar).length(Length::Size(80)))
        .primary_key(["id"])
        .definition()
        .unwrap()
}
/// Inspect normalized operations without using a live database.
fn operations(diff: &SchemaDiff) -> Vec<AlterOperation> {
    diff.alteration().unwrap().build().unwrap().operations
}
/// Equal definitions produce a typed no-op; different logical tables and invalid inputs error.
#[test]
fn no_changes_invalid_inputs_and_same_table_contract() {
    let source = source();
    assert_eq!(source.diff(&source.clone()).unwrap(), SchemaDiff::NoChanges);
    assert!(
        SchemaDiff::NoChanges
            .render(DialectId::Sqlite)
            .unwrap()
            .is_none()
    );
    assert!(SchemaDiff::NoChanges.alteration().is_none());
    assert!(SchemaDiff::NoChanges.rename_field("a", "b").is_err());
    let mut other = source.clone();
    other.name = "other".into();
    assert!(source.diff(&other).is_err());
    other.name = source.name.clone();
    other.fields.clear();
    assert!(source.diff(&other).is_err());
    assert_eq!(source.normalized().unwrap(), source);
}
/// Every field property, including length/precision, defaults and annotations compares by value.
#[test]
fn field_properties_are_structural_not_identity() {
    let source = source();
    let mut variants = Vec::new();
    let mut field = source.fields[1].clone();
    field.nullable = false;
    variants.push(field);
    let mut field = source.fields[1].clone();
    field.length = Some(Length::Size(100));
    variants.push(field);
    let mut field = source.fields[1].clone();
    field.default = Some("".into());
    variants.push(field);
    let mut field = source.fields[1].clone();
    field.comment = Some("name".into());
    variants.push(field);
    let mut field = source.fields[1].clone();
    field.data_type = SqlType::Text;
    field.length = None;
    variants.push(field);
    for field in variants {
        let mut target = source.clone();
        target.fields[1] = field.clone();
        assert_eq!(
            operations(&source.diff(&target).unwrap()),
            [AlterOperation::ChangeField(field)]
        );
    }
    let mut target = source.clone();
    target.fields[0].auto_increment = true;
    assert!(matches!(
        operations(&source.diff(&target).unwrap()).as_slice(),
        [AlterOperation::ChangeField(_)]
    ));
    target = source.clone();
    target.fields[0].unsigned = true;
    assert!(matches!(
        operations(&source.diff(&target).unwrap()).as_slice(),
        [AlterOperation::ChangeField(_)]
    ));
    let decimal = Create::new("decimal")
        .add_field(
            Field::new("amount", SqlType::Decimal).length(Length::Precision {
                precision: 10,
                scale: 2,
            }),
        )
        .definition()
        .unwrap();
    let mut target = decimal.clone();
    target.fields[0].length = Some(Length::Precision {
        precision: 12,
        scale: 2,
    });
    assert!(matches!(
        operations(&decimal.diff(&target).unwrap()).as_slice(),
        [AlterOperation::ChangeField(_)]
    ));
}
/// Field add/remove and explicit rename preserve changed definitions rather than guessing intent.
#[test]
fn diff_rename_same_changed_and_missing_halves() {
    let source = source();
    let mut target = source.clone();
    target.fields[1].name = "display".into();
    let diff = source.diff(&target).unwrap();
    assert!(matches!(
        operations(&diff).as_slice(),
        [AlterOperation::RemoveField(_), AlterOperation::AddField(_)]
    ));
    let renamed = diff.rename_field("name", "display").unwrap();
    assert_eq!(
        operations(&renamed),
        [AlterOperation::RenameField {
            from: "name".into(),
            to: "display".into()
        }]
    );
    assert!(
        renamed
            .render(DialectId::Sqlite)
            .unwrap()
            .unwrap()
            .queries()[0]
            .sql()
            .contains("RENAME COLUMN")
    );
    target.fields[1].nullable = false;
    let changed = source
        .diff(&target)
        .unwrap()
        .rename_field("name", "display")
        .unwrap();
    assert!(matches!(
        operations(&changed).as_slice(),
        [
            AlterOperation::RenameField { .. },
            AlterOperation::ChangeField(_)
        ]
    ));
    assert_eq!(
        changed.render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    assert!(changed.render(DialectId::Postgres).is_ok());
    assert!(changed.render(DialectId::Mysql).is_ok());
    assert!(
        source
            .diff(&target)
            .unwrap()
            .rename_field("name", "absent")
            .is_err()
    );
    let mut removed = source.clone();
    removed.fields.pop();
    assert!(
        source
            .diff(&removed)
            .unwrap()
            .rename_field("name", "display")
            .is_err()
    );
    let mut added = source.clone();
    added.fields.push(Field::new("new", SqlType::Text));
    assert!(
        source
            .diff(&added)
            .unwrap()
            .rename_field("name", "new")
            .is_err()
    );
    assert!(
        source
            .diff(&target)
            .unwrap()
            .rename_field("id", "display")
            .is_err()
    );
}
/// Ordered primary/regular/unique keys compare by value and produce remove-before-add changes.
#[test]
fn compound_keys_add_remove_change_and_order() {
    let source = source();
    let mut no_primary = source.clone();
    no_primary.primary.clear();
    assert_eq!(
        operations(&source.diff(&no_primary).unwrap()),
        [AlterOperation::RemovePrimary]
    );
    assert_eq!(
        operations(&no_primary.diff(&source).unwrap()),
        [AlterOperation::AddPrimary(vec!["id".into()])]
    );
    let mut compound = source.clone();
    compound.primary = vec!["name".into(), "id".into()];
    assert_eq!(
        operations(&source.diff(&compound).unwrap()),
        [
            AlterOperation::RemovePrimary,
            AlterOperation::AddPrimary(compound.primary.clone())
        ]
    );
    for unique in [false, true] {
        let mut indexed = source.clone();
        indexed.indexes.push(Index {
            name: "compound".into(),
            columns: vec!["id".into(), "name".into()],
            unique,
        });
        assert!(matches!(
            operations(&source.diff(&indexed).unwrap()).as_slice(),
            [AlterOperation::AddIndex(_)]
        ));
        assert_eq!(
            operations(&indexed.diff(&source).unwrap()),
            [AlterOperation::RemoveIndex("compound".into())]
        );
        let mut reversed = indexed.clone();
        reversed.indexes[0].columns.reverse();
        assert!(matches!(
            operations(&indexed.diff(&reversed).unwrap()).as_slice(),
            [AlterOperation::RemoveIndex(_), AlterOperation::AddIndex(_)]
        ));
        assert_eq!(
            indexed.diff(&indexed.clone()).unwrap(),
            SchemaDiff::NoChanges
        );
    }
}
/// Every FK field/action/table/casing change is visible; unchanged cloned definitions are a no-op.
#[test]
fn foreign_key_add_remove_change_and_case() {
    let base = source();
    let mut source = base.clone();
    source
        .foreign_keys
        .push(ForeignKey::new("parent", ["id"], "parent", ["id"]));
    assert!(matches!(
        operations(&base.diff(&source).unwrap()).as_slice(),
        [AlterOperation::AddForeignKey(_)]
    ));
    assert_eq!(
        operations(&source.diff(&base).unwrap()),
        [AlterOperation::RemoveForeignKey("parent".into())]
    );
    let mut targets = Vec::new();
    let mut target = source.clone();
    target.foreign_keys[0].table = "Parent".into();
    targets.push(target);
    let mut target = source.clone();
    target.foreign_keys[0].on_delete = ReferentialAction::Cascade;
    targets.push(target);
    let mut target = source.clone();
    target.foreign_keys[0].foreign_columns = vec!["other".into()];
    targets.push(target);
    for target in targets {
        assert!(matches!(
            operations(&source.diff(&target).unwrap()).as_slice(),
            [
                AlterOperation::RemoveForeignKey(_),
                AlterOperation::AddForeignKey(_)
            ]
        ));
    }
    assert_eq!(source.diff(&source.clone()).unwrap(), SchemaDiff::NoChanges);
}
/// Reconciled renames occur before dependent indexes/FKs, including additions on new fields.
#[test]
fn rename_orders_dependencies_and_added_foreign_fields() {
    let source = source();
    let mut target = source.clone();
    target.fields[1].name = "display".into();
    target.indexes.push(Index::new("display", ["display"]));
    let diff = source
        .diff(&target)
        .unwrap()
        .rename_field("name", "display")
        .unwrap();
    assert!(matches!(
        operations(&diff).as_slice(),
        [
            AlterOperation::RenameField { .. },
            AlterOperation::AddIndex(_)
        ]
    ));
    let mut target = source.clone();
    target
        .fields
        .push(Field::new("parent_id", SqlType::Integer));
    target
        .foreign_keys
        .push(ForeignKey::new("parent", ["parent_id"], "parent", ["id"]));
    let diff = source.diff(&target).unwrap();
    assert!(diff.render(DialectId::Postgres).is_ok());
    assert!(diff.render(DialectId::Mysql).is_ok());
    assert_eq!(
        diff.render(DialectId::Sqlite).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
}

/// Physical column order cannot silently become an empty migration.
#[test]
fn physical_field_reordering_is_explicitly_unsupported() {
    let source = source();
    let mut target = source.clone();
    target.fields.reverse();
    assert_eq!(
        source.diff(&target).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
}

/// Appending an SQL column cannot fulfill a target that inserts it before existing fields.
#[test]
fn diff_rejects_unrealizable_new_field_ordinals() {
    let source = source();
    let mut target = source.clone();
    target.fields.insert(1, Field::new("middle", SqlType::Text));
    let diff = source.diff(&target).unwrap();
    for dialect in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
        assert_eq!(
            diff.render(dialect).unwrap_err().kind(),
            ErrorKind::Unsupported
        );
    }
    // A genuine append is realizable and must not be rejected by an overbroad order guard.
    target.fields.swap(1, 2);
    for dialect in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
        assert!(source.diff(&target).unwrap().render(dialect).is_ok());
    }
}

/// Explicit rename retains an earlier ordinal that a remove/add pair would lose.
#[test]
fn reconciled_rename_preserves_middle_field_ordinal() {
    let source = Create::new("p")
        .add_field(Field::new("first", SqlType::Text))
        .add_field(Field::new("old", SqlType::Text))
        .add_field(Field::new("last", SqlType::Text))
        .definition()
        .unwrap();
    let mut target = source.clone();
    target.fields[1].name = "new".into();
    let diff = source.diff(&target).unwrap();
    for dialect in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
        assert_eq!(
            diff.render(dialect).unwrap_err().kind(),
            ErrorKind::Unsupported
        );
        let plan = diff
            .clone()
            .rename_field("old", "new")
            .unwrap()
            .render(dialect)
            .unwrap()
            .unwrap();
        assert_eq!(plan.queries().len(), 1);
        assert!(plan.queries()[0].sql().contains("RENAME COLUMN"));
    }
}
