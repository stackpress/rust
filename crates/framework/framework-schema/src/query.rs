//! Pure query contracts. Validation grants no operation capability and performs no database work.

use crate::{SchemaError, SchemaErrorKind, SchemaResult, ValueType};
use serde::{Deserialize, Serialize};

/// Typed scalar/list predicate selected by a generated field enum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "op",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Predicate<T> {
    /// Exact equality; null is represented by IsNull separately.
    Eq(T),
    /// Exact inequality.
    Ne(T),
    /// Greater than or equal, for ordered scalars.
    Ge(T),
    /// Less than or equal, for ordered scalars.
    Le(T),
    /// Explicit text pattern, bound as data by future execution.
    Like(T),
    /// Direct scalar JSON-array membership.
    Has(T),
    /// Direct scalar JSON-array nonmembership.
    Hasnt(T),
    /// SQL NULL / IS NOT NULL, independent of JSON null or missing paths.
    IsNull(bool),
}
impl<T: Serialize> Predicate<T> {
    /// Check type/operator applicability and public-wire budgets before future compilation.
    /// # Errors
    /// Refuses inappropriate operators, nullable misuse, lossy browser integers and oversized values.
    pub fn validate(&self, kind: &ValueType, multiple: bool, nullable: bool) -> SchemaResult<()> {
        let invalid = || {
            SchemaError::new(
                SchemaErrorKind::Invalid,
                "query predicate is incompatible with field",
            )
        };
        let scalar = match self {
            Self::IsNull(_) => return if nullable { Ok(()) } else { Err(invalid()) },
            Self::Has(value) | Self::Hasnt(value) if multiple => value,
            Self::Eq(value) | Self::Ne(value) if !multiple => value,
            Self::Ge(value) | Self::Le(value)
                if !multiple && !matches!(kind, ValueType::Boolean) =>
            {
                value
            }
            Self::Like(value) if !multiple && *kind == ValueType::String => value,
            _ => return Err(invalid()),
        };
        let value = crate::to_bounded_value(scalar)?;
        crate::check_browser_json(&value)?;
        if !crate::value_matches(kind, &value)
            || value.as_str().is_some_and(|value| value.len() > 4096)
        {
            return Err(invalid());
        }
        Ok(())
    }
}
/// Explicit ordering direction; unknown wire values never fall back to a default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    /// Increasing scalar order.
    Asc,
    /// Decreasing scalar order.
    Desc,
}
/// Explicit null placement, independent of a database's default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NullOrder {
    /// Nulls precede non-null values.
    First,
    /// Nulls follow non-null values.
    Last,
}
/// A generated selector plus explicit order/null policy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sort<T> {
    /// A generated allowlisted field selection.
    pub field: T,
    /// Explicit direction.
    pub direction: SortDirection,
    /// Explicit null policy.
    pub nulls: NullOrder,
}
/// Validate bounded query envelopes independently of SQL or native adapters.
/// # Errors
/// Refuses more than 100 filters, 16 sorts/includes, text over 4096 bytes, invalid pages or include depth.
pub fn query_budget(
    q: Option<&str>,
    filters: usize,
    sorts: usize,
    includes: &[String],
    skip: u64,
    take: u16,
) -> SchemaResult<()> {
    if !(1..=100).contains(&take)
        || skip > 100_000
        || filters > 100
        || sorts > 16
        || includes.len() > 16
        || q.is_some_and(|value| value.len() > 4096)
        || includes.iter().any(|path| {
            path.len() > 1024 || path.split('.').count() > 3 || path.split('.').any(str::is_empty)
        })
    {
        return Err(SchemaError::new(
            SchemaErrorKind::Limit,
            "query budget exceeded",
        ));
    }
    Ok(())
}
/// Check an include against public readable relation paths, bounded by the query envelope.
/// # Errors
/// Refuses unknown/private/scalar paths; no lookup URL or database access is inferred.
pub fn check_include(schema: &crate::UiSchema, model: &str, path: &str) -> SchemaResult<()> {
    let invalid = || SchemaError::new(SchemaErrorKind::Invalid, "include path is unavailable");
    let mut model = schema
        .models
        .iter()
        .find(|item| item.id == model)
        .ok_or_else(invalid)?;
    for part in path.split('.') {
        let relation = model
            .fields
            .iter()
            .find(|field| field.wire_name == part && field.readable)
            .and_then(|field| field.relation.as_ref())
            .ok_or_else(invalid)?;
        model = schema
            .models
            .iter()
            .find(|model| model.id == relation.target)
            .ok_or_else(invalid)?;
    }
    Ok(())
}
