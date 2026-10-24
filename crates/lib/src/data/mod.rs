//! Owned nested values, bounded paths and boundary parsers without host I/O.
//!
//! ```
//! use stackpress_lib::data::{Nest, ParseLimits, parse_query};
//! let mut data = parse_query("profile[name]=Ada&active=true", &ParseLimits::default())?;
//! assert_eq!(data.path_as::<String>("profile.name")?, Some("Ada".into()));
//! data.set_path("profile.name", "Grace")?;
//! assert_eq!(
//!     data.path("profile.name").and_then(|v| v.as_str()),
//!     Some("Grace")
//! );
//! # Ok::<(), stackpress_lib::Error>(())
//! ```

mod mutation;
mod nest;
mod parsers;
mod path;
mod value;

pub use nest::{IntoNest, Nest, VisitOutcome, make_array, make_object, should_be_an_array};
pub(crate) use parsers::{decode, encode, validate_mime};
pub use parsers::{
    parse_arg_string, parse_args, parse_body, parse_json_object, parse_multipart, parse_query,
};
pub use path::{DataPath, IntoPath, PathSegment};
pub use value::{FileData, SharedNative, Timestamp, Value};

/// Insertion-ordered map; use its iterators for filter/find/map and indexed reads.
pub type DataMap<K, V> = indexmap::IndexMap<K, V>;
/// Insertion-ordered unique set; cloning follows the payload's explicit semantics.
pub type DataSet<T> = indexmap::IndexSet<T>;

/// Finite budgets shared by constructors, mutation and buffered boundary parsers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseLimits {
    /// Maximum buffered input and aggregate string/byte payload size.
    pub max_bytes: usize,
    /// Maximum nested container/path depth, up to the implementation ceiling of 64.
    pub max_depth: usize,
    /// Maximum value nodes or multipart parts.
    pub max_entries: usize,
    /// Maximum header bytes within one multipart part.
    pub max_part_headers: usize,
}

impl Default for ParseLimits {
    /// Choose bounded defaults; callers can supply deliberate workload budgets.
    fn default() -> Self {
        // Use finite shared defaults; callers must explicitly select a larger bounded workload.
        Self {
            max_bytes: 1024 * 1024,
            max_depth: 64,
            max_entries: 10_000,
            max_part_headers: 8192,
        }
    }
}

/// Check a raw input before parsing or allocating its decoded representation.
pub(crate) fn check_input(bytes: usize, limits: &ParseLimits) -> crate::Result<()> {
    // Reject raw input before any decoded/candidate copy can multiply the buffered work.
    if bytes > limits.max_bytes {
        return Err(crate::Error::limit("input byte limit exceeded"));
    }
    Ok(())
}
