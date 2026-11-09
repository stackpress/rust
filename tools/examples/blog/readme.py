#!/usr/bin/env python3
"""Execute the documented finite first-success path in an isolated copied-out app."""
import os
from pathlib import Path
import re
import shlex
import shutil
import sqlite3
import sys
import subprocess
import json

ROOT = Path(__file__).resolve().parents[3]
BLOG = ROOT / 'examples/blog'
PROOFS = Path(__file__).resolve().parent
sys.path.insert(0, str(PROOFS))
from proof import bounded


def documented_commands(section):
    """Read actual shell examples from one README section, preserving argv quoting."""
    text = (BLOG / 'README.md').read_text().split(f'## {section}\n', 1)[1]
    text = text.split('\n## ', 1)[0]
    blocks = re.findall(r'^```sh\n(.*?)^```', text, re.MULTILINE | re.DOTALL)
    assert blocks, f'no documented commands in {section}'
    return [shlex.split(line) for block in blocks for line in block.splitlines() if line.strip()]


def run_commands(commands):
    """Run finite examples and retain real JSON query results for their walkthrough checks."""
    queries = []
    for command in commands:
        print('README:', shlex.join(command), flush=True)
        if 'query' in command:
            result = subprocess.run(command, stdout=subprocess.PIPE, text=True, timeout=180)
            print(result.stdout, end='')
            result.check_returncode()
            queries.append(json.loads(result.stdout.strip().splitlines()[-1])['results']['rows'])
        else:
            assert bounded(command, 1200) == 0, command
    return queries


def main():
    """Point only existing repo-local edges at this checkout; never resolve or change a lock."""
    source_lock = (BLOG / 'Cargo.lock').read_bytes()
    root_lock = (ROOT / 'Cargo.lock').read_bytes()
    root_manifest = (ROOT / 'Cargo.toml').read_bytes()
    source_manifest = (BLOG / 'Cargo.toml').read_bytes()
    parent=BLOG / '.build/readme'
    parent.mkdir(parents=True,exist_ok=True)
    # Keep one persistent consumer path with its own warm target: changing the copied source
    # directory while reusing a target can execute a prior top-level generator binary.
    destination = parent / 'project'
    shutil.copytree(BLOG, destination, dirs_exist_ok=True,ignore=shutil.ignore_patterns('.build', 'target', '.env'))
    # The first-success sequence must not reuse a previous database/history/generated tree.
    # Only this proof's owned output is retired; authored copied source stays reviewable.
    output = destination / '.build'
    if output.exists():
        assert output.resolve().parent == destination.resolve()
        shutil.rmtree(output)
    manifest = source_manifest.decode()
    # The README's unpublished-crate step must work in any fresh checkout. Derive its
    # absolute paths from the existing edges, not a machine-specific ignored review patch.
    edges = re.compile(r'path = "(\.\./\.\./(?:crates|tools)/[^"]+)"')
    selected = []
    def checkout_edge(match):
        source = (BLOG / match[1]).resolve()
        assert source.is_relative_to(ROOT) and (source / 'Cargo.toml').is_file(), source
        selected.append((match[0], 'path = "' + str(source) + '"'))
        return selected[-1][1]
    patched = edges.sub(checkout_edge, manifest)
    assert selected, 'no repo-local dependencies found'
    restored = patched
    for old, new in selected:
        restored = restored.replace(new, old)
    assert restored == manifest, 'copied manifest changed beyond existing path edges'
    (destination / 'Cargo.toml').write_text(patched)
    native = Path(os.environ['CARGO_TARGET_DIR'])
    os.environ['CARGO_TARGET_DIR'] = str(native.parent / 'readme')
    os.chdir(destination)
    commands = documented_commands('Setup')
    # The documented listener is checked as text only. Every preceding setup command runs.
    assert commands.pop() == ['cargo', 'run', '-p', 'blog-web', '--bin', 'serve', '--',
                              '<port>', 'config/develop.json']
    assert commands.pop() == ['cargo', 'xtask', 'build-browser']
    migration = parent / 'migration'
    try:
        assert run_commands(commands) == [[{'total': 2}]]
        # Browser consumer targets must not overwrite the main app's same-named products.
        os.environ['CARGO_TARGET_DIR'] = str(native.parent / 'readme-browser/native')
        run_commands([['cargo', 'xtask', 'build-browser']])
        shutil.rmtree(native.parent / 'readme-browser')
        os.environ['CARGO_TARGET_DIR'] = str(native.parent / 'readme')
        run_commands(documented_commands('Test'))
        # Keep the first-success project intact for nickname.py. The README migration gets
        # its own copied database, history, generated source and native compilation target.
        if migration.exists():
            assert migration.resolve().parent == parent.resolve()
            shutil.rmtree(migration)
        shutil.copytree(destination, migration)
        schema = (migration / 'schema.idea').read_text()
        walkthrough = (BLOG / 'README.md').read_text().split('## Add a column and migrate\n', 1)[1]
        field = re.findall(r'^```idea\n(.*?)^```', walkthrough, re.MULTILINE | re.DOTALL)
        assert len(field) == 1
        end = schema.index('\n}', schema.index('model Article'))
        (migration / 'schema.idea').write_text(schema[:end] + '\n' + field[0].rstrip() + schema[end:])
        with sqlite3.connect(migration / '.build/blog.sqlite') as connection:
            ids = [row[0] for row in connection.execute('SELECT id FROM article')]
        os.chdir(migration)
        os.environ['CARGO_TARGET_DIR'] = str(native.parent / 'readme-migration')
        rows, = run_commands(documented_commands('Add a column and migrate'))
        assert sorted(row['id'] for row in rows) == sorted(ids)
        assert all(row['subtitle'] is None for row in rows)
    finally:
        os.chdir(BLOG)
        os.environ['CARGO_TARGET_DIR'] = str(native)
        # Keep emitted products and copied inputs; completed consumer build targets are
        # disposable and must not consume the full gate's later coverage reservation.
        for name in ('readme-browser', 'readme-migration'):
            target = native.parent / name
            assert target.resolve().parent == native.parent.resolve()
            if target.exists():
                shutil.rmtree(target)
        assert (destination / 'Cargo.lock').read_bytes() == source_lock, 'copied lock changed'
        assert (BLOG / 'Cargo.lock').read_bytes() == source_lock, 'source lock changed'
        assert (BLOG / 'Cargo.toml').read_bytes() == source_manifest, 'source manifest changed'
        assert (ROOT / 'Cargo.lock').read_bytes() == root_lock, 'root lock changed'
        assert (ROOT / 'Cargo.toml').read_bytes() == root_manifest, 'root manifest changed'
        if migration.exists():
            assert (migration / 'Cargo.lock').read_bytes() == source_lock, 'migration lock changed'
    print('README setup, both browser bundles and nullable-column migration passed; copied/source/root locks unchanged; no listener.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
