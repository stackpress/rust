#!/usr/bin/env python3
"""Check Lane B doc links/claim anchors and compile exact Rust fences in external workspaces.

Record this outer script once. --compile uses existing consumer manifests/locks,
keeps every Cargo invocation locked/offline and gives each workspace its own target.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[4]
FRAMEWORK = ROOT / 'crates/framework'
DOCS = Path(__file__).resolve().parent


def slug(title):
    """Match the simple GitHub heading anchors used by these documentation pages."""
    return re.sub(r'[^\w\- ]', '', title.lower().replace('`', '')).replace(' ', '-')


def pages():
    """Include both crate entry points and every authored Markdown documentation page."""
    return sorted(path for name in ['csrf', 'email'] for path in [
        FRAMEWORK / f'framework-{name}/README.md',
        *(FRAMEWORK / f'framework-{name}/docs').rglob('*.md'),
    ])


def inspect():
    """Reject omitted examples, broken local links and structural/claim-anchor drift."""
    snippets = []
    documents = pages()
    for path in documents:
        text = path.read_text()
        visible = re.sub(r'^```[^\n]*\n.*?^```\s*$', '', text, flags=re.M | re.S)
        headings = re.findall(r'^(#{1,6}) (.+)$', visible, re.M)
        assert sum(level == '#' for level, _ in headings) == 1, path
        assert '\n\n\n' not in text, (path, 'consecutive blank lines')
        lines = visible.splitlines()
        for index, line in enumerate(lines):
            if re.match(r'^#{1,6} ', line):
                following = next((item for item in lines[index + 1:] if item.strip()), '')
                assert not following.startswith('#'), (path, line, 'heading needs orientation')
        for target in re.findall(r'(?<!!)\[[^\]]+\]\(([^)\s]+)\)', visible):
            if re.match(r'^[a-z]+:', target):
                raise AssertionError((path, 'external links need a separate check', target))
            file, _, anchor = target.partition('#')
            destination = (path.parent / file).resolve() if file else path
            assert destination.exists(), (path, target)
            if anchor:
                assert destination.suffix == '.md', (path, target)
                anchors = {slug(title) for _, title in re.findall(r'^(#{1,6}) (.+)$', destination.read_text(), re.M)}
                assert anchor in anchors, (path, target, anchors)
        matches = list(re.finditer(r'^```([^\n]*)\n(.*?)^```\s*$', text, re.M | re.S))
        assert text.count('```') == 2 * len(matches), (path, 'unclosed fence')
        for match in matches:
            assert match[1] == 'rust', (path, 'all examples must be tagged Rust')
            marker = text[:match.start()].rstrip().splitlines()[-1]
            mode = re.fullmatch(r'<!-- example: (direct|server) -->', marker)
            assert mode, (path, 'example needs explicit graph', marker)
            snippets.append({'path': str(path.relative_to(ROOT)), 'line': text[:match.start()].count('\n') + 1,
                             'mode': mode[1], 'body': match[2]})
    claims = json.loads((DOCS / 'claims.json').read_text())
    for claim in claims:
        assert claim['statement'] in (ROOT / claim['page']).read_text(), claim['id']
        for evidence in claim['evidence']:
            source = (ROOT / evidence['path']).read_text()
            for anchor in evidence['contains']:
                assert anchor in source, (claim['id'], evidence['path'], anchor)
    assert {item['mode'] for item in snippets} == {'direct', 'server'}
    print(json.dumps({'pages': len(documents), 'rust_fences': len(snippets), 'claim_anchors': len(claims),
                      'links': 'all local paths and anchors resolve'}, indent=2), flush=True)
    return snippets


def run(command, env):
    """Keep nested Cargo argv visible in the single outer benchmark record."""
    print('COMMAND:', *map(str, command), flush=True)
    subprocess.run(command, check=True, env=env)
    size = int(subprocess.check_output(['du', '-sk', os.environ['CARGO_TARGET_DIR']], text=True).split()[0])
    print(json.dumps({'combined_target_KiB': size, 'headroom_GiB': 19.4, 'cap_GiB': 20}), flush=True)
    assert size < 19.4 * 1024 * 1024, 'effort target reached headroom guard'


def compile_examples(snippets):
    """Copy exact examples to finite external consumers without editing repository manifests."""
    target = Path(os.environ['CARGO_TARGET_DIR'])
    assert target.is_absolute(), 'CARGO_TARGET_DIR must be an absolute effort target'
    external = Path(tempfile.mkdtemp(prefix='slice9b-docs-', dir='/private/tmp'))
    # Only explicitly local/finite examples run. Handlers accepting host state are compile-checked.
    executions = {
        'round_trip': 'sync', 'configured_service': 'value', 'first_email': 'async',
        'transport_configs': 'sync', 'inspect_delivery_mapping': 'async', 'inspect_options': 'sync',
        'message_options': 'sync', 'smtp_auth_configs': 'sync', 'native_round_trip': 'async', 'activation_modes': 'sync',
        'inspect_pool_binding': 'async', 'buffer_result': 'sync', 'inspect_metadata': 'sync',
        'factory_with_content': 'sync', 'reusable_event': 'async',
    }
    report = []
    for mode, template in [('direct', 'direct'), ('server', 'event')]:
        workspace = external / mode
        (workspace / 'src').mkdir(parents=True)
        original = FRAMEWORK / f'framework-email/consumers/{template}/Cargo.toml'
        manifest = re.sub(r'path = "([^"]+)"', lambda m: 'path = ' + json.dumps(str((original.parent / m[1]).resolve())), original.read_text())
        (workspace / 'Cargo.toml').write_text(manifest)
        lock = original.with_name('Cargo.lock').read_bytes()
        (workspace / 'Cargo.lock').write_bytes(lock)
        source = []
        tests = []
        for index, item in enumerate(snippets):
            if item['mode'] != mode:
                continue
            module = f'example_{index}'
            # Module isolation prevents unrelated examples from supplying missing imports/types.
            source.append(f"// {item['path']}:{item['line']}\npub mod {module} {{\n{item['body']}\n}}\n")
            for name, execution in executions.items():
                if re.search(rf'pub (?:async )?fn {name}\(', item['body']):
                    expression = f'super::{module}::{name}()' + ('.await' if execution == 'async' else '')
                    if execution != 'value':
                        expression += '.unwrap()'
                    tests.append(f'#[tokio::test(flavor = "current_thread")]\nasync fn {module}_{name}() {{ let _ = {expression}; }}')
            report.append({**{key: item[key] for key in ['path', 'line', 'mode']},
                           'sha256': hashlib.sha256(item['body'].encode()).hexdigest()})
        source.append('#[cfg(test)]\nmod checked_examples {\n' + '\n'.join(tests) + '\n}\n')
        (workspace / 'src/lib.rs').write_text('\n'.join(source))
        env = {**os.environ, 'CARGO_TARGET_DIR': str(target / f'docs-{mode}')}
        flags = ['--manifest-path', str(workspace / 'Cargo.toml'), '--locked', '--offline']
        run(['cargo', 'check', '--all-targets', *flags], env)
        run(['cargo', 'test', '--lib', *flags, '--', '--test-threads', '2'], env)
        assert (workspace / 'Cargo.lock').read_bytes() == lock
        print(f'{mode}: all fences compiled; {len(tests)} finite examples executed', flush=True)
    result = {'external_workspace': str(external), 'examples': report}
    (external / 'examples.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'external_workspace': str(external), 'checked_fences': len(report)}, indent=2), flush=True)


def main():
    """Make compilation opt-in so static link/claim checks need no Cargo invocation."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compile', action='store_true')
    args = parser.parse_args()
    examples = inspect()
    if args.compile:
        compile_examples(examples)


if __name__ == '__main__':
    main()
