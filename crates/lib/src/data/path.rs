//! Explicit literal keys/indices/append segments and checked common dot notation.

use crate::{Error, Result};
use std::{borrow::Cow, fmt};

/// More than the largest valid spelling (8192 key bytes + 64 machine indices/separators).
pub(super) const MAX_SOURCE_BYTES: usize = 16 * 1024;

/// Seal accepted path inputs so generic calls always use the same checked grammar.
mod sealed {
    pub trait Sealed {}
    impl Sealed for &str {}
    impl Sealed for &String {}
    impl Sealed for &super::DataPath {}
    impl Sealed for super::DataPath {}
}

/// A dot string or reusable parsed path accepted uniformly by Nest path methods.
pub trait IntoPath<'a>: sealed::Sealed {
    /// Parse strings; borrow an existing DataPath without copying its segments.
    ///
    /// # Errors
    /// Rejects malformed strings using the same limits as DataPath::dot.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_lib::IntoPath;
    /// assert_eq!("profile.name".into_path()?.to_string(), "profile.name");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    fn into_path(self) -> Result<Cow<'a, DataPath>>;
}
impl<'a> IntoPath<'a> for &'a str {
    /// Parse the common dot-string form into an owned temporary path.
    fn into_path(self) -> Result<Cow<'a, DataPath>> {
        // Strings produce owned checked segments because their spelling is not a parsed path.
        Ok(Cow::Owned(DataPath::dot(self)?))
    }
}
impl<'a> IntoPath<'a> for &'a String {
    /// Accept formatted/owned string references without a call-site as_str conversion.
    fn into_path(self) -> Result<Cow<'a, DataPath>> {
        // Forward String references through exactly the same dot grammar as str.
        self.as_str().into_path()
    }
}
impl<'a> IntoPath<'a> for &'a DataPath {
    /// Reuse validated segments for repeated reads and writes.
    fn into_path(self) -> Result<Cow<'a, DataPath>> {
        // Borrow reusable segments rather than allocating a new path for every operation.
        Ok(Cow::Borrowed(self))
    }
}
impl<'a> IntoPath<'a> for DataPath {
    /// Move a freshly built explicit path without an extra clone.
    fn into_path(self) -> Result<Cow<'a, DataPath>> {
        // Move explicit segments so their literal-key intent survives adaptation.
        Ok(Cow::Owned(self))
    }
}

/// One data addressing step; Append is never a magic empty literal key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathSegment {
    /// A literal object key, including punctuation when supplied explicitly.
    Key(String),
    /// A canonical numeric index; sparse writes remain keyed objects.
    Index(usize),
    /// The next numeric key after the existing maximum.
    Append,
}

/// A reusable path; common APIs also accept dot-notation strings directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataPath(pub(crate) Vec<PathSegment>);
impl fmt::Display for DataPath {
    /// Render ordinary dot keys plus quoted literal keys and explicit append markers.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Render indices and ordinary keys compactly, quoting literal punctuation/numeric keys to
        // avoid ambiguity.
        for (position, step) in self.0.iter().enumerate() {
            match step {
                PathSegment::Append => write!(f, "[]")?,
                PathSegment::Index(index) => {
                    if position > 0 {
                        write!(f, ".")?;
                    }
                    write!(f, "{index}")?;
                }
                PathSegment::Key(key)
                    if key
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                        && !key.bytes().all(|c| c.is_ascii_digit()) =>
                {
                    if position > 0 {
                        write!(f, ".")?;
                    }
                    write!(f, "{key}")?;
                }
                // Quote literal keys that a human might otherwise read as dot syntax or an index.
                PathSegment::Key(key) => write!(f, "[{key:?}]")?,
            }
        }
        Ok(())
    }
}
impl DataPath {
    /// Parse conventional dot notation.
    ///
    /// # Errors
    /// Rejects empty segments, oversized paths and overflowing numeric indices.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, PathSegment};
    /// let path = DataPath::dot("profile.name")?;
    /// assert_eq!(path.to_string(), "profile.name");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn dot(text: &str) -> Result<Self> {
        // The common form uses a fixed literal separator; no regex or escape language is inferred.
        Self::separated(text, '.')
    }
    /// Parse a chosen separator without interpreting regex or escape sequences.
    ///
    /// # Errors
    /// Rejects an empty/oversized path or an empty component.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, PathSegment};
    /// let path = DataPath::separated("profile/name", '/')?;
    /// assert_eq!(path.segments().len(), 2);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn separated(text: &str, separator: char) -> Result<Self> {
        // Parse every component before returning a path so partial paths never reach mutation.
        // Bound source and component count before allocating a key/vector for every supplied
        // segment.
        if text.len() > MAX_SOURCE_BYTES {
            return Err(Error::limit("path source byte limit exceeded"));
        }
        let mut steps = Vec::new();
        for part in text.split(separator) {
            if steps.len() == 64 {
                return Err(Error::limit("path depth limit exceeded"));
            }
            steps.push(segment(part)?);
        }
        Self::from_segments(steps)
    }
    /// Validate explicit literal/index/append segments.
    ///
    /// # Errors
    /// Rejects empty paths/keys, more than 64 segments or over 8192 key bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, PathSegment};
    /// let path = DataPath::from_segments([PathSegment::Key("literal.key".into())])?;
    /// assert_eq!(path.to_string(), "[\"literal.key\"]");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn from_segments(segments: impl IntoIterator<Item = PathSegment>) -> Result<Self> {
        // Accumulate bounded immutable segments; append is a typed step rather than an empty key.
        let mut path = Vec::new();
        let mut bytes = 0usize;
        for step in segments {
            // Literal keys contribute their actual byte length; typed indices have bounded machine
            // size.
            if let PathSegment::Key(key) = &step {
                if key.is_empty() {
                    return Err(Error::invalid("empty path component"));
                }
                bytes = bytes.saturating_add(key.len());
            }
            path.push(step);
            // Stop while consuming input so even a huge/infinite segment iterator cannot build an
            // unbounded path.
            if path.len() > 64 || bytes > 8192 {
                return Err(Error::limit("path limit exceeded"));
            }
        }
        // Root operations are separate APIs; an empty path must not accidentally replace the root.
        if path.is_empty() {
            return Err(Error::invalid("path is empty"));
        }
        Ok(Self(path))
    }
    /// Inspect immutable parsed segments.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, PathSegment};
    /// let path = DataPath::dot("rows.0")?;
    /// assert!(matches!(path.segments()[1], PathSegment::Index(0)));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn segments(&self) -> &[PathSegment] {
        // Borrow the validated sequence without exposing a way to bypass path bounds.
        &self.0
    }
}

/// Interpret only canonical nonnegative numeric strings as indices.
pub(crate) fn segment(text: &str) -> Result<PathSegment> {
    // Only canonical decimal spelling becomes an index; other text remains an exact key.
    if text.is_empty() {
        return Err(Error::invalid("empty path component"));
    }
    if text == "0" || (!text.starts_with('0') && text.bytes().all(|v| v.is_ascii_digit())) {
        return Ok(PathSegment::Index(
            text.parse()
                .map_err(|_| Error::limit("numeric path index overflow"))?,
        ));
    }
    Ok(PathSegment::Key(text.into()))
}
