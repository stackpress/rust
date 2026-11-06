#!/usr/bin/env python3
"""Prepare exactly one owned managed-smoke fixture and compile its host; never bind a listener."""
import argparse
import json
import os
from pathlib import Path
import sys

BLOG = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BLOG))
from proof import bounded


def main():
    """Run the real documented finite setup/populate/query commands before coordinator launch."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', default='managed-m4')
    args = parser.parse_args()
    root = BLOG / '.build' / args.directory
    root.mkdir(parents=True, exist_ok=False)
    config = root / 'config.json'
    config.write_text(json.dumps({'sqlite': {'path': str(root / 'blog.sqlite')}}))
    cargo = ['cargo', 'run', '--manifest-path', str(BLOG / 'Cargo.toml'), '-p', 'blog-web',
             '--locked', '--offline', '--bin']
    os.chdir(BLOG)
    commands = [cargo + ['setup', '--', str(root / 'blog.sqlite')],
                cargo + ['blog-web', '--', '-b', str(config), 'emit', 'populate'],
                cargo + ['blog-web', '--', '-b', str(config), 'emit', 'query',
                         'SELECT COUNT(*) AS total FROM article']]
    for command in commands:
        if bounded(command, 120):
            return 1
    # This is proof-fixture config only: source production webhook URLs remain untouched.
    config.write_text(json.dumps({'sqlite': {'path': str(root / 'blog.sqlite')},
                                  'api': {'webhooks': []}}))
    code = bounded(['cargo', 'build', '--manifest-path', str(BLOG / 'Cargo.toml'), '-p', 'blog-web',
                    '--bin', 'serve', '--bin', 'probe', '--locked', '--offline'], 180)
    if code:
        return code
    target = Path(os.environ['CARGO_TARGET_DIR'])
    repo = BLOG.parents[1]
    # This script is supplied to the coordinator only; preparation never executes it.
    launcher = root / 'run.sh'
    launcher.write_text(f'''#!/bin/sh
set -eu
cd '{BLOG}'
exec python3 '{repo / '.agents/scripts/build-benchmark.py'}' --store '/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks' run --label 'blog final coordinator managed localhost HTTP smoke' --kind other --scope consumer --cache warm -- '{target / 'debug/serve'}' "$1" '{config}'
''')
    print('Coordinator launch only:', f'devmetrics start --summary "Stackpress blog final localhost write/export smoke" -- \'bash {launcher} {{port}}\'')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
