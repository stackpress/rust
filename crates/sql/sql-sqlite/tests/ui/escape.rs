use stackpress_sql::{DialectId, QueryResult, Sql, Transaction};

fn escape(tx: &mut Transaction) -> impl Future<Output = stackpress_sql::Result<QueryResult>> + 'static {
    async move {
        tx.execute(&Sql::literal(DialectId::Sqlite, "SELECT 1")).await
    }
}

fn main() {}
