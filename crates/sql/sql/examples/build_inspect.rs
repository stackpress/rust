//! Run with `cargo run -p stackpress-sql --example build_inspect`.
//! Build all six families; inspect SQL and bind data without opening a database.

use stackpress_sql::{
    Alter, Create, Delete, DialectId, Expr, Field, Insert, JsonPath, JsonScalar, JsonSegment,
    Render, Result, Select, SqlType, Update,
};

/// Print a checked query without interpolating its values into SQL.
fn inspect(label: &str, builder: &impl Render) -> Result<()> {
    let query = builder.render(DialectId::Sqlite)?;
    println!("{label}: {}", query.sql());
    // Binds may contain private data; this local example prints only deliberate fixtures.
    println!("  binds: {:?}", query.binds());
    Ok(())
}

/// Render a small profile schema and its ordinary reads/writes.
fn main() -> Result<()> {
    let create = Create::new("profile")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("name", SqlType::Text))
        .add_field(Field::new("data", SqlType::Json))
        .primary_key(["id"]);
    for query in create.render(DialectId::Sqlite)?.queries() {
        println!("create: {}", query.sql());
    }
    let alter = Alter::new("profile").add_field(Field::new("active", SqlType::Boolean));
    for query in alter.render(DialectId::Sqlite)?.queries() {
        println!("alter: {}", query.sql());
    }

    inspect(
        "insert",
        &Insert::new("profile").columns(["name"]).values(["Ada"]),
    )?;
    let path = JsonPath::new([JsonSegment::Key("tags".into())]);
    let select = Select::new(["id", "name"])
        .from("profile")
        .and_where(Expr::any([Expr::eq("name", "Ada"), Expr::eq("id", 7)]))
        .where_json_contains("data", path, JsonScalar::String("admin".into()))
        .order("id")
        .limit(10);
    assert_eq!(select.build().selectors.len(), 2);
    inspect("select", &select)?;
    inspect(
        "update",
        &Update::new("profile")
            .set("name", "Grace")
            .where_eq("id", 7),
    )?;
    inspect("delete", &Delete::new("profile").where_eq("id", 7))?;
    Ok(())
}
