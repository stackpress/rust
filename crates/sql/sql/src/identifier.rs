//! Checked identifier segments and explicit table/selector aliases.

use crate::{DialectId, ErrorKind, Result, SqlError};

/// A name with explicit qualification; a string containing dots stays one segment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier(Vec<String>);

impl Identifier {
    /// Construct explicit qualification, such as `["profile", "id"]`.
    pub fn qualified(parts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self(parts.into_iter().map(Into::into).collect())
    }

    /// Inspect the original segments without interpreting their contents as SQL.
    pub fn segments(&self) -> &[String] {
        &self.0
    }

    /// Validate and quote each segment independently for the selected native renderer.
    pub(crate) fn quoted(&self, dialect: DialectId) -> Result<String> {
        // Bound qualification depth as well as each segment before allocating SQL.
        if self.0.is_empty() || self.0.len() > 64 {
            return Err(invalid("invalid SQL identifier qualification"));
        }
        self.0
            .iter()
            .map(|part| dialect.quote(part))
            .collect::<Result<Vec<_>>>()
            .map(|parts| parts.join("."))
    }
}

impl From<&str> for Identifier {
    /// Treat borrowed text as one literal segment, including embedded dots.
    fn from(value: &str) -> Self {
        Self(vec![value.into()])
    }
}

impl From<String> for Identifier {
    /// Take ownership of one literal segment.
    fn from(value: String) -> Self {
        Self(vec![value])
    }
}

/// An explicitly named relation with an optional single-segment alias.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableRef {
    /// Qualified relation name; aliasing never changes its segments.
    pub name: Identifier,
    /// An optional literal alias validated at rendering.
    pub alias: Option<String>,
}

impl TableRef {
    /// Name a table without a live connection.
    pub fn new(name: impl Into<Identifier>) -> Self {
        Self {
            name: name.into(),
            alias: None,
        }
    }

    /// Select a literal alias; dots and quotes have no special syntax.
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    /// Quote the relation and alias through their separate identifier boundaries.
    pub(crate) fn quoted(&self, dialect: DialectId) -> Result<String> {
        let mut sql = self.name.quoted(dialect)?;
        if let Some(alias) = &self.alias {
            // An alias is always one segment rather than another qualified name.
            sql.push_str(&format!(" AS {}", dialect.quote(alias)?));
        }
        Ok(sql)
    }
}

impl<T: Into<Identifier>> From<T> for TableRef {
    /// Convert a literal or explicitly qualified table name.
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// Quote a single bounded segment, rejecting controls and preserving literal punctuation.
pub(crate) fn quote(name: &str) -> Result<String> {
    if name.is_empty() || name.chars().any(char::is_control) {
        return Err(invalid("invalid SQL identifier"));
    }
    if name.len() > 1024 {
        return Err(SqlError::new(
            ErrorKind::Limit,
            "SQL identifier byte limit exceeded",
        ));
    }
    Ok(format!("\"{}\"", name.replace('"', "\"\"")))
}

/// Construct input errors without disclosing caller data.
pub(crate) fn invalid(message: &'static str) -> SqlError {
    SqlError::new(ErrorKind::InvalidQuery, message)
}

/// Construct a capability error without pretending unsupported syntax will execute.
pub(crate) fn unsupported(message: &'static str) -> SqlError {
    SqlError::new(ErrorKind::Unsupported, message)
}
