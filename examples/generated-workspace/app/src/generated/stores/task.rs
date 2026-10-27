//! Explicit generated Task store; no captured Engine or identity map.
use crate::generated::contracts::task::{CreateTask, Task};
use stackpress_framework_sql::{
    DataError, DataErrorKind, DataResult, DataSession, StorageCodec, StorageColumn,
};
use stackpress_sql::{Identifier, Insert, Row, Select, TableDefinition};

const _: () = assert!(stackpress_framework_sql::CONTRACT_REVISION == 1);
/// Same normalized input identity as the schema transform.
pub const FINGERPRINT: &str = "44c300781986cbf55c24eaab7df16d3ab96ecf98b7978b9a5e4cfafd61972d45";
/// Immutable field codecs, chosen by schema semantics.
pub const COLUMNS: &[StorageColumn] = &[
    StorageColumn {
        wire: "id",
        column: "id",
        codec: StorageCodec::Text,
        required: true,
        nonempty: false,
    },
    StorageColumn {
        wire: "projectId",
        column: "project_id",
        codec: StorageCodec::Text,
        required: true,
        nonempty: false,
    },
    StorageColumn {
        wire: "title",
        column: "title",
        codec: StorageCodec::Text,
        required: true,
        nonempty: true,
    },
    StorageColumn {
        wire: "done",
        column: "done",
        codec: StorageCodec::Boolean,
        required: true,
        nonempty: false,
    },
];
/// Pure inspectable table, query and codec functions.
pub struct TaskStore;
impl TaskStore {
    /// Build the checked storage definition; installation is always explicit.
    pub fn table() -> DataResult<TableDefinition> {
        let mut table = stackpress_sql::Create::new("task");
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("id", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field =
            stackpress_sql::Field::new("project_id", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("title", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("done", stackpress_sql::SqlType::Boolean).not_null();
        table = table.add_field(field);
        table = table.primary_key(["id"]);
        // Explicit owning FK; reverse collections have no column.
        let mut foreign = stackpress_sql::ForeignKey::new(
            "task_project_id_fk",
            ["project_id"],
            "project",
            ["id"],
        );
        foreign.on_delete = stackpress_sql::ReferentialAction::Restrict;
        table = table.add_foreign_key(foreign);
        Ok(table.definition()?)
    }
    /// Decode strict stored values, returning a fully owned contract.
    pub fn decode(row: Row) -> DataResult<Task> {
        let value: Task = stackpress_framework_sql::decode(row, COLUMNS)?;
        value.validate().map_err(|cause| {
            DataError::new("stored value validation failed")
                .with_kind(DataErrorKind::Decode)
                .with_cause(cause)
        })?;
        Ok(value)
    }
    /// Build one key-scoped ordinary read; no IO occurs here.
    pub fn select(key: &str) -> Select {
        let query = Select::new(
            COLUMNS
                .iter()
                .map(|column| Identifier::qualified(["task", column.column])),
        )
        .from("task")
        .where_eq("id", key);
        query.limit(1)
    }
}
/// Executing functions borrowing the caller's selected executor.
pub struct Tasks;
impl Tasks {
    /// Execute one key lookup; absence is distinct from decode/storage failure.
    pub async fn get(db: &mut DataSession<'_>, key: &str) -> DataResult<Option<Task>> {
        let mut rows = db.fetch(&TaskStore::select(key)).await?;
        rows.pop().map(TaskStore::decode).transpose()
    }
    /// Validate/prepare once and insert with authoritative statement-local RETURNING.
    pub async fn create(db: &mut DataSession<'_>, input: CreateTask) -> DataResult<Task> {
        if !db.dialect().capabilities().returning {
            return Err(DataError::new("M0 create requires RETURNING")
                .with_kind(DataErrorKind::Unsupported));
        }
        // Literal defaults are applied once, before the first database effect.
        let value = input.prepare().map_err(|cause| {
            DataError::new("create preparation failed")
                .with_kind(DataErrorKind::Validation)
                .with_cause(cause)
        })?;
        let values = stackpress_framework_sql::encode(&value, COLUMNS)?;
        let query = Insert::new("task")
            .columns(COLUMNS.iter().map(|column| column.column))
            .values(values)
            .returning(COLUMNS.iter().map(|column| column.column));
        let mut rows = db.fetch(&query).await?;
        if rows.len() != 1 {
            return Err(DataError::new("create returned an invalid row count")
                .with_kind(DataErrorKind::Decode));
        }
        TaskStore::decode(rows.remove(0))
    }
}
