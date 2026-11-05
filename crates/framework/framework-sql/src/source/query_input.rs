//! Source query carrier and the shared native paging/selector representation boundary.
use super::SourceResult as Result;
use serde::{Deserialize, Serialize};
use stackpress_framework_schema::source::{Record, Value};
/// Source select/filter envelope. Empty and missing maps have the same source defaults.
///
/// The generated source carrier retains arbitrary native q values until JS-style interpolation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StoreQuery {
    /// Source contains-search; Application declares name and website; Session ignores it.
    pub q: Option<Value>,
    /// Requested source paths; defaults to all base columns.
    pub columns: Vec<String>,
    /// One equal value per field (G53); active=-1 removes the implicit active predicate.
    pub eq: Record,
    /// Not-equal values; nested maps and arrays retain source index-path flattening.
    pub ne: Record,
    /// Greater-or-equal values.
    pub ge: Record,
    /// Less-or-equal values.
    pub le: Record,
    /// Contains-search values; % and _ retain SQL wildcard semantics.
    pub like: Record,
    /// JSON-array membership; every requested value must be present (G53).
    pub has: Record,
    /// Source currently compiles this as membership too, preserved deliberately.
    pub hasnt: Record,
    /// Source ASC/DESC map, case insensitive; invalid directions are skipped.
    pub sort: Record,
    /// Source offset.
    pub skip: u64,
    /// Source default 50; zero means unlimited.
    pub take: u64,
}
impl Default for StoreQuery {
    /// Preserve original generated select defaults rather than the framework public-query cap.
    fn default() -> Self {
        Self {
            q: None,
            columns: vec!["*".into()],
            eq: Record::new(),
            ne: Record::new(),
            ge: Record::new(),
            le: Record::new(),
            like: Record::new(),
            has: Record::new(),
            hasnt: Record::new(),
            sort: Record::new(),
            skip: 0,
            take: 50,
        }
    }
}
impl StoreQuery {
    /// Construct a focused equal-field query, retaining all ordinary defaults.
    pub fn eq(name: impl Into<String>, value: impl Into<Value>) -> Self {
        let mut query = Self::default();
        query.eq.insert(name.into(), value.into());
        query
    }
    /// Explicitly include inactive rows, as the original eq.active=-1 sentinel does.
    pub fn all(mut self) -> Self {
        self.eq.insert("active".into(), (-1).into());
        self
    }
}

impl StoreQuery {
    /// Extract the generated source query's known fields; unrelated request data is ignored.
    /// Typed paging is a Rust mechanics boundary, not a new public data allowlist.
    pub fn from_record(data: &Record) -> Result<StoreQuery> {
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
}
/// Source Array.isArray/every guard accepts an empty selector array.
pub(super) fn selected(value: Option<&Value>) -> Option<Vec<String>> {
    value?
        .as_array()?
        .iter()
        .map(|value| value.as_str().filter(|s| !s.is_empty()).map(str::to_owned))
        .collect()
}

/// Existing SQL builders require u64 paging. Missing selects source defaults; falsy
/// present values select no offset/limit. Integral JSON floats are ordinary JS numbers.
fn paging(value: Option<&Value>, default: u64) -> Result<u64> {
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
    Err(super::ActionError::new(
        500,
        "paging cannot be represented as a native nonnegative integer",
        Record::new(),
    ))
}

impl StoreQuery {
    /// Transfer a query to another generated store without JSON-erasing native Dates.
    /// Missing q remains missing; explicit null and insertion order are retained.
    pub fn to_record(&self) -> Record {
        let mut record = Record::new();
        if let Some(q) = &self.q {
            record.insert("q".into(), q.clone());
        }
        record.insert(
            "columns".into(),
            Value::Array(self.columns.iter().cloned().map(Value::String).collect()),
        );
        for (name, map) in [
            ("eq", &self.eq),
            ("ne", &self.ne),
            ("ge", &self.ge),
            ("le", &self.le),
            ("like", &self.like),
            ("has", &self.has),
            ("hasnt", &self.hasnt),
            ("sort", &self.sort),
        ] {
            record.insert(name.into(), Value::Object(map.clone()));
        }
        record.insert("skip".into(), self.skip.into());
        record.insert("take".into(), self.take.into());
        record
    }
}
