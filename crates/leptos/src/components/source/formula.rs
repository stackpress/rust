//! G55 arithmetic-only replacement for Frui's eval, identical on native and WASM targets.

/// Fill each source template once, then evaluate numbers, + - * / %, parentheses and unary -.
/// Unknown first remaining template becomes zero; malformed/non-arithmetic input returns zero.
/// IEEE Infinity/NaN are results, not parse errors, matching JavaScript arithmetic.
/// Data keys are normalized source field identifiers; arbitrary application keys are literal,
/// rather than being interpreted as JavaScript RegExp fragments as in the original component.
pub fn source_formula(formula: &str, value: f64, data: &[(String, f64)]) -> f64 {
    let mut expression = formula.replacen("{this}", &number(value), 1);
    for (key, value) in data {
        expression = expression.replacen(&format!("{{{key}}}"), &number(*value), 1);
    }
    if let Some(start) = expression.find('{')
        && let Some(end) = expression[start..].find('}')
    {
        expression.replace_range(start..=start + end, "0");
    }
    let mut parser = Parser {
        text: expression.as_bytes(),
        index: 0,
    };
    parser.evaluate().unwrap_or(0.0)
}

/// Frui substitutes Number(value) || 0; NaN and both signed zeros become zero.
fn number(value: f64) -> String {
    if value == 0.0 || value.is_nan() {
        "0".into()
    } else if value.is_infinite() {
        if value.is_sign_negative() {
            "-Infinity".into()
        } else {
            "Infinity".into()
        }
    } else {
        value.to_string()
    }
}

/// Arithmetic operator stack; explicit precedence avoids recursion for deeply nested input.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Operator {
    Open,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}
impl Operator {
    /// Higher precedence operators finish before a lower/equal left-associative operator.
    fn precedence(self) -> u8 {
        match self {
            Self::Open => 0,
            Self::Add | Self::Subtract => 1,
            Self::Multiply | Self::Divide | Self::Remainder => 2,
            Self::Negate => 3,
        }
    }
    /// Apply one operation, preserving IEEE division/remainder and signed zero.
    fn apply(self, values: &mut Vec<f64>) -> Option<()> {
        let right = values.pop()?;
        if self == Self::Negate {
            values.push(-right);
            return Some(());
        }
        let left = values.pop()?;
        values.push(match self {
            Self::Add => left + right,
            Self::Subtract => left - right,
            Self::Multiply => left * right,
            Self::Divide => left / right,
            Self::Remainder => left % right,
            Self::Open | Self::Negate => return None,
        });
        Some(())
    }
}

/// Borrowed token scanner and iterative operator/value stacks; no eval or recursive call tree.
struct Parser<'a> {
    text: &'a [u8],
    index: usize,
}
impl Parser<'_> {
    /// JavaScript accepts Unicode whitespace/BOM between arithmetic tokens.
    fn space(&mut self) {
        while let Some(c) = std::str::from_utf8(&self.text[self.index..])
            .ok()
            .and_then(|text| text.chars().next())
        {
            if !c.is_whitespace() && c != '\u{feff}' {
                break;
            }
            self.index += c.len_utf8();
        }
    }
    /// Reduce prefix minus operators after their numeric or parenthesized operand completes.
    fn negate(operators: &mut Vec<Operator>, values: &mut Vec<f64>) -> Option<()> {
        while operators.last() == Some(&Operator::Negate) {
            operators.pop()?.apply(values)?;
        }
        Some(())
    }
    /// Parse the complete arithmetic grammar with explicit operand/operator state.
    fn evaluate(&mut self) -> Option<f64> {
        let mut operators = Vec::<Operator>::new();
        let mut values = Vec::<f64>::new();
        let mut operand = true;
        loop {
            self.space();
            let Some(token) = self.text.get(self.index).copied() else {
                break;
            };
            if operand {
                match token {
                    b'-' => {
                        operators.push(Operator::Negate);
                        self.index += 1;
                    }
                    b'(' => {
                        operators.push(Operator::Open);
                        self.index += 1;
                    }
                    _ => {
                        values.push(self.number()?);
                        operand = false;
                        Self::negate(&mut operators, &mut values)?;
                    }
                }
                continue;
            }
            if token == b')' {
                self.index += 1;
                while operators
                    .last()
                    .is_some_and(|operator| *operator != Operator::Open)
                {
                    operators.pop()?.apply(&mut values)?;
                }
                if operators.pop() != Some(Operator::Open) {
                    return None;
                }
                Self::negate(&mut operators, &mut values)?;
                continue;
            }
            let operator = match token {
                b'+' => Operator::Add,
                b'-' => Operator::Subtract,
                b'*' => Operator::Multiply,
                b'/' => Operator::Divide,
                b'%' => Operator::Remainder,
                _ => return None,
            };
            // Equal precedence reduces first, preserving left associativity for / and %.
            while operators.last().is_some_and(|previous| {
                *previous != Operator::Open && previous.precedence() >= operator.precedence()
            }) {
                operators.pop()?.apply(&mut values)?;
            }
            operators.push(operator);
            self.index += 1;
            operand = true;
        }
        if operand {
            return None;
        }
        while let Some(operator) = operators.pop() {
            if operator == Operator::Open {
                return None;
            }
            operator.apply(&mut values)?;
        }
        if values.len() == 1 {
            values.pop()
        } else {
            None
        }
    }
    /// Decimal/scientific literals plus JavaScript numeric radix prefixes and overflow spellings.
    fn number(&mut self) -> Option<f64> {
        let start = self.index;
        for (token, value) in [
            (b"Infinity".as_slice(), f64::INFINITY),
            (b"NaN".as_slice(), f64::NAN),
        ] {
            if self.text.get(start..start + token.len()) == Some(token) {
                self.index += token.len();
                return Some(value);
            }
        }
        if self.text.get(start) == Some(&b'0')
            && let Some(radix) = self.text.get(start + 1).and_then(|prefix| match prefix {
                b'x' | b'X' => Some(16),
                b'o' | b'O' => Some(8),
                b'b' | b'B' => Some(2),
                _ => None,
            })
        {
            self.index += 2;
            let first = self.index;
            let mut value = 0.0;
            while let Some(digit) = self
                .text
                .get(self.index)
                .and_then(|byte| (*byte as char).to_digit(radix))
            {
                value = value * radix as f64 + digit as f64;
                self.index += 1;
            }
            return (self.index > first).then_some(value);
        }
        while self.text.get(self.index).is_some_and(u8::is_ascii_digit) {
            self.index += 1;
        }
        if self.text.get(self.index) == Some(&b'.') {
            self.index += 1;
            while self.text.get(self.index).is_some_and(u8::is_ascii_digit) {
                self.index += 1;
            }
        }
        if self
            .text
            .get(self.index)
            .is_some_and(|c| *c == b'e' || *c == b'E')
        {
            self.index += 1;
            if self
                .text
                .get(self.index)
                .is_some_and(|c| *c == b'+' || *c == b'-')
            {
                self.index += 1;
            }
            while self.text.get(self.index).is_some_and(u8::is_ascii_digit) {
                self.index += 1;
            }
        }
        std::str::from_utf8(&self.text[start..self.index])
            .ok()?
            .parse()
            .ok()
    }
}
