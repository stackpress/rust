//! Run with `cargo run -p stackpress-sql --example capabilities`.
//! Inspect every capability; flags describe rendering, not installed adapters.

use stackpress_sql::{DialectId, ErrorKind, Insert, Render, Result};

/// Print the full matrix and demonstrate a checked unsupported operation.
fn main() -> Result<()> {
    for dialect in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
        println!("{dialect:?}: {:#?}", dialect.capabilities());
    }
    let insert = Insert::new("profile")
        .columns(["name"])
        .values(["Ada"])
        .returning_all();
    let error = insert.render(DialectId::Mysql).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    println!("MySQL RETURNING: {error}");
    Ok(())
}
