//! Conservative lexical reuse policy without a driver fork or a general session reset.
//!
//! Query has no provenance marker. Only plain DML/SELECT and explicitly safe builtin
//! calls qualify. This does not inspect server-side trigger/routine definitions:
//! applications must not attach session-mutating triggers to reusable DML. Unknown
//! calls, CTEs, DDL and session/administrative constructs retire their lease.
use crate::execute::driver_error;
use sqlx::{Executor, MySqlConnection};
use stackpress_sql::{Query, Result};

/// Classify the already validated SQL without guessing provenance or interpolating tokens.
pub(crate) fn allowed(query: &Query) -> bool {
    allowed_sql(query.sql())
}
/// Admit only a bounded ordinary subset; quoted text/comments cannot hide executable tokens.
pub(crate) fn allowed_sql(sql: &str) -> bool {
    let bytes = sql.as_bytes();
    let mut index = 0;
    let mut command = None;
    let mut insert_target = false;
    let mut previous_dot = false;
    while index < bytes.len() {
        let Some(next) = skip_trivia(bytes, index) else {
            return false;
        };
        index = next;
        if index == bytes.len() {
            break;
        }
        let byte = bytes[index];
        // Variables/control syntax cannot be hidden among ordinary statement tokens.
        if byte == b'@' || byte == 0 {
            return false;
        }
        if byte == b';' {
            return command.is_some() && skip_trivia(bytes, index + 1) == Some(bytes.len());
        }
        if matches!(byte, b'\'' | b'"' | b'`' | b'[') {
            let Some(end) = quoted_end(bytes, index) else {
                return false;
            };
            index = end;
            let Some(next) = skip_trivia(bytes, index) else {
                return false;
            };
            if bytes.get(next) == Some(&b'(') && !insert_target {
                return false;
            }
            // A quoted INSERT table may be schema-qualified; it remains a table, not a call.
            insert_target = insert_target && bytes.get(next) == Some(&b'.');
            previous_dot = false;
        } else if byte.is_ascii_alphabetic() || byte == b'_' {
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            let word = &sql[start..index];
            if command.is_none() {
                if !["SELECT", "INSERT", "UPDATE", "DELETE"]
                    .iter()
                    .any(|known| word.eq_ignore_ascii_case(known))
                {
                    return false;
                }
                command = Some(word);
            }
            let update = command.is_some_and(|command| command.eq_ignore_ascii_case("UPDATE"));
            if (word.eq_ignore_ascii_case("SET") && !update)
                || [
                    "TEMPORARY",
                    "LOCK",
                    "UNLOCK",
                    "HANDLER",
                    "PREPARE",
                    "EXECUTE",
                    "DEALLOCATE",
                    "OUTFILE",
                    "DUMPFILE",
                    "PROCEDURE",
                    "SQL_CALC_FOUND_ROWS",
                ]
                .iter()
                .any(|token| word.eq_ignore_ascii_case(token))
            {
                return false;
            }
            let Some(next) = skip_trivia(bytes, index) else {
                return false;
            };
            if bytes.get(next) == Some(&b'(') && !insert_target {
                let qualified = previous_dot;
                if qualified || !safe_call(word) {
                    return false;
                }
            }
            // Only INSERT's target identifier may precede a column-list parenthesis.
            insert_target = (insert_target && bytes.get(next) == Some(&b'.'))
                || (command.is_some_and(|command| command.eq_ignore_ascii_case("INSERT"))
                    && word.eq_ignore_ascii_case("INTO"));
            previous_dot = false;
        } else {
            previous_dot = byte == b'.';
            index += 1;
        }
    }
    command.is_some()
}
/// Skip all core-admitted comment forms before classifying the next executable token.
fn skip_trivia(bytes: &[u8], mut index: usize) -> Option<usize> {
    loop {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if bytes[index..].starts_with(b"/*") {
            if bytes.get(index + 2) == Some(&b'!') {
                return None;
            }
            index += 2;
            while index + 1 < bytes.len() && !bytes[index..].starts_with(b"*/") {
                index += 1;
            }
            if index + 1 >= bytes.len() {
                return None;
            }
            index += 2;
        } else if bytes.get(index) == Some(&b'#')
            || (bytes[index..].starts_with(b"--")
                && bytes.get(index + 2).is_none_or(u8::is_ascii_whitespace))
        {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else {
            return Some(index);
        }
    }
}
/// Preserve core quote semantics; ambiguous/backslash/unclosed tokens retire.
fn quoted_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    let close = if bytes[index] == b'[' {
        b']'
    } else {
        bytes[index]
    };
    index += 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            return None;
        }
        if bytes[index] == close {
            index += 1;
            if close != b']' && bytes.get(index) == Some(&close) {
                index += 1;
            } else {
                return Some(index);
            }
        } else {
            index += 1;
        }
    }
    None
}
/// Explicit pure builtin/SQL grouping forms; unknown or session-state calls retire.
fn safe_call(word: &str) -> bool {
    [
        "VALUES",
        "IN",
        "EXISTS",
        "AS",
        "CAST",
        "CONVERT",
        "COUNT",
        "SUM",
        "MIN",
        "MAX",
        "AVG",
        "ABS",
        "COALESCE",
        "NULLIF",
        "IFNULL",
        "IF",
        "CONCAT",
        "LOWER",
        "UPPER",
        "LENGTH",
        "CHAR_LENGTH",
        "JSON_EXTRACT",
        "JSON_UNQUOTE",
        "JSON_CONTAINS",
        "JSON_ARRAY",
        "JSON_OBJECT",
        "CONNECTION_ID",
        "VERSION",
        "DATABASE",
    ]
    .iter()
    .any(|safe| word.eq_ignore_ascii_case(safe))
}
/// End any ordinary transaction before acknowledged allowlisted reuse.
pub(crate) async fn reset(connection: &mut MySqlConnection) -> Result<()> {
    // No general MySQL reset API is exposed by SQLx 0.9. Do not pretend this clears
    // unknown session variables/temp tables: only allowlisted clean leases reach here.
    connection.execute("ROLLBACK").await.map_err(driver_error)?;
    Ok(())
}
