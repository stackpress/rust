#!/usr/bin/env python3
"""Finite, offline browser pipeline. Invoke through the repository benchmark recorder."""
import os
from pathlib import Path
import shutil
import sys

BLOG = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BLOG))
from proof import bounded


def main():
    """Compile each renderer in its own target and emit its matching cached bindgen glue."""
    bindgen = shutil.which('wasm-bindgen')
    assert bindgen, 'the preinstalled wasm-bindgen CLI is required; no installation is attempted'
    original = os.environ.get('CARGO_TARGET_DIR')
    native = Path(original) if original else BLOG / 'target/native'
    target = native.parent
    try:
        for mode in ('hydrate', 'csr'):
            os.environ['CARGO_TARGET_DIR'] = str(target / mode)
            command = ['cargo', 'build', '--manifest-path', str(BLOG / 'Cargo.toml'), '-p', 'blog-client',
                       '--target', 'wasm32-unknown-unknown', '--no-default-features', '--features', mode,
                       '--locked', '--offline']
            if bounded(command):
                return 1
            output = BLOG / '.build/browser' / mode
            output.mkdir(parents=True, exist_ok=True)
            if bounded([bindgen, '--target', 'web', '--out-dir', str(output), '--out-name', 'blog',
                        str(target / mode / 'wasm32-unknown-unknown/debug/blog_client.wasm')]):
                return 1
    finally:
        if original is None:
            os.environ['CARGO_TARGET_DIR'] = str(native)
        else:
            os.environ['CARGO_TARGET_DIR'] = original
    # Native fixtures belong to their selected native proof; browser builds never create a
    # second native target merely because a caller supplied a renderer-specific target path.
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
