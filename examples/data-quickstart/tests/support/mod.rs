//! Install this exact generated revision into one caller-owned disposable SQLite file.

use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

/// Keep fixture creation independent of callable versus host-operation assertions.
pub async fn installed(path: &std::path::Path) -> Engine {
    let engine = Engine::new(
        SqliteDatabase::open(SqliteOptions::file(path).create_if_missing(true))
            .await
            .unwrap(),
    );
    for table in data_quickstart::generated::stores::schema::definition().unwrap() {
        engine
            .execute_plan(&table.create().render(engine.dialect()).unwrap())
            .await
            .unwrap();
    }
    engine
}
