#!/usr/bin/env python3
"""Finite pinned native/WASM builds; no installs or listener; outputs under the M0 receipt path."""
import os
from pathlib import Path
import subprocess
import shutil

ROOT = Path(__file__).resolve().parents[2]
LOGS = Path(os.environ.get('GALLERY_OUTPUT_DIR', '/private/tmp/s8-m0'))
# An explicit Cargo target root can share native/WASM output under a disk budget.
NATIVE = ROOT / os.environ.get('CARGO_TARGET_DIR', 'target/s8-native')
WASM = ROOT / os.environ.get('CARGO_TARGET_DIR', 'target/s8-wasm')

def run(name, command, target):
    """Retain exact output and stop on any finite command failure."""
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
    with (LOGS / f'{name}.log').open('w') as log:
        log.write('$ ' + ' '.join(map(str, command)) + '\n'); log.flush()
        subprocess.run(list(map(str, command)), cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=600, check=True)
    print('PASS ' + name, flush=True)

def main():
    """Build distinct modes with the already installed matching bindgen tool."""
    if subprocess.check_output(['wasm-bindgen','--version'], text=True).strip() != 'wasm-bindgen 0.2.129':
        raise RuntimeError('requires installed wasm-bindgen 0.2.129; no implicit install')
    LOGS.mkdir(parents=True,exist_ok=True)
    run('gallery-native-build',['cargo','build','--locked','-p','stackpress-example-ui-gallery-web'],NATIVE)
    for mode in ('hydrate','csr'):
        run(f'gallery-{mode}-build',['cargo','build','--locked','-p','stackpress-example-ui-gallery-client','--target','wasm32-unknown-unknown','--features',mode],WASM)
        output=LOGS / 'gallery-assets' / mode
        if output.exists(): shutil.rmtree(output)
        output.mkdir(parents=True)
        run(f'gallery-{mode}-bindgen',['wasm-bindgen','--target','web','--no-typescript','--out-dir',output,'--out-name','gallery',WASM / 'wasm32-unknown-unknown/debug/stackpress_example_ui_gallery_client.wasm'],WASM)
        shutil.copyfile(ROOT / 'examples/ui-gallery/bootstrap.js', output / 'start.js')
        import base64
        (output / 'sample.png').write_bytes(base64.b64decode('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aMesAAAAASUVORK5CYII='))

if __name__=='__main__': main()
