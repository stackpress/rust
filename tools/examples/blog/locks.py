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


ROOT = Path(__file__).resolve().parents[3]
BLOG = ROOT / 'examples/blog'
PROOFS = Path(__file__).resolve().parent
BASELINE = BLOG / '.build/lock-baseline.json'
EXAMPLE_LOCK = BLOG / 'Cargo.lock'
ROOT_LOCK = ROOT / 'Cargo.lock'
# G63 supersedes only G58 tuple placement; production approval adds exactly seven tuples.
G63_ADDITIONS = {
    ('data-encoding', '2.11.1',
     'registry+https://github.com/rust-lang/crates.io-index',
     '4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06'),
    ('rand', '0.9.4',
     'registry+https://github.com/rust-lang/crates.io-index',
     '44c5af06bb1b7d3216d91932aed5265164bf384dc89cd6ba05cf59a35f5f76ea'),
    ('rand_chacha', '0.9.0',
     'registry+https://github.com/rust-lang/crates.io-index',
     'd3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb'),
    ('rand_core', '0.9.3',
     'registry+https://github.com/rust-lang/crates.io-index',
     '99d9a13982dcf210057a8a78572b2217b667c3beacbf3a0d8b454f6f82837d38'),
    ('sha1', '0.10.6',
     'registry+https://github.com/rust-lang/crates.io-index',
     'e3bf829a2d51ab4a5ddf1352d8470c140cadc8301b2ae1789db023f01cedd6ba'),
    ('tokio-tungstenite', '0.29.0',
     'registry+https://github.com/rust-lang/crates.io-index',
     '8f72a05e828585856dacd553fba484c242c46e391fb0e58917c942ee9202915c'),
    ('tungstenite', '0.29.0',
     'registry+https://github.com/rust-lang/crates.io-index',
     '6c01152af293afb9c7c2a57e4b559c5620b421f6d133261c60dd2d0cdb38e6b8'),
}


def registry(data):
    """Preserve the exact ordered external identity, version, source and checksum tuples."""
    return [
        [p['name'], p['version'], p['source'], p.get('checksum')]
        for p in tomllib.loads(data.decode())['package']
        if p.get('source', '').startswith('registry+')
    ]


def verify_root_registry(current, expected):
    """Permit workspace-edge edits while freezing every external identity and checksum."""
    rows = registry(current)
    if len(expected) == 699:
        assert [row for row in rows if tuple(row) not in G63_ADDITIONS] == expected, 'root registry tuples/order changed'
        assert {tuple(row) for row in rows} - {tuple(row) for row in expected} == G63_ADDITIONS, 'G63 additions changed'
    else:
        assert rows == expected, 'root registry tuples/order changed'
    assert len(rows) == 706, 'unexpected root registry baseline'


def verify(baseline):
    """Reject registry mutation and every external tuple outside the root subset."""
    current = ROOT_LOCK.read_bytes()
    verify_root_registry(current, baseline['registry'])

    # Pruning unused root packages is expected in a standalone workspace. Neither a new
    # registry version/checksum outside the root nor a git-sourced package is permitted.
    example = EXAMPLE_LOCK.read_bytes()
    allowed = {tuple(p) for p in registry(current)}
    selected = registry(example)
    assert {tuple(p) for p in selected} <= allowed, 'blog registry tuples outside root'
    assert {tuple(p) for p in selected if p[0] == 'tungstenite'} == {row for row in G63_ADDITIONS if row[0] == 'tungstenite'}, 'blog handshake tuple changed'
    packages = tomllib.loads(example.decode())['package']
    # The sole WebSocket edge is build tooling, with just the HTTP upgrade feature enabled.
    manifest = tomllib.loads((BLOG / 'xtask/Cargo.toml').read_text())['dependencies']['tungstenite']
    assert manifest == {'version': '=0.29.0', 'default-features': False, 'features': ['handshake']}, 'G58 requires minimal handshake features'
    assert all(not p.get('source') or p['source'].startswith('registry+') for p in packages)
    print(f'locks: root 706 approved registry tuples; example {len(selected)} registry tuples are a pure root subset (G58 placement exception retired)')


def check():
    """Check against the committed root without depending on a resolution attempt marker."""
    # Workspace package/edge edits are legitimate before commit. Registry identities
    # remain anchored to HEAD; the outer gate separately freezes bytes during its run.
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
    assert len(baseline['registry']) == 706, 'unexpected root registry baseline'
    if release is None:
        assert not EXAMPLE_LOCK.exists(), 'example lock already exists; use check, not seed'
    else:
        assert release in ('g50-m3-exports', 'blog-web-stages', 'blog-post-g57', 'blog-plugin-crates', 'blog-xtask', 'sse-websocket'), 'manifest amendment lacks an explicit release'
        assert EXAMPLE_LOCK.exists(), 'manifest amendment needs the previous lock'
        if release in ('blog-plugin-crates', 'blog-xtask'):
            baseline['example_registry'] = registry(EXAMPLE_LOCK.read_bytes())
            assert len(baseline['example_registry']) == 437
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
        assert ROOT_LOCK.read_bytes() == root_bytes, 'root lock bytes changed during seed'
        verify(baseline)
    return result.returncode


def main():
    """Expose only the one-shot seed and later read-only invariant check."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('seed', 'check'))
    parser.add_argument('--release', choices=('g50-m3-exports', 'blog-web-stages', 'blog-post-g57', 'blog-plugin-crates', 'blog-xtask', 'sse-websocket'))
    args = parser.parse_args()
    if args.mode == 'seed':
        return seed(args.release)
    check()
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
