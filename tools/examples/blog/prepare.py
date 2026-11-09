#!/usr/bin/env python3
"""Prepare exactly one owned managed-smoke fixture and compile its host; never bind a listener."""
import argparse
import json
import os
from pathlib import Path
import shlex
import sys

ROOT = Path(__file__).resolve().parents[3]
BLOG = ROOT / 'examples/blog'
PROOFS = Path(__file__).resolve().parent
sys.path.insert(0, str(PROOFS))
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
    # Print a launch command for the prepared database; preparation never executes it.
    print('Replace <port> with a free port, then run from the blog directory:')
    print(f'cargo run -p blog-web --bin serve -- <port> {shlex.quote(str(config))}')
    print('Open http://localhost:<port>/')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
