#!/usr/bin/env python3
"""Check release crates for test/example/bench file reads escaping their package.

Scan all Rust source as a conservative superset of in-source cfg(test) modules,
plus tests/examples/benches and their support modules/scripts. Nested Cargo packages are
repository-only consumers: referencing their files is also a packaging violation.
Comments and ordinary test data are not interpreted as filesystem access. Dynamic
Rust include expressions fail closed unless composed from literals and
CARGO_MANIFEST_DIR. This is a source boundary check, not a replacement for testing
an extracted Cargo archive with its normalized manifest.
"""

import argparse
import ast
from dataclasses import dataclass
import json
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parents[2]
PENDING = {
    "stackpress-sql-postgres", "stackpress-sql-mysql", "stackpress-framework-sql",
    "stackpress-framework-admin", "stackpress-framework-email",
}
# Consume comments before strings, preserving offsets for line-number diagnostics.
LEXEME = re.compile(
    r'//[^\n]*|/\*|(?:b?r)(?P<hashes>\#*)"[\s\S]*?"(?P=hashes)'
    r'|b?"(?:\\.|[^"\\])*"', re.MULTILINE,
)
SCRIPT_LEXEME = re.compile(
    r'''//[^\n]*|/\*|'(?:\\.|[^'\\])*'|"(?:\\.|[^"\\])*"|`(?:\\.|[^`\\])*`''',
    re.MULTILINE,
)
STRING = re.compile(r'(?:b?r)(\#*)"([\s\S]*)"\1|b?"((?:\\.|[^"\\])*)"\Z')
INCLUDE = re.compile(r'\b(include_str|include_bytes|include)\s*!\s*([({\[])')
PATH = re.compile(
    r'#\s*\[(?:[^\]"\[]|"(?:\\.|[^"\\])*")*?\bpath\s*=\s*'
    r'(?P<literal>r(?P<hashes>\#*)"[\s\S]*?"(?P=hashes)|"(?:\\.|[^"\\])*")'
)
MANIFEST_PATH = re.compile(
    r'(?:Path|PathBuf)::(?:new|from)\s*\(\s*env!\s*\(\s*"CARGO_MANIFEST_DIR"\s*\)\s*\)'
    r'(?P<joins>(?:\s*\.join\s*\(\s*"(?:\\.|[^"\\])*"\s*\))*)'
)
JOIN = re.compile(r'\.join\s*\(\s*("(?:\\.|[^"\\])*")\s*\)')
FIXED_READ = re.compile(
    r'\b(?:read|read_to_string|read_dir|open|metadata|canonicalize|set_current_dir|current_dir)'
    r'\s*\(\s*("(?:\\.|[^"\\])*")'
)
SCRIPT_LITERAL = r'''(?:"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')'''
SCRIPT_READ = re.compile(r'\b(?:readFileSync|readFile|require)\s*\(\s*(' + SCRIPT_LITERAL + r')')
SCRIPT_JOIN = re.compile(r'\bpath\.join\s*\(\s*__dirname\s*,\s*(' + SCRIPT_LITERAL + r')')


@dataclass(frozen=True)
class Finding:
    """Keep package identity and source location alongside each actionable issue."""
    package: str
    source: Path
    line: int
    message: str


def uncomment(source, lexer=LEXEME):
    """Mask nested Rust comments without changing strings or diagnostic offsets."""
    chars = list(source)
    offset = 0
    while match := lexer.search(source, offset):
        start, end = match.span()
        if match.group() == "/*":
            depth = 1
            while depth and end < len(source):
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
        if source.startswith(("//", "/*"), start):
            chars[start:end] = ["\n" if char == "\n" else " " for char in source[start:end]]
        offset = end
    return "".join(chars)


def literal(expression):
    """Decode a complete Rust path literal, including raw strings."""
    match = STRING.fullmatch(expression.strip())
    if not match:
        return None
    if match[2] is not None:
        return match[2]
    try:
        return ast.literal_eval('"' + match[3] + '"')
    except (ValueError, SyntaxError):
        return None


def expression_end(source, opening):
    """Balance macro delimiters while ignoring delimiter characters in strings."""
    strings = {match.start(): match.end() for match in LEXEME.finditer(source)}
    depth, offset = 1, opening + 1
    while offset < len(source):
        if offset in strings:
            offset = strings[offset]
            continue
        char = source[offset]
        if char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
            if depth == 0:
                return offset
        offset += 1
    return len(source)


def include_path(expression, crate):
    """Resolve literal and concat!(env!(CARGO_MANIFEST_DIR), ...) includes."""
    direct = literal(expression)
    if direct is not None:
        return direct
    match = re.fullmatch(r'\s*concat!\s*\((.*)\)\s*', expression, re.DOTALL)
    if not match:
        return None
    body = re.sub(r'env!\s*\(\s*"CARGO_MANIFEST_DIR"\s*\)', lambda _: json.dumps(str(crate)), match[1])
    parts = []
    while body.strip().strip(",").strip():
        body = body.lstrip()
        token = LEXEME.match(body)
        if not token or (part := literal(token.group())) is None:
            return None
        parts.append(part)
        body = body[token.end():].lstrip()
        if body and not body.startswith(","):
            return None
        body = body.removeprefix(",")
    return "".join(parts)


def outside(destination, crate):
    """Reject escaped paths and files Cargo omits as nested packages."""
    destination = destination.resolve()
    if not destination.is_relative_to(crate):
        return "outside crate"
    for parent in (destination.parent, *destination.parents):
        if parent == crate:
            break
        if (parent / "Cargo.toml").is_file():
            return "inside a nested Cargo package omitted from the archive"
    return None


def scan_file(package, crate, path):
    """Inspect explicit compile-time reads and fixed runtime filesystem paths."""
    lexer = LEXEME if path.suffix == ".rs" else SCRIPT_LEXEME
    source = uncomment(path.read_text(), lexer)
    findings = []
    # Source strings are data, except where an access expression consumes them.
    strings = [(match.start(), match.end()) for match in lexer.finditer(source)]

    def report(offset, message):
        findings.append(Finding(package, path, source.count("\n", 0, offset) + 1, message))

    def check(offset, value, base):
        if problem := outside(base / value, crate):
            report(offset, f"{value}: {problem}")

    for match in INCLUDE.finditer(source):
        if any(start <= match.start() < end for start, end in strings):
            continue
        end = expression_end(source, match.end() - 1)
        value = include_path(source[match.end():end], crate)
        if value is None:
            report(match.start(), "unresolved include expression; use a package-local literal")
        else:
            check(match.start(), value, path.parent)
    for match in PATH.finditer(source):
        if not any(start <= match.start() < end for start, end in strings):
            # Conditional module paths have the same package boundary as an
            # unconditional #[path] attribute, even on an inactive feature.
            check(match.start(), literal(match["literal"]), path.parent)
    for match in MANIFEST_PATH.finditer(source):
        if any(start <= match.start() < end for start, end in strings):
            continue
        destination = crate
        for join in JOIN.finditer(match["joins"]):
            destination /= literal(join[1])
        check(match.start(), destination, crate)
    for match in FIXED_READ.finditer(source):
        if any(start <= match.start() < end for start, end in strings):
            continue
        value = literal(match[1])
        # Only fixed relative escapes are repository reads; OS tools, temporary
        # directories, and caller-supplied paths are not sibling source inputs.
        if value and value.startswith("../"):
            check(match.start(), value, crate)
    if path.suffix in {".js", ".cjs", ".mjs"}:
        for pattern, base in [(SCRIPT_READ, crate), (SCRIPT_JOIN, path.parent)]:
            for match in pattern.finditer(source):
                if any(start <= match.start() < end for start, end in strings):
                    continue
                value = ast.literal_eval(match[1])
                if pattern is SCRIPT_JOIN or value.startswith("../"):
                    check(match.start(), value, base)
    return findings


def scan_python(package, crate, path):
    """Resolve fixed __file__ anchors in proof scripts without executing them."""
    tree = ast.parse(path.read_text(), filename=str(path))
    anchors = {"__file__": path.resolve()}
    findings = []

    def evaluate(node):
        # Track only paths rooted in this script, keeping caller-supplied inputs
        # and temporary directories outside this repository-boundary analysis.
        if isinstance(node, ast.Name):
            return anchors.get(node.id)
        if isinstance(node, ast.Call):
            if isinstance(node.func, ast.Name) and node.func.id == "Path" and node.args:
                return evaluate(node.args[0])
            if isinstance(node.func, ast.Attribute) and node.func.attr in {"resolve", "absolute"}:
                return evaluate(node.func.value)
        if isinstance(node, ast.Attribute) and node.attr == "parent":
            if base := evaluate(node.value):
                return base.parent
        if isinstance(node, ast.Subscript) and isinstance(node.value, ast.Attribute) and node.value.attr == "parents":
            base = evaluate(node.value.value)
            if base and isinstance(node.slice, ast.Constant) and isinstance(node.slice.value, int):
                try:
                    return base.parents[node.slice.value]
                except IndexError:
                    return None
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
            base = evaluate(node.left)
            if base and isinstance(node.right, ast.Constant) and isinstance(node.right.value, str):
                return base / node.right.value
        return None

    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and (destination := evaluate(node.value)):
            for name in node.targets:
                if isinstance(name, ast.Name):
                    anchors[name.id] = destination
            if problem := outside(destination, crate):
                findings.append(Finding(package, path, node.lineno, f"fixed script anchor {destination}: {problem}"))
        if isinstance(node, ast.Call) and node.args:
            function = node.func.id if isinstance(node.func, ast.Name) else node.func.attr if isinstance(node.func, ast.Attribute) else ""
            argument = node.args[0]
            if function in {"open", "read", "read_text", "read_bytes"} and isinstance(argument, ast.Constant) and isinstance(argument.value, str) and argument.value.startswith("../"):
                if problem := outside(crate / argument.value, crate):
                    findings.append(Finding(package, path, node.lineno, f"{argument.value}: {problem}"))
    return findings


def candidates(root):
    """Use workspace membership rather than a drifting second crate catalog."""
    workspace = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]
    for directory in [".", *workspace["members"]]:
        crate = (root / directory).resolve()
        manifest = tomllib.loads((crate / "Cargo.toml").read_text())
        name = manifest["package"]["name"]
        if name == "stackpress" or directory.startswith("crates/") or directory == "tools/cli":
            yield name, crate


def scan_crate(package, crate):
    """Visit support modules too, without traversing nested consumer packages."""
    findings = []
    for directory in ("src", "tests", "examples", "benches"):
        # The facade's examples/ is a repository application collection, with no
        # Cargo example targets of its own. A future declared/autodiscovered
        # target re-enables the scan; nested application packages stay excluded.
        if crate == ROOT and directory == "examples":
            manifest = tomllib.loads((crate / "Cargo.toml").read_text())
            examples = crate / "examples"
            if not manifest.get("example") and not list(examples.glob("*.rs")) and not list(examples.glob("*/main.rs")):
                continue
        for path in sorted((crate / directory).rglob("*")):
            if not path.is_file() or path.suffix not in {".rs", ".py", ".js", ".cjs", ".mjs", ".sh"}:
                continue
            if any((parent / "Cargo.toml").exists() for parent in path.parents if parent != crate and parent.is_relative_to(crate)):
                continue
            if problem := outside(path, crate):
                findings.append(Finding(package, path, 1, problem))
            elif path.suffix == ".py":
                findings.extend(scan_python(package, crate, path))
            else:
                # Fixed literal read/open expressions are shared by the support
                # scripts and Rust. Rust-only include/path patterns simply do
                # not occur in the other support languages.
                findings.extend(scan_file(package, crate, path))
    return findings


def main():
    """Report all candidates, keeping the coverage lane's pending work explicit."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pending-coverage", action="store_true", help="report the five separately owned crates as pending")
    parser.add_argument("--package", action="append", help="restrict a focused check to this candidate")
    args = parser.parse_args()
    errors = 0
    selected = [(name, crate) for name, crate in candidates(ROOT) if not args.package or name in args.package]
    if args.package and set(args.package) - {name for name, _ in selected}:
        parser.error("unknown release candidate")
    for name, crate in selected:
        findings = scan_crate(name, crate)
        pending = args.pending_coverage and name in PENDING
        for finding in findings:
            print(f'{finding.source.relative_to(ROOT)}:{finding.line}: {"PENDING: " if pending else ""}{finding.message}')
        print(f'{name}: {"PENDING coverage worker" if pending else "PASS" if not findings else "FAIL"} ({len(findings)} references)')
        if not pending:
            errors += len(findings)
    print(f"Test boundaries: {len(selected)} candidates; {errors} errors outside pending work")
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(main())
