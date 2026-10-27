//! Explicit generated RoleProof store; no captured Engine or identity map.
use crate::generated::contracts::role_proof::{CreateRoleProof, RoleProof};
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
        wire: "text",
        column: "text",
        codec: StorageCodec::Text,
        required: false,
        nonempty: false,
    },
    StorageColumn {
        wire: "flag",
        column: "flag",
        codec: StorageCodec::Boolean,
        required: true,
        nonempty: false,
    },
    StorageColumn {
        wire: "zero",
        column: "zero",
        codec: StorageCodec::Integer,
        required: true,
        nonempty: false,
    },
    StorageColumn {
        wire: "rows",
        column: "rows",
        codec: StorageCodec::Json,
        required: false,
        nonempty: false,
    },
    StorageColumn {
        wire: "metadata",
        column: "metadata",
        codec: StorageCodec::Json,
        required: false,
        nonempty: false,
    },
    StorageColumn {
        wire: "score",
        column: "score",
        codec: StorageCodec::Integer,
        required: false,
        nonempty: false,
    },
    StorageColumn {
        wire: "slug",
        column: "slug",
        codec: StorageCodec::Text,
        required: false,
        nonempty: false,
    },
    StorageColumn {
        wire: "sensitive",
        column: "sensitive",
        codec: StorageCodec::Text,
        required: false,
        nonempty: false,
    },
    StorageColumn {
        wire: "hostOnly",
        column: "host_only",
        codec: StorageCodec::Text,
        required: false,
        nonempty: false,
    },
];
/// Pure inspectable table, query and codec functions.
pub struct RoleProofStore;
impl RoleProofStore {
    /// Build the checked storage definition; installation is always explicit.
    pub fn table() -> DataResult<TableDefinition> {
        let mut table = stackpress_sql::Create::new("role_proof");
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("id", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("text", stackpress_sql::SqlType::Text);
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("flag", stackpress_sql::SqlType::Boolean).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("zero", stackpress_sql::SqlType::Integer).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("rows", stackpress_sql::SqlType::Text);
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("metadata", stackpress_sql::SqlType::Text);
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("score", stackpress_sql::SqlType::Integer);
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("slug", stackpress_sql::SqlType::Text);
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("sensitive", stackpress_sql::SqlType::Text);
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("host_only", stackpress_sql::SqlType::Text);
        table = table.add_field(field);
        table = table.primary_key(["id"]);
        Ok(table.definition()?)
    }
    /// Decode strict stored values, returning a fully owned contract.
    pub fn decode(row: Row) -> DataResult<RoleProof> {
        let value: RoleProof = stackpress_framework_sql::decode(row, COLUMNS)?;
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
                .map(|column| Identifier::qualified(["role_proof", column.column])),
        )
        .from("role_proof")
        .where_eq("id", key);
        query.limit(1)
    }
}
/// Executing functions borrowing the caller's selected executor.
pub struct RoleProofs;
impl RoleProofs {
    /// Execute one key lookup; absence is distinct from decode/storage failure.
    pub async fn get(db: &mut DataSession<'_>, key: &str) -> DataResult<Option<RoleProof>> {
        let mut rows = db.fetch(&RoleProofStore::select(key)).await?;
        rows.pop().map(RoleProofStore::decode).transpose()
    }
    /// Validate/prepare once and insert with authoritative statement-local RETURNING.
    pub async fn create(db: &mut DataSession<'_>, input: CreateRoleProof) -> DataResult<RoleProof> {
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
        let query = Insert::new("role_proof")
            .columns(COLUMNS.iter().map(|column| column.column))
            .values(values)
            .returning(COLUMNS.iter().map(|column| column.column));
        let mut rows = db.fetch(&query).await?;
        if rows.len() != 1 {
            return Err(DataError::new("create returned an invalid row count")
                .with_kind(DataErrorKind::Decode));
        }
        RoleProofStore::decode(rows.remove(0))
    }
}
