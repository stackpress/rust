//! Source model catalogs, detached DDL/DML builders and SQL scalar codecs.
use super::{Result, SourceStatement, StoreQuery};
use crate::columns::Value;
use crate::columns::{self, Kind, Record, RecordSchema};
use stackpress_sql::{
    Alter, Create, DefaultValue, Delete, Field, ForeignKey, Index, Insert, Length,
    ReferentialAction, SchemaDiff, SqlType, SqlValue, Update,
};
use std::{marker::PhantomData, ops::Deref};

/// Closed model identity used by the two generated stores.
pub trait ModelKind: Clone + Send + Sync + 'static {
    /// Exact source schema name.
    const NAME: &'static str;
    /// Source SQL table name.
    const TABLE: &'static str;
    /// Construct the corresponding source schema.
    fn schema(seed: String) -> RecordSchema;
}
/// Profile's generated storage identity.
#[derive(Clone, Debug)]
pub struct ProfileModel;
impl ModelKind for ProfileModel {
    const NAME: &'static str = "Profile";
    const TABLE: &'static str = "profile";
    fn schema(seed: String) -> RecordSchema {
        crate::profile::ProfileSchema::new(seed).into_schema()
    }
}
/// Auth's generated storage identity.
#[derive(Clone, Debug)]
pub struct AuthModel;
impl ModelKind for AuthModel {
    const NAME: &'static str = "Auth";
    const TABLE: &'static str = "auth";
    fn schema(seed: String) -> RecordSchema {
        crate::auth::AuthSchema::new(seed).into_schema()
    }
}
/// Complete source Profile store.
pub type ProfileStore = Store<ProfileModel>;
/// Complete source Auth store, including its required inner Profile relation.
pub type AuthStore = Store<AuthModel>;

/// Detached store metadata; no engine, database handle or mutable transaction is retained.
#[derive(Clone, Debug)]
pub struct Store<M: ModelKind> {
    /// Model preparation and column codecs.
    pub schema: RecordSchema,
    /// Marker fixes the SQL table independently of dynamic input.
    marker: PhantomData<M>,
}
impl<M: ModelKind> Deref for Store<M> {
    type Target = RecordSchema;
    /// Expose generated schema methods directly on the source store.
    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}
impl<M: ModelKind> Store<M> {
    /// Retain the seed without creating or opening a database.
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            schema: M::schema(seed.into()),
            marker: PhantomData,
        }
    }
    /// Exact constant SQL table identity.
    pub fn table(&self) -> &'static str {
        M::TABLE
    }

    /// Source table definition with global Auth token uniqueness and CASCADE profile deletion.
    /// SQLite cannot retain source column comments; that renderer mechanics limitation does
    /// not change data, defaults, indexes or foreign key actions.
    pub fn create(&self) -> Create {
        let mut create = Create::new(M::TABLE);
        for column in &self.columns {
            let data_type = match column.kind {
                Kind::Strings | Kind::Object => SqlType::Json,
                Kind::Boolean => SqlType::Boolean,
                Kind::Date => SqlType::DateTime,
                _ => SqlType::Varchar,
            };
            let mut field = Field::new(column.sql, data_type.clone());
            if data_type == SqlType::Varchar {
                field = field.length(Length::Size(255));
            }
            if !column.nullable {
                field = field.not_null();
            }
            // These are source SQL defaults, independent of generated CUID/clock preparation.
            field = match column.name {
                "type" => field.default(if M::NAME == "Profile" {
                    "person"
                } else {
                    "username"
                }),
                "roles" => field.default("[\"GUEST\"]"),
                "tags" => field.default("[]"),
                "references" => field.default("{}"),
                "active" => field.default(true),
                "verified" => field.default(false),
                "created" | "updated" | "consumed" => field.default(DefaultValue::CurrentTimestamp),
                _ => field,
            };
            create = create.add_field(field);
        }
        create = create.primary_key(["id"]);
        let names: &[&str] = if M::NAME == "Profile" {
            &["name", "type", "active", "created", "updated"]
        } else {
            &[
                "profile_id",
                "type",
                "verified",
                "consumed",
                "active",
                "created",
                "updated",
            ]
        };
        for name in names {
            create = create.add_index(Index::new(format!("{}_{name}_index", M::TABLE), [*name]));
        }
        if M::NAME == "Auth" {
            let mut foreign =
                ForeignKey::new("auth_profile_id_foreign", ["profile_id"], "profile", ["id"]);
            foreign.on_delete = ReferentialAction::Cascade;
            foreign.on_update = ReferentialAction::Restrict;
            create = create
                .add_index(Index::new("auth_token_unique", ["token"]).unique())
                .add_foreign_key(foreign);
        }
        create
    }

    /// Compare the source catalog against a caller's requested target, without introspection.
    pub fn alter(&self, to: Option<&Create>) -> Result<Alter> {
        let Some(to) = to else {
            return Ok(Alter::new(M::TABLE));
        };
        let diff = self.create().definition()?.diff(&to.definition()?)?;
        Ok(match diff {
            SchemaDiff::Changes { alteration, .. } => *alteration,
            _ => Alter::new(M::TABLE),
        })
    }

    /// Map supplied values to source SQL spelling and enable native token/secret codecs.
    pub fn scalarize(&self, values: &Record) -> Result<Vec<(String, SqlValue)>> {
        let mut result = Vec::new();
        for column in &self.columns {
            if let Some(value) = values.get(column.name)
                && let Some(value) = column.serialize(Some(value), &self.seed, true, self.offset)?
            {
                result.push((column.sql.into(), self.to_sql_value(column.name, &value)?));
            }
        }
        Ok(result)
    }

    /// Decode actual supplied SQL fields only; partial projections do not acquire defaults.
    pub fn unscalarize(&self, values: &Record) -> Result<Record> {
        let mut result = Record::new();
        for column in &self.columns {
            if let Some(value) = values.get(column.sql)
                && let Some(value) =
                    column.unserialize(Some(value), &self.seed, true, self.offset)?
            {
                result.insert(column.name.into(), value);
            }
        }
        Ok(result)
    }

    /// Source toSqlValue conversion after column encoding, with nullable image/references.
    pub fn to_sql_value(&self, name: &str, value: &Value) -> Result<SqlValue> {
        stackpress_framework_sql::source::to_sql_value(self.column(name), value, self.offset)
    }

    /// Adapt RETURNING to the renderer's native capability; source MySQL ignores this request.
    pub fn insert_for(&self, input: &Record, dialect: stackpress_sql::DialectId) -> Result<Insert> {
        let values = self.scalarize(input)?;
        let insert = Insert::new(M::TABLE)
            .columns(values.iter().map(|(name, _)| name.clone()))
            .values(values.into_iter().map(|(_, value)| value));
        Ok(if dialect.capabilities().returning {
            insert.returning_all()
        } else {
            insert
        })
    }

    /// Source insertion requests RETURNING where the SQL dialect supports it.
    pub fn insert(&self, input: &Record) -> Result<Insert> {
        let values = self.scalarize(input)?;
        Ok(Insert::new(M::TABLE)
            .columns(values.iter().map(|(name, _)| name.clone()))
            .values(values.into_iter().map(|(_, value)| value))
            .returning_all())
    }

    /// Source update mutates input.updated before scalarization; returned action rows are
    /// assembled earlier and deliberately do not acquire this new timestamp.
    pub fn update(
        &self,
        query: &StoreQuery,
        input: &mut Record,
        dialect: stackpress_sql::DialectId,
    ) -> Result<SourceStatement<Update>> {
        input.insert("updated".into(), columns::date(columns::now()));
        let mut update = Update::new(M::TABLE);
        for (name, value) in self.scalarize(input)? {
            update = update.set(name, value);
        }
        if let Some(filter) = self.where_clause(query, dialect)? {
            update = update.and_where(filter);
        } else {
            update = update.all_rows();
        }
        Ok(SourceStatement(update))
    }

    /// Physical source deletion, with the same active default as reads and updates.
    pub fn delete(
        &self,
        query: &StoreQuery,
        dialect: stackpress_sql::DialectId,
    ) -> Result<SourceStatement<Delete>> {
        let delete = Delete::new(M::TABLE);
        Ok(SourceStatement(
            if let Some(filter) = self.where_clause(query, dialect)? {
                delete.and_where(filter)
            } else {
                delete.all_rows()
            },
        ))
    }
}
