//! Checked source, Rust and SQL identity spelling; no IO or mutable global names.

use super::invalid;
use crate::SchemaResult;
use sha2::{Digest, Sha256};

/// Convert source identity to a portable bounded module/SQL name, refusing injection.
pub fn portable_name(source: &str) -> SchemaResult<String> {
    if source.is_empty()
        || source.len() > 512
        || source.contains("__")
        || !source
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        || !source.as_bytes()[0].is_ascii_alphabetic()
    {
        return Err(invalid("source name is not portable"));
    }
    let mut result = String::new();
    let bytes = source.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if byte.is_ascii_uppercase()
            && index > 0
            && (bytes[index - 1].is_ascii_lowercase()
                || (index + 1 < bytes.len() && bytes[index + 1].is_ascii_lowercase()))
        {
            result.push('_');
        }
        result.push((*byte as char).to_ascii_lowercase());
    }
    if is_keyword(&result) {
        result.push('_');
    }
    if result.len() > 63 {
        let digest = format!("{:x}", Sha256::digest(source.as_bytes()));
        result.truncate(46);
        result.push('_');
        result.push_str(&digest[..16]);
    }
    Ok(result)
}

/// Derive conventional type spelling from the already checked portable name.
pub(super) fn type_name(source: &str) -> SchemaResult<String> {
    let name = portable_name(source)?;
    let mut result = String::new();
    for part in name.split('_').filter(|part| !part.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.extend(chars);
        }
    }
    // Self has special Rust grammar and cannot be a raw type identifier.
    if result == "Self" {
        result.push_str("Value");
    }
    Ok(result)
}

/// Members share the portable module name; a keyword suffix is explicit and deterministic.
pub(super) fn member_name(portable: &str) -> String {
    portable.into()
}

/// Reserved Rust words cannot become module declarations supplied to the aggregate.
fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "gen"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}
