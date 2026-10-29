#!/usr/bin/env python3
"""Finite independent-consumer proofs. Recorder-wrap this entry; no installs/listeners."""
import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
TARGET = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target/issue17')).resolve()
OUT = Path(os.environ.get('INDEPENDENT_OUTPUT_DIR', '/private/tmp/leptos-independent')).resolve()
MANIFEST = str(HERE / 'Cargo.toml')
CARGO = ['cargo']
OPTIONS = ['--manifest-path', MANIFEST, '--locked', '--offline']


def run(args, *, output=None, timeout=600, cwd=ROOT, env=None):
    """Enforce a deadline, cap the aggregate target, and kill/reap unfinished owned children."""
    print('$ ' + shlex.join(map(str, args)), flush=True)
    settings = dict(os.environ, CARGO_TARGET_DIR=str(TARGET))
    settings.update(env or {})
    stream = output.open('w') if output else None
    started = time.monotonic()
    try:
        with subprocess.Popen(list(map(str, args)), cwd=cwd, env=settings, stdout=stream,
                              start_new_session=True) as child:
            try:
                while child.poll() is None:
                    if time.monotonic() - started > timeout:
                        raise TimeoutError('independent proof deadline')
                    if TARGET.exists():
                        usage = subprocess.run(['du', '-sk', str(TARGET)], capture_output=True,
                                               text=True, timeout=15)
                        # Cargo may remove a transient file during du; the outer supervisor
                        # also checks the aggregate continuously and rejects other errors.
                        vanished = usage.stderr.splitlines()
                        transient = vanished and all(line.startswith('du: ' + str(TARGET) + '/') and
                            line.endswith(': No such file or directory') for line in vanished)
                        if usage.returncode and not (usage.returncode == 1 and transient):
                            raise RuntimeError('cannot measure complete target: ' + usage.stderr)
                        if int(usage.stdout.split()[0]) > 8 * 1024 * 1024:
                            raise RuntimeError('independent target exceeds 8 GiB')
                    time.sleep(.25)
                if child.returncode:
                    raise subprocess.CalledProcessError(child.returncode, args)
            finally:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGKILL)
                child.wait()
    finally:
        if stream:
            stream.close()


def native():
    """Test defaults and the two useful SSR helper selections without unifying modes."""
    for features in ['', 'ssr', 'ssr,async,rich,format,stream']:
        run(CARGO + ['test', *OPTIONS, '--all-targets'] + (['--features', features] if features else []))
    run(CARGO + ['fmt', '--manifest-path', MANIFEST, '--', '--check'])
    run(CARGO + ['clippy', *OPTIONS, '--all-targets', '--features', 'ssr,async,rich,format,stream', '--', '-D', 'warnings'])
    run(CARGO + ['test', *OPTIONS, '--doc', '--features', 'ssr,async,rich,format,stream'])
    run(CARGO + ['doc', *OPTIONS, '--no-deps', '--features', 'ssr,async,rich,format,stream'], env={'RUSTDOCFLAGS': '-D warnings'})


def build():
    """Copy matching real native/hydrate/CSR artifacts before the shared output is reused."""
    version = subprocess.check_output(['wasm-bindgen', '--version'], text=True).strip()
    assert version == 'wasm-bindgen 0.2.129', 'use the approved installed bindgen; do not install'
    for variant, helpers in [('base', []), ('helpers', ['async', 'rich', 'format'])]:
        destination = OUT / variant
        destination.mkdir(parents=True, exist_ok=True)
        features = ','.join(['ssr', *helpers])
        run(CARGO + ['build', *OPTIONS, '--example', 'document', '--features', features])
        shutil.copy2(TARGET / 'debug/examples/document', destination / 'document')
        for mode in ['hydrate', 'csr']:
            for theme, locale in [('light', 'en'), ('dark', 'fr')]:
                run([destination / 'document', mode, theme, locale],
                    output=destination / f'{mode}-{theme}-{locale}.html', timeout=20)
            run(CARGO + ['build', *OPTIONS, '--lib', '--target', 'wasm32-unknown-unknown',
                         '--features', ','.join([mode, *helpers])])
            assets = destination / mode
            if assets.exists():
                shutil.rmtree(assets)
            assets.mkdir()
            run(['wasm-bindgen', '--target', 'web', '--no-typescript', '--out-dir', assets,
                 '--out-name', 'independent', TARGET / 'wasm32-unknown-unknown/debug/leptos_independent.wasm'])
            shutil.copy2(HERE / 'bootstrap.js', assets / 'start.js')
    print('PASS separate independent native/hydrate/CSR artifacts', flush=True)


def browser():
    """Run real Chromium against finite interception using only already installed tooling."""
    if not os.environ.get('INDEPENDENT_PLAYWRIGHT'):
        raise RuntimeError('INDEPENDENT_PLAYWRIGHT must name an already installed package')
    run(['node', HERE / 'browser.cjs'], timeout=180)


def main():
    """Dispatch one explicit finite proof group."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['native', 'build', 'browser', 'graphs', 'docs'])
    action = parser.parse_args().action
    OUT.mkdir(parents=True, exist_ok=True)
    if action in ['native', 'build', 'browser']:
        globals()[action]()
    else:
        run(['python3', HERE / (action + '.py')], timeout=1200)


if __name__ == '__main__':
    main()
