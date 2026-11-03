//! Cohesive generated schema operations shared by the two source models.

use super::{Column, Kind};
use crate::columns::Value;
use stackpress_framework_schema::SchemaResult;

/// Insertion-ordered source record; missing/null and observable JSON/CSV order are retained.
pub use stackpress_framework_schema::source::Record;

/// Immutable model columns and source seed; defaults are generated per access.
#[derive(Clone, Debug)]
pub struct RecordSchema {
    /// Source model name.
    pub name: &'static str,
    /// Generated source-order columns.
    pub columns: Vec<Column>,
    /// Seed used only at the Auth storage boundary.
    pub(crate) seed: String,
    /// Explicit local UTC offset for Date conversion.
    pub(crate) offset: i32,
}

impl RecordSchema {
    /// Construct source descriptors; no SQL, plugin registration or default generation occurs.
    pub(crate) fn new(name: &'static str, seed: String) -> Self {
        let names: &[&str] = if name == "Profile" {
            &[
                "id",
                "image",
                "name",
                "type",
                "roles",
                "tags",
                "references",
                "active",
                "created",
                "updated",
            ]
        } else {
            &[
                "id",
                "profileId",
                "type",
                "token",
                "secret",
                "verified",
                "consumed",
                "active",
                "created",
                "updated",
            ]
        };
        let columns = names.iter().map(|field| column(field, name)).collect();
        Self {
            name,
            columns,
            seed,
            offset: 0,
        }
    }

    /// Select the host's local offset explicitly; UTC is the deterministic native default.
    pub fn with_offset_minutes(mut self, offset: i32) -> Self {
        self.offset = offset;
        self
    }

    /// Resolve a source column, retaining source camelCase spelling.
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == name)
    }

    /// Fresh CUID and timestamps with the source's literal defaults and omitted image/name.
    pub fn defaults_at(&self, now: i64) -> SchemaResult<Record> {
        self.columns
            .iter()
            .filter_map(|column| match column.defaults_at(now) {
                Ok(Some(value)) => Some(Ok((column.name.into(), value))),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            })
            .collect()
    }

    /// Generate defaults at the current native clock, as the original getter does.
    pub fn defaults(&self) -> SchemaResult<Record> {
        self.defaults_at(now())
    }

    /// Keep declared stored fields only, optionally generating defaults first.
    pub fn filter(&self, value: &Record, populate: bool) -> SchemaResult<Record> {
        let filtered = value
            .iter()
            .filter(|(name, _)| self.column(name).is_some())
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
        if populate {
            self.populate(&filtered)
        } else {
            Ok(filtered)
        }
    }

    /// Spread defaults then input, retaining extra members just as source populate does.
    pub fn populate(&self, value: &Record) -> SchemaResult<Record> {
        let mut result = self.defaults()?;
        result.extend(value.clone());
        Ok(result)
    }

    /// Return only field errors, or None; required checks every source column.
    pub fn assert(&self, value: &Record, required: bool) -> Option<Record> {
        let errors: Record = self
            .columns
            .iter()
            .filter(|column| required || value.contains_key(column.name))
            .filter_map(|column| {
                column
                    .assert(value.get(column.name))
                    .map(|error| (column.name.into(), Value::String(error)))
            })
            .collect();
        (!errors.is_empty()).then_some(errors)
    }

    /// Prepare columns; storage=true enables token encryption and secret hashing.
    pub fn serialize(&self, value: &Record, storage: bool) -> SchemaResult<Record> {
        self.columns
            .iter()
            .filter_map(|column| {
                match column.serialize(value.get(column.name), &self.seed, storage, self.offset) {
                    Ok(Some(value)) => Some(Ok((column.name.into(), value))),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                }
            })
            .collect()
    }

    /// Decode columns; omitted list columns deliberately become empty arrays.
    pub fn unserialize(&self, value: &Record, storage: bool) -> SchemaResult<Record> {
        self.columns
            .iter()
            .filter_map(|column| {
                match column.unserialize(value.get(column.name), &self.seed, storage, self.offset) {
                    Ok(Some(value)) => Some(Ok((column.name.into(), value))),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                }
            })
            .collect()
    }
}

/// Native clock in milliseconds, without acquiring an external host service.
pub(crate) fn now() -> i64 {
    stackpress_framework_schema::source::now()
}

/// One shared metadata definition prevents Auth/Profile equivalent columns from diverging.
fn column(name: &'static str, model: &str) -> Column {
    let kind = match name {
        "roles" | "tags" => Kind::Strings,
        "references" => Kind::Object,
        "active" | "verified" => Kind::Boolean,
        "created" | "updated" | "consumed" => Kind::Date,
        "token" => Kind::Token,
        "secret" => Kind::Secret,
        _ => Kind::String,
    };
    let required_message = match name {
        "id" => Some("Value is required."),
        "name" => Some("Name is required"),
        "type" => Some("Type is required"),
        "profileId" => Some("Profile is required"),
        "token" => Some("Token is required"),
        "secret" => Some("Secret is required"),
        _ => None,
    };
    Column {
        name,
        sql: if name == "profileId" {
            "profile_id"
        } else {
            name
        },
        kind,
        nullable: matches!(name, "image" | "references"),
        required_message,
        minimum: (name == "token").then_some(5),
        default: match name {
            "type" => Some(serde_json::Value::String(
                if model == "Profile" {
                    "person"
                } else {
                    "username"
                }
                .into(),
            )),
            "roles" => Some(serde_json::json!(["GUEST"])),
            "tags" => Some(serde_json::json!([])),
            "references" => Some(serde_json::json!({})),
            "active" => Some(true.into()),
            "verified" => Some(false.into()),
            _ => None,
        },
    }
}
