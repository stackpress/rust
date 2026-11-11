"""Check Rust file references without mistaking strings/comments for Rust modules.

This is a conservative source audit, not a Rust macro expander. Literal paths and
concat!(env!("CARGO_MANIFEST_DIR"), ...) are supported. Computed include expressions
fail closed. Conditional modules are checked even when their cfg is disabled.
"""
from dataclasses import dataclass
from pathlib import Path
import re


@dataclass(frozen=True)
class Token:
    """A source token retaining its original line for actionable diagnostics."""
    value: str
    line: int
    string: bool = False


def tokens(source):
    """Skip nested comments and keep literals opaque, including generated Rust strings."""
    output = []
    index = 0
    line = 1
    while index < len(source):
        start = index
        char = source[index]
        if source.startswith('//', index):
            end = source.find('\n', index)
            index = len(source) if end < 0 else end
        elif source.startswith('/*', index):
            depth = 1
            index += 2
            while depth and index < len(source):
                if source.startswith('/*', index):
                    depth += 1
                    index += 2
                elif source.startswith('*/', index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
        else:
            raw = re.match(r'(?:b|c)?r(\#*)"', source[index:])
            if raw:
                opening = index + raw.end()
                closing = '"' + raw[1]
                end = source.find(closing, opening)
                if end < 0:
                    raise ValueError(f'unterminated raw string at line {line}')
                output.append(Token(source[opening:end], line, True))
                index = end + len(closing)
            elif char == '"' or (char in 'bc' and source[index:index + 2].endswith('"')):
                index += 1 if char == '"' else 2
                opening = index
                while index < len(source) and source[index] != '"':
                    index += 2 if source[index] == '\\' else 1
                # File paths in this workspace use ordinary ASCII escape sequences.
                value = source[opening:index]
                value = re.sub(r'\\([\\"])', r'\1', value)
                output.append(Token(value, line, True))
                index += 1
            elif char == "'" and (match := re.match(r"'(?:\\.|[^'\\\n])'", source[index:])):
                index += match.end()
            elif match := re.match(r'(?:r#)?[A-Za-z_][A-Za-z_0-9]*', source[index:]):
                output.append(Token(match[0].removeprefix('r#'), line))
                index += match.end()
            else:
                if not char.isspace():
                    output.append(Token(char, line))
                index += 1
        line += source[start:index].count('\n')
    return output


def literal_expression(items, root):
    """Evaluate only literal/concat/manifest-dir expressions; reject dynamic paths."""
    values = [t.value for t in items]
    if len(items) == 1 and items[0].string:
        return items[0].value
    if values[:3] == ['concat', '!', '('] and values[-1:] == [')']:
        parts = []
        start = 3
        depth = 0
        for i in range(3, len(items)):
            value = items[i].value
            if (value == ',' and depth == 0) or i == len(items) - 1:
                if i > start:
                    parts.append(literal_expression(items[start:i], root))
                start = i + 1
            elif value in ('(', '[', '{'):
                depth += 1
            elif value in (')', ']', '}'):
                depth -= 1
        return ''.join(parts)
    if values == ['env', '!', '(', 'CARGO_MANIFEST_DIR', ')']:
        return str(root)
    raise ValueError('computed file reference: ' + ' '.join(values))


def references(source, root, target_roots=()):
    """Yield location, spelling and allowed resolutions for each actual file reference."""
    items = tokens(source.read_text())
    # A non-root foo.rs owns implicit children under foo/; #[path] is file-relative.
    module_dir = source.parent if source.stem in ('lib', 'main', 'mod') or source in target_roots else source.with_suffix('')
    scopes = []
    inline = []
    pending_path = None
    i = 0
    while i < len(items):
        token = items[i]
        values = [t.value for t in items[i:i + 7]]
        if values[:4] == ['#', '[', 'path', '='] and items[i + 4].string:
            pending_path = items[i + 4]
            i += 6
            continue
        if token.value in ('include_str', 'include_bytes', 'include') and values[1:3] == ['!', '(']:
            end = i + 3
            depth = 1
            while end < len(items) and depth:
                if items[end].value == '(' and not items[end].string:
                    depth += 1
                elif items[end].value == ')' and not items[end].string:
                    depth -= 1
                end += 1
            expression = items[i + 3:end - 1]
            if expression and expression[-1].value == ',':
                expression = expression[:-1]
            try:
                literal = literal_expression(expression, root)
            except ValueError as error:
                yield token.line, str(error), []
            else:
                yield token.line, token.value + '!' + literal, [source.parent / literal]
            i = end
            continue
        if token.value == 'mod' and len(values) >= 3 and values[2] in (';', '{'):
            name, delimiter = values[1:3]
            if delimiter == ';':
                if pending_path:
                    base = source.parent.joinpath(*inline)
                    yield token.line, '#[path] ' + pending_path.value, [base / pending_path.value]
                else:
                    base = module_dir.joinpath(*inline) / name
                    yield token.line, 'mod ' + name, [base.with_suffix('.rs'), base / 'mod.rs']
                pending_path = None
                i += 3
                continue
            # Inline module braces change child module resolution, ordinary braces do not.
            inline.append(pending_path.value if pending_path else name)
            pending_path = None
            scopes.append(True)
            i += 3
            continue
        if token.value == '{' and not token.string:
            scopes.append(False)
        elif token.value == '}' and not token.string and scopes:
            if scopes.pop():
                inline.pop()
        i += 1


def check_source(root, relative, packaged, target_roots=()):
    """Require every resolution to remain inside the crate and its Cargo package list."""
    failures = []
    for line, spelling, choices in references(root / relative, root, target_roots):
        valid = []
        for choice in choices:
            try:
                name = choice.resolve().relative_to(root.resolve()).as_posix()
            except ValueError:
                continue
            if name in packaged:
                valid.append(name)
        if not valid:
            failures.append(f'{relative}:{line}: {spelling} missing from package')
    return failures
