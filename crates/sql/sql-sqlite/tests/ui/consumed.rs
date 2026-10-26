use stackpress_sql::Transaction;

async fn completed(tx: Transaction) {
    tx.commit().await.unwrap();
    tx.rollback().await.unwrap();
}

fn main() {}
