//! Strict bounded RFC-style CSV; LF and CRLF records, embedded quoted line breaks, no recovery.
use super::{Limits, invalid};
use stackpress_framework_schema::SchemaResult;

/// Exact quote state; only delimiters may follow a closed quoted cell.
#[derive(Clone, Copy, PartialEq)]
enum State {
    Start,
    Plain,
    Quoted,
    Closed,
}
/// Validate byte/cell/column/row budgets before each allocation or append.
/// # Errors
/// Rejects malformed UTF-8, quote transitions, lone record CRs and structural/budget excess.
pub fn parse(bytes: &[u8], limits: Limits) -> SchemaResult<Vec<Vec<String>>> {
    let limits = limits.validate()?;
    if bytes.len() > limits.bytes {
        return Err(invalid("CSV file byte limit exceeded"));
    }
    std::str::from_utf8(bytes).map_err(|cause| invalid("CSV must be UTF-8").with_cause(cause))?;
    let mut records = Vec::new();
    let mut row = Vec::new();
    let mut cell = Vec::new();
    let mut state = State::Start;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        // Once the header and allowed rows are complete, refuse before allocating another cell.
        if records.len() > limits.rows {
            return Err(invalid("CSV row limit exceeded"));
        }
        // Inside quotes every byte is data except a doubled or closing quote.
        if state == State::Quoted {
            if byte == b'"' {
                if bytes.get(index + 1) == Some(&b'"') {
                    push(&mut cell, b'"', limits)?;
                    index += 1;
                } else {
                    state = State::Closed;
                }
            } else {
                push(&mut cell, byte, limits)?;
            }
        } else if byte == b',' || byte == b'\n' || byte == b'\r' {
            finish_cell(&mut row, &mut cell, limits)?;
            state = State::Start;
            if byte != b',' {
                if byte == b'\r' {
                    if bytes.get(index + 1) != Some(&b'\n') {
                        return Err(invalid("CSV record CR requires LF"));
                    }
                    index += 1;
                }
                finish_row(&mut records, &mut row, limits)?;
            }
        } else if byte == b'"' && state == State::Start {
            state = State::Quoted;
        } else {
            if byte == b'"' || state == State::Closed {
                return Err(invalid("invalid CSV quote transition"));
            }
            push(&mut cell, byte, limits)?;
            state = State::Plain;
        }
        index += 1;
    }
    if state == State::Quoted {
        return Err(invalid("unterminated CSV quoted cell"));
    }
    if state != State::Start || !row.is_empty() || !cell.is_empty() {
        finish_cell(&mut row, &mut cell, limits)?;
        finish_row(&mut records, &mut row, limits)?;
    }
    if records.len() < 2 {
        return Err(invalid("CSV needs a header and at least one data row"));
    }
    Ok(records)
}
/// Admit one data byte before growing the current cell.
fn push(cell: &mut Vec<u8>, byte: u8, limits: Limits) -> SchemaResult<()> {
    if cell.len() >= limits.cell {
        return Err(invalid("CSV cell byte limit exceeded"));
    }
    cell.push(byte);
    Ok(())
}
/// Transfer a validated UTF-8 cell into a column slot without making a second data copy.
fn finish_cell(row: &mut Vec<String>, cell: &mut Vec<u8>, limits: Limits) -> SchemaResult<()> {
    if row.len() >= limits.columns {
        return Err(invalid("CSV column limit exceeded"));
    }
    row.push(
        String::from_utf8(std::mem::take(cell))
            .map_err(|cause| invalid("CSV cell must be UTF-8").with_cause(cause))?,
    );
    Ok(())
}
/// Keep the header plus at most the configured number of complete data records.
fn finish_row(
    records: &mut Vec<Vec<String>>,
    row: &mut Vec<String>,
    limits: Limits,
) -> SchemaResult<()> {
    if records.len() > limits.rows {
        return Err(invalid("CSV row limit exceeded"));
    }
    records.push(std::mem::take(row));
    Ok(())
}
/// Deterministic CRLF output, quoting every cell and doubling quotes; charge before append.
/// # Errors
/// Refuses configured bounds instead of returning a partial CSV.
pub fn write(records: &[Vec<String>], limits: Limits) -> SchemaResult<String> {
    write_records(records, limits, true)
}
/// G36 exports have row/byte ceilings; import-only column and cell ceilings do not narrow them.
pub(super) fn write_export(records: &[Vec<String>], limits: Limits) -> SchemaResult<String> {
    write_records(records, limits, false)
}
/// Share escaping and output charging, applying extra import budgets only for ordinary records.
fn write_records(
    records: &[Vec<String>],
    limits: Limits,
    import_limits: bool,
) -> SchemaResult<String> {
    let limits = limits.validate()?;
    let mut out = String::new();
    if records.is_empty() || records.len() > limits.rows + 1 {
        return Err(invalid("CSV row limit exceeded"));
    }
    for row in records {
        if row.is_empty() || (import_limits && row.len() > limits.columns) {
            return Err(invalid("CSV column limit exceeded"));
        }
        for (index, cell) in row.iter().enumerate() {
            if import_limits && cell.len() > limits.cell {
                return Err(invalid("CSV cell byte limit exceeded"));
            }
            if index > 0 {
                append(&mut out, ",", limits)?;
            }
            append(&mut out, "\"", limits)?;
            for part in cell.split_inclusive('"') {
                append(&mut out, part, limits)?;
                if part.ends_with('"') {
                    append(&mut out, "\"", limits)?;
                }
            }
            append(&mut out, "\"", limits)?;
        }
        append(&mut out, "\r\n", limits)?;
    }
    Ok(out)
}
/// Charge output bytes before append; callers discard all partial output on failure.
fn append(out: &mut String, text: &str, limits: Limits) -> SchemaResult<()> {
    if text.len() > limits.bytes.saturating_sub(out.len()) {
        return Err(invalid("CSV output byte limit exceeded; narrow the filter"));
    }
    out.push_str(text);
    Ok(())
}
