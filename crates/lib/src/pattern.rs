//! Explicit literal, Express-style, Rust-regex and neutral method/path matching.

use crate::{DataMap, Error, ErrorKind, Result};
use std::borrow::Cow;

/// Captures belong to one matched listener, never a mutable global event slot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MatchData {
    /// Positional wildcard/regex captures; unmatched regex groups remain None.
    pub args: Vec<Option<String>>,
    /// Named parameter/regex captures.
    pub params: DataMap<String, String>,
}

/// A normalized request method, not an Any matcher or a transport authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Method(Cow<'static, str>);
impl Method {
    /// Standard GET method without parsing or allocation.
    pub const GET: Self = Self(Cow::Borrowed("GET"));
    /// Standard HEAD method without parsing or allocation.
    pub const HEAD: Self = Self(Cow::Borrowed("HEAD"));
    /// Standard POST method without parsing or allocation.
    pub const POST: Self = Self(Cow::Borrowed("POST"));
    /// Standard PUT method without parsing or allocation.
    pub const PUT: Self = Self(Cow::Borrowed("PUT"));
    /// Standard DELETE method without parsing or allocation.
    pub const DELETE: Self = Self(Cow::Borrowed("DELETE"));
    /// Standard CONNECT method without parsing or allocation.
    pub const CONNECT: Self = Self(Cow::Borrowed("CONNECT"));
    /// Standard OPTIONS method without parsing or allocation.
    pub const OPTIONS: Self = Self(Cow::Borrowed("OPTIONS"));
    /// Standard TRACE method without parsing or allocation.
    pub const TRACE: Self = Self(Cow::Borrowed("TRACE"));
    /// Standard PATCH method without parsing or allocation.
    pub const PATCH: Self = Self(Cow::Borrowed("PATCH"));

    /// Validate an HTTP-style token and normalize it to uppercase.
    ///
    /// # Errors
    /// Rejects empty, oversized, whitespace or non-token input.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Method;
    /// assert_eq!(Method::new("post")?.as_str(), "POST");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new(method: &str) -> Result<Self> {
        // Validate the entire method token before uppercasing; Any remains a matcher-only concept.
        if method.is_empty()
            || method.len() > 128
            || !method
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"!#$%&'*+-.^_|~".contains(&c) || c == 0x60)
        {
            return Err(Error::invalid("invalid method token"));
        }
        Ok(Self(Cow::Owned(method.to_ascii_uppercase())))
    }
    /// Inspect the normalized token.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Method;
    /// assert_eq!(Method::GET.as_str(), "GET");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_str(&self) -> &str {
        // Borrow the normalized token without allocating another route/event string.
        &self.0
    }
}
impl Default for Method {
    /// Use GET for an independently constructed request.
    fn default() -> Self {
        // A default request has a real GET verb, not a wildcard or fictional ALL verb.
        Self::GET
    }
}

/// Method selection is distinct from real request verbs.
#[derive(Debug)]
pub enum MethodPattern {
    /// Match one normalized token.
    Exact(Method),
    /// Match any valid uppercase method token.
    Any,
}
impl MethodPattern {
    /// Match the standard GET method without fallible constant validation.
    pub const GET: Self = Self::Exact(Method::GET);
    /// Match the standard HEAD method without fallible constant validation.
    pub const HEAD: Self = Self::Exact(Method::HEAD);
    /// Match the standard POST method without fallible constant validation.
    pub const POST: Self = Self::Exact(Method::POST);
    /// Match the standard PUT method without fallible constant validation.
    pub const PUT: Self = Self::Exact(Method::PUT);
    /// Match the standard DELETE method without fallible constant validation.
    pub const DELETE: Self = Self::Exact(Method::DELETE);
    /// Match the standard CONNECT method without fallible constant validation.
    pub const CONNECT: Self = Self::Exact(Method::CONNECT);
    /// Match the standard OPTIONS method without fallible constant validation.
    pub const OPTIONS: Self = Self::Exact(Method::OPTIONS);
    /// Match the standard TRACE method without fallible constant validation.
    pub const TRACE: Self = Self::Exact(Method::TRACE);
    /// Match the standard PATCH method without fallible constant validation.
    pub const PATCH: Self = Self::Exact(Method::PATCH);

    /// Validate an exact method matcher.
    ///
    /// # Errors
    /// Returns invalid method-token input.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::MethodPattern;
    /// let selected = MethodPattern::exact("PURGE")?;
    /// assert!(matches!(selected, MethodPattern::Exact(_)));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn exact(method: &str) -> Result<Self> {
        // Wrap only a validated real verb when constructing an exact method matcher.
        Ok(Self::Exact(Method::new(method)?))
    }
}

/// Original route metadata is retained separately from compiled expressions.
#[derive(Debug)]
pub struct Route {
    /// Normalized method, or None for Any.
    pub method: Option<String>,
    /// Original Express-style path.
    pub path: String,
}

/// One positional capture's intended interpretation.
enum Capture {
    Param(String),
    Wildcard,
}
/// Typed matcher storage; no compiled regex display string becomes registration identity.
enum Matcher {
    Exact,
    Express {
        regexp: regex::Regex,
        mapping: Vec<Capture>,
        separator: char,
    },
    Regex(regex::Regex),
}

/// A compiled immutable event pattern. No ECMAScript regex compatibility is implied.
///
/// ```
/// use stackpress_lib::EventPattern;
/// let pattern = EventPattern::express("/profiles/:id", '/')?;
/// assert_eq!(
///     pattern.captures("/profiles/42")?.unwrap().params["id"],
///     "42"
/// );
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub struct EventPattern {
    /// Original contribution spelling for diagnostics and route metadata.
    source: String,
    /// Full typed identity including matcher vocabulary, flags and separator.
    identity: String,
    /// Immutable compiled matcher with its listener-specific capture interpretation.
    matcher: Matcher,
    /// Original method/path metadata only for neutral route contributions.
    route: Option<Route>,
}

/// Compile with a finite source/automaton budget, reporting a registration error.
fn compile(source: &str, insensitive: bool) -> Result<regex::Regex> {
    // Bound both source length and compiled automaton size before storing a listener matcher.
    if source.len() > 8192 {
        return Err(Error::limit("pattern byte limit exceeded"));
    }
    regex::RegexBuilder::new(source)
        .case_insensitive(insensitive)
        .size_limit(1024 * 1024)
        .build()
        .map_err(|cause| {
            Error::new("event pattern could not be compiled")
                .with_kind(ErrorKind::Registration)
                .with_cause(cause)
        })
}

/// Parse Express tokens and escape every literal fragment before regex compilation.
fn fragment(source: &str, separator: char) -> Result<(String, Vec<Capture>)> {
    // Scan literal and dynamic portions separately so literal regex punctuation never broadens
    // matches.
    if source.len() > 8192 {
        return Err(Error::limit("pattern byte limit exceeded"));
    }
    // Character scanning keeps a Unicode separator intact while only ASCII names are dynamic
    // syntax.
    let chars: Vec<char> = source.chars().collect();
    let (mut expression, mut mapping, mut literal) = (String::new(), Vec::new(), String::new());
    let (mut i, mut dynamic) = (0, false);
    while i < chars.len() {
        let character = chars[i];
        // Ordinary characters accumulate as literal text and separate adjacent dynamic tokens.
        if character != ':' && character != '*' {
            literal.push(character);
            dynamic = false;
            i += 1;
            continue;
        }
        // Two dynamic tokens without intervening literal text have ambiguous capture boundaries.
        if dynamic {
            return Err(
                Error::invalid("adjacent dynamic pattern tokens are ambiguous")
                    .with_kind(ErrorKind::Registration),
            );
        }
        // Escape the completed literal before appending any generated capture expression.
        expression.push_str(&regex::escape(&std::mem::take(&mut literal)));
        dynamic = true;
        let (pattern, capture, next) = dynamic_token(&chars, i, separator)?;
        // Reject duplicate named captures before extending the parallel regex/mapping tables.
        if let Capture::Param(name) = &capture
            && mapping
                .iter()
                .any(|previous| matches!(previous,Capture::Param(previous) if previous==name))
        {
            return Err(Error::invalid("duplicate parameter name"));
        }
        expression.push_str(&pattern);
        mapping.push(capture);
        i = next;
    }
    expression.push_str(&regex::escape(&literal));
    Ok((expression, mapping))
}

/// Read one named parameter or wildcard, leaving literal escaping to the outer scan.
fn dynamic_token(
    chars: &[char],
    start: usize,
    separator: char,
) -> Result<(String, Capture, usize)> {
    // One dynamic token owns exactly one capture; adjacency/duplicates are checked by the outer
    // scan.
    let segment = format!("([^{}]+)", regex::escape(&separator.to_string()));
    let mut end = start + 1;
    if chars[start] == ':' {
        while end < chars.len()
            && (chars[end].is_ascii_alphanumeric() || chars[end] == '_' || chars[end] == '-')
            && chars[end] != separator
        {
            end += 1;
        }
        // A colon must introduce a name; an empty capture name cannot be inspected reliably.
        if end == start + 1 {
            return Err(Error::invalid("parameter name is empty"));
        }
        let name = chars[start + 1..end].iter().collect();
        return Ok((segment, Capture::Param(name), end));
    }
    // A double star may span separators; a single star remains nonempty within one segment.
    if chars.get(end) == Some(&'*') {
        end += 1;
        if chars.get(end) == Some(&'*') {
            return Err(Error::invalid("invalid star run"));
        }
        Ok(("(.*)".into(), Capture::Wildcard, end))
    } else {
        Ok((segment, Capture::Wildcard, end))
    }
}

/// Collect Express capture meanings without injecting them into a request.
fn express_captures(
    regexp: &regex::Regex,
    mapping: &[Capture],
    separator: char,
    event: &str,
) -> Option<MatchData> {
    // Use the immutable first match and its listener-specific mapping, with no global regex cursor.
    let captures = regexp.captures(event)?;
    let mut data = MatchData::default();
    for (index, kind) in mapping.iter().enumerate() {
        // A missing optional capture cannot overwrite another listener's parameter map.
        let Some(capture) = captures.get(index + 1) else {
            continue;
        };
        match kind {
            Capture::Param(name) => {
                data.params.insert(name.clone(), capture.as_str().into());
            }
            // Flatten only this declared separator into positional wildcard arguments.
            Capture::Wildcard => data.args.extend(
                capture
                    .as_str()
                    .split(separator)
                    .map(|part| Some(part.into())),
            ),
        }
    }
    Some(data)
}

/// Preserve absent positional groups and collect only present named Rust-regex groups.
fn regex_captures(regexp: &regex::Regex, event: &str) -> Option<MatchData> {
    // Retain unmatched groups as None and keep named capture visibility separate from positions.
    let captures = regexp.captures(event)?;
    // Skip the full match; preserve optional group holes rather than shifting later capture
    // positions.
    let args = captures
        .iter()
        .skip(1)
        .map(|capture| capture.map(|value| value.as_str().into()))
        .collect();
    let mut params = DataMap::new();
    // Include only present named groups while retaining the regex's declared name order.
    for name in regexp.capture_names().flatten() {
        if let Some(value) = captures.name(name) {
            params.insert(name.into(), value.as_str().into());
        }
    }
    Some(MatchData { args, params })
}

impl EventPattern {
    /// Match literal event text; validation happens before registration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{EventPattern, MethodPattern};
    /// assert!(
    ///     EventPattern::exact("file.txt")
    ///         .captures("fileXtxt")?
    ///         .is_none()
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn exact(source: &str) -> Self {
        // Literal patterns retain punctuation verbatim; registration validates their finite size.
        Self {
            source: source.into(),
            identity: format!("exact:{source}"),
            matcher: Matcher::Exact,
            route: None,
        }
    }
    /// Compile explicit Express tokens with a chosen single-character separator.
    ///
    /// # Errors
    /// Rejects malformed/ambiguous tokens, duplicate parameters or regex budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{EventPattern, MethodPattern};
    /// let pattern = EventPattern::express("/profiles/:id", '/')?;
    /// assert_eq!(pattern.captures("/profiles/7")?.unwrap().params["id"], "7");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn express(source: &str, separator: char) -> Result<Self> {
        // Reserve colon/star for syntax and reject separators that would make token parsing
        // ambiguous.
        if separator.is_control() || separator == ':' || separator == '*' {
            return Err(Error::invalid("invalid pattern separator"));
        }
        let (expression, mapping) = fragment(source, separator)?;
        Ok(Self {
            source: source.into(),
            identity: format!("express:{separator}:{source}"),
            matcher: Matcher::Express {
                regexp: compile(&format!("^{expression}$"), false)?,
                mapping,
                separator,
            },
            route: None,
        })
    }
    /// Compile Rust-regex syntax; anchors and case-insensitivity are explicit.
    ///
    /// # Errors
    /// Rejects unsupported syntax (including JS lookaround/backreferences) or limits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{EventPattern, MethodPattern};
    /// let pattern = EventPattern::regex(r"^item-(\d+)$", false)?;
    /// assert_eq!(
    ///     pattern.captures("item-7")?.unwrap().args[0].as_deref(),
    ///     Some("7")
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn regex(source: &str, case_insensitive: bool) -> Result<Self> {
        // Store the original syntax/flags as identity instead of reparsing a compiled expression
        // display.
        Ok(Self {
            source: source.into(),
            identity: format!("regex:{case_insensitive}:{source}"),
            matcher: Matcher::Regex(compile(source, case_insensitive)?),
            route: None,
        })
    }
    /// Compile neutral method/path event matching, retaining original route metadata.
    ///
    /// # Errors
    /// Rejects invalid/oversized paths or token expressions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{EventPattern, MethodPattern};
    /// let pattern = EventPattern::route(MethodPattern::GET, "/profiles/:id")?;
    /// assert!(pattern.captures("GET /profiles/7")?.is_some());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn route(method: MethodPattern, path: &str) -> Result<Self> {
        // Keep route metadata alongside matching; no HTTP resource or capture injection is created.

        // Require an absolute route path before compiling its independent method/path parts.
        if !path.starts_with('/') {
            return Err(Error::invalid("route path must start with slash"));
        }
        let (path_expression, mapping) = fragment(path, '/')?;
        // Exact verbs become escaped literals; Any accepts valid method tokens without being a
        // request verb.
        let (method_expression, method_name) = match method {
            MethodPattern::Exact(method) => {
                (regex::escape(method.as_str()), Some(method.0.into_owned()))
            }
            MethodPattern::Any => (r"[A-Z0-9!#$%&'*+\-.^_|\x60~]+".into(), None),
        };
        let source = format!("{} {path}", method_name.as_deref().unwrap_or("ANY"));
        let dynamic = method_name.is_none() || !mapping.is_empty();
        // Parameterized/Any routes tolerate trailing slashes; exact literal routes preserve strict
        // equality.
        let matcher = if dynamic {
            Matcher::Express {
                regexp: compile(&format!("^{method_expression} {path_expression}/*$"), false)?,
                mapping,
                separator: '/',
            }
        } else {
            Matcher::Exact
        };
        Ok(Self {
            identity: format!("route:{source}"),
            source,
            matcher,
            route: Some(Route {
                method: method_name,
                path: path.into(),
            }),
        })
    }
    /// Inspect the original expression, never a canonicalization/reparse key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{EventPattern, MethodPattern};
    /// assert_eq!(EventPattern::exact("health").source(), "health");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn source(&self) -> &str {
        // Return the original contribution text for diagnostics and inspection.
        &self.source
    }
    /// Inspect route metadata only when this matcher was a route contribution.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{EventPattern, MethodPattern};
    /// let pattern = EventPattern::route(MethodPattern::Any, "/health")?;
    /// assert!(pattern.route_metadata().unwrap().method.is_none());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn route_metadata(&self) -> Option<&Route> {
        // Only route constructors populate metadata; an arbitrary matching event is not a transport
        // route.
        self.route.as_ref()
    }
    /// Compare full typed pattern identity without collapsing capture-name mappings.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{EventPattern, MethodPattern};
    /// assert!(
    ///     !EventPattern::exact("health").same_pattern(&EventPattern::regex("health", false)?)
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn same_pattern(&self, other: &Self) -> bool {
        // Typed identity includes syntax kind, flags and separator, preserving capture-name
        // differences.
        self.identity == other.identity
    }
    /// Validate literal registration input before mutating a builder.
    pub(crate) fn validate(&self) -> Result<()> {
        // Fail registration before table mutation if literal contribution text violates the input
        // contract.
        if self.source.is_empty() || self.source.len() > 8192 {
            return Err(Error::invalid("event pattern is empty/oversized"));
        }
        Ok(())
    }
    /// Extract first-match captures without mutable regex cursors or global state.
    ///
    /// # Errors
    /// Rejects an oversized event input.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{EventPattern, MethodPattern};
    /// let pattern = EventPattern::express("/profiles/:id", '/')?;
    /// assert_eq!(pattern.captures("/profiles/7")?.unwrap().params["id"], "7");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn captures(&self, event: &str) -> Result<Option<MatchData>> {
        // Check event size once, then delegate to the chosen immutable matching vocabulary.

        // Bound regex work before asking any matcher to inspect the event string.
        if event.len() > 8192 {
            return Err(Error::limit("event byte limit exceeded"));
        }
        Ok(match &self.matcher {
            Matcher::Exact => (self.source == event).then(MatchData::default),
            Matcher::Express {
                regexp,
                mapping,
                separator,
            } => express_captures(regexp, mapping, *separator, event),
            Matcher::Regex(regexp) => regex_captures(regexp, event),
        })
    }
}
