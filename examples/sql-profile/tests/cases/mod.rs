//! App-owned fixtures and distinct behavioral proofs, without hidden database setup at startup.

mod parity;
mod storage;

use std::path::Path;

use serde_json::json;
use stackpress_server::{ConfigLayers, Server};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

pub(super) use parity::{literals, parity};
pub(super) use storage::{failures, setup, startup};

/// Give the app its own file layer, preserving profiles defaults and later overrides.
fn layers(path: &Path, limit: usize) -> ConfigLayers {
    let mut layers = ConfigLayers::new();
    layers
        .value(
            "test",
            json!({"sqlite":{"path":path},"profiles":{"limit":limit}}),
        )
        .unwrap();
    layers
}

/// Await shutdown before the owning tempfile root leaves scope.
async fn close(app: Server) {
    app.host()
        .shared_service::<Engine>()
        .unwrap()
        .close()
        .await
        .unwrap();
}

/// Open a writable fixture for explicit test/operator effects, never through application startup.
async fn writable(path: &Path) -> Engine {
    Engine::new(
        SqliteDatabase::open(SqliteOptions::file(path))
            .await
            .unwrap(),
    )
}
