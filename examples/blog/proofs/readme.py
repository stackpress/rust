#!/usr/bin/env python3
"""Execute the documented finite first-success path in an isolated copied-out app."""
import os
from pathlib import Path
import re
import shutil
import sys
import subprocess
import json

BLOG = Path(__file__).resolve().parents[1]
ROOT = BLOG.parents[1]
sys.path.insert(0, str(BLOG))
from proof import bounded


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
    # This is the README's explicit mkdir step before the managed writer publishes output.
    output.mkdir()
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
    cargo = ['cargo', 'run', '--locked', '--offline', '-p']
    commands = [cargo + ['blog-tool', '--', 'generate-app'],
                cargo + ['blog-tool', '--', 'generate-app', '--check'],
                cargo + ['blog-web', '--bin', 'setup', '--', '.build/blog.sqlite'],
                cargo + ['blog-web', '--bin', 'blog-web', '--', '-b', 'config/develop.json', 'emit', 'revision'],
                cargo + ['blog-web', '--bin', 'blog-web', '--', '-b', 'config/develop.json', 'emit', 'populate'],
                cargo + ['blog-web', '--bin', 'blog-web', '--', '-b', 'config/develop.json', 'emit', 'query',
                         'SELECT COUNT(*) AS total FROM article']]
    try:
        for command in commands[:-1]:
            if bounded(command, 180):
                return 1
        query=subprocess.run(commands[-1],stdout=subprocess.PIPE,text=True,timeout=120)
        print(query.stdout,end='')
        assert query.returncode==0
        assert json.loads(query.stdout.strip().splitlines()[-1])['results']['rows'][0]['total']==2
    finally:
        assert (destination / 'Cargo.lock').read_bytes() == source_lock, 'copied lock changed'
        assert (BLOG / 'Cargo.lock').read_bytes() == source_lock, 'source lock changed'
        assert (BLOG / 'Cargo.toml').read_bytes() == source_manifest, 'source manifest changed'
        assert (ROOT / 'Cargo.lock').read_bytes() == root_lock, 'root lock changed'
        assert (ROOT / 'Cargo.toml').read_bytes() == root_manifest, 'root manifest changed'
    print('Copied app: generate/check/setup/revision/populate/query passed; copied/source/root locks unchanged; no listener.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
