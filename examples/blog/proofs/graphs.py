#!/usr/bin/env python3
"""Assert current normal dependency edges using locked/offline cargo tree, never target output."""
import os
from pathlib import Path
import subprocess

BLOG = Path(__file__).resolve().parents[1]


def graph(package, *features):
    """Read only this manifest's selected renderer graph; no resolver mutation is allowed."""
    command = ['cargo', 'tree', '--manifest-path', str(BLOG / 'Cargo.toml'), '-p', package,
               '--locked', '--offline', '-e', 'normal', '--prefix', 'none', *features]
    result = subprocess.check_output(command, text=True, timeout=60)
    return {line.split(' v')[0] for line in result.splitlines() if ' v' in line}


def main():
    """Keep generators outside native runtime and native resources outside both browser graphs."""
    native = graph('blog-web')
    forbidden_native = {'blog-tool', 'stackpress-idea-parser', 'stackpress-idea-transformer'}
    assert not native & forbidden_native, native & forbidden_native
    print(f'native: {len(native)} packages; no Idea parser/transformer or blog-tool')
    for mode in ('hydrate', 'csr'):
        browser = graph('blog-client', '--target', 'wasm32-unknown-unknown',
                        '--no-default-features', '--features', mode)
        forbidden = {name for name in browser if name.startswith(('stackpress-sql', 'stackpress-server', 'stackpress-idea'))}
        forbidden |= browser & {'stackpress-framework-sql', 'stackpress-framework-models',
                                'stackpress-framework-identity', 'stackpress-framework-email',
                                'stackpress-cli', 'lettre', 'reqwest', 'ring', 'blog-tool', 'blog-web'}
        assert not forbidden, (mode, forbidden)
        print(f'{mode}: {len(browser)} packages; no native SQL/server/mail/identity/codegen')


if __name__ == '__main__':
    main()
