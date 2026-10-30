#!/usr/bin/env python3
"""Check session doc structure/links/claim anchors and exact external consumer examples.

Record this outer proof once. --compile reuses existing locked/offline consumer graphs;
--format formats only Markdown Rust fences. No repository Rust source/lock is written.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[4]
CRATE = Path(__file__).resolve().parents[1]
DOCS = Path(__file__).resolve().parent
FENCES = re.compile(r'^```([^\n]*)\n(.*?)^```\s*$', re.M | re.S)
MODES = {'direct', 'server', 'facade', 'browser'}


def pages():
    """Keep all entry/detail pages in the same mechanical review scope."""
    return [CRATE / 'README.md', *sorted(DOCS.rglob('*.md'))]


def slug(title):
    """Match the simple GitHub anchors authored in this documentation set."""
    return re.sub(r'[^\w\- ]', '', title.lower().replace('`', '')).replace(' ', '-')


def document(path, text):
    """Reject unclassified fences and malformed Markdown before compiling any code."""
    visible = FENCES.sub('', text)
    headings = re.findall(r'^(#{1,6}) (.+)$', visible, re.M)
    assert sum(level == '#' for level, _ in headings) == 1, path
    assert '\n\n\n' not in text, (path, 'consecutive blank lines')
    lines = visible.splitlines()
    for index, line in enumerate(lines):
        if re.match(r'^#{1,6} ', line):
            following = next((value for value in lines[index + 1:] if value.strip()), '')
            assert not following.startswith('#'), (path, line, 'heading needs orientation')
    for target in re.findall(r'(?<!!)\[[^\]]+\]\(([^)\s]+)\)', visible):
        assert not re.match(r'^[a-z]+:', target), (path, 'external link requires explicit review', target)
        name, _, anchor = target.partition('#')
        destination = (path.parent / name).resolve() if name else path
        assert destination.exists(), (path, target)
        if anchor:
            assert destination.suffix == '.md', (path, target)
            anchors = {slug(title) for _, title in re.findall(r'^(#{1,6}) (.+)$', destination.read_text(), re.M)}
            assert anchor in anchors, (path, target)
    matches = list(FENCES.finditer(text))
    assert text.count('```') == 2 * len(matches), (path, 'unclosed fence')
    snippets = []
    for match in matches:
        assert match[1] == 'rust', (path, 'every runnable example must be plain Rust')
        marker = text[:match.start()].rstrip().splitlines()[-1]
        mode = re.fullmatch(r'<!-- example: (\w+) -->', marker)
        assert mode and mode[1] in MODES, (path, 'example needs explicit graph', marker)
        snippets.append({'path': str(path.relative_to(ROOT)), 'line': text[:match.start()].count('\n') + 1,
                         'mode': mode[1], 'body': match[2]})
    return snippets


def check_claim(claim):
    """Check exact statement/evidence anchors without claiming automatic semantic proof."""
    assert claim['statement'] in (ROOT / claim['page']).read_text(), claim['id']
    for evidence in claim['evidence']:
        source = (ROOT / evidence['path']).read_text()
        for anchor in evidence['contains']:
            assert anchor in source, (claim['id'], evidence['path'], anchor)


def negative_controls(claims):
    """Exercise omitted example, missing target/anchor and stale source evidence in memory."""
    path = CRATE / 'README.md'
    probes = [
        lambda: document(path, '# Probe\n\nContent.\n\n```rust\nfn omitted() {}\n```\n'),
        lambda: document(path, '# Probe\n\n[Missing](docs/missing-docs-negative-control.md)\n'),
        lambda: document(path, '# Probe\n\n[Missing](docs/concepts.md#missing-docs-negative-control)\n'),
        lambda: check_claim({**claims[0], 'evidence': [
            {'path': claims[0]['evidence'][0]['path'], 'contains': ['MISSING-DOCS-NEGATIVE-CONTROL']}]}),
    ]
    for index, probe in enumerate(probes):
        try:
            probe()
        except AssertionError:
            continue
        raise AssertionError(f'negative control {index} unexpectedly accepted')


def inspect():
    """Inspect every Markdown page, exact fence and claim anchor."""
    snippets = [item for path in pages() for item in document(path, path.read_text())]
    assert {item['mode'] for item in snippets} == MODES
    claims = json.loads((DOCS / 'claims.json').read_text())
    assert len({claim['id'] for claim in claims}) == len(claims)
    errors = []
    for claim in claims:
        try:
            check_claim(claim)
        except AssertionError as error:
            errors.append(str(error))
    assert not errors, errors
    negative_controls(claims)
    print(json.dumps({'pages': len(pages()), 'rust_fences': len(snippets), 'claims': len(claims),
                      'negative_controls': 4, 'links': 'all local paths/anchors resolve'}, indent=2), flush=True)
    return snippets


def format_examples():
    """Use rustfmt on isolated fence bodies and write only their Markdown containers."""
    for path in pages():
        text = path.read_text()
        def formatted(match):
            assert match[1] == 'rust', path
            body = subprocess.check_output(['rustfmt', '--edition', '2024', '--emit', 'stdout'],
                                           input=match[2], text=True)
            return '```rust\n' + body + '```\n'
        path.write_text(FENCES.sub(formatted, text))


def run(command, env):
    """Print nested Cargo argv into the single outer recorder log; the effort wrapper bounds it."""
    print('COMMAND:', *map(str, command), flush=True)
    subprocess.run(command, env=env, check=True)


def compile_examples(snippets):
    """Give every fence an isolated module in an unchanged existing consumer graph."""
    target = Path(os.environ['CARGO_TARGET_DIR'])
    assert target.is_absolute(), 'the effort target must be absolute'
    external = Path(tempfile.mkdtemp(prefix='slice9a-docs-', dir='/private/tmp'))
    root_lock = (ROOT / 'Cargo.lock').read_bytes()
    executions = {
        'first_session': 'async', 'explicit_permission': 'async', 'profile_defaults': 'sync',
        'configured_session': 'value', 'identity_host': 'async', 'direct_signout': 'async',
        'owned_hooks': 'async', 'otp_template': 'unit', 'sqlite_profile': 'async',
        'ordered_csv': 'sync', 'facade_session': 'async',
    }
    report = []
    for mode, template in [('direct', 'direct'), ('server', 'plugin'), ('facade', 'facade'), ('browser', 'browser')]:
        workspace = external / mode
        (workspace / 'src').mkdir(parents=True)
        original = CRATE / f'consumers/{template}/Cargo.toml'
        manifest = re.sub(r'path = "([^"]+)"', lambda match: 'path = ' + json.dumps(
            str((original.parent / match[1]).resolve())), original.read_text())
        (workspace / 'Cargo.toml').write_text(manifest)
        lock = original.with_name('Cargo.lock').read_bytes()
        (workspace / 'Cargo.lock').write_bytes(lock)
        modules, tests = [], []
        for index, item in enumerate(snippets):
            if item['mode'] != mode:
                continue
            name = f'example_{index}'
            # No scaffold import leaks between examples; the complete fence is copied verbatim.
            file = workspace / f'src/{name}.rs'
            file.write_text(item['body'])
            run(['rustfmt', '--edition', '2024', '--check', str(file)], os.environ)
            modules.append(f'// {item["path"]}:{item["line"]}\npub mod {name};')
            for function, execution in executions.items():
                if not re.search(rf'pub (?:async )?fn {function}\(', item['body']):
                    continue
                expression = f'super::{name}::{function}()' + ('.await' if execution == 'async' else '')
                if execution not in {'value', 'unit'}:
                    expression += '.unwrap()'
                tests.append(f'#[tokio::test(flavor="current_thread")]\nasync fn {name}_{function}() {{ let _ = {expression}; }}')
            report.append({key: item[key] for key in ['path', 'line', 'mode']} | {
                'sha256': hashlib.sha256(item['body'].encode()).hexdigest()})
        if mode != 'browser':
            modules.append('#[cfg(test)]\nmod checked_examples {\n' + '\n'.join(tests) + '\n}')
        (workspace / 'src/lib.rs').write_text('\n'.join(modules) + '\n')
        env = {**os.environ, 'CARGO_TARGET_DIR': str(target / f'consumer-session-{template}')}
        flags = ['--manifest-path', str(workspace / 'Cargo.toml'), '--locked', '--offline']
        if mode == 'browser':
            for renderer in ['hydrate', 'csr']:
                run(['cargo', 'check', '--lib', *flags, '--target', 'wasm32-unknown-unknown', '--features', renderer], env)
        else:
            run(['cargo', 'check', '--all-targets', *flags], env)
            run(['cargo', 'test', '--lib', *flags, '--', '--test-threads', '2'], env)
        assert (workspace / 'Cargo.lock').read_bytes() == lock
        assert original.with_name('Cargo.lock').read_bytes() == lock
        assert (ROOT / 'Cargo.lock').read_bytes() == root_lock
        print(f'{mode}: exact fences compiled; {len(tests)} finite examples executed', flush=True)
    result = {'external_workspace': str(external), 'examples': report}
    (external / 'examples.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'external_workspace': str(external), 'checked_fences': len(report)}, indent=2), flush=True)


def main():
    """Static checks are the default; compilation/formatting are explicit recorded operations."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compile', action='store_true')
    parser.add_argument('--format', action='store_true')
    args = parser.parse_args()
    if args.format:
        format_examples()
    snippets = inspect()
    if args.compile:
        compile_examples(snippets)


if __name__ == '__main__':
    main()
