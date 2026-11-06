#!/usr/bin/env python3
"""Check both real default generator trees without changing bytes or modification times."""
from pathlib import Path
import sys

BLOG = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BLOG))
from proof import bounded


def snapshot(tree):
    """Capture every managed file so a check cannot repair drift without detection."""
    return {str(path.relative_to(BLOG)): (path.read_bytes(), path.stat().st_mtime_ns)
            for path in (BLOG / '.build' / tree).rglob('*') if path.is_file()}


def main():
    """Invoke the official CLI's native/application checks against existing generated output."""
    before = {name: snapshot(name) for name in ['generated', 'app-generated']}
    command = ['cargo', 'run', '--manifest-path', str(BLOG / 'Cargo.toml'), '-p', 'blog-tool',
               '--locked', '--offline', '--']
    for action in ['generate', 'generate-app']:
        assert bounded(command + [action, '--check'], 120) == 0
        assert {name: snapshot(name) for name in before} == before
    print('Both default trees fresh; exact bytes and mtimes unchanged.')


if __name__ == '__main__':
    main()
