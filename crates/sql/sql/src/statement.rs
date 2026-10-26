//! Bounded lexical validation for explicitly trusted ordinary statements.
//!
//! This gate rejects batches, transaction/connection control and dialect-specific
//! escape hazards. It is not a SQL grammar parser or a replacement for author trust
//! and native prepared-parameter validation. All state is local to one rendering.

use crate::{DialectId, ErrorKind, QueryLimits, Result, SqlError};

/// Construct a safe lexical/identifier failure without echoing caller input.
fn invalid_sql() -> SqlError {
    SqlError::new(
        ErrorKind::InvalidQuery,
        "invalid SQL statement or identifier",
    )
}

/// Admit one statement using the selected dialect's bounded lexical subset.
pub(crate) fn validate(sql: &str, dialect: DialectId) -> Result<()> {
    let bytes = sql.as_bytes();
    if bytes.contains(&0) {
        return Err(invalid_sql());
    }
    let mut index = 0;
    let mut ended = false;
    let mut first_word = None;
    while index < bytes.len() {
        let byte = bytes[index];
        // Whitespace/comments remain legal after the final statement terminator.
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if (bytes[index..].starts_with(b"--")
            && (dialect != DialectId::Mysql
                || bytes.get(index + 2).is_none_or(u8::is_ascii_whitespace)))
            || (dialect == DialectId::Mysql && byte == b'#')
        {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            if dialect == DialectId::Mysql && bytes.get(index + 2) == Some(&b'!') {
                return Err(invalid_sql());
            }
            index += 2;
            while index + 1 < bytes.len() && !bytes[index..].starts_with(b"*/") {
                index += 1;
            }
            if index + 1 >= bytes.len() {
                return Err(invalid_sql());
            }
            index += 2;
            continue;
        }
        // A second executable token is a batch, even if no whitespace separates it.
        if ended || byte == 0 {
            return Err(invalid_sql());
        }
        if byte == b';' {
            ended = true;
            index += 1;
            continue;
        }
        // Every string slice begins/ends at an ASCII delimiter or digit boundary;
        // advancing through UTF-8 continuation bytes never creates an unchecked str slice.
        if dialect == DialectId::Postgres && byte == b'$' {
            let tail = &sql[index + 1..];
            let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
            if digits > 0 {
                let ordinal = tail[..digits].parse::<usize>().map_err(|_| invalid_sql())?;
                if ordinal == 0 || ordinal > QueryLimits::default().binds {
                    return Err(invalid_sql());
                }
                index += digits + 1;
                continue;
            }
            // Dollar-quoted literals retain semicolons/question marks without replacement.
            let tag_end = tail.find('$').ok_or_else(invalid_sql)?;
            if !tail[..tag_end]
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return Err(invalid_sql());
            }
            let delimiter = &sql[index..index + tag_end + 2];
            let body = index + delimiter.len();
            let end = sql[body..].find(delimiter).ok_or_else(invalid_sql)?;
            index = body + end + delimiter.len();
            continue;
        }
        if dialect == DialectId::Postgres && bytes[index..].starts_with(b"::") {
            index += 2;
            continue;
        }
        if dialect == DialectId::Postgres && byte == b'?' {
            return Err(invalid_sql());
        }
        if matches!(byte, b'\'' | b'"' | b'`' | b'[') {
            let close = if byte == b'[' { b']' } else { byte };
            index += 1;
            loop {
                if index >= bytes.len() || bytes[index] == 0 {
                    return Err(invalid_sql());
                }
                if dialect != DialectId::Sqlite && bytes[index] == b'\\' {
                    return Err(SqlError::new(
                        ErrorKind::Unsupported,
                        "raw network SQL backslash quoting is outside the lexical subset; bind data instead",
                    ));
                }
                if bytes[index] == close {
                    index += 1;
                    // Bracket identifiers terminate at the first right bracket.
                    if close != b']' && bytes.get(index) == Some(&close) {
                        index += 1;
                        continue;
                    }
                    break;
                }
                index += 1;
            }
            continue;
        }
        if matches!(byte, b':' | b'@' | b'$')
            || (byte == b'?' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit))
        {
            return Err(SqlError::new(
                ErrorKind::Unsupported,
                "only anonymous SQL parameters are supported",
            ));
        }
        if byte.is_ascii_alphabetic() {
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            first_word.get_or_insert(&sql[start..index]);
            continue;
        }
        index += 1;
    }
    // Connection policy and transaction control cannot be bypassed through raw SQL.
    let word = first_word.ok_or_else(invalid_sql)?.to_ascii_uppercase();
    if !matches!(
        word.as_str(),
        "SELECT"
            | "WITH"
            | "INSERT"
            | "UPDATE"
            | "DELETE"
            | "CREATE"
            | "DROP"
            | "ALTER"
            | "EXPLAIN"
    ) {
        return Err(SqlError::new(
            ErrorKind::Unsupported,
            "SQL command is outside the ordinary statement API",
        ));
    }
    Ok(())
}
