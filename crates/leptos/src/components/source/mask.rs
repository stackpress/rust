//! G55 Rust subset of Inputmask: 9/a/* tokens, escapes, optional groups and finite quantifiers.

/// One common Inputmask position and whether it belongs to an optional suffix/group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceMaskToken {
    /// An ASCII decimal digit.
    Digit(bool),
    /// Inputmask's Latin/Cyrillic letter definition.
    Letter(bool),
    /// A digit or the same letter definition.
    Alphanumeric(bool),
    /// Fixed punctuation; escaped token characters are literal too.
    Literal(char, bool),
}
impl SourceMaskToken {
    /// Source token admission; no regex or alias language is inferred.
    fn accepts(&self, c: char) -> bool {
        let letter = c.is_ascii_alphabetic()
            || ('\u{00c0}'..='\u{00ff}').contains(&c)
            || ('\u{0410}'..='\u{044f}').contains(&c)
            || matches!(c, 'Ё' | 'ё' | 'µ');
        match self {
            Self::Digit(_) => c.is_ascii_digit(),
            Self::Letter(_) => letter,
            Self::Alphanumeric(_) => letter || c.is_ascii_digit(),
            Self::Literal(expected, _) => c == *expected,
        }
    }
    /// Optional quantifier/group positions can be omitted from the visible tail.
    fn optional(&self) -> bool {
        match self {
            Self::Digit(optional)
            | Self::Letter(optional)
            | Self::Alphanumeric(optional)
            | Self::Literal(_, optional) => *optional,
        }
    }
    /// Clone one repeated token with its quantifier optionality.
    fn repeated(&self, optional: bool) -> Self {
        match self {
            Self::Digit(_) => Self::Digit(optional),
            Self::Letter(_) => Self::Letter(optional),
            Self::Alphanumeric(_) => Self::Alphanumeric(optional),
            Self::Literal(c, _) => Self::Literal(*c, optional),
        }
    }
}

/// Compile the documented finite common mask grammar; unsupported syntax returns None.
/// Supported: 9, a, *, backslash escaping, optional [...], {n}, {n,m}.
/// Alternation, regex/aliases, group quantifiers and open-ended quantifiers are unsupported.
pub fn source_mask(pattern: &str) -> Option<Vec<SourceMaskToken>> {
    let mut chars = pattern.chars().peekable();
    let mut tokens = Vec::<SourceMaskToken>::new();
    let mut optional = 0usize;
    while let Some(c) = chars.next() {
        match c {
            '[' => optional += 1,
            ']' => {
                optional = optional.checked_sub(1)?;
            }
            '\\' => tokens.push(SourceMaskToken::Literal(chars.next()?, optional > 0)),
            '{' => {
                let previous = tokens.pop()?;
                let mut count = String::new();
                loop {
                    let c = chars.next()?;
                    if c == '}' {
                        break;
                    }
                    count.push(c);
                }
                let (min, max) = if let Some((min, max)) = count.split_once(',') {
                    (min.parse::<usize>().ok()?, max.parse::<usize>().ok()?)
                } else {
                    let count = count.parse::<usize>().ok()?;
                    (count, count)
                };
                if max < min {
                    return None;
                }
                for index in 0..max {
                    tokens.push(previous.repeated(previous.optional() || index >= min));
                }
            }
            '9' => tokens.push(SourceMaskToken::Digit(optional > 0)),
            'a' => tokens.push(SourceMaskToken::Letter(optional > 0)),
            '*' => tokens.push(SourceMaskToken::Alphanumeric(optional > 0)),
            '|' | '}' => return None,
            c => tokens.push(SourceMaskToken::Literal(c, optional > 0)),
        }
    }
    (optional == 0).then_some(tokens)
}

/// Fill a common mask, skipping invalid characters and retaining source placeholders when focused.
/// An unsupported mask is left as ordinary text, with no false claim of Inputmask coverage.
pub fn source_mask_value(pattern: &str, value: &str, focused: bool) -> String {
    let Some(tokens) = source_mask(pattern) else {
        return value.into();
    };
    if value.is_empty() && !focused {
        return String::new();
    }
    let mut input = value.chars().peekable();
    let mut output = String::new();
    for token in tokens {
        if let SourceMaskToken::Literal(c, optional) = token {
            if optional && input.peek().is_none() {
                continue;
            }
            output.push(c);
            if input.peek() == Some(&c) {
                input.next();
            }
            continue;
        }
        let found = input.by_ref().find(|c| token.accepts(*c));
        if let Some(c) = found {
            output.push(c);
        } else if !token.optional() {
            output.push('_');
        }
    }
    output
}

/// Backspace/Delete target editable slots rather than fixed punctuation or placeholder tails.
/// The returned cursor uses UTF-16 units, matching native input selection APIs.
pub fn source_mask_delete(
    pattern: &str,
    text: &str,
    start: usize,
    end: usize,
    backward: bool,
) -> Option<(String, usize)> {
    let tokens = source_mask(pattern)?;
    let mut units = text.encode_utf16().collect::<Vec<_>>();
    let start = start.min(units.len());
    let end = end.max(start).min(units.len());
    let editable = |index: usize| {
        tokens
            .get(index)
            .is_some_and(|token| !matches!(token, SourceMaskToken::Literal(..)))
    };
    let position = if start != end {
        (start..end).find(|index| editable(*index))
    } else if backward {
        (0..start)
            .rev()
            .find(|index| editable(*index) && units.get(*index) != Some(&95))
    } else {
        (start..units.len()).find(|index| editable(*index))
    }?;
    if start != end {
        for (index, unit) in units.iter_mut().enumerate().take(end).skip(start) {
            if editable(index) {
                *unit = 95;
            }
        }
    } else {
        units[position] = 95;
    }
    Some((
        source_mask_value(pattern, &String::from_utf16_lossy(&units), true),
        position,
    ))
}
