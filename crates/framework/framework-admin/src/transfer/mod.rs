//! G36 bounded CSV transfer: pure parsing/projection only; application hosts own authorization.
//!
//! ```
//! use stackpress_framework_admin::transfer::{Limits, parse};
//! let limits = Limits { rows: 25, ..Limits::default() };
//! let records = parse(b"id,name\na,Ada", limits)?;
//! assert_eq!(records[1], ["a", "Ada"]);
//! # Ok::<(), stackpress_framework_schema::SchemaError>(())
//! ```
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_framework_schema::{
    FieldIssue, SchemaError, SchemaErrorKind, SchemaResult, UiField, UiModel, ValueType,
};
mod csv;
pub use csv::{parse, write};

/// App-selected ceilings. Every entry may only be lowered from the G36 maximum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Limits {
    /// Import file / exported CSV bytes (UTF-8, including delimiters).
    pub bytes: usize,
    /// Data rows, excluding the one header row; also the atomic batch maximum.
    pub rows: usize,
    /// Import-only distinct public header columns.
    pub columns: usize,
    /// Import-only raw decoded CSV cell bytes before typed conversion.
    pub cell: usize,
}
impl Default for Limits {
    /// Fixed coordinator-approved maxima; applications can only select smaller nonzero ceilings.
    fn default() -> Self {
        Self {
            bytes: 1_048_576,
            rows: 100,
            columns: 100,
            cell: 65_536,
        }
    }
}
impl Limits {
    /// Refuse widening or unusable zero budgets before reading/allocating transfer data.
    /// # Errors
    /// Returns a safe configuration error for any invalid ceiling.
    pub fn validate(self) -> SchemaResult<Self> {
        let max = Self::default();
        if self.bytes == 0
            || self.bytes > max.bytes
            || self.rows == 0
            || self.rows > max.rows
            || self.columns == 0
            || self.columns > max.columns
            || self.cell == 0
            || self.cell > max.cell
        {
            return Err(invalid("invalid transfer limits"));
        }
        Ok(self)
    }
}
/// Strict upload request; confirmation resends the exact browser-owned file and is revalidated.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upload {
    /// UTF-8 CSV, never a server file path.
    pub csv: String,
    /// Explicit opt-in; ordinary imports preserve apostrophes literally.
    #[serde(default)]
    pub decode_export_guards: bool,
    /// Confirmation is explicit and cannot be supplied by the preview response.
    #[serde(default)]
    pub confirm: bool,
}
/// One row's public preview. All values are inert data, including raw guard forms.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    /// One-based CSV record number, with header record numbered one.
    pub number: usize,
    /// Raw decoded CSV cells before sentinel/guard conversion.
    pub raw: Vec<String>,
    /// Approved decoded creation members; omission is represented by absence.
    pub values: Value,
    /// Safe field/row failures; no SQL, rejected native cause or credentials.
    pub errors: Vec<FieldIssue>,
}
/// Every parsed row is present, in input order. No preview performs writes.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    /// Validated public wire names in file order.
    pub headers: Vec<String>,
    /// All bounded data records.
    pub rows: Vec<Row>,
}
impl Preview {
    /// A confirm is available only if every row passed conversion, contracts and conflict checks.
    pub fn valid(&self) -> bool {
        !self.rows.is_empty() && self.rows.iter().all(|row| row.errors.is_empty())
    }
}
/// A public error with no input value or native diagnostic text.
pub fn issue(field: Option<&str>, code: &str, message: &str) -> FieldIssue {
    FieldIssue {
        path: field.into_iter().map(str::to_owned).collect(),
        code: code.into(),
        message: message.into(),
    }
}
/// Select public stored fields, independently of roles and virtual related collections.
pub fn fields(model: &UiModel, import: bool) -> Vec<&UiField> {
    model
        .fields
        .iter()
        .filter(|f| !f.sensitive && if import { f.writable } else { f.readable })
        .filter(|f| !matches!(f.value_type.value, ValueType::Model(_)))
        .collect()
}
/// Parse all bounded rows, then check exact headers and convert cell values without a database.
/// # Errors
/// Refuses structural CSV/header/budget errors. Typed cell errors are retained per row.
pub fn preview(model: &UiModel, upload: &Upload, limits: Limits) -> SchemaResult<Preview> {
    let mut records = parse(upload.csv.as_bytes(), limits)?;
    let headers = records.remove(0);
    let allowed = fields(model, true);
    let mut seen = std::collections::BTreeSet::new();
    let mut columns = Vec::new();
    // Resolve exact public wire names before creating any typed row values.
    for name in &headers {
        if !seen.insert(name) {
            return Err(invalid("duplicate CSV header"));
        }
        columns.push(
            *allowed
                .iter()
                .find(|field| field.wire_name == *name)
                .ok_or_else(|| invalid("unknown or nonwritable CSV header"))?,
        );
    }
    let mut rows = Vec::new();
    for (index, raw) in records.into_iter().enumerate() {
        let mut values = serde_json::Map::new();
        let mut errors = Vec::new();
        // A mismatched record is retained as an invalid row; no extra cells are silently dropped.
        if raw.len() != headers.len() {
            errors.push(issue(None, "columns", "CSV row has inconsistent columns"));
        } else {
            for (field, cell) in columns.iter().zip(&raw) {
                match decode(field, cell, upload.decode_export_guards) {
                    Ok(Some(value)) => {
                        values.insert(field.wire_name.clone(), value);
                    }
                    Ok(None) => {}
                    Err(_) => errors.push(issue(
                        Some(&field.wire_name),
                        "cell",
                        "Invalid CSV value or required omission",
                    )),
                }
            }
        }
        rows.push(Row {
            number: index + 2,
            raw,
            values: Value::Object(values),
            errors,
        });
    }
    Ok(Preview { headers, rows })
}
/// Decode reserved presence and explicitly selected export guards without a truthiness fallback.
fn decode(field: &UiField, raw: &str, guards: bool) -> SchemaResult<Option<Value>> {
    let raw = if guards && (raw.starts_with("''") || raw.strip_prefix('\'').is_some_and(dangerous))
    {
        &raw[1..]
    } else {
        raw
    };
    // Presence tokens precede literal escaping, so a doubled backslash cannot become a token.
    if raw == "\\O" {
        return if field.input_required {
            Err(invalid("required CSV value omitted"))
        } else {
            Ok(None)
        };
    }
    if raw == "\\N" {
        return Ok(Some(Value::Null));
    }
    let literal = raw.starts_with("\\\\");
    let raw = if literal { &raw[1..] } else { raw };
    if !literal && raw.starts_with('\\') {
        return Err(invalid("literal CSV backslash must be escaped"));
    }
    // Scalar string-backed wire types stay canonical strings. Collections always use JSON.
    let value = if !field.value_type.multiple
        && matches!(
            field.value_type.value,
            ValueType::String
                | ValueType::Enum(_)
                | ValueType::Date
                | ValueType::Time
                | ValueType::Datetime
        ) {
        Value::String(raw.into())
    } else {
        serde_json::from_str(raw)
            .map_err(|cause| invalid("invalid typed CSV cell").with_cause(cause))?
    };
    Ok(Some(value))
}
/// Formula-leading strings are guarded. Original apostrophes/backslashes have distinct escapes.
fn encode(value: &Value) -> SchemaResult<String> {
    Ok(match value {
        Value::Null => "\\N".into(),
        Value::String(text) if text.starts_with('\\') => format!("\\{text}"),
        Value::String(text) if text.starts_with('\'') || dangerous(text) => format!("'{text}"),
        Value::String(text) => text.clone(),
        value => serde_json::to_string(value)
            .map_err(|cause| invalid("CSV value encoding failed").with_cause(cause))?,
    })
}
/// G36 string-only spreadsheet formula prefixes; JSON numbers retain their numeric wire form.
fn dangerous(text: &str) -> bool {
    text.starts_with(['=', '+', '-', '@', '\t', '\r'])
}
/// Export every selected public stored value. Missing projection members are errors, never omission.
/// # Errors
/// Refuses excess rows/bytes, malformed rows or incomplete public projections.
pub fn export(model: &UiModel, rows: &[Value], limits: Limits) -> SchemaResult<String> {
    let limits = limits.validate()?;
    if rows.len() > limits.rows {
        return Err(invalid("Export exceeds row limit; narrow the filter"));
    }
    // Descriptor order fixes the header and every value column independently of object key order.
    let columns = fields(model, false);
    if columns.is_empty() {
        return Err(invalid("CSV export has no public stored columns"));
    }
    let mut records = vec![columns.iter().map(|f| f.wire_name.clone()).collect()];
    for row in rows {
        let row = row.get("row").unwrap_or(row);
        records.push(
            columns
                .iter()
                .map(|f| {
                    row.get(&f.wire_name)
                        .ok_or_else(|| invalid("incomplete public export row"))
                        .and_then(encode)
                })
                .collect::<SchemaResult<Vec<_>>>()?,
        );
    }
    csv::write_export(&records, limits)
}
/// Safe parse/config context; concrete codec causes are retained only in the private error chain.
pub(crate) fn invalid(message: &'static str) -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, message)
}

/// Root-owned import progress. Unknown is terminal for this mounted form and never retryable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    /// No request has been submitted for the selected draft.
    Idle,
    /// Read-only parsing, contract validation and conflict checks are pending.
    Previewing,
    /// Preview returned, with every row's errors available for review.
    Previewed,
    /// One explicitly confirmed atomic batch is awaiting acknowledgement.
    Applying,
    /// Acknowledged count from the actual generated batch result.
    Imported(usize),
    /// Safe known failure message; the browser retains the file draft.
    Failed(String),
    /// A write may have taken effect; no imported claim or automatic retry is permitted.
    Unknown,
}
impl Phase {
    /// Pending work or uncertainty blocks another request and file/encoding changes.
    pub fn locked(&self) -> bool {
        matches!(self, Self::Previewing | Self::Applying | Self::Unknown)
    }
    /// Public progress never substitutes a saved/imported claim for an uncertain acknowledgement.
    pub fn message(&self) -> String {
        match self {
            Self::Idle => "Choose a CSV file, then preview. Nothing has been written.".into(),
            Self::Previewing => "Validating every row; nothing is being written…".into(),
            Self::Previewed => "Preview complete; nothing has been written.".into(),
            Self::Applying => "Applying one atomic batch…".into(),
            Self::Imported(count) => format!("Imported {count} rows from the submitted file."),
            Self::Failed(message) => message.clone(),
            Self::Unknown => concat!(
                "Write outcome is unknown. Rows may have been imported. ",
                "Verify the records before starting a new import; no automatic retry."
            )
            .into(),
        }
    }

    /// Interpret only an acknowledged import count; malformed or lost write replies stay unknown.
    pub fn reply(write: bool, status: u16, envelope: &Value) -> Self {
        // Explicit uncertainty always wins, even if a malformed transport labels the reply OK.
        if write && envelope.pointer("/errors/outcome").and_then(Value::as_str) == Some("unknown") {
            return Self::Unknown;
        }
        if (200..300).contains(&status) {
            // A contradictory/missing public status cannot acknowledge a submitted write.
            if !envelope
                .get("code")
                .and_then(Value::as_u64)
                .is_some_and(|code| (200..300).contains(&code))
            {
                return if write {
                    Self::Unknown
                } else {
                    Self::Failed("Invalid preview acknowledgement; draft preserved".into())
                };
            }
            if !write {
                return Self::Previewed;
            }
            return envelope
                .pointer("/results/imported")
                .and_then(Value::as_u64)
                .filter(|count| (1..=100).contains(count))
                .and_then(|n| usize::try_from(n).ok())
                .map(Self::Imported)
                .unwrap_or(Self::Unknown);
        }
        Self::Failed(
            envelope
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("Import request failed; draft preserved")
                .chars()
                .take(512)
                .collect(),
        )
    }
}
