//! Source field declarations and per-access defaults over the shared column mechanics.
use super::{Column, Kind, Record, Value};
use stackpress_framework_schema::{SchemaResult, ValueProviders, providers::BuiltinProviders};
use stackpress_framework_session::{columns::RecordSchema, profile::ProfileSchema};
use std::{marker::PhantomData, ops::Deref};

/// Finite source model identity; implementations supply owned schema columns.
pub trait ModelKind: Clone + Send + Sync + 'static {
    /// Exact source model name.
    const NAME: &'static str;
    /// Exact SQL table name.
    const TABLE: &'static str;
}
/// Source Application identity.
#[derive(Clone, Debug)]
pub struct ApplicationModel;
impl ModelKind for ApplicationModel {
    const NAME: &'static str = "Application";
    const TABLE: &'static str = "application";
}
/// Source persisted Session identity.
#[derive(Clone, Debug)]
pub struct SessionModel;
impl ModelKind for SessionModel {
    const NAME: &'static str = "Session";
    const TABLE: &'static str = "session";
}
/// Source Application schema, including plaintext generated secret.
pub type ApplicationSchema = Schema<ApplicationModel>;
/// Source API Session schema, including applicationId and profileId.
pub type SessionSchema = Schema<SessionModel>;

/// API field catalog using public Profile schema composition and shared column codecs.
#[derive(Clone, Debug)]
pub struct Schema<M: ModelKind> {
    /// Shared record operations; API defaults override both generated string columns.
    pub(crate) record: RecordSchema,
    /// Fixes the catalog identity without retaining a mutable host resource.
    marker: PhantomData<M>,
}
impl<M: ModelKind> Default for Schema<M> {
    fn default() -> Self {
        Self::new()
    }
}
impl<M: ModelKind> Deref for Schema<M> {
    type Target = RecordSchema;
    fn deref(&self) -> &Self::Target {
        &self.record
    }
}
impl<M: ModelKind> Schema<M> {
    /// Compose a new catalog without generating values or retaining external resources.
    pub fn new() -> Self {
        // Public name/columns are the merged composition boundary. No Profile field is
        // redefined here; its record machinery supplies source Date/list coercions.
        let mut record = ProfileSchema::default().into_schema();
        record.name = M::NAME;
        let names: &[&str] = if M::NAME == "Application" {
            &[
                "id",
                "profileId",
                "name",
                "logo",
                "website",
                "secret",
                "scopes",
                "active",
                "expires",
                "created",
                "updated",
            ]
        } else {
            &[
                "id",
                "applicationId",
                "profileId",
                "secret",
                "scopes",
                "active",
                "expires",
                "created",
                "updated",
            ]
        };
        record.columns = names.iter().map(|name| column(name)).collect();
        Self {
            record,
            marker: PhantomData,
        }
    }
    /// Native source model descriptor for direct consumers, including both forward
    /// relations and Application's inverse `session` metadata (not a store join).
    pub fn model(&self) -> stackpress_framework_schema::Model {
        let mut model = self.record.model();
        for field in &mut model.fields {
            if field.name == "secret" {
                field.value_rule = Some(stackpress_framework_schema::ValueRule {
                    provider: "cuid".into(),
                    args: Vec::new(),
                    revision: "cuid2-3.3.0".into(),
                });
                field.input_required = false;
            }
            if field.name == "expires" {
                field.value_rule = None;
                field.input_required = true;
            }
            if field.name == "website" {
                field.searchable = true;
            }
        }
        // The common RecordSchema supplies scalar descriptions and the Profile edge.
        // API's additional edges are composed without redefining external Profile fields.
        let mut edge = model
            .fields
            .iter()
            .find(|field| field.name == "profile")
            .expect("shared relation descriptor")
            .clone();
        if M::NAME == "Application" {
            edge.name = "session".into();
            edge.rust_name = "session".into();
            edge.column = "session".into();
            edge.value_type = stackpress_framework_schema::ValueType::Model("Session".into());
            edge.required = false;
            edge.multiple = true;
            edge.relation = None;
        } else {
            edge.name = "application".into();
            edge.rust_name = "application".into();
            edge.column = "application".into();
            edge.value_type = stackpress_framework_schema::ValueType::Model("Application".into());
            edge.relation = Some(stackpress_framework_schema::Relation {
                local: "applicationId".into(),
                foreign: "id".into(),
                inverse: None,
            });
        }
        model.fields.push(edge);
        model.query = vec!["*".into(), "profile.*".into()];
        if M::NAME == "Session" {
            model.query.push("application.*".into());
        }
        model
    }
    /// Explicit native local offset, matching the shared Date representation.
    pub fn with_offset_minutes(mut self, offset: i32) -> Self {
        self.record = self.record.with_offset_minutes(offset);
        self
    }
    /// Fresh source defaults at a supplied clock and value-provider boundary.
    pub fn defaults_with(&self, now: i64, providers: &dyn ValueProviders) -> SchemaResult<Record> {
        let mut result = Record::new();
        for column in &self.columns {
            let value = if matches!(column.name, "id" | "secret") {
                Some(Value::from(providers.generate("cuid", &[])?))
            } else {
                column.defaults_at(now)?
            };
            if let Some(value) = value {
                result.insert(column.name.into(), value);
            }
        }
        Ok(result)
    }
    /// Use the shared built-in CUID provider for both id and secret.
    pub fn defaults_at(&self, now: i64) -> SchemaResult<Record> {
        self.defaults_with(now, &BuiltinProviders)
    }
    /// Generate source defaults on every access.
    pub fn defaults(&self) -> SchemaResult<Record> {
        self.defaults_at(now())
    }
    /// Spread defaults followed by input, preserving source override semantics.
    pub fn populate(&self, value: &Record) -> SchemaResult<Record> {
        let mut result = self.defaults()?;
        result.extend(value.clone());
        Ok(result)
    }
    /// Keep stored fields, optionally applying freshly generated defaults first.
    pub fn filter(&self, value: &Record, populate: bool) -> SchemaResult<Record> {
        let filtered = self.record.filter(value, false)?;
        if populate {
            self.populate(&filtered)
        } else {
            Ok(filtered)
        }
    }
}
/// Millisecond native clock; no provider, database or listener is acquired.
pub(crate) fn now() -> i64 {
    stackpress_framework_schema::source::now()
}
/// Exact stored source fields; API secrets use String, never the Auth secret hash codec.
fn column(name: &'static str) -> Column {
    Column {
        name,
        sql: match name {
            "profileId" => "profile_id",
            "applicationId" => "application_id",
            _ => name,
        },
        kind: match name {
            "scopes" => Kind::Strings,
            "active" => Kind::Boolean,
            "expires" | "created" | "updated" => Kind::Date,
            _ => Kind::String,
        },
        nullable: matches!(name, "logo" | "website"),
        required_message: match name {
            "id" => Some("Value is required."),
            "profileId" => Some("Profile is required"),
            "applicationId" => Some("App is required"),
            "name" => Some("Name is required"),
            "expires" => Some("Expires is required"),
            _ => None,
        },
        minimum: None,
        default: match name {
            "scopes" => Some(serde_json::json!([])),
            "active" => Some(true.into()),
            _ => None,
        },
    }
}
