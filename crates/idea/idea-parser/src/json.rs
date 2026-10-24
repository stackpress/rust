//! Checked JSON import/export boundary for schema config and typed reference envelopes.
//!
//! The codec owns raw decoding/serialization; config owns normalized schema structure.
//! Public methods enforce both policies before exposing owned data to callers.

mod codec;
mod config;

use crate::{
    AttributeValue, CompiledSchema, ErrorCode, IdeaError, ParseLimits, Reference, SchemaConfig,
};
use indexmap::IndexMap;
use serde_json::Value;

pub(crate) use codec::{encode, measure, read_json};
pub(crate) use config::schema_from_value;
use config::{fields, object, required};

impl SchemaConfig {
    /// Import ordered normalized JSON, rejecting duplicates and malformed configuration.
    ///
    /// JSON strings are literal data. Use [`CompiledSchema::from_json`] for typed references.
    /// Limits bound input bytes, JSON container depth and materialized export size.
    ///
    /// # Errors
    /// Returns named `<json>` syntax, validation, duplicate or resource diagnostics.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::{SchemaConfig, ParseLimits};
    /// let schema = SchemaConfig::from_json(r#"{"prop":{"P":{"x":1}}}"#, ParseLimits::default())?;
    /// assert_eq!(schema.prop["P"]["x"], 1);
    /// # Ok::<(), stackpress_idea_parser::IdeaError>(())
    /// ```
    pub fn from_json(source: &str, limits: ParseLimits) -> Result<Self, IdeaError> {
        Self::from_json_named("<json>", source, limits)
    }

    /// Import normalized JSON with an explicit logical diagnostic name.
    ///
    /// # Errors
    /// Returns the same checked errors as [`Self::from_json`].
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::{SchemaConfig, ParseLimits};
    /// assert!(SchemaConfig::from_json_named("empty.json", "{}", ParseLimits::default()).is_ok());
    /// ```
    pub fn from_json_named(
        file: &str,
        source: &str,
        limits: ParseLimits,
    ) -> Result<Self, IdeaError> {
        let result = read_json(file, source, limits).and_then(|value| schema_from_value(&value));
        result.map_err(|error| locate_json(error, file, source))
    }

    /// Validate public, manually assembled config before clone, export or finalization.
    ///
    /// Existing public config fields remain compatible. Direct `Clone`/Serde on arbitrary
    /// caller-built recursive `Value`s does not provide this checked boundary.
    ///
    /// # Errors
    /// Returns invalid names/structure, duplicate declarations/columns or resource limits.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::{SchemaConfig, ParseLimits};
    /// SchemaConfig::default().validate(ParseLimits::default())?;
    /// # Ok::<(), stackpress_idea_parser::IdeaError>(())
    /// ```
    pub fn validate(&self, limits: ParseLimits) -> Result<(), IdeaError> {
        // Reject caller-built wide sections before visiting their values or allocating JSON.
        let maximum = limits.max_collection();
        if self.r#use.len() > maximum
            || self.plugin.len() > maximum
            || self.prop.len() > maximum
            || self.r#enum.len() > maximum
            || self.r#type.len() > maximum
            || self.model.len() > maximum
        {
            return Err(limit("Schema collection budget exceeded"));
        }
        // Inspect borrowed recursive values before invoking recursive Serde or Clone.
        for values in self
            .plugin
            .values()
            .chain(self.prop.values())
            .chain(self.r#enum.values())
        {
            if values.len() > maximum {
                return Err(limit("Member collection budget exceeded"));
            }
            for value in values.values() {
                validate_value(value, limits)?;
            }
        }
        for value in self.r#type.values().chain(self.model.values()) {
            if value.columns.len() > maximum {
                return Err(limit("Column collection budget exceeded"));
            }
            validate_attributes(&value.attributes, limits)?;
            for column in &value.columns {
                validate_attributes(&column.attributes, limits)?;
            }
        }
        // A bounded serializer prevents a wide caller-built config from allocating its JSON.
        let encoded = encode(self, limits)?;
        let value = read_json("<config>", &encoded, limits)?;
        schema_from_value(&value)?;
        Ok(())
    }

    /// Export checked ordered config as compact JSON with no ambient inputs or IO.
    ///
    /// # Errors
    /// Rejects invalid manually constructed config or output exceeding the supplied limits.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::{SchemaConfig, ParseLimits};
    /// assert_eq!(SchemaConfig::default().to_json(ParseLimits::default())?, "{}");
    /// # Ok::<(), stackpress_idea_parser::IdeaError>(())
    /// ```
    pub fn to_json(&self, limits: ParseLimits) -> Result<String, IdeaError> {
        self.validate(limits)?;
        encode(self, limits)
    }
}

impl CompiledSchema {
    /// Export config together with actual reference sites; literal lookalikes remain data.
    ///
    /// # Errors
    /// Rejects malformed config/reference sites, invalid coordinates or exceeded budgets.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::*;
    /// let compiled = CompiledSchema { config: SchemaConfig::default(), references: vec![] };
    /// assert!(compiled.to_json(ParseLimits::default())?.contains("references"));
    /// # Ok::<(), IdeaError>(())
    /// ```
    pub fn to_json(&self, limits: ParseLimits) -> Result<String, IdeaError> {
        self.config.validate(limits)?;
        crate::references::validate_references(&self.config, &self.references, limits)?;
        encode(self, limits)
    }

    /// Import the typed config/reference envelope using duplicate-aware bounded decoding.
    ///
    /// # Errors
    /// Rejects unknown fields, invalid config/sites/coordinates and exceeded budgets.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::*;
    /// let compiled = CompiledSchema::from_json(r#"{"config":{},"references":[]}"#,
    ///     ParseLimits::default())?;
    /// assert!(compiled.references.is_empty());
    /// # Ok::<(), IdeaError>(())
    /// ```
    pub fn from_json(source: &str, limits: ParseLimits) -> Result<Self, IdeaError> {
        let result = (|| {
            // Typed envelopes have a closed shape; plain-schema imports never infer references.
            let value = read_json("<json>", source, limits)?;
            let object = object(&value)?;
            fields(object, &["config", "references"])?;
            let config = schema_from_value(required(object, "config")?)?;
            let references: Vec<Reference> =
                serde_json::from_value(required(object, "references")?.clone())
                    .map_err(|_| validation("Invalid typed reference records"))?;
            crate::references::validate_references(&config, &references, limits)?;
            Ok(Self { config, references })
        })();
        result.map_err(|error| locate_json(error, "<json>", source))
    }
}

/// Check metadata values without recursively serializing untrusted caller-built data.
fn validate_attributes(
    attributes: &IndexMap<String, AttributeValue>,
    limits: ParseLimits,
) -> Result<(), IdeaError> {
    if attributes.len() > limits.max_collection() {
        return Err(limit("Attribute collection budget exceeded"));
    }
    for value in attributes.values() {
        if let AttributeValue::Array(values) = value {
            if values.len() > limits.max_collection() {
                return Err(limit("Argument collection budget exceeded"));
            }
            for value in values {
                validate_value(value, limits)?;
            }
        }
    }
    Ok(())
}

/// Bound borrowed value depth, nodes, key/string bytes before clone or serialization.
pub(crate) fn validate_value(root: &Value, limits: ParseLimits) -> Result<(), IdeaError> {
    // Work holds borrowed nodes, so rejecting a hostile tree does not recursively drop it.
    let mut pending = vec![(root, 1usize)];
    let mut bytes = 0usize;
    let mut nodes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        // Check aggregate traversal work before queuing another node's children.
        nodes += 1;
        if nodes > limits.max_nodes() {
            return Err(limit("JSON node budget exceeded"));
        }
        if matches!(value, Value::Array(_) | Value::Object(_))
            && depth > limits.max_depth.min(ParseLimits::HARD_DEPTH)
        {
            return Err(limit("JSON nesting limit exceeded"));
        }
        match value {
            Value::Array(values) => {
                // Width is checked before extending the traversal stack.
                if values.len() > limits.max_collection() {
                    return Err(limit("JSON collection budget exceeded"));
                }
                pending.extend(values.iter().map(|value| (value, depth + 1)));
            }
            Value::Object(values) => {
                // Key bytes contribute to allocation pressure just like string values.
                if values.len() > limits.max_collection() {
                    return Err(limit("JSON collection budget exceeded"));
                }
                for (key, value) in values {
                    bytes = bytes.saturating_add(key.len());
                    pending.push((value, depth + 1));
                }
            }
            Value::String(value) => bytes = bytes.saturating_add(value.len()),
            _ => {}
        }
        if bytes > limits.max_bytes {
            return Err(limit("JSON byte budget exceeded"));
        }
    }
    Ok(())
}

/// Assign source coordinates to structural errors lacking a lexical location.
fn locate_json(error: IdeaError, file: &str, source: &str) -> IdeaError {
    if error.file.is_empty() {
        error.with_span(0, 0).locate(file, source)
    } else {
        error
    }
}

/// Produce a sanitized structural failure without including config or environment data.
pub(crate) fn validation(message: &str) -> IdeaError {
    IdeaError::new(ErrorCode::ValidationError, message)
}

/// Produce a sanitized budget failure shared by checked JSON operations.
fn limit(message: &str) -> IdeaError {
    IdeaError::new(ErrorCode::Limit, message)
}
