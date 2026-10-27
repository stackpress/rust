//! Aggregated loss inspection remains available when a required transition cannot render.
#![cfg(feature = "migrations")]

use stackpress_framework_sql::migrations::{
    DialectId, MigrationPlanner, RenameMap, RevisionSnapshot,
};
use stackpress_sql::{Create, Field, SqlType};

/// A missing backfill cannot hide a different column/table loss or storage conversion.
#[test]
fn aggregates_before_precondition_refusal() {
    let retained = Create::new("retained")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("optional", SqlType::Text))
        .add_field(Field::new("removed", SqlType::Text))
        .add_field(Field::new("retyped", SqlType::Text))
        .primary_key(["id"])
        .definition()
        .unwrap();
    let removed = Create::new("removed_table")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .primary_key(["id"])
        .definition()
        .unwrap();
    let from = RevisionSnapshot::new("review", "v1", vec![retained.clone(), removed]).unwrap();
    let mut retained = retained;
    retained.fields.retain(|field| field.name != "removed");
    retained.fields[1].nullable = false;
    retained.fields[2].data_type = SqlType::Integer;
    retained
        .fields
        .push(Field::new("new_required", SqlType::Text).not_null());
    let to = RevisionSnapshot::new("review", "v2", vec![retained]).unwrap();
    let findings =
        MigrationPlanner::review(&from, &to, &RenameMap::new(), DialectId::Sqlite).unwrap();
    for kind in [
        "drop_table",
        "drop_column",
        "retype_or_narrow",
        "nullable_to_required",
        "required_addition_without_backfill",
    ] {
        assert!(
            findings
                .iter()
                .any(|finding| finding.kind == kind && finding.destructive),
            "{kind}"
        );
    }
    assert!(MigrationPlanner::between(&from, &to, RenameMap::new(), DialectId::Sqlite).is_err());
}
