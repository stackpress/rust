//! Typed JSON scalars, bounded document encoding and per-query checked paths.
//!
//! SQLite reads JSON as text. Encoding is explicit and does not change native row types.
//! This module does not guess that arbitrary text is a JSON document or a date.

use crate::identifier::invalid;
use crate::{ErrorKind, Result, SqlError, SqlValue};

/// An exact JSON scalar; JSON null is distinct from SQL NULL.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{JsonScalar, JsonValue, SqlValue};
/// let document = JsonValue::Object(vec![(
///     "tags".into(),
///     JsonValue::Array(vec![
///         JsonValue::Scalar(JsonScalar::String("admin".into())),
///         JsonValue::Scalar(JsonScalar::Null),
///     ]),
/// )]);
/// assert_eq!(document.to_json()?, r#"{"tags":["admin",null]}"#);
/// assert_ne!(
///     SqlValue::json(&JsonValue::Scalar(JsonScalar::Null))?,
///     SqlValue::Null
/// );
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum JsonScalar {
    /// JSON null.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonScalar, JsonValue};
    /// let document = JsonValue::Scalar(JsonScalar::Null);
    /// assert!(!document.to_json()?.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Null,
    /// JSON boolean, distinct from the JSON number 0/1.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonScalar, JsonValue};
    /// let document = JsonValue::Scalar(JsonScalar::Bool(true));
    /// assert!(!document.to_json()?.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Bool(bool),
    /// Exact signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonScalar, JsonValue};
    /// let document = JsonValue::Scalar(JsonScalar::Integer(7));
    /// assert!(!document.to_json()?.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Integer(i64),
    /// Finite floating-point number.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonScalar, JsonValue};
    /// let document = JsonValue::Scalar(JsonScalar::Real(1.5));
    /// assert!(!document.to_json()?.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Real(f64),
    /// JSON string, including empty strings and escaped controls.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonScalar, JsonValue};
    /// let document = JsonValue::Scalar(JsonScalar::String("Ada".into()));
    /// assert!(!document.to_json()?.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    String(String),
}

impl JsonScalar {
    /// Return SQLite json_type's spelling, validating nonfinite numbers.
    pub(crate) fn kind(&self) -> Result<&'static str> {
        match self {
            Self::Null => Ok("null"),
            Self::Bool(true) => Ok("true"),
            Self::Bool(false) => Ok("false"),
            Self::Integer(_) => Ok("integer"),
            Self::Real(value) if value.is_finite() => Ok("real"),
            Self::Real(_) => Err(invalid("JSON number must be finite")),
            Self::String(_) => Ok("text"),
        }
    }

    /// Convert an explicitly selected scalar to its extracted SQLite bind representation.
    pub(crate) fn bind(&self) -> SqlValue {
        match self {
            Self::Null => SqlValue::Null,
            Self::Bool(value) => SqlValue::Bool(*value),
            Self::Integer(value) => SqlValue::Integer(*value),
            Self::Real(value) => SqlValue::Real(*value),
            Self::String(value) => SqlValue::Text(value.clone()),
        }
    }
}

/// Explicit structured JSON input with finite depth, node and byte budgets on encoding.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{JsonScalar, JsonValue, SqlValue};
/// let document = JsonValue::Object(vec![(
///     "tags".into(),
///     JsonValue::Array(vec![
///         JsonValue::Scalar(JsonScalar::String("admin".into())),
///         JsonValue::Scalar(JsonScalar::Null),
///     ]),
/// )]);
/// assert_eq!(document.to_json()?, r#"{"tags":["admin",null]}"#);
/// assert_ne!(
///     SqlValue::json(&JsonValue::Scalar(JsonScalar::Null))?,
///     SqlValue::Null
/// );
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    /// One JSON scalar.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonScalar, JsonValue};
    /// let document = JsonValue::Scalar(JsonScalar::Null);
    /// assert!(!document.to_json()?.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Scalar(JsonScalar),
    /// Ordered array members.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::JsonValue;
    /// let document = JsonValue::Array(vec![]);
    /// assert!(!document.to_json()?.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Array(Vec<JsonValue>),
    /// Ordered unique keys; duplicate keys are rejected rather than overwritten.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonScalar, JsonValue};
    /// let document = JsonValue::Object(vec![(
    ///     "active".into(),
    ///     JsonValue::Scalar(JsonScalar::Bool(true)),
    /// )]);
    /// assert!(!document.to_json()?.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Encode at most 64 levels, 10,000 nodes and 1 MiB of JSON text.
    ///
    /// # Errors
    /// Rejects duplicate object keys, nonfinite reals and resource-budget overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonScalar, JsonValue, SqlValue};
    /// let document = JsonValue::Object(vec![(
    ///     "tags".into(),
    ///     JsonValue::Array(vec![
    ///         JsonValue::Scalar(JsonScalar::String("admin".into())),
    ///         JsonValue::Scalar(JsonScalar::Null),
    ///     ]),
    /// )]);
    /// assert_eq!(document.to_json()?, r#"{"tags":["admin",null]}"#);
    /// assert_ne!(
    ///     SqlValue::json(&JsonValue::Scalar(JsonScalar::Null))?,
    ///     SqlValue::Null
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn to_json(&self) -> Result<String> {
        let mut output = String::new();
        let mut nodes = 0;
        self.encode(&mut output, 0, &mut nodes)?;
        Ok(output)
    }

    /// Walk in source order, checking depth/nodes before descending into a child.
    fn encode(&self, output: &mut String, depth: usize, nodes: &mut usize) -> Result<()> {
        *nodes += 1;
        if depth > 64 || *nodes > 10_000 {
            return Err(json_limit());
        }
        match self {
            Self::Scalar(value) => {
                value.kind()?;
                match value {
                    JsonScalar::Null => output.push_str("null"),
                    JsonScalar::Bool(value) => {
                        output.push_str(if *value { "true" } else { "false" })
                    }
                    JsonScalar::Integer(value) => output.push_str(&value.to_string()),
                    JsonScalar::Real(value) => {
                        let encoded = value.to_string();
                        output.push_str(&encoded);
                        // Preserve an explicit real as a JSON real even when mathematically integral.
                        if !encoded.contains(['.', 'e', 'E']) {
                            output.push_str(".0");
                        }
                    }
                    JsonScalar::String(value) => output.push_str(&json_string(value)?),
                }
            }
            Self::Array(values) => {
                output.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    value.encode(output, depth + 1, nodes)?;
                }
                output.push(']');
            }
            Self::Object(values) => {
                output.push('{');
                let mut keys = std::collections::HashSet::new();
                for (index, (key, value)) in values.iter().enumerate() {
                    // Duplicate keys have ambiguous reads across dialects, so do not emit them.
                    if !keys.insert(key) {
                        return Err(invalid("duplicate JSON object key"));
                    }
                    if index != 0 {
                        output.push(',');
                    }
                    output.push_str(&json_string(key)?);
                    output.push(':');
                    value.encode(output, depth + 1, nodes)?;
                }
                output.push('}');
            }
        }
        if output.len() > 1024 * 1024 {
            return Err(json_limit());
        }
        Ok(())
    }
}

/// One checked path component; the key `"01"` is never inferred to be an index.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Index, JsonPath, JsonPathOptions, JsonSegment};
/// let path = JsonPath::parse("tags:01", JsonPathOptions { separator: ':' })?;
/// assert_eq!(
///     path.segments(),
///     &[
///         JsonSegment::Key("tags".into()),
///         JsonSegment::Key("01".into())
///     ]
/// );
/// let indexed = JsonPath::new([JsonSegment::Key("tags".into()), JsonSegment::Index(0)]);
/// assert_eq!(indexed.segments().len(), 2);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JsonSegment {
    /// A literal object key.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonPath, JsonSegment};
    /// let path = JsonPath::new([JsonSegment::Key("01".into())]);
    /// assert_eq!(path.segments().len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Key(String),
    /// A nonnegative array index, limited to signed 32-bit range by validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Index, JsonPath, JsonSegment};
    /// let path = JsonPath::new([JsonSegment::Index(0)]);
    /// assert_eq!(path.segments().len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Index(u32),
}

/// Per-query parsing options for a convenience key path; no mutable global dialect state.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Index, JsonPath, JsonPathOptions, JsonSegment};
/// let path = JsonPath::parse("tags:01", JsonPathOptions { separator: ':' })?;
/// assert_eq!(
///     path.segments(),
///     &[
///         JsonSegment::Key("tags".into()),
///         JsonSegment::Key("01".into())
///     ]
/// );
/// let indexed = JsonPath::new([JsonSegment::Key("tags".into()), JsonSegment::Index(0)]);
/// assert_eq!(indexed.segments().len(), 2);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JsonPathOptions {
    /// Separator between literal keys. Array indices use the explicit segment API.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::JsonPathOptions;
    /// let value = JsonPathOptions::default();
    /// assert_eq!(value.separator, '.');
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub separator: char,
}

impl Default for JsonPathOptions {
    /// Use a dot between keys without guessing numeric components.
    fn default() -> Self {
        Self { separator: '.' }
    }
}

/// A JSON path with at most 64 components and 16 KiB of encoded text.
///
/// The empty typed path selects the root. Convenience parsing is key-only, uses
/// a per-query separator (default `.`), and rejects empty components. Numeric-looking
/// keys stay keys. Typed paths are validated at render time: at most 64 components,
/// 16 KiB encoded path text, and indices at most i32::MAX. Paths remain data, not SQL.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Index, JsonPath, JsonPathOptions, JsonSegment};
/// let path = JsonPath::parse("tags:01", JsonPathOptions { separator: ':' })?;
/// assert_eq!(
///     path.segments(),
///     &[
///         JsonSegment::Key("tags".into()),
///         JsonSegment::Key("01".into())
///     ]
/// );
/// let indexed = JsonPath::new([JsonSegment::Key("tags".into()), JsonSegment::Index(0)]);
/// assert_eq!(indexed.segments().len(), 2);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JsonPath(Vec<JsonSegment>);

impl JsonPath {
    /// Construct a root or explicitly typed key/index path, validated at render time.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Index, JsonPath, JsonPathOptions, JsonSegment};
    /// let path = JsonPath::parse("tags:01", JsonPathOptions { separator: ':' })?;
    /// assert_eq!(
    ///     path.segments(),
    ///     &[
    ///         JsonSegment::Key("tags".into()),
    ///         JsonSegment::Key("01".into())
    ///     ]
    /// );
    /// let indexed = JsonPath::new([JsonSegment::Key("tags".into()), JsonSegment::Index(0)]);
    /// assert_eq!(indexed.segments().len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn new(segments: impl IntoIterator<Item = JsonSegment>) -> Self {
        Self(segments.into_iter().collect())
    }

    /// Parse a key-only convenience path; empty/ambiguous components are errors.
    ///
    /// # Errors
    ///
    /// Rejects invalid separators, empty components and path depth/size/index violations.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Index, JsonPath, JsonPathOptions, JsonSegment};
    /// let path = JsonPath::parse("tags:01", JsonPathOptions { separator: ':' })?;
    /// assert_eq!(
    ///     path.segments(),
    ///     &[
    ///         JsonSegment::Key("tags".into()),
    ///         JsonSegment::Key("01".into())
    ///     ]
    /// );
    /// let indexed = JsonPath::new([JsonSegment::Key("tags".into()), JsonSegment::Index(0)]);
    /// assert_eq!(indexed.segments().len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn parse(path: &str, options: JsonPathOptions) -> Result<Self> {
        if options.separator.is_control()
            || options.separator.is_alphanumeric()
            || matches!(options.separator, '"' | '\\' | '[' | ']' | '$')
        {
            return Err(invalid("invalid JSON path separator"));
        }
        if path.is_empty() {
            return Err(invalid("empty JSON convenience path"));
        }
        let segments = path
            .split(options.separator)
            .map(|key| JsonSegment::Key(key.into()))
            .collect::<Vec<_>>();
        if segments
            .iter()
            .any(|segment| matches!(segment, JsonSegment::Key(key) if key.is_empty()))
        {
            return Err(invalid("empty JSON convenience path component"));
        }
        let parsed = Self(segments);
        parsed.sqlite_path()?;
        Ok(parsed)
    }

    /// Inspect typed components in their traversal order.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Index, JsonPath, JsonPathOptions, JsonSegment};
    /// let path = JsonPath::parse("tags:01", JsonPathOptions { separator: ':' })?;
    /// assert_eq!(
    ///     path.segments(),
    ///     &[
    ///         JsonSegment::Key("tags".into()),
    ///         JsonSegment::Key("01".into())
    ///     ]
    /// );
    /// let indexed = JsonPath::new([JsonSegment::Key("tags".into()), JsonSegment::Index(0)]);
    /// assert_eq!(indexed.segments().len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn segments(&self) -> &[JsonSegment] {
        &self.0
    }

    /// Encode a bound SQLite path; key punctuation can never become SQL syntax.
    pub(crate) fn sqlite_path(&self) -> Result<String> {
        if self.0.len() > 64 {
            return Err(json_limit());
        }
        let mut path = String::from("$");
        for segment in &self.0 {
            match segment {
                JsonSegment::Key(key) => {
                    path.push('.');
                    path.push_str(&json_string(key)?);
                }
                JsonSegment::Index(index) => {
                    if *index > i32::MAX as u32 {
                        return Err(invalid("JSON array index out of range"));
                    }
                    path.push_str(&format!("[{index}]"));
                }
            }
            if path.len() > 16 * 1024 {
                return Err(json_limit());
            }
        }
        Ok(path)
    }
}

/// Encode one JSON string with standard escapes and a finite preallocation bound.
pub(crate) fn json_string(value: &str) -> Result<String> {
    if value.len() > 1024 * 1024 {
        return Err(json_limit());
    }
    let mut encoded = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => encoded.push_str("\\\""),
            '\\' => encoded.push_str("\\\\"),
            c if c <= '\u{1f}' => encoded.push_str(&format!("\\u{:04x}", c as u32)),
            c => encoded.push(c),
        }
        if encoded.len() > 1024 * 1024 {
            return Err(json_limit());
        }
    }
    encoded.push('"');
    Ok(encoded)
}

/// Report bounded JSON/path failure without including caller content.
fn json_limit() -> SqlError {
    SqlError::new(ErrorKind::Limit, "SQL JSON limit exceeded")
}

/// Reject PostgreSQL JSONB's unsupported U+0000 strings/keys before generating SQL.
pub(crate) fn validate_postgres_json(value: &JsonValue) -> Result<()> {
    // Encoding establishes finite depth/nodes first; the following recursive walk is bounded.
    value.to_json()?;
    postgres_json_strings(value)
}

/// Walk an already bounded JSON document without allocating replacement strings.
fn postgres_json_strings(value: &JsonValue) -> Result<()> {
    match value {
        JsonValue::Scalar(JsonScalar::String(text)) if text.contains('\0') => {
            return Err(crate::identifier::unsupported(
                "PostgreSQL JSONB cannot retain NUL strings",
            ));
        }
        JsonValue::Array(values) => {
            for value in values {
                postgres_json_strings(value)?;
            }
        }
        JsonValue::Object(values) => {
            for (key, value) in values {
                if key.contains('\0') {
                    return Err(crate::identifier::unsupported(
                        "PostgreSQL JSONB cannot retain NUL keys",
                    ));
                }
                postgres_json_strings(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}
