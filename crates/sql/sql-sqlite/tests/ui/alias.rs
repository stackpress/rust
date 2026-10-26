use stackpress_sql::{DialectId, Sql, Transaction};

async fn alias(tx: &mut Transaction) {
    let query = Sql::literal(DialectId::Sqlite, "SELECT 1");
    let first = tx.execute(&query);
    let second = tx.execute(&query);
    let _ = tokio::join!(first, second);
}

fn main() {}
