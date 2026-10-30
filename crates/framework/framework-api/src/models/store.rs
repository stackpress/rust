//! Source model catalogs, detached DDL/DML builders and SQL scalar codecs.
use super::Value;
use super::{Kind, Record, Schema};
use super::{ModelKind, Result, StoreQuery};
use crate::sql::SourceStatement;
use stackpress_framework_session::storage::Store as CodecStore;
use stackpress_sql::{
    Alter, Create, DefaultValue, Delete, Field, ForeignKey, Index, Insert, Length,
    ReferentialAction, SchemaDiff, SqlType, SqlValue, Update,
};
use std::{marker::PhantomData, ops::Deref};

/// Detached store metadata; no engine, database handle or mutable transaction is retained.
#[derive(Clone, Debug)]
pub struct Store<M: ModelKind> {
    /// Model preparation and column codecs.
    pub schema: Schema<M>,
    codec: CodecStore<CodecModel<M>>,
    /// Shared explicit local UTC offset for predicates, decode and native SQL encoding.
    pub(crate) offset: i32,
    /// Marker fixes the SQL table independently of dynamic input.
    marker: PhantomData<M>,
}
impl<M: ModelKind> Deref for Store<M> {
    type Target = Schema<M>;
    /// Expose generated schema methods directly on the source store.
    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}
impl<M: ModelKind> Store<M> {
    /// Retain the seed without creating or opening a database.
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            schema: Schema::new(),
            codec: CodecStore::new(seed),
            offset: 0,
            marker: PhantomData,
        }
    }
    /// Choose the same explicit local offset for preparation, filtering and SQL codecs.
    pub fn with_offset_minutes(mut self, offset: i32) -> Self {
        self.offset = offset;
        self.schema = self.schema.with_offset_minutes(offset);
        self.codec.schema = self.codec.schema.with_offset_minutes(offset);
        self
    }
    /// Exact constant SQL table identity.
    pub fn table(&self) -> &'static str {
        M::TABLE
    }

    /// Source table definition; secret is plaintext and expires has no implicit TTL.
    pub fn create(&self) -> Create {
        let mut create = Create::new(M::TABLE);
        for column in &self.columns {
            let kind = match column.kind {
                Kind::Strings => SqlType::Json,
                Kind::Boolean => SqlType::Boolean,
                Kind::Date => SqlType::DateTime,
                _ => SqlType::Varchar,
            };
            let mut field = Field::new(column.sql, kind.clone());
            if kind == SqlType::Varchar {
                field = field.length(Length::Size(255));
            }
            if !column.nullable {
                field = field.not_null();
            }
            field = match column.name {
                "scopes" => field.default("[]"),
                "active" => field.default(true),
                "created" | "updated" => field.default(DefaultValue::CurrentTimestamp),
                _ => field,
            };
            create = create.add_field(field);
        }
        create = create.primary_key(["id"]);
        let indexes: &[&str] = if M::NAME == "Application" {
            &[
                "profile_id",
                "name",
                "website",
                "active",
                "expires",
                "created",
                "updated",
            ]
        } else {
            &[
                "application_id",
                "profile_id",
                "active",
                "expires",
                "created",
                "updated",
            ]
        };
        for name in indexes {
            create = create.add_index(Index::new(format!("{}_{name}_index", M::TABLE), [*name]));
        }
        for (local, target) in [("profile_id", "profile"), ("application_id", "application")] {
            if local == "application_id" && M::NAME != "Session" {
                continue;
            }
            let mut foreign = ForeignKey::new(
                format!("{}_{local}_foreign", M::TABLE),
                [local],
                target,
                ["id"],
            );
            foreign.on_delete = ReferentialAction::Cascade;
            foreign.on_update = ReferentialAction::Restrict;
            create = create.add_foreign_key(foreign);
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

    /// Source scalarization uses the merged column codecs, with API String secrets.
    pub fn scalarize(&self, values: &Record) -> Result<Vec<(String, SqlValue)>> {
        self.codec.scalarize(values)
    }
    /// Decode supplied SQL columns only; omitted projected fields stay absent.
    pub fn unscalarize(&self, values: &Record) -> Result<Record> {
        self.codec.unscalarize(values)
    }
    /// Source native SQL scalar conversion.
    pub fn to_sql_value(&self, name: &str, value: &Value) -> Result<SqlValue> {
        self.codec.to_sql_value(name, value)
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
        input.insert("updated".into(), super::date(super::schema::now()));
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

/// Native codec identity composed from the API catalog through the public schema fields.
#[derive(Clone, Debug)]
struct CodecModel<M: ModelKind>(PhantomData<M>);
impl<M: ModelKind> stackpress_framework_session::storage::ModelKind for CodecModel<M> {
    const NAME: &'static str = M::NAME;
    const TABLE: &'static str = M::TABLE;
    fn schema(_: String) -> stackpress_framework_session::columns::RecordSchema {
        Schema::<M>::new().record
    }
}
/// Direct Application store.
pub type ApplicationStore = Store<super::ApplicationModel>;
/// Direct persisted API Session store.
pub type SessionStore = Store<super::SessionModel>;
