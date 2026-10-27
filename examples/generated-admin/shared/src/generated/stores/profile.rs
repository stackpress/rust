//! Explicit generated Profile store; no captured Engine or identity map.
use crate::generated::contracts::profile::{CreateProfile, Profile};
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
        wire: "active",
        column: "active",
        codec: StorageCodec::Boolean,
        required: true,
        nonempty: false,
    },
    StorageColumn {
        wire: "address",
        column: "address",
        codec: StorageCodec::Json,
        required: false,
        nonempty: false,
    },
    StorageColumn {
        wire: "name",
        column: "name",
        codec: StorageCodec::Text,
        required: true,
        nonempty: true,
    },
];
/// Pure inspectable table, query and codec functions.
pub struct ProfileStore;
impl ProfileStore {
    /// Build the checked storage definition; installation is always explicit.
    pub fn table() -> DataResult<TableDefinition> {
        let mut table = stackpress_sql::Create::new("profile");
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("id", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field =
            stackpress_sql::Field::new("active", stackpress_sql::SqlType::Boolean).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("address", stackpress_sql::SqlType::Text);
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("name", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        table = table.primary_key(["id"]);
        // Storage constraints/indexes are authoritative; UI hints cannot replace them.
        table = table.add_index(stackpress_sql::Index::new("profile_name_index", ["name"]));
        Ok(table.definition()?)
    }
    /// Decode strict stored values, returning a fully owned contract.
    pub fn decode(row: Row) -> DataResult<Profile> {
        let value: Profile = stackpress_framework_sql::decode(row, COLUMNS)?;
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
                .map(|column| Identifier::qualified(["profile", column.column])),
        )
        .from("profile")
        .where_eq("id", key);
        // Ordinary reads exclude inactive records, including key lookups.
        let query = query.where_eq(Identifier::qualified(["profile", "active"]), true);
        query.limit(1)
    }
}
/// Executing functions borrowing the caller's selected executor.
pub struct Profiles;
impl Profiles {
    /// Execute one key lookup; absence is distinct from decode/storage failure.
    pub async fn get(db: &mut DataSession<'_>, key: &str) -> DataResult<Option<Profile>> {
        let mut rows = db.fetch(&ProfileStore::select(key)).await?;
        rows.pop().map(ProfileStore::decode).transpose()
    }
    /// Validate/prepare once and insert with authoritative statement-local RETURNING.
    pub async fn create(db: &mut DataSession<'_>, input: CreateProfile) -> DataResult<Profile> {
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
        let query = Insert::new("profile")
            .columns(COLUMNS.iter().map(|column| column.column))
            .values(values)
            .returning(COLUMNS.iter().map(|column| column.column));
        let mut rows = db.fetch(&query).await?;
        if rows.len() != 1 {
            return Err(DataError::new("create returned an invalid row count")
                .with_kind(DataErrorKind::Decode));
        }
        ProfileStore::decode(rows.remove(0))
    }
}
