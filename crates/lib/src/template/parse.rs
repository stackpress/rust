//! Literal-delimiter scanning and bounded structural block parsing.

use super::{TemplateError, TemplateErrorKind, TemplateHelper, TemplateOptions, TemplateResult};
use serde_json::Value;
use std::{collections::BTreeMap, ops::Range};

/// One owned lookup path, validated independently of runtime data.
pub(super) type Path = Vec<String>;

/// A path argument remains lazy; JSON literal arguments are owned by the template.
pub(super) enum Argument {
    Path(Path),
    Literal(Value),
}

/// A node's source range also locates runtime failures without retaining input data.
pub(super) struct Node {
    pub(super) span: Range<usize>,
    pub(super) kind: Kind,
}

/// Parsed structure makes repeated block names and nearest-block else unambiguous.
pub(super) enum Kind {
    Text,
    Value(Path),
    Helper {
        name: String,
        arguments: Vec<Argument>,
    },
    If {
        path: Path,
        inverted: bool,
        yes: Vec<Node>,
        no: Vec<Node>,
    },
    Each {
        path: Path,
        item: Option<String>,
        key: Option<String>,
        body: Vec<Node>,
    },
}

/// A structural terminator returned to the immediately enclosing block.
enum End {
    Eof,
    Else(Range<usize>),
    Close(String, Range<usize>),
}

/// Cursor offsets always come from UTF-8 string boundaries or ASCII delimiters.
struct Parser<'a> {
    source: &'a str,
    options: &'a TemplateOptions,
    helpers: &'a BTreeMap<String, TemplateHelper>,
    cursor: usize,
}

/// Parse every branch while rejecting a top-level close or else marker.
pub(super) fn compile(
    source: &str,
    options: &TemplateOptions,
    helpers: &BTreeMap<String, TemplateHelper>,
) -> TemplateResult<Vec<Node>> {
    let mut parser = Parser {
        source,
        options,
        helpers,
        cursor: 0,
    };
    let (nodes, end) = parser.sequence(0)?;
    match end {
        End::Eof => Ok(nodes),
        End::Else(span) | End::Close(_, span) => {
            Err(syntax("unexpected block terminator").at(span))
        }
    }
}

impl Parser<'_> {
    /// Read siblings until EOF, else, or a close belonging to the caller.
    fn sequence(&mut self, depth: usize) -> TemplateResult<(Vec<Node>, End)> {
        let mut nodes = Vec::new();
        let open = self.options.delimiters[0].as_str();
        let close = self.options.delimiters[1].as_str();
        while self.cursor < self.source.len() {
            let Some(offset) = self.source[self.cursor..].find(open) else {
                nodes.push(Node {
                    span: self.cursor..self.source.len(),
                    kind: Kind::Text,
                });
                self.cursor = self.source.len();
                break;
            };
            let start = self.cursor + offset;
            if start > self.cursor {
                nodes.push(Node {
                    span: self.cursor..start,
                    kind: Kind::Text,
                });
            }
            let content_start = start + open.len();
            // A doubled opener is literal text; it never starts a nested tag.
            if self.source[content_start..].starts_with(open) {
                nodes.push(Node {
                    span: start..content_start,
                    kind: Kind::Text,
                });
                self.cursor = content_start + open.len();
                continue;
            }
            let offset = self.source[content_start..]
                .find(close)
                .ok_or_else(|| syntax("unclosed tag").at(start..self.source.len()))?;
            let content_end = content_start + offset;
            self.cursor = content_end + close.len();
            let span = start..self.cursor;
            let tag = self.source[content_start..content_end].trim_ascii();
            if tag == "|" {
                return Ok((nodes, End::Else(span)));
            }
            if let Some(tag) = tag.strip_prefix('/') {
                return Ok((nodes, End::Close(tag.into(), span)));
            }
            // The bounded block parser consumes its own close before returning another sibling.
            let kind = if let Some(tag) = tag.strip_prefix('#') {
                self.block(tag, depth, span.clone())?
            } else {
                self.inline(tag).map_err(|error| error.at(span.clone()))?
            };
            nodes.push(Node { span, kind });
        }
        Ok((nodes, End::Eof))
    }

    /// Parse one supported opener, child lists, and the exact nearest matching close.
    fn block(&mut self, tag: &str, depth: usize, span: Range<usize>) -> TemplateResult<Kind> {
        if depth >= self.options.limits.max_nesting {
            return Err(
                TemplateError::new(TemplateErrorKind::Limit, "block nesting limit").at(span),
            );
        }
        let (kind, arguments) = tag
            .split_once(':')
            .ok_or_else(|| syntax("invalid block opener").at(span.clone()))?;
        let tokens = tokens(arguments).map_err(|error| error.at(span.clone()))?;
        let name = tokens
            .first()
            .ok_or_else(|| syntax("block requires a path").at(span.clone()))?;
        let path = path(name).map_err(|error| error.at(span.clone()))?;
        if !matches!(kind, "?" | "!" | "@")
            || (kind != "@" && tokens.len() != 1)
            || tokens.len() > 3
        {
            return Err(syntax("unsupported block or arguments").at(span));
        }
        let item = tokens.get(1).map(|name| (*name).to_owned());
        let key = tokens.get(2).map(|name| (*name).to_owned());
        if item.as_ref().is_some_and(|name| !identifier(name))
            || key.as_ref().is_some_and(|name| !identifier(name))
            || (item.is_some() && item == key)
        {
            return Err(syntax("invalid or duplicate loop aliases").at(span));
        }
        let (yes, end) = self.sequence(depth + 1)?;
        let (no, end) = match end {
            End::Else(else_span) if kind == "@" => {
                return Err(syntax("loop cannot have else").at(else_span));
            }
            End::Else(_) => self.sequence(depth + 1)?,
            end => (Vec::new(), end),
        };
        // Closing tokens contain only block kind and path, never opener aliases.
        let expected = format!("{kind}:{name}");
        match end {
            End::Close(actual, _) if actual == expected => {}
            End::Close(_, span) | End::Else(span) => {
                return Err(syntax("mismatched block terminator").at(span));
            }
            End::Eof => return Err(syntax("unclosed block").at(span)),
        }
        if kind == "@" {
            Ok(Kind::Each {
                path,
                item,
                key,
                body: yes,
            })
        } else {
            Ok(Kind::If {
                path,
                inverted: kind == "!",
                yes,
                no,
            })
        }
    }

    /// Distinguish a strict value path from a registered inline helper call.
    fn inline(&self, tag: &str) -> TemplateResult<Kind> {
        let Some((name, source)) = tag.split_once(':') else {
            return Ok(Kind::Value(path(tag)?));
        };
        let name = name.trim_ascii();
        if !self.helpers.contains_key(name) {
            return Err(TemplateError::new(
                TemplateErrorKind::UnknownHelper,
                "unregistered inline helper",
            ));
        }
        let tokens = tokens(source)?;
        let first = tokens
            .first()
            .ok_or_else(|| syntax("helper requires a path"))?;
        let mut arguments = vec![Argument::Path(path(first)?)];
        for token in tokens.iter().skip(1) {
            // JSON literals cannot become implicit unquoted string fallbacks.
            let literal = token.starts_with('"')
                || matches!(*token, "null" | "true" | "false")
                || token
                    .starts_with(|character: char| character.is_ascii_digit() || character == '-');
            if literal {
                let value = serde_json::from_str::<Value>(token)
                    .map_err(|_| syntax("invalid JSON argument"))?;
                arguments.push(Argument::Literal(value));
            } else {
                arguments.push(Argument::Path(path(token)?));
            }
        }
        Ok(Kind::Helper {
            name: name.into(),
            arguments,
        })
    }
}

/// Tokenize JSON strings without splitting their escaped quotes or embedded spaces.
fn tokens(source: &str) -> TemplateResult<Vec<&str>> {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    let mut result = Vec::new();
    while cursor < bytes.len() {
        if bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
            continue;
        }
        let start = cursor;
        if bytes[cursor] == b'"' {
            cursor += 1;
            let mut closed = false;
            while cursor < bytes.len() {
                match bytes[cursor] {
                    b'\\' => cursor = cursor.saturating_add(2).min(bytes.len()),
                    b'"' => {
                        cursor += 1;
                        closed = true;
                        break;
                    }
                    _ => cursor += 1,
                }
            }
            if !closed || (cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace()) {
                return Err(syntax("invalid quoted argument"));
            }
        } else {
            while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
        }
        result.push(&source[start..cursor]);
    }
    Ok(result)
}

/// Accept only named roots and unambiguous dotted object/array segments.
fn path(source: &str) -> TemplateResult<Path> {
    let segments: Vec<_> = source.split('.').collect();
    let first = segments.first().copied().unwrap_or_default();
    let root = identifier(first)
        || matches!(
            first,
            "@root" | "@this" | "@key" | "@index" | "@first" | "@last" | "@odd" | "@even"
        );
    if !root
        || segments
            .iter()
            .skip(1)
            .any(|segment| !identifier(segment) && !index(segment))
    {
        return Err(syntax("invalid value path"));
    }
    Ok(segments.into_iter().map(str::to_owned).collect())
}

/// The deliberately portable ASCII identifier grammar used for paths and aliases.
pub(super) fn identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// Canonical decimal spelling avoids interpreting `01` as the first array element.
fn index(value: &str) -> bool {
    value == "0"
        || (!value.starts_with('0')
            && !value.is_empty()
            && value.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Construct a syntax error without echoing untrusted tag text.
fn syntax(message: &'static str) -> TemplateError {
    TemplateError::new(TemplateErrorKind::Syntax, message)
}
