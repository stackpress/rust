//! Strict bounded boundary parsing; supplied arguments never pass through a query string.

use super::{DataMap, DataPath, FileData, Nest, ParseLimits, PathSegment, Value, check_input};
use crate::{Error, Result};

/// Decode percent escapes once, optionally treating plus as space.
pub(crate) fn decode(text: &str, plus: bool) -> Result<String> {
    // Process bytes once: a decoded percent sign is payload, never another escape to interpret.
    let input = text.as_bytes();
    let mut output = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        // Require two hexadecimal digits before consuming an escape; truncation cannot read beyond
        // input.
        if input[i] == b'%' {
            let a = input.get(i + 1).and_then(|b| (*b as char).to_digit(16));
            let b = input.get(i + 2).and_then(|b| (*b as char).to_digit(16));
            // Reject malformed escapes instead of lossy replacement that could change field
            // identity.
            let (Some(a), Some(b)) = (a, b) else {
                return Err(Error::invalid("malformed percent encoding"));
            };
            output.push((a * 16 + b) as u8);
            i += 3;
        } else {
            // Query decoding treats plus as space; cookie decoding deliberately leaves plus
            // literal.
            output.push(if plus && input[i] == b'+' {
                b' '
            } else {
                input[i]
            });
            i += 1;
        }
    }
    // Validate the decoded bytes after escapes; invalid UTF-8 is a boundary error.
    String::from_utf8(output).map_err(|_| Error::invalid("decoded input is not UTF-8"))
}

/// Encode UTF-8 bytes using the upstream encodeURIComponent unreserved vocabulary.
pub(crate) fn encode(text: &str) -> String {
    // Encode UTF-8 bytes rather than characters so non-ASCII cookies round-trip losslessly.
    let mut output = String::new();
    for byte in text.bytes() {
        // Keep only the declared unreserved vocabulary literal; delimiters are always escaped.
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            output.push(byte as char);
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}

/// Parse a strict bracket path; append is explicit and malformed suffixes are errors.
fn bracket_path(text: &str) -> Result<DataPath> {
    // Separate the literal root from bracket components before accepting any nested write.
    // The enclosing body budget may be larger; path spelling still has its own finite work limit.
    if text.len() > super::path::MAX_SOURCE_BYTES {
        return Err(Error::limit("path source byte limit exceeded"));
    }
    let split = text.find('[').unwrap_or(text.len());
    let root = &text[..split];
    // A closing bracket without an opening one cannot be a valid root component.
    if root.contains(']') {
        return Err(Error::invalid("unexpected closing bracket"));
    }
    let mut steps = vec![super::path::segment(root)?];
    let mut rest = &text[split..];
    // Consume complete bracket pairs; trailing free text is rejected rather than appended to a key.
    while !rest.is_empty() {
        // Reject a deep bracket chain before allocating all of its intermediate segment values.
        if steps.len() == 64 {
            return Err(Error::limit("path depth limit exceeded"));
        }
        let Some(inner) = rest.strip_prefix('[') else {
            return Err(Error::invalid("invalid bracket path suffix"));
        };
        let Some(end) = inner.find(']') else {
            return Err(Error::invalid("unclosed bracket path"));
        };
        let key = &inner[..end];
        if key.contains('[') {
            return Err(Error::invalid("invalid nested bracket"));
        }
        // Empty brackets mean append; empty literal keys remain invalid everywhere else.
        steps.push(if key.is_empty() {
            PathSegment::Append
        } else {
            super::path::segment(key)?
        });
        rest = &inner[end + 1..];
    }
    DataPath::from_segments(steps)
}

/// Coerce only explicit finite JSON scalars and valid object/array JSON values.
fn scalar(text: &str, limits: &ParseLimits) -> Result<Value> {
    // Only exact JSON spellings coerce; empty strings and malformed JSON-like input remain text.
    match text {
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        "null" => return Ok(Value::Null),
        _ => {}
    }
    // A malformed JSON-like scalar remains text; valid structured JSON is real data.
    if ((text.starts_with('{') && text.ends_with('}'))
        || (text.starts_with('[') && text.ends_with(']')))
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(text)
    {
        return Value::from_json_with(value, limits);
    }
    // Use the finite JSON-number parser rather than accepting NaN, infinity or partial numeric
    // prefixes.
    if let Ok(number) = text.parse::<serde_json::Number>() {
        return Ok(Value::Number(number));
    }
    Ok(Value::String(text.into()))
}

/// Parse URL-encoded input with optional leading ?, strict decoding and nested keys.
///
/// # Errors
/// Rejects malformed escapes/UTF-8/paths, structural conflicts or resource limits.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::data::{
/// #     ParseLimits, parse_arg_string, parse_args, parse_body, parse_json_object, parse_multipart,
/// #     parse_query,
/// # };
/// let limits = ParseLimits::default();
/// let data = parse_query("active=false&profile[name]=Ada", &limits)?;
/// assert_eq!(data.path_as::<bool>("active")?, Some(false));
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn parse_query(text: &str, limits: &ParseLimits) -> Result<Nest> {
    // Bound the raw input before decoding, then insert pairs in their supplied order.
    check_input(text.len(), limits)?;
    let mut nest = Nest::with_limits(*limits);
    for (i, pair) in text
        // An optional query marker and empty separators add no fields; source order determines
        // replacement.
        .strip_prefix('?')
        .unwrap_or(text)
        .split('&')
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        // Enforce a pair limit before parsing this pair so repeated keys cannot bypass input work
        // limits.
        if i >= limits.max_entries {
            return Err(Error::limit("query entry limit exceeded"));
        }
        // Split only the first equals sign; additional equals signs belong to the supplied value.
        // Each parameter has one name/value boundary; equals signs inside its value remain payload.
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = decode(key, true)?;
        nest.set_path(&bracket_path(&key)?, scalar(&decode(value, true)?, limits)?)?;
    }
    Ok(nest)
}

/// Parse supplied argv directly, preserving delimiter characters inside values.
///
/// # Errors
/// Rejects a dangling long option, malformed path or exceeded budget.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::data::{
/// #     ParseLimits, parse_arg_string, parse_args, parse_body, parse_json_object, parse_multipart,
/// #     parse_query,
/// # };
/// let limits = ParseLimits::default();
/// let args = ["--token".into(), "a&b=c".into()];
/// assert_eq!(
///     parse_args(&args, 0, &limits)?.path_as::<String>("token")?,
///     Some("a&b=c".into())
/// );
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn parse_args(args: &[String], skip: usize, limits: &ParseLimits) -> Result<Nest> {
    // Bound aggregate argv bytes before selecting options; values never become query syntax.

    // Checked accumulation rejects arithmetic overflow before comparing the byte budget.
    let size = args.iter().try_fold(0usize, |sum, arg| {
        sum.checked_add(arg.len())
            .ok_or_else(|| Error::limit("argument size overflow"))
    })?;
    check_input(size, limits)?;
    let mut nest = Nest::with_limits(*limits);
    // A skip beyond the supplied argv produces empty input instead of indexing panic.
    let args = args.get(skip..).unwrap_or_default();
    let (mut i, mut position, mut options) = (0usize, 0usize, true);
    while i < args.len() {
        if i >= limits.max_entries {
            return Err(Error::limit("argument entry limit exceeded"));
        }
        let arg = &args[i];
        i += 1;
        // The terminator switches all later arguments to positional values, including leading minus
        // signs.
        if options && arg == "--" {
            options = false;
            continue;
        }
        // A lone dash contributes no short flags and is deliberately ignored.
        if options && arg == "-" {
            continue;
        }
        // Long options consume either their inline value or exactly the following argv item.
        if options && let Some(option) = arg.strip_prefix("--") {
            let (key, value) = if let Some(pair) = option.split_once('=') {
                pair
            } else {
                // A missing long-option value is an error; it must not be silently treated as true.
                let value = args
                    .get(i)
                    .ok_or_else(|| Error::invalid("long option is missing its value"))?;
                i += 1;
                (option, value.as_str())
            };
            nest.set_path(&bracket_path(key)?, scalar(value, limits)?)?;
        } else if options && arg.starts_with('-') && arg.parse::<serde_json::Number>().is_err() {
            // Grouped short flags become independent true fields; non-alphanumeric flags are
            // rejected.
            for flag in arg[1..].chars() {
                if !flag.is_ascii_alphanumeric() {
                    return Err(Error::invalid("invalid short flag"));
                }
                nest.set(&flag.to_string(), true)?;
            }
        } else {
            // Numeric arguments, including negative numbers, retain positional identity and scalar
            // coercion.
            nest.set(&position.to_string(), scalar(arg, limits)?)?;
            position += 1;
        }
    }
    Ok(nest)
}

/// Tokenize whitespace/quoted argument text without shell expansion or execution.
///
/// # Errors
/// Rejects unclosed quotes or any argv parser failure.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::data::{
/// #     ParseLimits, parse_arg_string, parse_args, parse_body, parse_json_object, parse_multipart,
/// #     parse_query,
/// # };
/// let limits = ParseLimits::default();
/// assert_eq!(
///     parse_arg_string("--name 'Ada Lovelace'", &limits)?.path_as::<String>("name")?,
///     Some("Ada Lovelace".into())
/// );
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn parse_arg_string(text: &str, limits: &ParseLimits) -> Result<Nest> {
    // This is a small tokenizer only: quotes group text and no shell expansion is performed.
    check_input(text.len(), limits)?;
    let (mut quote, mut started) = (None, false);
    let mut buffer = String::new();
    let mut args = Vec::new();
    for character in text.chars() {
        // Inside a quote only its matching delimiter changes tokenizer state; whitespace remains
        // payload.
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            } else {
                buffer.push(character);
            }
        } else if character == '\'' || character == '"' {
            // Opening quotes mark an argument as started so an explicitly quoted empty value
            // survives.
            quote = Some(character);
            started = true;
        } else if character.is_whitespace() {
            // Only a started token is emitted; repeated unquoted whitespace contributes no
            // argument.
            if started {
                args.push(std::mem::take(&mut buffer));
                started = false;
            }
        } else {
            buffer.push(character);
            started = true;
        }
        if args.len() > limits.max_entries {
            return Err(Error::limit("argument entry limit exceeded"));
        }
    }
    // Reject an unfinished quote before handing any collected argv to the option parser.
    if quote.is_some() {
        return Err(Error::invalid("unclosed argument quote"));
    }
    if started {
        args.push(buffer);
    }
    parse_args(&args, 0, limits)
}

/// Parse JSON with an object root and the caller's finite budgets.
///
/// # Errors
/// Rejects malformed/non-object JSON and excessive input/depth/entries.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::data::{
/// #     ParseLimits, parse_arg_string, parse_args, parse_body, parse_json_object, parse_multipart,
/// #     parse_query,
/// # };
/// let limits = ParseLimits::default();
/// assert!(parse_json_object("[]", &limits).is_err());
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn parse_json_object(text: &str, limits: &ParseLimits) -> Result<Nest> {
    // Check bytes before parsing and rich-tree limits before accepting the object root.
    check_input(text.len(), limits)?;
    let parsed =
        serde_json::from_str(text).map_err(|cause| Error::from_source(cause).with_code(400))?;
    Nest::from_value(Value::from_json_with(parsed, limits)?, *limits)
}

/// Split media/disposition parameters, respecting quotes and escaped characters.
pub(crate) fn parameters(text: &str) -> Result<(String, DataMap<String, String>)> {
    // Separate framing from value decoding so quoted delimiters are never split as parameters.
    let mut parts = split_parameters(text)?;
    // Normalize only the media/disposition essence; quoted parameter content keeps its original
    // case/spaces.
    let essence = parts.remove(0).trim().to_ascii_lowercase();
    let mut parameters = DataMap::new();
    for part in parts {
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| Error::invalid("invalid media parameter"))?;
        let key = key.trim().to_ascii_lowercase();
        // Reject duplicate names after case normalization so casing cannot select ambiguous
        // metadata.
        if !crate::headers::token(&key) || parameters.contains_key(&key) {
            return Err(Error::invalid("duplicate/empty media parameter"));
        }
        let value = value.trim();
        let value = parameter_value(value)?;
        parameters.insert(key, value);
    }
    Ok((essence, parameters))
}

/// Split on unquoted semicolons, retaining quote/escape syntax for value decoding.
fn split_parameters(text: &str) -> Result<Vec<String>> {
    // Track quote/escape state while retaining syntax needed to decode a value exactly once.
    let mut parts = Vec::new();
    let (mut quoted, mut escaped) = (false, false);
    let mut current = String::new();
    for character in text.chars() {
        // An escaped character cannot toggle quotes or terminate a parameter.
        if escaped {
            current.push(character);
            escaped = false;
        } else if quoted && character == '\\' {
            // A backslash has escape meaning only inside a quoted parameter.
            current.push(character);
            escaped = true;
        } else if character == '"' {
            current.push(character);
            quoted = !quoted;
        } else if character == ';' && !quoted {
            parts.push(std::mem::take(&mut current));
        } else {
            current.push(character);
        }
    }
    // Do not accept truncated quote/escape state as a complete parameter list.
    if quoted || escaped {
        return Err(Error::invalid("unclosed media parameter quote"));
    }
    parts.push(current);
    Ok(parts)
}

/// Decode a single token or quoted string without losing quoted filename spaces.
fn parameter_value(value: &str) -> Result<String> {
    // Unquote only a complete quoted string; unquoted parameters must be tokens.
    let value = if let Some(quoted) = value.strip_prefix('"') {
        let quoted = quoted
            .strip_suffix('"')
            .ok_or_else(|| Error::invalid("invalid media parameter quote"))?;
        let mut decoded = String::new();
        let mut characters = quoted.chars();
        // Remove quoting syntax while preserving meaningful spaces inside it.
        while let Some(character) = characters.next() {
            // Quoted-pair escaping preserves the following character while removing only its escape
            // syntax.
            if character == '\\' {
                decoded.push(
                    characters
                        .next()
                        .ok_or_else(|| Error::invalid("truncated quoted escape"))?,
                );
            } else if character == '"' {
                return Err(Error::invalid("unescaped media parameter quote"));
            } else {
                decoded.push(character);
            }
        }
        decoded
    } else {
        // Unquoted whitespace/control/punctuation cannot be smuggled into parameter framing.
        if !crate::headers::token(value) {
            return Err(Error::invalid("invalid unquoted media parameter"));
        }
        value.to_owned()
    };
    Ok(value)
}

/// Validate bounded MIME syntax/parameters without choosing a body parser.
pub(crate) fn validate_mime(text: &str) -> Result<()> {
    // Validate bounded syntax only; parser/transport selection belongs to the caller.
    if text.len() > 8192 || text.bytes().any(|b| b < 0x20 || b == 0x7f) {
        return Err(Error::invalid("invalid media type"));
    }
    let (kind, _) = parameters(text)?;
    // Require a real type/subtype pair instead of treating an arbitrary token as a media type.
    let (kind, subtype) = kind
        .split_once('/')
        .ok_or_else(|| Error::invalid("media type requires type/subtype"))?;
    if !crate::headers::token(kind) || !crate::headers::token(subtype) {
        return Err(Error::invalid("invalid media type token"));
    }
    Ok(())
}

/// Find a literal byte sequence without interpreting file contents as UTF-8.
fn find(bytes: &[u8], needle: &[u8]) -> Option<usize> {
    // Byte windows preserve binary payloads and do not impose UTF-8 on uploaded files.
    bytes.windows(needle.len()).position(|part| part == needle)
}

/// Parse bounded multipart bytes with an explicit boundary and required termination.
///
/// # Errors
/// Rejects invalid/truncated framing, malformed headers/text or exceeded budgets.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::data::{
/// #     ParseLimits, parse_arg_string, parse_args, parse_body, parse_json_object, parse_multipart,
/// #     parse_query,
/// # };
/// let limits = ParseLimits::default();
/// let data = parse_multipart(
///     b"--B\r\nContent-Disposition: form-data; name=\"x\"\r\n\r\nfalse\r\n--B--",
///     "B",
///     &limits,
/// )?;
/// assert_eq!(data.path_as::<bool>("x")?, Some(false));
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn parse_multipart(body: &[u8], boundary: &str, limits: &ParseLimits) -> Result<Nest> {
    // Reject oversized input and invalid boundaries before scanning any part payload.
    check_input(body.len(), limits)?;
    if boundary.is_empty()
        || boundary.len() > 70
        || boundary.ends_with(' ')
        || !boundary
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"'()+_,-./:=? ".contains(&c))
    {
        return Err(Error::invalid("invalid multipart boundary"));
    }
    let delimiter = format!("--{boundary}").into_bytes();
    // A multipart body must begin with its declared boundary; preamble guessing is not performed.
    if !body.starts_with(&delimiter) {
        return Err(Error::invalid("multipart initial boundary missing"));
    }
    let mut position = delimiter.len();
    let mut nest = Nest::with_limits(*limits);
    let mut parts = 0usize;
    loop {
        // Only a closing suffix terminates the message, and only an optional final CRLF may follow.
        if body.get(position..position + 2) == Some(b"--") {
            let tail = &body[position + 2..];
            if !tail.is_empty() && tail != b"\r\n" {
                return Err(Error::invalid("invalid multipart closing boundary"));
            }
            return Ok(nest);
        }
        // Nonfinal boundaries must start the next header block with exact CRLF framing.
        if body.get(position..position + 2) != Some(b"\r\n") {
            return Err(Error::invalid("invalid multipart boundary framing"));
        }
        position += 2;
        // Count even ignored parts so missing dispositions cannot bypass the work budget.
        parts += 1;
        if parts > limits.max_entries {
            return Err(Error::limit("multipart part limit exceeded"));
        }
        let (fields, content_start) = part_headers(body, position, limits)?;
        // Only valid delimiter suffixes end a part; boundary-like bytes remain data.
        let marker = [b"\r\n".as_slice(), delimiter.as_slice()].concat();
        let content_end = part_end(body, content_start, &marker)?;
        if let Some((path, value)) = part_value(&fields, &body[content_start..content_end], limits)?
        {
            nest.set_path(&path, value)?;
        }
        // Advance past the framed delimiter; the next loop decides continuation versus termination.
        position = content_end + marker.len();
    }
}

/// Validate a bounded part header block and return the first content byte.
fn part_headers(
    body: &[u8],
    position: usize,
    limits: &ParseLimits,
) -> Result<(DataMap<String, String>, usize)> {
    // Locate complete framing before decoding header text or allocating the field map.
    let (header_end, content_start) = if body[position..].starts_with(b"\r\n") {
        (position, position + 2)
    } else {
        let end = find(&body[position..], b"\r\n\r\n")
            .ok_or_else(|| Error::invalid("multipart headers are truncated"))?
            + position;
        (end, end + 4)
    };
    // Bound one complete header block before decoding or splitting its fields.
    if header_end - position > limits.max_part_headers {
        return Err(Error::limit("multipart part header limit exceeded"));
    }
    let headers = std::str::from_utf8(&body[position..header_end])
        .map_err(|_| Error::invalid("multipart headers are not UTF-8"))?;
    let mut fields = DataMap::new();
    // Parse repeated header lines explicitly; duplicate normalized names are rejected.
    for line in headers.split("\r\n").filter(|s| !s.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| Error::invalid("invalid multipart header"))?;
        let name = name.trim().to_ascii_lowercase();
        if fields.insert(name, value.trim().to_owned()).is_some() {
            return Err(Error::invalid("duplicate multipart header"));
        }
    }
    Ok((fields, content_start))
}

/// Accept only framed boundary suffixes so boundary-like file bytes survive intact.
fn part_end(body: &[u8], content_start: usize, marker: &[u8]) -> Result<usize> {
    // Skip boundary-looking bytes unless their following suffix proves real multipart framing.
    let mut search = content_start;
    let content_end = loop {
        let offset = find(&body[search..], marker)
            .ok_or_else(|| Error::invalid("multipart closing boundary missing"))?;
        let found = search + offset;
        // A literal byte match alone is insufficient: only continuation/closing suffixes end this
        // part.
        let suffix = body.get(found + marker.len()..found + marker.len() + 2);
        if suffix == Some(b"--") || suffix == Some(b"\r\n") {
            break found;
        }
        // A false boundary match remains file payload; resume scanning after that literal
        // candidate.
        search = found + marker.len();
    };
    Ok(content_end)
}

/// Decode form fields/files; parts without a named form-data disposition are ignored.
fn part_value(
    fields: &DataMap<String, String>,
    content: &[u8],
    limits: &ParseLimits,
) -> Result<Option<(DataPath, Value)>> {
    // Ignore unnamed/non-form parts; filename presence, including empty, selects binary file data.

    // Parts without Content-Disposition are permitted but contribute no form field.
    if let Some(disposition) = fields.get("content-disposition") {
        let (kind, metadata) = parameters(disposition)?;
        if kind == "form-data"
            && let Some(name) = metadata.get("name").filter(|v| !v.is_empty())
        {
            // Even an empty filename denotes a file; file bytes are never decoded as form text.
            let value = if let Some(filename) = metadata.get("filename") {
                Value::File(FileData::new(
                    content.to_vec(),
                    filename,
                    fields
                        .get("content-type")
                        .map(String::as_str)
                        .unwrap_or("application/octet-stream"),
                ))
            } else {
                scalar(
                    std::str::from_utf8(content)
                        .map_err(|_| Error::invalid("multipart text is not UTF-8"))?,
                    limits,
                )?
            };
            return Ok(Some((bracket_path(name)?, value)));
        }
    }
    Ok(None)
}

/// Dispatch supported media types without guessing unknown input into empty data.
///
/// # Errors
/// Rejects unsupported MIME, missing boundary, invalid UTF-8 or parser failures.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::data::{
/// #     ParseLimits, parse_arg_string, parse_args, parse_body, parse_json_object, parse_multipart,
/// #     parse_query,
/// # };
/// let limits = ParseLimits::default();
/// let data = parse_body("application/json", br#"{"id":7}"#, &limits)?;
/// assert_eq!(data.path_as::<u64>("id")?, Some(7));
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn parse_body(content_type: &str, body: &[u8], limits: &ParseLimits) -> Result<Nest> {
    // Choose an explicit supported media type; unknown bodies never silently become empty data.
    check_input(body.len(), limits)?;
    let (kind, parameters) = parameters(content_type)?;
    if kind == "multipart/form-data" {
        return parse_multipart(
            body,
            parameters
                .get("boundary")
                .ok_or_else(|| Error::invalid("multipart boundary is required"))?,
            limits,
        );
    }
    // Non-multipart supported forms require valid UTF-8 before their text parser runs.
    let text = std::str::from_utf8(body).map_err(|_| Error::invalid("body text is not UTF-8"))?;
    if kind == "application/json" || kind == "text/json" || kind.ends_with("+json") {
        parse_json_object(text, limits)
    } else if kind == "application/x-www-form-urlencoded" {
        parse_query(text, limits)
    } else {
        Err(Error::invalid("unsupported body media type").with_code(415))
    }
}
