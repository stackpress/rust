#!/usr/bin/env python3
"""Finite no-listener native/WASM builds using exactly two owned target trees."""
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUT = Path('/private/tmp/s8-stream')
NATIVE = ROOT / 'target/s8-native'
WASM = ROOT / 'target/s8-wasm'


def run(name, args, target):
    """Retain exact commands and diagnostics outside disposable compilation trees."""
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
    with (OUT / (name + '.log')).open('w') as log:
        log.write('$ ' + ' '.join(map(str, args)) + '\n'); log.flush()
        subprocess.run(list(map(str, args)), cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=600)
    print('PASS ' + name, flush=True)


def main():
    """Require installed tooling; never install dependencies or launch a listener."""
    OUT.mkdir(exist_ok=True)
    if subprocess.check_output(['wasm-bindgen', '--version'], text=True).strip() != 'wasm-bindgen 0.2.129':
        raise RuntimeError('requires already installed wasm-bindgen 0.2.129')
    run('native-build', ['cargo', 'build', '--locked', '-p', 'stackpress-example-ui-stream-web'], NATIVE)
    run('hydrate-build', ['cargo', 'build', '--locked', '-p', 'stackpress-example-ui-stream-client', '--target', 'wasm32-unknown-unknown', '--features', 'hydrate'], WASM)
    assets = OUT / 'assets'; assets.mkdir(exist_ok=True)
    run('bindgen', ['wasm-bindgen', '--target', 'web', '--no-typescript', '--out-dir', assets, '--out-name', 'stream', WASM / 'wasm32-unknown-unknown/debug/stackpress_example_ui_stream_client.wasm'], WASM)
    shutil.copyfile(ROOT / 'examples/ui-stream/bootstrap.js', assets / 'start.js')
    shutil.copy2(NATIVE / 'debug/stackpress-example-ui-stream-web', OUT / 'stream-host')


if __name__ == '__main__': main()
