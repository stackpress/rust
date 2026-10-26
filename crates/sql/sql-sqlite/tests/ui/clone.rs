use stackpress_sql::{Engine, Transaction};
use stackpress_sql_sqlite::SqliteDatabase;

fn clone_state(engine: Engine, tx: Transaction, database: SqliteDatabase) {
    let _ = engine.clone();
    let _ = tx.clone();
    let _ = database.clone();
}

fn main() {}
