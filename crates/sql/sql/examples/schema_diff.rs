//! Run with `cargo run -p stackpress-sql --example schema_diff`.
//! Compare supplied definitions and explicitly reconcile a field rename.

use stackpress_sql::{Create, DialectId, Field, Result, SchemaDiff, SqlType};

/// Inspect ordered migration statements without installing or introspecting anything.
fn main() -> Result<()> {
    let source = Create::new("profile")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("name", SqlType::Text))
        .primary_key(["id"])
        .definition()?;
    assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
    let mut target = source.clone();
    target.fields[1].name = "display_name".into();
    let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    let alteration = diff
        .alteration()
        .expect("explicit rename has an alteration");
    println!("operations: {:?}", alteration.build()?.operations);
    if let Some(plan) = diff.render(DialectId::Sqlite)? {
        for query in plan.queries() {
            println!("{} | {:?}", query.sql(), query.binds());
        }
    }
    Ok(())
}
