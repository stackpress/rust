//! Explicit generated Project store; no captured Engine or identity map.
use crate::generated::contracts::project::{CreateProject, Project};
use stackpress_framework_sql::{
    DataError, DataErrorKind, DataResult, DataSession, StorageCodec, StorageColumn,
};
use stackpress_sql::{Identifier, Insert, Row, Select, TableDefinition};

const _: () = assert!(stackpress_framework_sql::CONTRACT_REVISION == 1);
/// Same normalized input identity as the schema transform.
pub const FINGERPRINT: &str = "ed7339d88345035c2d45e410c7349278ec7e85bc9880a88c691f3f3ddfccfcf3";
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
        wire: "ownerId",
        column: "owner_id",
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
        wire: "state",
        column: "state",
        codec: StorageCodec::Text,
        required: true,
        nonempty: false,
    },
    StorageColumn {
        wire: "tags",
        column: "tags",
        codec: StorageCodec::Json,
        required: true,
        nonempty: false,
    },
];
/// Pure inspectable table, query and codec functions.
pub struct ProjectStore;
impl ProjectStore {
    /// Build the checked storage definition; installation is always explicit.
    pub fn table() -> DataResult<TableDefinition> {
        let mut table = stackpress_sql::Create::new("project");
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("id", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field =
            stackpress_sql::Field::new("owner_id", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("title", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("state", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("tags", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        table = table.primary_key(["id"]);
        // Storage constraints/indexes are authoritative; UI hints cannot replace them.
        table = table.add_index(stackpress_sql::Index::new("project_title_index", ["title"]));
        // Explicit owning FK; reverse collections have no column.
        let mut foreign =
            stackpress_sql::ForeignKey::new("project_owner_id_fk", ["owner_id"], "profile", ["id"]);
        foreign.on_delete = stackpress_sql::ReferentialAction::Restrict;
        table = table.add_foreign_key(foreign);
        Ok(table.definition()?)
    }
    /// Decode strict stored values, returning a fully owned contract.
    pub fn decode(row: Row) -> DataResult<Project> {
        let value: Project = stackpress_framework_sql::decode(row, COLUMNS)?;
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
                .map(|column| Identifier::qualified(["project", column.column])),
        )
        .from("project")
        .where_eq("id", key);
        query.limit(1)
    }
}
/// Executing functions borrowing the caller's selected executor.
pub struct Projects;
impl Projects {
    /// Execute one key lookup; absence is distinct from decode/storage failure.
    pub async fn get(db: &mut DataSession<'_>, key: &str) -> DataResult<Option<Project>> {
        let mut rows = db.fetch(&ProjectStore::select(key)).await?;
        rows.pop().map(ProjectStore::decode).transpose()
    }
    /// Validate/prepare once and insert with authoritative statement-local RETURNING.
    pub async fn create(db: &mut DataSession<'_>, input: CreateProject) -> DataResult<Project> {
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
        let query = Insert::new("project")
            .columns(COLUMNS.iter().map(|column| column.column))
            .values(values)
            .returning(COLUMNS.iter().map(|column| column.column));
        let mut rows = db.fetch(&query).await?;
        if rows.len() != 1 {
            return Err(DataError::new("create returned an invalid row count")
                .with_kind(DataErrorKind::Decode));
        }
        ProjectStore::decode(rows.remove(0))
    }
}
