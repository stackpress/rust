#!/usr/bin/env python3
"""Check aggregate docs/claims and compile exact fences in frozen external consumer graphs.

Record this outer command once. --format changes only Markdown fences. --compile writes
disposable /private/tmp workspaces, never repository source, manifests or locks.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[4]
CRATE = Path(__file__).resolve().parents[1]
DOCS = Path(__file__).resolve().parent
FENCES = re.compile(r'^```([^\n]*)\n(.*?)^```\s*$', re.M | re.S)
GRAPHS = {'full': 'full', 'headless': 'headless', 'facade': 'facade',
          'browser': 'website', 'buildhost': 'build-host'}
EXECUTIONS = {'first_aggregate': 'async', 'catalog_host': 'async', 'finite_serve': 'async',
              'packaging_lifecycle': 'async', 'facade_host': 'async', 'absent_client': 'async',
              'generation_options': 'sync', 'sqlite_transaction': 'async',
              'translation': 'unit', 'build_registry': 'sync', 'compose_application': 'sync'}


def pages():
    """Include the entry and every detail page in the same review inventory."""
    return [CRATE / 'README.md', *sorted(DOCS.rglob('*.md'))]


def slug(title):
    """Resolve the simple GitHub heading anchors authored by this doc set."""
    return re.sub(r'[^\w\- ]', '', title.lower().replace('`', '')).replace(' ', '-')


def document(path, text):
    """Reject malformed Markdown, unresolved links and unclassified executable fences."""
    visible = FENCES.sub('', text)
    headings = re.findall(r'^(#{1,6}) (.+)$', visible, re.M)
    assert sum(level == '#' for level, _ in headings) == 1, path
    assert '\n\n\n' not in text, (path, 'consecutive blank lines')
    lines = visible.splitlines()
    for index, line in enumerate(lines):
        if re.match(r'^#{1,6} ', line):
            following = next((item for item in lines[index + 1:] if item.strip()), '')
            assert not following.startswith('#'), (path, line, 'heading needs orientation')
    for target in re.findall(r'(?<!!)\[[^\]]+\]\(([^)\s]+)\)', visible):
        assert not re.match(r'^[a-z]+:', target), (path, 'external link needs review', target)
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
        assert match[1] == 'rust', (path, 'plain Rust examples only')
        marker = text[:match.start()].rstrip().splitlines()[-1]
        mode = re.fullmatch(r'<!-- example: (\w+) -->', marker)
        assert mode and mode[1] in GRAPHS, (path, 'example needs graph classification', marker)
        snippets.append({'path': str(path.relative_to(ROOT)), 'line': text[:match.start()].count('\n') + 1,
                         'mode': mode[1], 'body': match[2]})
    return snippets


def check_claim(claim):
    """Pin exact page statements and source anchors; semantic review remains necessary."""
    assert claim['statement'] in (ROOT / claim['page']).read_text(), claim['id']
    assert claim['evidence'], (claim['id'], 'missing evidence')
    for item in claim['evidence']:
        source = (ROOT / item['path']).read_text()
        assert item['contains'], claim['id']
        for anchor in item['contains']:
            assert anchor in source, (claim['id'], item['path'], anchor)


def inventory(claims, snippets, page_paths):
    """Every page has a claim; every finite example has an explicit execution selection."""
    assert len({item['id'] for item in claims}) == len(claims), 'duplicate claim identity'
    assert {item['page'] for item in claims} == set(page_paths), 'claim page inventory'
    assert {item['mode'] for item in snippets} == set(GRAPHS), 'example graph inventory'
    functions = [name for item in snippets for name in EXECUTIONS
                 if re.search(rf'pub (?:async )?fn {name}\(', item['body'])]
    assert sorted(functions) == sorted(EXECUTIONS), 'finite execution inventory'


def negative_controls(claims, snippets):
    """Prove structural, anchor, page-inventory and execution omissions fail closed."""
    path = CRATE / 'README.md'
    paths = [str(page.relative_to(ROOT)) for page in pages()]
    probes = [
        lambda: document(path, '# Probe\n\nText.\n\n```rust\nfn omitted() {}\n```\n'),
        lambda: document(path, '# Probe\n\n[Missing](docs/missing-docs-control.md)\n'),
        lambda: document(path, '# Probe\n\n[Missing](docs/concepts.md#missing-docs-control)\n'),
        lambda: check_claim({**claims[0], 'evidence': [
            {'path': claims[0]['evidence'][0]['path'], 'contains': ['MISSING-DOCS-CONTROL']}]}),
        lambda: inventory(claims + [claims[0]], snippets, paths),
        lambda: inventory(claims, snippets, paths + ['MISSING-PAGE-CONTROL']),
        lambda: inventory(claims, [item for item in snippets if 'pub async fn first_aggregate(' not in item['body']], paths),
    ]
    for index, probe in enumerate(probes):
        try:
            probe()
        except AssertionError:
            continue
        raise AssertionError(f'negative control {index} unexpectedly accepted')
    return len(probes)


def inspect():
    """Review every page, current claim and exact example body."""
    snippets, errors = [], []
    for page in pages():
        try:
            snippets.extend(document(page, page.read_text()))
        except AssertionError as error:
            errors.append(str(error))
    assert not errors, errors
    claims = json.loads((DOCS / 'claims.json').read_text())
    errors = []
    for claim in claims:
        try:
            check_claim(claim)
        except AssertionError as error:
            errors.append(str(error))
    assert not errors, errors
    inventory(claims, snippets, [str(page.relative_to(ROOT)) for page in pages()])
    controls = negative_controls(claims, snippets)
    print(json.dumps({'pages': len(pages()), 'rust_fences': len(snippets), 'claims': len(claims),
                      'negative_controls': controls, 'links': 'all local paths/anchors resolve'}, indent=2), flush=True)
    return snippets


def run(command, env):
    """Bound each process group externally, terminating/reaping before a timeout is reported."""
    print('COMMAND:', *map(str, command), flush=True)
    child = subprocess.Popen(command, env=env, start_new_session=True)
    try:
        child.wait(timeout=900)
    finally:
        if child.poll() is None:
            os.killpg(child.pid, signal.SIGTERM)
            try:
                child.wait(timeout=2)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
            raise TimeoutError('documentation command deadline; process group reaped')
    assert child.returncode == 0, (command, child.returncode)


def format_examples():
    """Format isolated exact Rust bodies, writing only their Markdown containers."""
    for page in pages():
        def formatted(match):
            assert match[1] == 'rust', page
            body = subprocess.check_output(['rustfmt', '--edition', '2024', '--emit', 'stdout'],
                                           input=match[2], text=True, timeout=30)
            return '```rust\n' + body + '```\n'
        page.write_text(FENCES.sub(formatted, page.read_text()))


def registry(data):
    """Compare complete registry identities without flattening duplicate versions."""
    return {(p['name'], p['version'], p['source'], p.get('checksum'))
            for p in tomllib.loads(data.decode())['package'] if p.get('source', '').startswith('registry+')}


def compile_examples(snippets):
    """Compile exact modules with no hidden imports or manifest/lock resolution."""
    target = Path(os.environ['CARGO_TARGET_DIR'])
    assert target.is_absolute(), 'supply absolute effort CARGO_TARGET_DIR'
    external = Path(tempfile.mkdtemp(prefix='slice11-docs-', dir='/private/tmp'))
    root_lock = (ROOT / 'Cargo.lock').read_bytes()
    report, executions = [], []
    for mode, template in GRAPHS.items():
        workspace = external / mode
        (workspace / 'src').mkdir(parents=True)
        original = CRATE / f'consumers/{template}/Cargo.toml'
        manifest = re.sub(r'path = "([^"]+)"', lambda match: 'path = ' + json.dumps(
            str((original.parent / match[1]).resolve())), original.read_text())
        (workspace / 'Cargo.toml').write_text(manifest)
        lock = original.with_name('Cargo.lock').read_bytes()
        assert registry(lock) <= registry(root_lock), template
        (workspace / 'Cargo.lock').write_bytes(lock)
        modules, tests = [], []
        for index, item in enumerate(snippets):
            if item['mode'] != mode:
                continue
            name = f'example_{index}'
            file = workspace / f'src/{name}.rs'
            file.write_text(item['body'])
            run(['rustfmt', '--edition', '2024', '--check', str(file)], os.environ)
            modules.append(f'// {item["path"]}:{item["line"]}\npub mod {name};')
            invoked = []
            for function, kind in EXECUTIONS.items():
                if not re.search(rf'pub (?:async )?fn {function}\(', item['body']):
                    continue
                expression = f'super::{name}::{function}()' + ('.await' if kind == 'async' else '')
                if kind != 'unit':
                    expression += '.unwrap()'
                attribute = '#[tokio::test(flavor="current_thread")]\nasync' if kind == 'async' else '#[test]\n'
                tests.append(f'{attribute} fn {name}_{function}() {{ let _ = {expression}; }}')
                invoked.append(function)
                executions.append(function)
            report.append({key: item[key] for key in ['path', 'line', 'mode']} | {
                'sha256': hashlib.sha256(item['body'].encode()).hexdigest(), 'executed': invoked,
                'compile_only': not invoked})
        modules.append('#[cfg(test)]\nmod checked_examples {\n' + '\n'.join(tests) + '\n}')
        (workspace / 'src/lib.rs').write_text('\n'.join(modules) + '\n')
        env = {**os.environ, 'CARGO_TARGET_DIR': str(target / f'consumer-aggregate-{template}')}
        flags = ['--manifest-path', str(workspace / 'Cargo.toml'), '--locked', '--offline']
        if mode == 'browser':
            run(['cargo', 'check', '--lib', *flags, '--features', 'ssr'], env)
            for renderer in ['hydrate', 'csr']:
                run(['cargo', 'check', '--lib', *flags, '--target', 'wasm32-unknown-unknown', '--features', renderer], env)
        else:
            run(['cargo', 'check', '--all-targets', *flags], env)
            run(['cargo', 'test', '--lib', *flags, '--', '--test-threads', '2'], env)
        assert (workspace / 'Cargo.lock').read_bytes() == lock
        assert original.with_name('Cargo.lock').read_bytes() == lock
        assert (ROOT / 'Cargo.lock').read_bytes() == root_lock
        print(f'{mode}: exact fences compiled; {len(tests)} finite executions passed', flush=True)
    assert sorted(executions) == sorted(EXECUTIONS)
    result = {'external_workspace': str(external), 'examples': report, 'executions': executions}
    (external / 'examples.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'external_workspace': str(external), 'checked_fences': len(report),
                      'finite_executions': len(executions)}, indent=2), flush=True)


def main():
    """Static inspection by default; explicit switches opt into formatting/compilation."""
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
