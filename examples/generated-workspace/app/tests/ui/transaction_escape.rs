use stackpress_framework_sql::DataSession;
use stackpress_sql::Transaction;
fn escape(mut transaction: Transaction) -> DataSession<'static> {
    DataSession::transaction(&mut transaction)
}
fn main() {}
