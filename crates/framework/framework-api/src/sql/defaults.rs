//! Source generated query defaults used by detail/get/search and management consumers.
use crate::models::{ModelKind, Record, StoreQuery, Value};
/// Source model @query columns, including plaintext credentials and joined Profile.
pub fn columns<M: ModelKind>() -> Vec<String> {
    let mut columns = vec!["*".into(), "profile.*".into()];
    if M::NAME == "Session" {
        columns.push("application.*".into());
    }
    columns
}
/// Extract the generated source query's known fields; unrelated request data is ignored.
/// Typed paging is a Rust mechanics boundary, not a new public data allowlist.
pub fn query(data: &Record) -> crate::models::Result<StoreQuery> {
    let map = |name: &str| {
        data.get(name)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
    };
    Ok(StoreQuery {
        q: data.get("q").cloned(),
        columns: selected(data.get("columns")).unwrap_or_else(|| vec!["*".into()]),
        eq: map("eq"),
        ne: map("ne"),
        ge: map("ge"),
        le: map("le"),
        like: map("like"),
        has: map("has"),
        hasnt: map("hasnt"),
        sort: map("sort"),
        skip: paging(data.get("skip"), 0)?,
        take: paging(data.get("take"), 50)?,
    })
}
/// Source Array.isArray/every guard accepts an empty selector array.
pub fn selected(value: Option<&Value>) -> Option<Vec<String>> {
    value?
        .as_array()?
        .iter()
        .map(|value| value.as_str().filter(|s| !s.is_empty()).map(str::to_owned))
        .collect()
}

/// Existing SQL builders require u64 paging. Missing selects source defaults; falsy
/// present values select no offset/limit. Integral JSON floats are ordinary JS numbers.
fn paging(value: Option<&Value>, default: u64) -> crate::models::Result<u64> {
    let Some(value) = value else {
        return Ok(default);
    };
    if value.is_null() || value.as_bool() == Some(false) || value.as_str() == Some("") {
        return Ok(0);
    }
    if let Value::Number(number) = value {
        if let Some(number) = number.as_u64() {
            return Ok(number);
        }
        if let Some(number) = number
            .as_f64()
            .filter(|number| *number >= 0.0 && number.fract() == 0.0 && *number < u64::MAX as f64)
        {
            return Ok(number as u64);
        }
    }
    // This native representation failure is a Rust SQL mechanics limit, not a new
    // maximum page size or a silent fallback that changes the source selection.
    Err(crate::models::ActionError::new(
        500,
        "paging cannot be represented as a native nonnegative integer",
        Record::new(),
    ))
}
