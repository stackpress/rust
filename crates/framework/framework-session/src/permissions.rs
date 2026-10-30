//! Source permission derivation, separate from the framework's escaped router patterns.
//!
//! Rust regex syntax is the existing mechanical boundary: unsupported ECMAScript constructs
//! return a typed compile error. Global matches start fresh per call; sticky means start=0.

use crate::{SessionPermission, SessionRoute};
use regex::{Regex, RegexBuilder};

/// Match an event against one source permission, preserving unescaped regex punctuation.
/// # Errors
/// Returns unsupported/invalid Rust expression syntax rather than changing it into a literal.
pub fn match_event(permit: &str, permission: &str) -> Result<bool, regex::Error> {
    // The source exact shortcut precedes all expression parsing.
    if permit == permission {
        return Ok(true);
    }
    let derived = permission
        .replace('*', "([^-]+)")
        .replace("([^-]+)([^-]+)", "(.*)");
    matches(permit, permission, &format!("^{derived}$"))
}
/// Match method first, then the source path/parameter/wildcard expression.
/// # Errors
/// Returns invalid or unsupported expression syntax after the source exact shortcuts.
pub fn match_route(permit: &SessionRoute, permission: &SessionRoute) -> Result<bool, regex::Error> {
    if permission.method != "ALL" && permission.method != permit.method {
        return Ok(false);
    }
    if permission.route == permit.route {
        return Ok(true);
    }
    // Source parameter names include hyphens and become exactly one wildcard segment.
    let params = Regex::new(":[a-zA-Z0-9_-]+")?;
    let derived = params
        .replace_all(&permission.route, "*")
        .replace('*', "([^/]+)")
        .replace("([^/]+)([^/]+)", "(.*)");
    matches(&permit.route, &permission.route, &format!("^{derived}$"))
}
/// Match one requested permission against some grant; errors preserve source iteration order.
/// # Errors
/// Propagates the first evaluated invalid/unsupported expression.
pub fn match_any(
    permit: &SessionPermission,
    grants: &[SessionPermission],
) -> Result<bool, regex::Error> {
    // matchAnyEvent checks includes before compiling any earlier expression.
    if let SessionPermission::Event(event) = permit
        && grants
            .iter()
            .any(|grant| matches!(grant, SessionPermission::Event(g) if g == event))
    {
        return Ok(true);
    }
    for grant in grants {
        let matched = match (permit, grant) {
            (SessionPermission::Event(p), SessionPermission::Event(g)) => match_event(p, g)?,
            (SessionPermission::Route(p), SessionPermission::Route(g)) => match_route(p, g)?,
            _ => false,
        };
        if matched {
            return Ok(true);
        }
    }
    Ok(false)
}
/// Recognize only the source slash-delimited form, including its allowed JS flags.
fn explicit(source: &str) -> Option<(&str, &str)> {
    let tail = source.strip_prefix('/')?;
    let last = tail.rfind('/')?;
    let (expression, flags) = (&tail[..last], &tail[last + 1..]);
    (!expression.is_empty() && flags.chars().all(|c| "igmsuy".contains(c)))
        .then_some((expression, flags))
}
/// Compile flags explicitly; each invocation owns a fresh match cursor as upstream does.
fn matches(input: &str, source: &str, derived: &str) -> Result<bool, regex::Error> {
    let (expression, flags) = explicit(source).unwrap_or((derived, "ig"));
    let mut seen = String::new();
    for flag in flags.chars() {
        if seen.contains(flag) {
            return Err(regex::Error::Syntax(
                "duplicate permission regex flag".into(),
            ));
        }
        seen.push(flag);
    }
    let regexp = RegexBuilder::new(expression)
        .case_insensitive(flags.contains('i'))
        .multi_line(flags.contains('m'))
        .dot_matches_new_line(flags.contains('s'))
        .build()?;
    // y starts the fresh JS cursor at zero; g otherwise has no persistent lastIndex here.
    Ok(regexp
        .find(input)
        .is_some_and(|m| !flags.contains('y') || m.start() == 0))
}
