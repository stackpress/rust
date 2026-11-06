#!/usr/bin/env python3
"""Seed this example lock exactly once offline, then check registry/root-byte invariants.

Run through the repository build-benchmark recorder. ``seed`` performs the sole unlocked
offline resolution; ``check`` reads committed locks without a marker or Cargo operation. Subsequent Cargo commands use
--locked --offline. The durable attempt marker prevents an accidental second resolution.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tomllib


BLOG = Path(__file__).resolve().parents[1]
ROOT = BLOG.parents[1]
BASELINE = BLOG / '.build/lock-baseline.json'
EXAMPLE_LOCK = BLOG / 'Cargo.lock'
ROOT_LOCK = ROOT / 'Cargo.lock'


def registry(data):
    """Preserve the exact ordered external identity, version, source and checksum tuples."""
    return [
        [p['name'], p['version'], p['source'], p.get('checksum')]
        for p in tomllib.loads(data.decode())['package']
        if p.get('source', '').startswith('registry+')
    ]


def verify(baseline):
    """Reject root mutation or an external tuple not present in the original root lock."""
    current = ROOT_LOCK.read_bytes()
    assert hashlib.sha256(current).hexdigest() == baseline['root_sha256'], 'root lock bytes changed'
    assert registry(current) == baseline['registry'], 'root registry tuples/order changed'
    assert len(baseline['registry']) == 699, 'unexpected root registry baseline'

    # Pruning unused root packages is expected in a standalone workspace. Neither a new
    # registry version/checksum nor a git-sourced package is a permitted workspace edge.
    example = EXAMPLE_LOCK.read_bytes()
    allowed = {tuple(p) for p in baseline['registry']}
    selected = registry(example)
    assert all(tuple(p) in allowed for p in selected), 'example gained an external tuple'
    packages = tomllib.loads(example.decode())['package']
    assert all(not p.get('source') or p['source'].startswith('registry+') for p in packages)
    print(f'locks: root 699 identical tuples/bytes; example {len(selected)} registry tuples are an exact subset')


def check():
    """Check against the committed root without depending on a resolution attempt marker."""
    # Git reads local committed source only. Comparing the worktree bytes with that source
    # also detects an accidental root-lock edit, rather than blessing it as a new baseline.
    committed = subprocess.check_output(['git', '-C', str(ROOT), 'show', 'HEAD:Cargo.lock'])
    verify({'root_sha256': hashlib.sha256(committed).hexdigest(),
            'registry': registry(committed)})


def seed(release=None):
    """Resolve once for initial setup or one explicitly authorized manifest amendment."""
    marker = BASELINE if release is None else BASELINE.with_name(f'lock-{release}.json')
    root_bytes = ROOT_LOCK.read_bytes()
    baseline = {
        'root_sha256': hashlib.sha256(root_bytes).hexdigest(),
        'registry': registry(root_bytes),
    }
    assert len(baseline['registry']) == 699, 'unexpected root registry baseline'
    if release is None:
        assert not EXAMPLE_LOCK.exists(), 'example lock already exists; use check, not seed'
    else:
        assert release in ('g50-m3-exports', 'blog-web-stages', 'blog-post-g57'), 'manifest amendment lacks an explicit release'
        assert EXAMPLE_LOCK.exists(), 'manifest amendment needs the previous lock'
    BASELINE.parent.mkdir(parents=True, exist_ok=True)

    # Claim the attempt before Cargo runs. Failed attempts remain evidence and never authorize
    # another unlocked resolution merely because the child process returned an error.
    with marker.open('x') as handle:
        json.dump(baseline, handle, indent=2)
        handle.write('\n')
    if release is not None:
        # Retain the old lock as evidence before the approved root-seeded amendment.
        marker.with_suffix('.previous.lock').write_bytes(EXAMPLE_LOCK.read_bytes())
    with EXAMPLE_LOCK.open('xb' if release is None else 'wb') as handle:
        handle.write(root_bytes)

    try:
        result = subprocess.run(
            ['cargo', 'tree', '--manifest-path', str(BLOG / 'Cargo.toml'),
             '--workspace', '--offline', '-e', 'normal', '--prefix', 'none'],
            stdout=subprocess.DEVNULL, check=False,
        )
    finally:
        # This must be the first action after resolution, including a failed Cargo process.
        verify(baseline)
    return result.returncode


def main():
    """Expose only the one-shot seed and later read-only invariant check."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('seed', 'check'))
    parser.add_argument('--release', choices=('g50-m3-exports', 'blog-web-stages', 'blog-post-g57'))
    args = parser.parse_args()
    if args.mode == 'seed':
        return seed(args.release)
    check()
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
