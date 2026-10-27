//! Strict one-decode URL state; changing a filter never executes an operation or navigation.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stackpress_framework_schema::{
    SchemaError, SchemaErrorKind, SchemaResult, SortDirection, UiModel,
};
use std::collections::BTreeMap;

/// Browser-owned search state, checked again against actual generated query contracts by hosts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SearchQuery {
    /// Literal search text, never SQL or an expression.
    pub q: Option<String>,
    /// Exact scalar equality filters; null means remove the filter.
    pub filters: BTreeMap<String, Value>,
    /// One explicitly selected sort; the cycle is absent, descending, ascending, absent.
    pub order: Option<(String, SortDirection)>,
    /// Bounded offset in rows.
    pub skip: u64,
    /// Page size, 1 through 100.
    pub limit: u16,
}
impl Default for SearchQuery {
    /// Begin at the first twenty rows without a filter or order.
    fn default() -> Self {
        Self {
            q: None,
            filters: BTreeMap::new(),
            order: None,
            skip: 0,
            limit: 20,
        }
    }
}
impl SearchQuery {
    /// Replace an equality filter; explicit null removes it while false/zero stay concrete.
    pub fn filter(&mut self, field: &str, value: Value) {
        if value.is_null() {
            self.filters.remove(field);
        } else {
            self.filters.insert(field.into(), value);
        }
        self.skip = 0;
    }
    /// Cycle one order field without any ambient URL mutation.
    pub fn cycle_order(&mut self, field: &str) {
        self.order = match &self.order {
            Some((selected, SortDirection::Desc)) if selected == field => {
                Some((field.into(), SortDirection::Asc))
            }
            Some((selected, SortDirection::Asc)) if selected == field => None,
            _ => Some((field.into(), SortDirection::Desc)),
        };
        self.skip = 0;
    }
    /// Validate names, scalar values and query budgets before building the actual query envelope.
    /// # Errors
    /// Refuses private/unsupported fields, oversized values and invalid paging.
    pub fn operation(&self, model: &UiModel) -> SchemaResult<Value> {
        stackpress_framework_schema::query_budget(
            self.q.as_deref(),
            self.filters.len(),
            usize::from(self.order.is_some()),
            &[],
            self.skip,
            self.limit,
        )?;
        if self.q.is_some()
            && !model
                .fields
                .iter()
                .any(|field| field.searchable && field.readable)
        {
            return Err(invalid("search is unavailable"));
        }
        let mut filters = Vec::new();
        for (name, value) in &self.filters {
            let field = model
                .fields
                .iter()
                .find(|field| {
                    field.wire_name == *name
                        && field.filterable
                        && field.readable
                        && !field.sensitive
                })
                .ok_or_else(|| invalid("filter is unavailable"))?;
            // Null removes a URL filter; explicit SQL-null predicates are not inferred.
            if value.is_null() {
                continue;
            }
            stackpress_framework_schema::Predicate::Eq(value.clone()).validate(
                &field.value_type.value,
                false,
                field.nullable,
            )?;
            filters.push(json!({"field":name,"predicate":{"op":"eq","value":value}}));
        }
        let mut sort = Vec::new();
        if let Some((name, direction)) = &self.order {
            if !model.fields.iter().any(|field| {
                field.wire_name == *name && field.sortable && field.readable && !field.sensitive
            }) {
                return Err(invalid("order is unavailable"));
            }
            sort.push(json!({"field":name,"direction":direction,"nulls":"last"}));
        }
        Ok(
            json!({"q":self.q,"filters":filters,"sort":sort,"includes":[],"skip":self.skip,"take":self.limit,"total":true}),
        )
    }
    /// Encode one bounded JSON parameter with byte-level percent encoding.
    /// # Errors
    /// Refuses state that exceeds the finite URL budget.
    pub fn query(&self) -> SchemaResult<String> {
        let json = serde_json::to_string(self)
            .map_err(|cause| invalid("search encoding failed").with_cause(cause))?;
        if json.len() > 4096 {
            return Err(invalid("search URL budget exceeded"));
        }
        Ok(format!("state={}", encode(&json)))
    }
    /// Decode only a single state parameter, once; callers validate it against their model.
    /// # Errors
    /// Refuses duplicates, unknown parameters, malformed escapes/UTF-8/JSON and oversized input.
    pub fn from_query(query: &str) -> SchemaResult<Self> {
        if query.is_empty() {
            return Ok(Self::default());
        }
        let raw = query
            .strip_prefix("state=")
            .filter(|raw| !raw.contains('&') && raw.len() <= 12_288)
            .ok_or_else(|| invalid("invalid search URL"))?;
        let json = decode(raw)?;
        if json.len() > 4096 {
            return Err(invalid("search URL budget exceeded"));
        }
        serde_json::from_str(&json)
            .map_err(|cause| invalid("invalid search state").with_cause(cause))
    }
}

/// Encode UTF-8 bytes as inert URL data, preserving only RFC unreserved ASCII.
pub fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

/// Decode percent escapes exactly once; plus is not silently converted into a space.
/// # Errors
/// Refuses malformed escapes, raw reserved characters and invalid UTF-8.
pub fn decode(value: &str) -> SchemaResult<String> {
    if value.len() > 12_288 {
        return Err(invalid("URL value budget exceeded"));
    }
    let bytes = value.as_bytes();
    let mut result = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' {
            let end = index
                .checked_add(3)
                .filter(|end| *end <= bytes.len())
                .ok_or_else(|| invalid("invalid URL escape"))?;
            let part = std::str::from_utf8(&bytes[index + 1..end])
                .map_err(|cause| invalid("invalid URL escape").with_cause(cause))?;
            result.push(
                u8::from_str_radix(part, 16)
                    .map_err(|cause| invalid("invalid URL escape").with_cause(cause))?,
            );
            index = end;
        } else if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            result.push(byte);
            index += 1;
        } else {
            return Err(invalid("invalid URL character"));
        }
    }
    String::from_utf8(result).map_err(|cause| invalid("invalid URL text").with_cause(cause))
}

/// Preserve a safe typed context without echoing the rejected query.
fn invalid(context: &'static str) -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, context)
}
